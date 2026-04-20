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

pub struct BoxFrame<'a> {
    title: Option<&'a str>,
    borders: Borders,
    palette: &'a Palette,
}

impl<'a> BoxFrame<'a> {
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            title: None,
            borders: Borders::ALL,
            palette,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

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
