//! Live preview runner for `ono preview <name>`.
//!
//! Sets up raw mode + alternate screen, runs a ~30 fps render loop for the
//! requested component until the user presses q / Esc / Ctrl+C, then
//! restores the terminal.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use figlet_rs::FIGlet;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget};
use ratatui::{Frame, Terminal};

use crate::components::boot::Boot;
use crate::components::dashboard::{Dashboard, DashboardState};
use crate::components::map::{MapState, WorldMap};
use crate::components::splash::{Banner, Splash};
use crate::components::statusbar::Statusbar;
use crate::elements::boxed::BoxFrame;
use crate::elements::percentage::Percentage;
use crate::elements::progress::{Progress, ProgressStyle};
use crate::elements::sparkline::Sparkline as SparklineElem;
use crate::elements::spinner::Spinner;
use crate::elements::typewriter::Typewriter;
use crate::theme::Theme;

const FRAME_TARGET: Duration = Duration::from_millis(33);

pub fn run(name: &str, theme: Theme) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = dispatch(&mut terminal, name, theme);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn dispatch(term: &mut Term, name: &str, theme: Theme) -> io::Result<()> {
    match name {
        "box" => run_loop(term, theme, |f, _| draw_box(f, theme)),
        "progress" => run_loop(term, theme, |f, elapsed| draw_progress(f, theme, elapsed)),
        "spinner" => run_loop(term, theme, |f, elapsed| draw_spinner(f, theme, elapsed)),
        "percentage" => run_loop(term, theme, |f, elapsed| draw_percentage(f, theme, elapsed)),
        "sparkline" => run_loop_stateful(term, theme, SparkState::new(), |f, theme, elapsed, st| {
            st.push(elapsed);
            draw_sparkline(f, theme, st);
        }),
        "typewriter" => run_loop(term, theme, |f, elapsed| draw_typewriter(f, theme, elapsed)),
        "statusbar" => run_loop(term, theme, |f, elapsed| draw_statusbar(f, theme, elapsed)),
        "boot" => run_loop(term, theme, |f, elapsed| draw_boot(f, theme, elapsed)),
        "splash" => {
            let font = FIGlet::standard().expect("figlet standard font");
            let banner = Banner::from_text("ono", &font);
            run_loop(term, theme, move |f, elapsed| {
                draw_splash(f, theme, &banner, elapsed);
            })
        }
        "dashboard" => {
            let start = Instant::now();
            let state = DashboardState::new(start);
            run_loop_stateful(term, theme, state, |f, theme, _elapsed, st| {
                let now = Instant::now();
                st.tick(now);
                draw_dashboard(f, theme, st, now);
            })
        }
        "map" => {
            let start = Instant::now();
            let state = MapState::new(start);
            run_loop_stateful(term, theme, state, |f, theme, _elapsed, st| {
                let now = Instant::now();
                st.tick(now);
                draw_map(f, theme, st, now);
            })
        }
        other => {
            eprintln!("no preview implementation for `{other}`");
            Ok(())
        }
    }
}

fn run_loop<F>(term: &mut Term, theme: Theme, mut draw: F) -> io::Result<()>
where
    F: FnMut(&mut Frame, Duration),
{
    let palette = theme.palette();
    let start = Instant::now();
    let mut last_frame = start;
    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start);
        term.draw(|f| {
            let area = f.area();
            Block::default()
                .style(Style::default().bg(palette.bg))
                .render(area, f.buffer_mut());
            draw(f, elapsed);
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

fn run_loop_stateful<S, F>(
    term: &mut Term,
    theme: Theme,
    mut state: S,
    mut draw: F,
) -> io::Result<()>
where
    F: FnMut(&mut Frame, Theme, Duration, &mut S),
{
    let palette = theme.palette();
    let start = Instant::now();
    let mut last_frame = start;
    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start);
        term.draw(|f| {
            let area = f.area();
            Block::default()
                .style(Style::default().bg(palette.bg))
                .render(area, f.buffer_mut());
            draw(f, theme, elapsed, &mut state);
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

fn centered_line(area: Rect, width: u16) -> Rect {
    let w = width.min(area.width);
    let h = 1.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + area.height.saturating_sub(1) / 2;
    Rect { x, y, width: w, height: h }
}

fn footer_hint(f: &mut Frame, theme: Theme) {
    let palette = theme.palette();
    let area = f.area();
    if area.height < 2 {
        return;
    }
    let text = format!(" q to quit · theme: {} ", theme.name());
    let w = (text.chars().count() as u16).min(area.width);
    let rect = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: w,
        height: 1,
    };
    Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(palette.dim).bg(palette.bg),
    )))
    .render(rect, f.buffer_mut());
}

