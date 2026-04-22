//! Component: boot — animated boot log.
//!
//! A typewriter-revealed list of steps with a trailing spinner on the active
//! step. Pure function of elapsed time — the caller advances the clock each
//! frame. Idle-pause loops indefinitely when all steps finish.

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::super::theme::{Knobs, Palette};

/// Terminal marker for a completed [`Step`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    /// Green `[ok]` marker.
    Ok,
    /// Amber `[!!]` marker.
    Warn,
}

/// A single line in the boot log.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    /// Text revealed with the typewriter effect.
    pub label: &'static str,
    /// Milliseconds the spinner runs after reveal completes.
    pub pending_ms: u64,
    /// Outcome marker drawn once the step is finished.
    pub outcome: StepOutcome,
}

/// Default list of steps, used when no custom sequence is supplied.
pub const DEFAULT_STEPS: &[Step] = &[
    Step { label: "resolving package registry", pending_ms: 380, outcome: StepOutcome::Ok },
    Step { label: "fetching manifest · ono@0.0.4", pending_ms: 240, outcome: StepOutcome::Ok },
    Step { label: "auditing 14 components", pending_ms: 620, outcome: StepOutcome::Ok },
    Step { label: "compiling templates", pending_ms: 1180, outcome: StepOutcome::Ok },
    Step { label: "checking terminal capabilities", pending_ms: 420, outcome: StepOutcome::Warn },
    Step { label: "warming up renderer", pending_ms: 540, outcome: StepOutcome::Ok },
    Step { label: "rendering catalog · 14 components ready", pending_ms: 200, outcome: StepOutcome::Ok },
];

/// Animated boot log. Steps reveal with a typewriter effect, then a spinner
/// runs for `pending_ms`, then an outcome marker is drawn. After the final
/// step the whole sequence pauses for `idle_pause_ms` and loops.
///
/// ```no_run
/// use std::time::Duration;
/// use ono::components::boot::Boot;
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 60, 12));
/// # let area = buf.area;
///
/// let theme = Theme::Forest;
/// Boot::new(theme.palette(), theme.knobs())
///     .header("> booting my-app")
///     .elapsed(Duration::from_millis(2400))
///     .render(area, &mut buf);
/// ```
pub struct Boot<'a> {
    header: &'a str,
    footer: &'a str,
    steps: &'a [Step],
    elapsed: Duration,
    intro_ms: u64,
    idle_pause_ms: u64,
    palette: &'a Palette,
    knobs: &'a Knobs,
}

impl<'a> Boot<'a> {
    /// Construct a boot log with defaults. Use the builder to override before
    /// rendering; required state is just the `elapsed` clock.
    pub fn new(palette: &'a Palette, knobs: &'a Knobs) -> Self {
        Self {
            header: "› booting ono",
            footer: "q to quit",
            steps: DEFAULT_STEPS,
            elapsed: Duration::ZERO,
            intro_ms: 600,
            idle_pause_ms: 3200,
            palette,
            knobs,
        }
    }

    /// First line in the log (drawn before any step). Default `"› booting ono"`.
    pub fn header(mut self, header: &'a str) -> Self {
        self.header = header;
        self
    }

    /// Last line, drawn after all steps. Default `"q to quit"`.
    pub fn footer(mut self, footer: &'a str) -> Self {
        self.footer = footer;
        self
    }

    /// Override the default step sequence.
    pub fn steps(mut self, steps: &'a [Step]) -> Self {
        self.steps = steps;
        self
    }

