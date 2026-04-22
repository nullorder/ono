//! Element: percentage — inline percentage readout.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;

use super::super::theme::Palette;

/// Inline percentage readout, rendered in the palette's `bright` color.
///
/// The input is clamped to `0.0..=1.0`.
///
/// ```no_run
/// use ono::elements::percentage::Percentage;
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 8, 1));
/// # let area = buf.area;
///
/// let palette = Theme::Forest.palette();
/// Percentage::new(0.72, palette).decimals(1).render(area, &mut buf); // "72.0%"
/// ```
pub struct Percentage<'a> {
    value: f32,
    decimals: u8,
    palette: &'a Palette,
}

impl<'a> Percentage<'a> {
    /// Construct a percentage readout. `value` is clamped to `0.0..=1.0`.
    pub fn new(value: f32, palette: &'a Palette) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            decimals: 0,
            palette,
        }
    }

    /// Number of decimal places (capped at 4). Defaults to 0.
    pub fn decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals.min(4);
        self
    }
}

impl Widget for Percentage<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let pct = self.value * 100.0;
        let text = format!("{pct:.*}%", self.decimals as usize);
        Line::from(text)
            .style(Style::default().fg(self.palette.bright))
            .render(area, buf);
    }
}
