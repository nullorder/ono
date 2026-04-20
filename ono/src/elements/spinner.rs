//! Element: spinner.
//!
//! Stateless: the caller advances `tick` each frame. This makes the widget
//! trivially composable (no interior mutability, no hidden timer) at the
//! cost of the caller owning the clock — which is the right tradeoff for
//! Ratatui-style render loops.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use super::super::theme::Palette;

pub const SPINNER_DOTS: &[char] = &['◐', '◓', '◑', '◒'];
pub const SPINNER_BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
pub const SPINNER_MINIMAL: &[char] = &['·', '◦', '○', '◦'];

pub struct Spinner<'a> {
    frames: &'a [char],
    tick: u64,
    label: Option<&'a str>,
    palette: &'a Palette,
}

impl<'a> Spinner<'a> {
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            frames: SPINNER_DOTS,
            tick: 0,
            label: None,
            palette,
        }
    }

    pub fn frames(mut self, frames: &'a [char]) -> Self {
        if !frames.is_empty() {
            self.frames = frames;
        }
        self
    }

    pub fn tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl Widget for Spinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let frame = self.frames[(self.tick as usize) % self.frames.len()];
        let mut spans = vec![Span::styled(
            frame.to_string(),
            Style::default().fg(self.palette.accent),
        )];
        if let Some(label) = self.label {
            spans.push(Span::styled(
                format!(" {label}"),
                Style::default().fg(self.palette.primary),
            ));
        }
        Line::from(spans).render(area, buf);
    }
}