fn draw_box(f: &mut Frame, theme: Theme) {
    let palette = theme.palette();
    let area = f.area();
    let inner = Rect {
        x: area.x + 4,
        y: area.y + 2,
        width: area.width.saturating_sub(8),
        height: area.height.saturating_sub(4),
    };
    BoxFrame::new(palette)
        .title(" box · bordered panel ")
        .render(inner, f.buffer_mut());

    let label = Line::from(Span::styled(
        "a thin Block wrapper — palette-sourced borders and title",
        Style::default().fg(palette.dim).bg(palette.bg),
    ));
    let content = Rect {
        x: inner.x + 2,
        y: inner.y + 1,
        width: inner.width.saturating_sub(4),
        height: 1,
    };
    Paragraph::new(label).render(content, f.buffer_mut());
    footer_hint(f, theme);
}

fn draw_progress(f: &mut Frame, theme: Theme, elapsed: Duration) {
    let palette = theme.palette();
    let t = elapsed.as_secs_f32();
    let pct = (((t * 0.25).sin() * 0.5 + 0.5).clamp(0.0, 1.0)) as f32;

    let width = 48u16;
    let bar_rect = centered_line(f.area(), width);

    let unicode_rect = Rect { y: bar_rect.y.saturating_sub(2), ..bar_rect };
    let ascii_rect = Rect { y: bar_rect.y + 2, ..bar_rect };

    Progress::new(pct, palette)
        .width(width)
        .label("unicode")
        .show_percent(true)
        .style(ProgressStyle::Unicode)
        .render(unicode_rect, f.buffer_mut());

    Progress::new(pct, palette)
        .width(width)
        .label("ascii  ")
        .show_percent(true)
        .style(ProgressStyle::Ascii)
        .render(ascii_rect, f.buffer_mut());

    footer_hint(f, theme);
}

fn draw_spinner(f: &mut Frame, theme: Theme, elapsed: Duration) {
    let palette = theme.palette();
    let knobs = theme.knobs();
    let tick = (elapsed.as_millis() / 100) as u64;
    let rect = centered_line(f.area(), 24);
    Spinner::new(palette)
        .frames(knobs.spinner)
        .tick(tick)
        .label("spinning up services")
        .render(rect, f.buffer_mut());
    footer_hint(f, theme);
}

fn draw_percentage(f: &mut Frame, theme: Theme, elapsed: Duration) {
    let palette = theme.palette();
    let t = elapsed.as_secs_f32();
    let v = ((t * 0.3).sin() * 0.5 + 0.5).clamp(0.0, 1.0);

    let rect = centered_line(f.area(), 12);
    let above = Rect { y: rect.y.saturating_sub(1), ..rect };
    Paragraph::new(Line::from(Span::styled(
        "percentage",
        Style::default().fg(palette.dim).bg(palette.bg),
    )))
    .render(above, f.buffer_mut());

    Percentage::new(v, palette)
        .decimals(1)
        .render(rect, f.buffer_mut());
    footer_hint(f, theme);
}

struct SparkState {
    values: Vec<f32>,
    last_push_ms: u128,
}

impl SparkState {
    fn new() -> Self {
        Self { values: Vec::with_capacity(200), last_push_ms: 0 }
    }

    fn push(&mut self, elapsed: Duration) {
        let now = elapsed.as_millis();
        if now.saturating_sub(self.last_push_ms) < 80 {
            return;
        }
        self.last_push_ms = now;
        let t = elapsed.as_secs_f32();
        let v = (t * 1.3).sin() * 0.5 + 0.5 + (t * 5.1).sin() * 0.1;
        if self.values.len() >= 200 {
            self.values.remove(0);
        }
        self.values.push(v);
    }
}

