//! Element: percentage — inline percentage readout.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;

use super::super::theme::Palette;

pub struct Percentage<'a> {
    value: f32,
    decimals: u8,
    palette: &'a Palette,
}

impl<'a> Percentage<'a> {
    pub fn new(value: f32, palette: &'a Palette) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            decimals: 0,
            palette,
        }
    }

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
