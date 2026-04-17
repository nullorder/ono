use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Map, MapResolution, Points};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Terminal;

use experiments::{color_rgb, rgb, scale_rgb, Theme, Xorshift};

const FRAME_TARGET: Duration = Duration::from_millis(33);
const X_BOUNDS: [f64; 2] = [-180.0, 180.0];
const Y_BOUNDS: [f64; 2] = [-65.0, 80.0];
const TRACE_DURATION_MS: (u64, u64) = (2400, 4200);
const TRACE_SPAWN_INTERVAL_MS: (u64, u64) = (280, 720);
const MAX_ACTIVE_TRACES: usize = 10;

struct City {
    code: &'static str,
    lon: f64,
    lat: f64,
    phase: f32,
}

const CITIES: &[(&str, f64, f64)] = &[
    ("NYC", -74.0, 40.7),
    ("LAX", -118.2, 34.1),
    ("SAO", -46.6, -23.5),
    ("LDN", -0.1, 51.5),
    ("PAR", 2.4, 48.9),
    ("FRA", 8.7, 50.1),
    ("LOS", 3.4, 6.5),
    ("NBO", 36.8, -1.3),
    ("DXB", 55.3, 25.2),
    ("BLR", 77.6, 13.0),
    ("SIN", 103.8, 1.4),
    ("TYO", 139.7, 35.7),
    ("SEL", 127.0, 37.6),
    ("SYD", 151.2, -33.9),
];

const ROUTES: &[(usize, usize)] = &[
    (0, 3),
    (3, 5),
    (1, 11),
    (10, 13),
    (0, 2),
    (9, 3),
    (12, 11),
    (4, 11),
    (7, 8),
    (1, 12),
    (0, 11),
    (8, 10),
    (3, 9),
];

#[derive(Clone, Copy)]
struct Trace {
    route_idx: usize,
    start: Instant,
    duration: Duration,
}

struct State {
    cities: Vec<City>,
    traces: Vec<Trace>,
    next_spawn: Instant,
    rng: Xorshift,
    total_packets: u64,
}

impl State {
    fn new(now: Instant) -> Self {
        let cities = CITIES
            .iter()
            .enumerate()
            .map(|(i, (code, lon, lat))| City {
                code,
                lon: *lon,
                lat: *lat,
                phase: i as f32 * 0.73,
            })
            .collect();
        Self {
            cities,
            traces: Vec::with_capacity(MAX_ACTIVE_TRACES),
            next_spawn: now,
            rng: Xorshift::new(0xACE1),
            total_packets: 0,
        }
    }

    fn tick(&mut self, now: Instant) {
        self.traces.retain(|t| now.duration_since(t.start) < t.duration);
        if now >= self.next_spawn && self.traces.len() < MAX_ACTIVE_TRACES {
            let route_idx = (self.rng.next() as usize) % ROUTES.len();
            let duration_ms = TRACE_DURATION_MS.0
                + (self.rng.next() as u64) % (TRACE_DURATION_MS.1 - TRACE_DURATION_MS.0);
            self.traces.push(Trace {
                route_idx,
                start: now,
                duration: Duration::from_millis(duration_ms),
            });
            self.total_packets += 1;
            let gap = TRACE_SPAWN_INTERVAL_MS.0
                + (self.rng.next() as u64) % (TRACE_SPAWN_INTERVAL_MS.1 - TRACE_SPAWN_INTERVAL_MS.0);
            self.next_spawn = now + Duration::from_millis(gap);
        }
    }
}

fn trace_position(trace: &Trace, now: Instant) -> (f64, f64, f64) {
    let (src, dst) = ROUTES[trace.route_idx];
    let src = &CITIES[src];
    let dst = &CITIES[dst];
    let elapsed = now.duration_since(trace.start).as_secs_f64();
    let total = trace.duration.as_secs_f64();
    let t = (elapsed / total).clamp(0.0, 1.0);
    let eased = 0.5 - 0.5 * (std::f64::consts::PI * t).cos();
    let lon = src.1 + (dst.1 - src.1) * eased;
    let lat_mid = (src.2 + dst.2) / 2.0 + (src.2 - dst.2).abs() * 0.35;
    let a = 1.0 - t;
    let b = t;
    let bulge = 4.0 * t * (1.0 - t);
    let lat = src.2 * a + dst.2 * b + (lat_mid - (src.2 + dst.2) / 2.0) * bulge;
    (lon, lat, t)
}