    /// Monotonically-increasing clock driving the animation. Typically
    /// `Instant::now().duration_since(start)`.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = elapsed;
        self
    }

    /// Delay in milliseconds before the first step reveal begins (default 600).
    pub fn intro_ms(mut self, ms: u64) -> Self {
        self.intro_ms = ms;
        self
    }

    /// Delay in milliseconds after the last step before the sequence loops
    /// (default 3200).
    pub fn idle_pause_ms(mut self, ms: u64) -> Self {
        self.idle_pause_ms = ms;
        self
    }

    fn cycle_ms(&self) -> u64 {
        let reveal = self.knobs.reveal_ms_per_char.max(1);
        let mut total = self.intro_ms;
        for s in self.steps {
            total += s.label.chars().count() as u64 * reveal;
            total += s.pending_ms;
        }
        total + self.idle_pause_ms
    }

    fn state_at(&self, looped_ms: u64) -> FrameState {
        let reveal = self.knobs.reveal_ms_per_char.max(1);
        let mut cursor_ms = self.intro_ms;
        if looped_ms < cursor_ms {
            return FrameState { active_idx: 0, revealed_chars: 0, spinning: false, idle: false };
        }
        for (i, step) in self.steps.iter().enumerate() {
            let label_len = step.label.chars().count();
            let reveal_end = cursor_ms + label_len as u64 * reveal;
            let pending_end = reveal_end + step.pending_ms;
            if looped_ms < reveal_end {
                let into = looped_ms - cursor_ms;
                return FrameState {
                    active_idx: i,
                    revealed_chars: ((into / reveal) as usize).min(label_len),
                    spinning: false,
                    idle: false,
                };
            }
            if looped_ms < pending_end {
                return FrameState {
                    active_idx: i,
                    revealed_chars: label_len,
                    spinning: true,
                    idle: false,
                };
            }
            cursor_ms = pending_end;
        }
        FrameState { active_idx: self.steps.len(), revealed_chars: 0, spinning: false, idle: true }
    }

    fn lines(&self, state: &FrameState, t: f32) -> Vec<Line<'static>> {
        let palette = self.palette;
        let knobs = self.knobs;
        let mut lines: Vec<Line<'static>> = Vec::with_capacity(self.steps.len() + 4);

        lines.push(Line::from(Span::styled(
            self.header.to_string(),
            Style::default().fg(palette.bright).bg(palette.bg),
        )));
        lines.push(Line::from(Span::raw(" ")));

        let spinner_frames = if knobs.spinner.is_empty() { &['·'][..] } else { knobs.spinner };
        let spinner_idx = ((t * 10.0) as usize) % spinner_frames.len().max(1);
        let cursor_on = ((t * knobs.cursor_blink_hz * 2.0) as usize) % 2 == 0;

        for (i, step) in self.steps.iter().enumerate() {
            if i > state.active_idx {
                continue;
            }
            let done = i < state.active_idx;
            let prefix_color = if done {
                match step.outcome {
                    StepOutcome::Ok => palette.bright,
                    StepOutcome::Warn => palette.accent,
                }
            } else {
                palette.dim
            };
            let prefix_text = if done {
                match step.outcome {
                    StepOutcome::Ok => "[ok]",
                    StepOutcome::Warn => "[!!]",
                }
            } else {
                "[..]"
            };
            let label_text = if done {
                step.label.to_string()
            } else {
                let end = char_boundary(step.label, state.revealed_chars);
                step.label[..end].to_string()
            };

            let mut spans = vec![
                Span::styled(
                    prefix_text.to_string(),
                    Style::default().fg(prefix_color).bg(palette.bg),
                ),
                Span::raw("  "),
                Span::styled(
                    label_text,
                    Style::default().fg(palette.primary).bg(palette.bg),
                ),
            ];

            if i == state.active_idx && state.spinning {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    spinner_frames[spinner_idx].to_string(),
                    Style::default().fg(palette.bright).bg(palette.bg),
                ));
            } else if i == state.active_idx && !state.spinning && cursor_on {
                spans.push(Span::styled(
                    "▋".to_string(),
                    Style::default().fg(palette.bright).bg(palette.bg),
                ));
            }

            lines.push(Line::from(spans));
        }

        if state.idle {
            lines.push(Line::from(Span::raw(" ")));
            let cursor = if cursor_on { "▋" } else { " " };
            lines.push(Line::from(vec![
                Span::styled("›".to_string(), Style::default().fg(palette.bright).bg(palette.bg)),
                Span::raw(" "),
                Span::styled(cursor.to_string(), Style::default().fg(palette.bright).bg(palette.bg)),
            ]));
        }

        lines.push(Line::from(Span::raw(" ")));
        lines.push(Line::from(Span::styled(
            self.footer.to_string(),
            Style::default().fg(palette.dim).bg(palette.bg),
        )));

        lines
    }
}

struct FrameState {
    active_idx: usize,
    revealed_chars: usize,
    spinning: bool,
    idle: bool,
}

fn char_boundary(s: &str, n_chars: usize) -> usize {
    s.char_indices().nth(n_chars).map(|(i, _)| i).unwrap_or(s.len())
}

impl Widget for Boot<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let cycle = self.cycle_ms().max(1);
        let elapsed_ms = self.elapsed.as_millis() as u64;
        let looped = elapsed_ms % cycle;
        let state = self.state_at(looped);
        let t = self.elapsed.as_secs_f32();
        let lines = self.lines(&state, t);
        Paragraph::new(lines).render(area, buf);
    }
}
