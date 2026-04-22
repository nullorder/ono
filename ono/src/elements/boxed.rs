//! Element: box — bordered panel.
//!
//! Thin wrapper over `ratatui::widgets::Block` that sources its border and
//! title colors from a palette, so swapping themes requires no edits here.
//!
//! (`boxed` because `box` is a reserved keyword. The spec and CLI name
//! remain "box".)

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Widget};

use super::super::theme::Palette;

/// Bordered panel with an optional title.
///
/// ```no_run
/// use ono::elements::boxed::BoxFrame;
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
/// # let area = buf.area;
///
/// let palette = Theme::Forest.palette();
/// BoxFrame::new(palette).title("metrics").render(area, &mut buf);
/// ```
pub struct BoxFrame<'a> {
    title: Option<&'a str>,
    borders: Borders,
    palette: &'a Palette,
}

impl<'a> BoxFrame<'a> {
    /// Construct a fully-bordered box with no title.
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            title: None,
            borders: Borders::ALL,
            palette,
        }
    }

    /// Set the title rendered into the top border.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Restrict which sides render a border (default `Borders::ALL`).
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }
}

impl Widget for BoxFrame<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::default()
            .borders(self.borders)
            .border_style(Style::default().fg(self.palette.border));
        if let Some(title) = self.title {
            block = block.title(
                Line::from(title).style(Style::default().fg(self.palette.primary)),
            );
        }
        block.render(area, buf);
    }
}
