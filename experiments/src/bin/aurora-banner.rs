use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use figlet_rs::FIGlet;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::Style as TuiStyle;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

use experiments::{color_rgb, rgb, scale_rgb, Theme, Xorshift, GLITCH_CHARS};

const TEXT: &str = "ono";
const TAGLINE: &str = "beautiful terminal UI components";
const FRAME_TARGET: Duration = Duration::from_millis(33);
const FLICKER_DIM: f32 = 0.6;

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    glyph_idx: usize,
}

struct Banner {
    grid: Vec<Vec<Option<Cell>>>,
    width: usize,
    height: usize,
    eye_glyphs: [Option<usize>; 2],
}

impl Banner {
    fn from_text(text: &str, font: &FIGlet) -> Self {
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
}

fn trim_blank_rows(mut lines: Vec<String>) -> Vec<String> {
    while lines.last().map_or(false, |l| l.trim().is_empty()) {
        lines.pop();
    }
    while lines.first().map_or(false, |l| l.trim().is_empty()) {
        lines.remove(0);
    }
    lines
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

struct SporadicEvent {
    next_at: Instant,
    active_until: Option<Instant>,
    rng: Xorshift,
    interval_min: f32,
    interval_max: f32,
    duration: Duration,
}

impl SporadicEvent {
    fn new(now: Instant, interval_min: f32, interval_max: f32, duration_ms: u64) -> Self {
        let mut s = Self {
            next_at: now,
            active_until: None,
            rng: Xorshift::new(0x9E3779B9),
            interval_min,
            interval_max,
            duration: Duration::from_millis(duration_ms),
        };
        s.schedule_next(now);
        s
    }

    fn schedule_next(&mut self, now: Instant) {
        let secs = self.interval_min + self.rng.unit() * (self.interval_max - self.interval_min);
        self.next_at = now + Duration::from_secs_f32(secs);
    }

    fn active(&mut self, now: Instant) -> bool {
        if let Some(end) = self.active_until {
            if now < end {
                return true;
            }
            self.active_until = None;
        }
        if now >= self.next_at {
            self.active_until = Some(now + self.duration);
            self.schedule_next(now);
            return true;
        }
        false
    }
}

fn banner_lines(
    banner: &Banner,
    style: Theme,
    t: f32,
    flicker_active: bool,
    glitch_active: bool,
    glitch_rng: &mut Xorshift,
) -> Vec<Line<'static>> {
    let knobs = style.knobs();
    let palette = style.palette();
    let width = banner.width.max(1) as f32;
    let scanline_row = if knobs.scanline && banner.height > 0 {
        Some(((t * knobs.scanline_speed_rows_per_sec) as usize) % banner.height)
    } else {
        None
    };

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
                        let phase = x as f32 / width + t / knobs.gradient_period_secs;
                        let mut rgb_color = style.gradient(phase);

                        let mut brightness =
                            pulse_factor(c.glyph_idx, banner.eye_glyphs, t, knobs.pulse_amplitude);
                        if flicker_active && knobs.idle_flicker {
                            brightness *= FLICKER_DIM;
                        }
                        if scanline_row == Some(y) {
                            brightness *= knobs.scanline_boost;
                        }
                        rgb_color = scale_rgb(rgb_color, brightness);

                        let mut ch = c.ch;
                        if glitch_active && knobs.glitch && glitch_rng.unit() < 0.18 {
                            let gi = (glitch_rng.next() as usize) % GLITCH_CHARS.len();
                            ch = GLITCH_CHARS[gi];
                            rgb_color = color_rgb(palette.accent);
                        }

                        Span::styled(
                            ch.to_string(),
                            TuiStyle::default().fg(rgb(rgb_color)).bg(palette.bg),
                        )
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

fn main() -> io::Result<()> {
    let style = Theme::parse_from_args();
    let font = FIGlet::standard().expect("figlet standard font");
    let banner = Banner::from_text(TEXT, &font);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &banner, style);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, banner: &Banner, style: Theme) -> io::Result<()> {
    let start = Instant::now();
    let knobs = style.knobs();
    let palette = style.palette();

    let mut flicker = SporadicEvent::new(start, 8.0, 12.0, 40);
    let mut glitch = SporadicEvent::new(
        start,
        knobs.glitch_interval_min.max(0.1),
        knobs.glitch_interval_max.max(0.2),
        knobs.glitch_duration_ms.max(1),
    );
    let mut glitch_rng = Xorshift::new(0xC0FFEE);
    let mut last_frame = start;

    loop {
        let now = Instant::now();
        let t = now.duration_since(start).as_secs_f32();
        let flicker_active = knobs.idle_flicker && flicker.active(now);
        let glitch_active = knobs.glitch && glitch.active(now);

        let lines = banner_lines(banner, style, t, flicker_active, glitch_active, &mut glitch_rng);

        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(Block::default().style(TuiStyle::default().bg(palette.bg)), area);

            let banner_w = banner.width as u16;
            let banner_h = banner.height as u16;
            let tag_w = TAGLINE.chars().count() as u16;

            let total_h = banner_h + 2;
            let vpad = area.height.saturating_sub(total_h) / 2;
            let hpad = area.width.saturating_sub(banner_w) / 2;

            let banner_area = Rect {
                x: area.x + hpad,
                y: area.y + vpad,
                width: banner_w.min(area.width.saturating_sub(hpad)),
                height: banner_h.min(area.height.saturating_sub(vpad)),
            };
            f.render_widget(Paragraph::new(lines), banner_area);

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
                    TAGLINE,
                    TuiStyle::default().fg(palette.dim).bg(palette.bg),
                )));
                f.render_widget(tag, tag_area);
            }

            let hint = format!(" · {} · ", style.name());
            let hint_w = hint.chars().count() as u16;
            let hint_area = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1),
                width: hint_w.min(area.width),
                height: 1,
            };
            let hint_para = Paragraph::new(Line::from(Span::styled(
                hint,
                TuiStyle::default().fg(palette.dim).bg(palette.bg),
            )));
            f.render_widget(hint_para, hint_area);
        })?;

        let remaining = FRAME_TARGET.saturating_sub(last_frame.elapsed());
        if event::poll(remaining)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && is_quit(key.code, key.modifiers) {
                    return Ok(());
                }
            }
        }
        last_frame = Instant::now();
    }
}

fn is_quit(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(
        (code, modifiers),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL)
    ) || matches!(code, KeyCode::Char('q') | KeyCode::Esc)
}
