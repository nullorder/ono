//! Component: splash — animated FIGlet wordmark.
//!
//! Matches `specs/components/splash.toml`. Gradient sweeps across the large
//! text, per-eye pulse animates the two O's out of phase, an optional
//! scanline rolls down, and an occasional idle flicker dims the whole
//! banner for a frame.
//!
//! Stateless: caller owns the clock and passes `elapsed`. Build the
//! `Banner` once (FIGlet conversion is non-trivial) and render each frame.

use std::time::Duration;

use figlet_rs::FIGlet;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::super::theme::{lerp_rgb, scale_rgb, Knobs, Palette};

const FLICKER_DIM: f32 = 0.6;
const FLICKER_PERIOD_MS: u64 = 9000;
const FLICKER_WINDOW_MS: u64 = 40;

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    glyph_idx: usize,
}

/// Pre-computed FIGlet grid used by [`Splash`]. Build once, render every
/// frame — FIGlet conversion is non-trivial and does not need to repeat.
///
/// The first two `'o'` characters (if any) are tracked as "eye" glyphs, which
/// [`Splash`] pulses out of phase to animate the wordmark's face.
pub struct Banner {
    grid: Vec<Vec<Option<Cell>>>,
    width: usize,
    height: usize,
    eye_glyphs: [Option<usize>; 2],
}

impl Banner {
    /// Build a banner grid from `text` using `font`. FIGlet conversion is
    /// non-trivial — construct once and reuse across frames.
    ///
    /// # Panics
    /// Panics if the supplied FIGlet font cannot render a character in `text`.
    pub fn from_text(text: &str, font: &FIGlet) -> Self {
        let mut glyph_lines: Vec<Vec<String>> = text
            .chars()
            .map(|ch| {
                let fig = font
                    .convert(&ch.to_string())
                    .unwrap_or_else(|| panic!("figlet failed to render {:?}", ch));
                trim_blank_rows(fig.to_string().lines().map(str::to_string).collect())
            })
            .collect();

        let height = glyph_lines.iter().map(Vec::len).max().unwrap_or(0);
        for lines in &mut glyph_lines {
            while lines.len() < height {
                lines.push(String::new());
            }
        }

        let widths: Vec<usize> = glyph_lines
            .iter()
            .map(|lines| lines.iter().map(|l| l.chars().count()).max().unwrap_or(0))
            .collect();
        let total_width: usize = widths.iter().sum();

        let mut grid: Vec<Vec<Option<Cell>>> = vec![vec![None; total_width]; height];
        let mut eye_glyphs: [Option<usize>; 2] = [None, None];
        let mut eye_cursor = 0;
        let mut x_cursor = 0;

        for (gi, (lines, ch)) in glyph_lines.iter().zip(text.chars()).enumerate() {
            if ch == 'o' && eye_cursor < eye_glyphs.len() {
                eye_glyphs[eye_cursor] = Some(gi);
                eye_cursor += 1;
            }
            for (y, line) in lines.iter().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    if !c.is_whitespace() {
                        let gx = x_cursor + x;
                        if gx < total_width {
                            grid[y][gx] = Some(Cell { ch: c, glyph_idx: gi });
                        }
                    }
                }
            }
            x_cursor += widths[gi];
        }

        Self { grid, width: total_width, height, eye_glyphs }
    }

    /// Grid width in cells.
    pub fn width(&self) -> u16 {
        self.width as u16
    }

    /// Grid height in rows.
    pub fn height(&self) -> u16 {
        self.height as u16
    }
}

fn trim_blank_rows(mut lines: Vec<String>) -> Vec<String> {
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    while lines.first().is_some_and(|l| l.trim().is_empty()) {
        lines.remove(0);
    }
    lines
}

/// Animated FIGlet wordmark with gradient sweep, optional per-eye pulse,
/// scanline, and idle flicker.
///
/// ```no_run
/// use std::time::Duration;
/// use figlet_rs::FIGlet;
/// use ono::components::splash::{Banner, Splash};
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 80, 16));
/// # let area = buf.area;
///
/// let theme = Theme::Forest;
/// let font = FIGlet::standard().unwrap();
/// let banner = Banner::from_text("ono", &font);
///
/// Splash::new(&banner, theme.palette(), theme.knobs())
///     .tagline("beautiful terminal UI components")
///     .elapsed(Duration::from_millis(1200))
///     .render(area, &mut buf);
/// ```
pub struct Splash<'a> {
    banner: &'a Banner,
    tagline: &'a str,
    pulse: bool,
    scanline: bool,
    flicker: bool,
    elapsed: Duration,
    palette: &'a Palette,
    knobs: &'a Knobs,
}

impl<'a> Splash<'a> {
    /// Construct a splash over a pre-built [`Banner`]. Defaults enable pulse,
    /// scanline, and flicker; the tagline is `"beautiful terminal UI
    /// components"`.
    pub fn new(banner: &'a Banner, palette: &'a Palette, knobs: &'a Knobs) -> Self {
        Self {
            banner,
            tagline: "beautiful terminal UI components",
            pulse: true,
            scanline: true,
            flicker: true,
            elapsed: Duration::ZERO,
            palette,
            knobs,
        }
    }

