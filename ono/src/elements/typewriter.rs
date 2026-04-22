//! Element: typewriter — progressive text reveal.
//!
//! Caller supplies the progress as a fraction (0.0..=1.0). A blinking cursor
//! can be appended while the reveal is in progress; the caller advances the
//! `tick` for its blink phase.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use super::super::theme::Palette;

/// Progressive text reveal, one scalar at a time, with an optional blinking
/// caret.
///
/// Stateless: the caller drives both the reveal progress and the caret blink.
///
/// ```no_run
/// use ono::elements::typewriter::Typewriter;
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
/// # let area = buf.area;
///
/// let palette = Theme::Forest.palette();
/// Typewriter::new("boot sequence complete", palette)
///     .progress(0.5)
///     .cursor(true)
///     .cursor_blink(3)
///     .render(area, &mut buf);
/// ```
pub struct Typewriter<'a> {
    text: &'a str,
    progress: f32,
    cursor: bool,
    cursor_blink_tick: u64,
    palette: &'a Palette,
}

impl<'a> Typewriter<'a> {
    /// Construct a typewriter. Defaults: fully revealed, caret on.
    pub fn new(text: &'a str, palette: &'a Palette) -> Self {
        Self {
            text,
            progress: 1.0,
            cursor: true,
            cursor_blink_tick: 0,
            palette,
        }
    }

    /// Fraction of the text to reveal (0.0..=1.0, clamped). Counted in Unicode
    /// scalars.
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Show or hide the trailing caret.
    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }

    /// Caller-owned tick for caret-blink phase (caret is shown when
    /// `tick % 2 == 0`, or always while the reveal is still in progress).
    pub fn cursor_blink(mut self, tick: u64) -> Self {
        self.cursor_blink_tick = tick;
        self
    }
}

impl Widget for Typewriter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let total = self.text.chars().count();
        let revealed_count = (self.progress * total as f32).round() as usize;
        let revealed: String = self.text.chars().take(revealed_count).collect();
        let done = revealed_count >= total;

        let mut spans = vec![Span::styled(
            revealed,
            Style::default().fg(self.palette.primary),
        )];
        if self.cursor && (!done || self.cursor_blink_tick % 2 == 0) {
            spans.push(Span::styled(
                "▌",
                Style::default().fg(self.palette.bright),
            ));
        }
        Line::from(spans).render(area, buf);
    }
}
