//! Demo crate — what a user's project looks like after running:
//!
//!   ono add splash
//!   ono add boot
//!   ono add dashboard
//!   ono add statusbar
//!
//! The `src/ono/` tree was generated verbatim by those commands. Nothing
//! here imports from the `ono` crate at runtime — every component lives in
//! your own tree and is yours to modify.

mod ono;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use figlet_rs::FIGlet;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};
use ratatui::{Frame, Terminal};

use ono::components::boot::Boot;
use ono::components::dashboard::{Dashboard, DashboardState};
use ono::components::splash::{Banner, Splash};
use ono::components::statusbar::Statusbar;
use ono::theme::{Knobs, Palette, DEFAULT_THEME};

const FRAME_TARGET: Duration = Duration::from_millis(33);
const SPLASH_MS: u64 = 3000;
const BOOT_MS: u64 = 7000;

enum Scene {
    Splash,
    Boot,
    Dashboard,
}

fn scene_at(elapsed: Duration) -> Scene {
    let ms = elapsed.as_millis() as u64;
    if ms < SPLASH_MS {
        Scene::Splash
    } else if ms < SPLASH_MS + BOOT_MS {
        Scene::Boot
    } else {
        Scene::Dashboard
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn run(term: &mut Term) -> io::Result<()> {
    let theme = DEFAULT_THEME;
    let palette = theme.palette();
    let knobs = theme.knobs();

    let font = FIGlet::standard().expect("figlet standard font");
    let banner = Banner::from_text("ono", &font);

    let start = Instant::now();
    let mut dash_state = DashboardState::new(start);

    loop {
        let frame_start = Instant::now();
        let elapsed = frame_start.duration_since(start);
        let boot_elapsed = elapsed.saturating_sub(Duration::from_millis(SPLASH_MS));
        dash_state.tick(frame_start);

        term.draw(|f| {
            let area = f.area();
            Block::default()
                .style(Style::default().bg(palette.bg))
                .render(area, f.buffer_mut());

            match scene_at(elapsed) {
                Scene::Splash => {
                    Splash::new(&banner, palette, knobs)
                        .elapsed(elapsed)
                        .render(area, f.buffer_mut());
                }
                Scene::Boot => {
                    Boot::new(palette, knobs)
                        .elapsed(boot_elapsed)
                        .render(area, f.buffer_mut());
                }
                Scene::Dashboard => {
                    render_dashboard(f, &dash_state, frame_start, palette, knobs, elapsed);
                }
            }
        })?;

        let deadline = frame_start + FRAME_TARGET;
        let remaining = deadline.saturating_duration_since(Instant::now());
        if event::poll(remaining)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && is_quit(key.code, key.modifiers) {
                    return Ok(());
                }
            }
        }
    }
}

fn render_dashboard(
    f: &mut Frame,
    state: &DashboardState,
    now: Instant,
    palette: &Palette,
    knobs: &Knobs,
    elapsed: Duration,
) {
    let area = f.area();
    if area.height < 3 {
        return;
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    Dashboard::new(state, now, palette, knobs)
        .subtitle("ratatui demo")
        .render(rows[0], f.buffer_mut());

    let t = elapsed.as_secs_f32();
    let pct = ((t * 0.15).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let tick = (elapsed.as_millis() / 100) as u64;
    Statusbar::new(palette)
        .label("rendering")
        .percent(pct)
        .width(rows[1].width)
        .show_spinner(true)
        .show_progress(true)
        .show_percent(true)
        .tick(tick)
        .render(rows[1], f.buffer_mut());
}

fn is_quit(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(
        (code, modifiers),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL)
    ) || matches!(code, KeyCode::Char('q') | KeyCode::Esc)
}