fn draw_sparkline(f: &mut Frame, theme: Theme, state: &SparkState) {
    let palette = theme.palette();
    let area = f.area();
    let width = 80u16.min(area.width.saturating_sub(4));
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height / 2,
        width,
        height: 1,
    };
    let label = Rect { y: rect.y.saturating_sub(1), ..rect };
    Paragraph::new(Line::from(Span::styled(
        "sparkline",
        Style::default().fg(palette.dim).bg(palette.bg),
    )))
    .render(label, f.buffer_mut());
    SparklineElem::new(&state.values, palette)
        .width(width)
        .render(rect, f.buffer_mut());
    footer_hint(f, theme);
}

fn draw_typewriter(f: &mut Frame, theme: Theme, elapsed: Duration) {
    let palette = theme.palette();
    let knobs = theme.knobs();
    let text = "beautiful terminal UI components";
    let total = text.chars().count() as u64;
    let reveal_ms = total * knobs.reveal_ms_per_char;
    let cycle = reveal_ms + 2500;
    let looped = (elapsed.as_millis() as u64) % cycle.max(1);
    let progress = if looped < reveal_ms {
        looped as f32 / reveal_ms.max(1) as f32
    } else {
        1.0
    };
    let blink_tick = (elapsed.as_secs_f32() * knobs.cursor_blink_hz * 2.0) as u64;

    let rect = centered_line(f.area(), text.chars().count() as u16 + 2);
    Typewriter::new(text, palette)
        .progress(progress)
        .cursor(true)
        .cursor_blink(blink_tick)
        .render(rect, f.buffer_mut());
    footer_hint(f, theme);
}

fn draw_statusbar(f: &mut Frame, theme: Theme, elapsed: Duration) {
    let palette = theme.palette();
    let knobs = theme.knobs();
    let t = elapsed.as_secs_f32();
    let pct = ((t * 0.15).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let tick = (elapsed.as_millis() / 100) as u64;
    let _ = knobs;

    let rect = centered_line(f.area(), 64);
    Statusbar::new(palette)
        .label("Working")
        .percent(pct)
        .width(64)
        .show_spinner(true)
        .show_progress(true)
        .show_percent(true)
        .tick(tick)
        .render(rect, f.buffer_mut());
    footer_hint(f, theme);
}

fn draw_boot(f: &mut Frame, theme: Theme, elapsed: Duration) {
    let palette = theme.palette();
    let knobs = theme.knobs();
    let area = f.area();
    let inner = Rect {
        x: area.x + 4,
        y: area.y + 2,
        width: area.width.saturating_sub(8),
        height: area.height.saturating_sub(4),
    };
    Boot::new(palette, knobs)
        .elapsed(elapsed)
        .render(inner, f.buffer_mut());
}

fn draw_splash(f: &mut Frame, theme: Theme, banner: &Banner, elapsed: Duration) {
    let palette = theme.palette();
    let knobs = theme.knobs();
    Splash::new(banner, palette, knobs)
        .elapsed(elapsed)
        .render(f.area(), f.buffer_mut());

    let area = f.area();
    if area.height >= 1 {
        let text = format!(" {} · q to quit ", theme.name());
        let w = (text.chars().count() as u16).min(area.width);
        let rect = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: w,
            height: 1,
        };
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(palette.dim).bg(palette.bg),
        )))
        .render(rect, f.buffer_mut());
    }
}

fn draw_dashboard(f: &mut Frame, theme: Theme, state: &DashboardState, now: Instant) {
    let palette = theme.palette();
    let knobs = theme.knobs();
    let border = outer_border(theme);
    Dashboard::new(state, now, palette, knobs)
        .subtitle(&format!("command center · {}", theme.name()))
        .border_type(border)
        .render(f.area(), f.buffer_mut());
}

fn draw_map(f: &mut Frame, theme: Theme, state: &MapState, now: Instant) {
    let palette = theme.palette();
    let border = outer_border(theme);
    WorldMap::new(state, now, palette)
        .subtitle(&format!("global traffic · {}", theme.name()))
        .border_type(border)
        .render(f.area(), f.buffer_mut());
}

fn outer_border(theme: Theme) -> BorderType {
    match theme {
        Theme::Forest => BorderType::Rounded,
        #[cfg(feature = "theme-retro")]
        Theme::Retro => BorderType::Rounded,
        #[cfg(feature = "theme-minimal")]
        Theme::Minimal => BorderType::Plain,
        #[cfg(feature = "theme-cyber")]
        Theme::Cyber => BorderType::Double,
    }
}

