//! Element: sparkline — unicode block-glyph mini-chart.
//!
//! Inputs are raw values; normalization is done here so the caller can pass
//! a ring buffer of whatever they're measuring without thinking about scale.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use super::super::theme::Palette;

const LEVELS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Block-glyph mini-chart. The caller passes a slice of values; auto-scaling
/// to the min/max of the visible window happens here.
///
/// When more values are provided than cells are available, the trailing
/// window is shown (newest data on the right).
///
/// ```no_run
/// use ono::elements::sparkline::Sparkline;
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
/// # let area = buf.area;
///
/// let palette = Theme::Forest.palette();
/// let samples: Vec<f32> = (0..40).map(|i| (i as f32 * 0.3).sin()).collect();
/// Sparkline::new(&samples, palette).width(20).render(area, &mut buf);
/// ```
pub struct Sparkline<'a> {
    values: &'a [f32],
    width: Option<u16>,
    palette: &'a Palette,
}

impl<'a> Sparkline<'a> {
    /// Construct a sparkline. Values are auto-scaled at render time.
    pub fn new(values: &'a [f32], palette: &'a Palette) -> Self {
        Self {
            values,
            width: None,
            palette,
        }
    }

    /// Fix the render width in cells. Defaults to the available area width.
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width.max(1));
        self
    }
}

impl Widget for Sparkline<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 || self.values.is_empty() {
            return;
        }
        let cells = self.width.unwrap_or(area.width).min(area.width);
        if cells == 0 {
            return;
        }

        // Take the last `cells` values (newest on the right).
        let take = (cells as usize).min(self.values.len());
        let tail = &self.values[self.values.len() - take..];

        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for &v in tail {
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        if !lo.is_finite() {
            return;
        }
        let range = (hi - lo).max(f32::EPSILON);

        let mut out = String::with_capacity(cells as usize);
        // Leading padding if we have fewer values than cells.
        for _ in take..(cells as usize) {
            out.push(' ');
        }
        for &v in tail {
            let t = ((v - lo) / range).clamp(0.0, 1.0);
            let idx = (t * (LEVELS.len() - 1) as f32).round() as usize;
            out.push(LEVELS[idx]);
        }

        Line::from(Span::styled(
            out,
            Style::default().fg(self.palette.primary),
        ))
        .render(area, buf);
    }
}