fn main() -> io::Result<()> {
    let theme = Theme::parse_from_args();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, theme);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, theme: Theme) -> io::Result<()> {
    let start = Instant::now();
    let mut state = State::new(start);
    let mut last_frame = start;
    let palette = theme.palette();

    let outer_border = match theme {
        #[cfg(feature = "theme-cyber")]
        Theme::Cyber => BorderType::Double,
        #[cfg(feature = "theme-minimal")]
        Theme::Minimal => BorderType::Plain,
        Theme::Retro => BorderType::Rounded,
    };

    loop {
        let now = Instant::now();
        state.tick(now);
        let t = now.duration_since(start).as_secs_f32();

        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(Block::default().style(TuiStyle::default().bg(palette.bg)), area);

            let outer = Block::default()
                .borders(Borders::ALL)
                .border_type(outer_border)
                .border_style(TuiStyle::default().fg(palette.primary).bg(palette.bg))
                .title(Line::from(vec![
                    Span::styled(
                        " ono ",
                        TuiStyle::default()
                            .fg(palette.bright)
                            .bg(palette.bg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("· global traffic · {} ", theme.name()),
                        TuiStyle::default().fg(palette.dim).bg(palette.bg),
                    ),
                ]))
                .title_bottom(
                    Line::from(vec![Span::styled(
                        " q to quit ",
                        TuiStyle::default().fg(palette.dim).bg(palette.bg),
                    )])
                    .alignment(Alignment::Left),
                )
                .style(TuiStyle::default().bg(palette.bg));
            let inner = outer.inner(area);
            f.render_widget(outer, area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(inner);

            let city_points: Vec<(f64, f64)> = state
                .cities
                .iter()
                .map(|c| (c.lon, c.lat))
                .collect();

            let trace_dots: Vec<(f64, f64, f64)> = state
                .traces
                .iter()
                .map(|tr| trace_position(tr, now))
                .collect();

            let canvas = Canvas::default()
                .marker(Marker::Braille)
                .x_bounds(X_BOUNDS)
                .y_bounds(Y_BOUNDS)
                .background_color(palette.bg)
                .paint(|ctx| {
                    ctx.draw(&Map {
                        resolution: MapResolution::High,
                        color: palette.border,
                    });

                    for &(src, dst) in ROUTES {
                        ctx.draw(&CanvasLine {
                            x1: CITIES[src].1,
                            y1: CITIES[src].2,
                            x2: CITIES[dst].1,
                            y2: CITIES[dst].2,
                            color: palette.dim,
                        });
                    }

                    ctx.draw(&Points {
                        coords: &city_points,
                        color: palette.primary,
                    });

                    let bright_rgb = color_rgb(palette.bright);
                    for (city, offset) in state.cities.iter().zip(0u32..) {
                        let amp = (t * 1.4 + city.phase).sin() * 0.35 + 0.65;
                        let scaled = scale_rgb(bright_rgb, amp);
                        let _ = offset;
                        ctx.draw(&Points {
                            coords: &[(city.lon, city.lat)],
                            color: rgb(scaled),
                        });
                    }

                    for (lon, lat, frac) in &trace_dots {
                        let intensity = 1.0 - (frac - 0.5).abs() as f32 * 1.6;
                        let scaled = scale_rgb(color_rgb(palette.accent), intensity.max(0.35));
                        ctx.draw(&Points {
                            coords: &[(*lon, *lat)],
                            color: rgb(scaled),
                        });
                    }

                    for city in &state.cities {
                        let (dx, dy) = match city.code {
                            "PAR" => (-12.0, -4.0),
                            "FRA" => (3.0, 3.0),
                            "LDN" => (-12.0, 3.0),
                            "NYC" => (-14.0, -3.0),
                            "LAX" => (-14.0, -3.0),
                            "SEL" => (-12.0, 4.0),
                            "TYO" => (3.0, -4.0),
                            "DXB" => (3.0, 4.0),
                            "BLR" => (3.0, -4.0),
                            _ => (3.0, -3.0),
                        };
                        ctx.print(
                            city.lon + dx,
                            city.lat + dy,
                            Line::from(Span::styled(
                                city.code,
                                TuiStyle::default().fg(palette.dim),
                            )),
                        );
                    }
                });
            f.render_widget(canvas, rows[0]);

            let active = state.traces.len();
            let packets_per_sec = state.total_packets as f64 / t.max(1.0) as f64;
            let avg_hop_ms = 42.0 + (t * 0.6).sin() * 12.0;

            let stats = Paragraph::new(Line::from(vec![
                Span::styled("  active routes  ", TuiStyle::default().fg(palette.dim).bg(palette.bg)),
                Span::styled(
                    format!("{:<4}", active),
                    TuiStyle::default()
                        .fg(palette.bright)
                        .bg(palette.bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("   throughput  ", TuiStyle::default().fg(palette.dim).bg(palette.bg)),
                Span::styled(
                    format!("{:>5.1} pkt/s", packets_per_sec),
                    TuiStyle::default()
                        .fg(palette.bright)
                        .bg(palette.bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("   avg hop  ", TuiStyle::default().fg(palette.dim).bg(palette.bg)),
                Span::styled(
                    format!("{:>4.0}ms", avg_hop_ms),
                    TuiStyle::default()
                        .fg(palette.primary)
                        .bg(palette.bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("   total sent  {:<6}", state.total_packets),
                    TuiStyle::default().fg(palette.dim).bg(palette.bg),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(TuiStyle::default().fg(palette.border).bg(palette.bg))
                    .style(TuiStyle::default().bg(palette.bg)),
            );
            f.render_widget(stats, rows[1]);
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
