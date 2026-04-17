use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::Style as TuiStyle;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

use experiments::Theme;

const FRAME_TARGET: Duration = Duration::from_millis(33);
const IDLE_PAUSE_MS: u64 = 3200;
const HEADER: &str = "› booting ono v0.0.4";
const FOOTER: &str = "experiment · q to quit";

struct Step {
    label: &'static str,
    pending_ms: u64,
    outcome: Outcome,
}

#[derive(Clone, Copy)]
enum Outcome {
    Ok,
    Warn,
}

const STEPS: &[Step] = &[
    Step { label: "resolving package registry", pending_ms: 380, outcome: Outcome::Ok },
    Step { label: "fetching manifest · ono@0.0.4", pending_ms: 240, outcome: Outcome::Ok },
    Step { label: "auditing 14 components", pending_ms: 620, outcome: Outcome::Ok },
    Step { label: "compiling templates", pending_ms: 1180, outcome: Outcome::Ok },
    Step { label: "checking terminal capabilities", pending_ms: 420, outcome: Outcome::Warn },
    Step { label: "warming up renderer", pending_ms: 540, outcome: Outcome::Ok },
    Step { label: "rendering catalog · 14 components ready", pending_ms: 200, outcome: Outcome::Ok },
];

fn cycle_ms(reveal_ms_per_char: u64) -> u64 {
    let mut total: u64 = 600;
    for s in STEPS {
        total += s.label.chars().count() as u64 * reveal_ms_per_char;
        total += s.pending_ms;
    }
    total + IDLE_PAUSE_MS
}

struct FrameState {
    active_idx: usize,
    revealed_chars: usize,
    spinning: bool,
    idle: bool,
}

fn compute_state(elapsed: Duration, reveal_ms_per_char: u64) -> FrameState {
    let mut cursor_ms: u64 = 600;
    let t_ms = elapsed.as_millis() as u64;

    if t_ms < cursor_ms {
        return FrameState { active_idx: 0, revealed_chars: 0, spinning: false, idle: false };
    }

    for (i, step) in STEPS.iter().enumerate() {
        let label_len = step.label.chars().count();
        let reveal_ms = label_len as u64 * reveal_ms_per_char;
        let reveal_end = cursor_ms + reveal_ms;
        let pending_end = reveal_end + step.pending_ms;

        if t_ms < reveal_end {
            let into = t_ms - cursor_ms;
            let revealed = (into / reveal_ms_per_char.max(1)) as usize;
            return FrameState {
                active_idx: i,
                revealed_chars: revealed.min(label_len),
                spinning: false,
                idle: false,
            };
        }
        if t_ms < pending_end {
            return FrameState {
                active_idx: i,
                revealed_chars: label_len,
                spinning: true,
                idle: false,
            };
        }
        cursor_ms = pending_end;
    }

    FrameState { active_idx: STEPS.len(), revealed_chars: 0, spinning: false, idle: true }
}

fn char_boundary(s: &str, n_chars: usize) -> usize {
    s.char_indices().nth(n_chars).map(|(i, _)| i).unwrap_or(s.len())
}

fn render_lines(state: &FrameState, style: Theme, t: f32) -> Vec<Line<'static>> {
    let palette = style.palette();
    let knobs = style.knobs();
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(STEPS.len() + 4);

    lines.push(Line::from(Span::styled(
        HEADER,
        TuiStyle::default().fg(palette.bright).bg(palette.bg),
    )));
    lines.push(Line::from(Span::raw(" ")));

    let spinner_idx = ((t * 10.0) as usize) % knobs.spinner.len().max(1);
    let cursor_on = ((t * knobs.cursor_blink_hz * 2.0) as usize) % 2 == 0;

    for (i, step) in STEPS.iter().enumerate() {
        if i > state.active_idx {
            continue;
        }
        let done = i < state.active_idx;
        let prefix_color = if done {
            match step.outcome {
                Outcome::Ok => palette.bright,
                Outcome::Warn => palette.accent,
            }
        } else {
            palette.dim
        };
        let prefix_text = if done {
            match step.outcome {
                Outcome::Ok => "[ok]",
                Outcome::Warn => "[!!]",
            }
        } else {
            "[..]"
        };

        let label_style = if done {
            TuiStyle::default().fg(palette.primary).bg(palette.bg)
        } else {
            TuiStyle::default().fg(palette.primary).bg(palette.bg)
        };

        let label_text = if done {
            step.label.to_string()
        } else {
            step.label[..char_boundary(step.label, state.revealed_chars)].to_string()
        };

        let mut spans = vec![
            Span::styled(prefix_text, TuiStyle::default().fg(prefix_color).bg(palette.bg)),
            Span::raw("  "),
            Span::styled(label_text, label_style),
        ];

        if i == state.active_idx && state.spinning {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                knobs.spinner[spinner_idx].to_string(),
                TuiStyle::default().fg(palette.bright).bg(palette.bg),
            ));
        } else if i == state.active_idx && !state.spinning && cursor_on {
            spans.push(Span::styled(
                "▋",
                TuiStyle::default().fg(palette.bright).bg(palette.bg),
            ));
        }

        lines.push(Line::from(spans));
    }

    if state.idle {
        lines.push(Line::from(Span::raw(" ")));
        let prompt_cursor = if cursor_on { "▋" } else { " " };
        lines.push(Line::from(vec![
            Span::styled("›", TuiStyle::default().fg(palette.bright).bg(palette.bg)),
            Span::raw(" "),
            Span::styled(prompt_cursor, TuiStyle::default().fg(palette.bright).bg(palette.bg)),
        ]));
    }

    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(Span::styled(
        format!("{} · {}", FOOTER, style.name()),
        TuiStyle::default().fg(palette.dim).bg(palette.bg),
    )));

    lines
}

fn main() -> io::Result<()> {
    let style = Theme::parse_from_args();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, style);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, style: Theme) -> io::Result<()> {
    let start = Instant::now();
    let knobs = style.knobs();
    let palette = style.palette();
    let cycle = Duration::from_millis(cycle_ms(knobs.reveal_ms_per_char));
    let mut last_frame = start;

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start);
        let looped_ms = (elapsed.as_millis() as u64) % (cycle.as_millis() as u64).max(1);
        let state = compute_state(Duration::from_millis(looped_ms), knobs.reveal_ms_per_char);
        let t = elapsed.as_secs_f32();
        let lines = render_lines(&state, style, t);

        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::default().style(TuiStyle::default().bg(palette.bg)),
                area,
            );

            let max_line_w = STEPS
                .iter()
                .map(|s| s.label.chars().count() + 10)
                .max()
                .unwrap_or(40) as u16;
            let block_w = max_line_w.max(HEADER.chars().count() as u16).min(area.width);
            let block_h = (lines.len() as u16).min(area.height);
            let hpad = area.width.saturating_sub(block_w) / 2;
            let vpad = area.height.saturating_sub(block_h) / 2;
            let inner = Rect {
                x: area.x + hpad,
                y: area.y + vpad,
                width: block_w,
                height: block_h,
            };
            f.render_widget(Paragraph::new(lines), inner);
        })?;

        let remaining = FRAME_TARGET.saturating_sub(last_frame.elapsed());
        if event::poll(remaining)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && is_quit(key.code, key.modifiers) {
                    return Ok(());
                }
            }
        }
        last_frame = Instant::now();
    }
}

fn is_quit(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(
        (code, modifiers),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL)
    ) || matches!(code, KeyCode::Char('q') | KeyCode::Esc)
}
