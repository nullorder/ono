//! Element: progress — horizontal progress bar.
//!
//! Matches `specs/elements/progress.toml`. Unicode style uses 8-subcell
//! block glyphs for smooth fills; ASCII style uses `[==  ]`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use super::super::theme::Palette;

/// Rendering style for [`Progress`]. Defaults to [`ProgressStyle::Unicode`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProgressStyle {
    /// Eighth-block glyphs for smooth sub-cell fills.
    Unicode,
    /// `[== ]` form for terminals without block-character support.
    Ascii,
}

/// Horizontal progress bar.
///
/// The input percent is clamped to `0.0..=1.0`.
///
/// ```no_run
/// use ono::elements::progress::{Progress, ProgressStyle};
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
/// # let area = buf.area;
///
/// let palette = Theme::Forest.palette();
/// Progress::new(0.42, palette)
///     .width(30)
///     .label("install")
///     .show_percent(true)
///     .style(ProgressStyle::Unicode)
///     .render(area, &mut buf);
/// ```
pub struct Progress<'a> {
    percent: f32,
    width: u16,
    label: Option<&'a str>,
    show_percent: bool,
    style: ProgressStyle,
    palette: &'a Palette,
}

impl<'a> Progress<'a> {
    /// Construct a progress bar. `percent` is clamped to `0.0..=1.0`.
    pub fn new(percent: f32, palette: &'a Palette) -> Self {
        Self {
            percent: percent.clamp(0.0, 1.0),
            width: 20,
            label: None,
            show_percent: false,
            style: ProgressStyle::Unicode,
            palette,
        }
    }

    /// Total width in cells, including the bar track (default 20).
    pub fn width(mut self, width: u16) -> Self {
        self.width = width.max(1);
        self
    }

    /// Label shown before the bar.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Append a right-aligned `  42%` readout after the bar.
    pub fn show_percent(mut self, show: bool) -> Self {
        self.show_percent = show;
        self
    }

    /// Choose Unicode block-glyph or ASCII rendering.
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    fn bar_spans(&self) -> Vec<Span<'static>> {
        let fill_style = Style::default().fg(self.palette.primary);
        let track_style = Style::default().fg(self.palette.dim);
        match self.style {
            ProgressStyle::Unicode => {
                let total_eighths = (self.width as f32) * 8.0;
                let filled_eighths = (self.percent * total_eighths).round() as u32;
                let full = (filled_eighths / 8) as u16;
                let remainder = (filled_eighths % 8) as usize;
                let partial = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'][remainder];

                let mut spans = vec![];
                if full > 0 {
                    spans.push(Span::styled("█".repeat(full as usize), fill_style));
                }
                if full < self.width {
                    if remainder > 0 {
                        spans.push(Span::styled(partial.to_string(), fill_style));
                        let empty_cells = self.width - full - 1;
                        if empty_cells > 0 {
                            spans.push(Span::styled(" ".repeat(empty_cells as usize), track_style));
                        }
                    } else {
                        let empty_cells = self.width - full;
                        spans.push(Span::styled(" ".repeat(empty_cells as usize), track_style));
                    }
                }
                spans
            }
            ProgressStyle::Ascii => {
                let inner_width = self.width.saturating_sub(2).max(1);
                let filled = ((self.percent * inner_width as f32).round() as u16).min(inner_width);
                let empty = inner_width - filled;
                vec![
                    Span::styled("[", track_style),
                    Span::styled("=".repeat(filled as usize), fill_style),
                    Span::styled(" ".repeat(empty as usize), track_style),
                    Span::styled("]", track_style),
                ]
            }
        }
    }
}

impl Widget for Progress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let mut spans: Vec<Span> = Vec::new();
        if let Some(label) = self.label {
            spans.push(Span::styled(
                format!("{label} "),
                Style::default().fg(self.palette.primary),
            ));
        }
        spans.extend(self.bar_spans());
        if self.show_percent {
            spans.push(Span::styled(
                format!(" {:>3}%", (self.percent * 100.0).round() as u32),
                Style::default().fg(self.palette.bright),
            ));
        }
        Line::from(spans).render(area, buf);
    }
}
