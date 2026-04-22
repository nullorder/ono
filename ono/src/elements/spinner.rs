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

const DEFAULT_FRAMES: &[char] = &['◐', '◓', '◑', '◒'];

/// Single-cell animated spinner with optional trailing label.
///
/// Stateless: advance `tick` each frame and re-render. The theme-canonical
/// frame set can be pulled from `Theme::knobs().spinner`.
///
/// ```no_run
/// use ono::elements::spinner::Spinner;
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
/// # let area = buf.area;
///
/// let theme = Theme::Forest;
/// let tick = 7; // caller-owned frame counter
/// Spinner::new(theme.palette())
///     .frames(theme.knobs().spinner)
///     .tick(tick)
///     .label("loading")
///     .render(area, &mut buf);
/// ```
pub struct Spinner<'a> {
    frames: &'a [char],
    tick: u64,
    label: Option<&'a str>,
    palette: &'a Palette,
}

impl<'a> Spinner<'a> {
    /// Construct a spinner with the default frame set.
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            frames: DEFAULT_FRAMES,
            tick: 0,
            label: None,
            palette,
        }
    }

    /// Override the frame set. Empty slices are ignored (default is kept).
    pub fn frames(mut self, frames: &'a [char]) -> Self {
        if !frames.is_empty() {
            self.frames = frames;
        }
        self
    }

    /// Current frame counter (caller-owned; typically an integer advanced each
    /// animation tick).
    pub fn tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    /// Label rendered one cell right of the spinner glyph.
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