    /// Replace the line under the wordmark. Pass `""` to hide it.
    pub fn tagline(mut self, tagline: &'a str) -> Self {
        self.tagline = tagline;
        self
    }

    /// Toggle the per-eye pulse animation.
    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    /// Toggle the rolling scanline (only drawn when `knobs.scanline` is true).
    pub fn scanline(mut self, scanline: bool) -> Self {
        self.scanline = scanline;
        self
    }

    /// Toggle the idle dim-flicker (only drawn when `knobs.idle_flicker` is true).
    pub fn flicker(mut self, flicker: bool) -> Self {
        self.flicker = flicker;
        self
    }

    /// Monotonic clock driving the animation. Typically
    /// `Instant::now().duration_since(start)`.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = elapsed;
        self
    }

    fn banner_lines(&self) -> Vec<Line<'static>> {
        let t = self.elapsed.as_secs_f32();
        let banner = self.banner;
        let knobs = self.knobs;
        let palette = self.palette;

        let scanline_on = self.scanline && knobs.scanline && banner.height > 0;
        let scanline_row = if scanline_on {
            Some(((t * knobs.scanline_speed_rows_per_sec) as usize) % banner.height)
        } else {
            None
        };

        let flicker_on = self.flicker
            && knobs.idle_flicker
            && (self.elapsed.as_millis() as u64 % FLICKER_PERIOD_MS) < FLICKER_WINDOW_MS;

        let pulse_amp = if self.pulse { knobs.pulse_amplitude } else { 0.0 };
        let period = knobs.gradient_period_secs.max(0.5);
        let width = banner.width.max(1) as f32;

        let g_start = color_rgb(palette.secondary);
        let g_mid = color_rgb(palette.primary);
        let g_end = color_rgb(palette.bright);

        banner
            .grid
            .iter()
            .enumerate()
            .map(|(y, row)| {
                let spans: Vec<Span<'static>> = row
                    .iter()
                    .enumerate()
                    .map(|(x, cell)| match cell {
                        None => Span::raw(" "),
                        Some(c) => {
                            let phase = x as f32 / width + t / period;
                            let mut rgb = tri_gradient(g_start, g_mid, g_end, phase);

                            let mut brightness =
                                pulse_factor(c.glyph_idx, banner.eye_glyphs, t, pulse_amp);
                            if flicker_on {
                                brightness *= FLICKER_DIM;
                            }
                            if scanline_row == Some(y) {
                                brightness *= knobs.scanline_boost;
                            }
                            rgb = scale_rgb(rgb, brightness);

                            Span::styled(
                                c.ch.to_string(),
                                Style::default()
                                    .fg(Color::Rgb(rgb.0, rgb.1, rgb.2))
                                    .bg(palette.bg),
                            )
                        }
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

fn pulse_factor(glyph_idx: usize, eyes: [Option<usize>; 2], t: f32, amp: f32) -> f32 {
    use std::f32::consts::TAU;
    if amp == 0.0 {
        return 1.0;
    }
    if eyes[0] == Some(glyph_idx) {
        1.0 + amp * (t * TAU / 2.3).sin()
    } else if eyes[1] == Some(glyph_idx) {
        1.0 + amp * (t * TAU / 2.9 + 1.0).sin()
    } else {
        1.0
    }
}

fn color_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

fn tri_gradient(a: (u8, u8, u8), b: (u8, u8, u8), c: (u8, u8, u8), phase: f32) -> (u8, u8, u8) {
    let p = phase.rem_euclid(1.0);
    let tri = if p < 0.5 { p * 2.0 } else { 2.0 - p * 2.0 };
    if tri < 0.5 {
        lerp_rgb(a, b, tri * 2.0)
    } else {
        lerp_rgb(b, c, (tri - 0.5) * 2.0)
    }
}

impl Widget for Splash<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let palette = self.palette;
        let banner_w = self.banner.width();
        let banner_h = self.banner.height();
        let tag_w = self.tagline.chars().count() as u16;
        let total_h = banner_h + if tag_w > 0 { 2 } else { 0 };
        let vpad = area.height.saturating_sub(total_h) / 2;
        let hpad = area.width.saturating_sub(banner_w) / 2;

        let banner_area = Rect {
            x: area.x + hpad,
            y: area.y + vpad,
            width: banner_w.min(area.width.saturating_sub(hpad)),
            height: banner_h.min(area.height.saturating_sub(vpad)),
        };
        let lines = self.banner_lines();
        Paragraph::new(lines).render(banner_area, buf);

        if tag_w > 0 {
            let tag_y = banner_area.y + banner_area.height + 1;
            if tag_y < area.y + area.height {
                let tag_hpad = area.width.saturating_sub(tag_w) / 2;
                let tag_area = Rect {
                    x: area.x + tag_hpad,
                    y: tag_y,
                    width: tag_w.min(area.width.saturating_sub(tag_hpad)),
                    height: 1,
                };
                let tag = Paragraph::new(Line::from(Span::styled(
                    self.tagline.to_string(),
                    Style::default().fg(palette.dim).bg(palette.bg),
                )));
                tag.render(tag_area, buf);
            }
        }
    }
}
