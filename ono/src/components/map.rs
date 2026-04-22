//! Component: map — animated global traffic map.
//!
//! Ratatui canvas with a world map, a fixed set of cities, routes drawn
//! between them, and small traces that ride along each route. State holds
//! the live traces and spawn scheduler; caller advances with `tick(now)`.

use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Map, MapResolution, Points};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

use super::super::theme::{scale_rgb, Palette, Xorshift};

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

/// Mutable animation state for [`WorldMap`]. Call [`MapState::tick`] each
/// frame to spawn new packet traces and drop finished ones.
pub struct MapState {
    cities: Vec<City>,
    traces: Vec<Trace>,
    next_spawn: Instant,
    rng: Xorshift,
    clock_base: Instant,
    total_packets: u64,
}

impl MapState {
    /// Initialize state with the current frame's `Instant` as the clock base.
    pub fn new(now: Instant) -> Self {
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
            clock_base: now,
            total_packets: 0,
        }
    }

    /// Advance the simulation to `now`: retire finished traces, maybe spawn new
    /// ones. Call once per frame before rendering.
    pub fn tick(&mut self, now: Instant) {
        self.traces
            .retain(|t| now.duration_since(t.start) < t.duration);
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
                + (self.rng.next() as u64)
                    % (TRACE_SPAWN_INTERVAL_MS.1 - TRACE_SPAWN_INTERVAL_MS.0);
            self.next_spawn = now + Duration::from_millis(gap);
        }
    }

    fn elapsed(&self, now: Instant) -> Duration {
        now.duration_since(self.clock_base)
    }
}

fn trace_position(trace: &Trace, now: Instant) -> (f64, f64, f64) {
    let (src_idx, dst_idx) = ROUTES[trace.route_idx];
    let src = &CITIES[src_idx];
    let dst = &CITIES[dst_idx];
    let elapsed = now.duration_since(trace.start).as_secs_f64();
    let total = trace.duration.as_secs_f64().max(1e-6);
    let t = (elapsed / total).clamp(0.0, 1.0);
    let bulge = 4.0 * t * (1.0 - t);
    let lon = src.1 + (dst.1 - src.1) * t;
    let lat_mid = (src.2 + dst.2) / 2.0 + (src.2 - dst.2).abs() * 0.35;
    let lat =
        src.2 * (1.0 - t) + dst.2 * t + (lat_mid - (src.2 + dst.2) / 2.0) * bulge;
    (lon, lat, t)
}

fn color_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

fn rgb(c: (u8, u8, u8)) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

/// Stylized world map with animated packet traces between cities.
///
/// The state is owned by a [`MapState`] that the caller ticks each frame.
///
/// ```no_run
/// use std::time::Instant;
/// use ono::components::map::{MapState, WorldMap};
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
/// # let area = buf.area;
///
/// let palette = Theme::Forest.palette();
/// let now = Instant::now();
/// let mut state = MapState::new(now);
/// state.tick(now);
///
/// WorldMap::new(&state, now, palette)
///     .title("my-app")
///     .subtitle("traffic")
///     .render(area, &mut buf);
/// ```
pub struct WorldMap<'a> {
    state: &'a MapState,
    now: Instant,
    title: &'a str,
    subtitle: &'a str,
    border_type: BorderType,
    palette: &'a Palette,
}

impl<'a> WorldMap<'a> {
    /// Construct a world map view. `now` is the current frame's timestamp —
    /// traces are positioned relative to it.
    pub fn new(state: &'a MapState, now: Instant, palette: &'a Palette) -> Self {
        Self {
            state,
            now,
            title: "ono",
            subtitle: "global traffic",
            border_type: BorderType::Rounded,
            palette,
        }
    }

    /// Title rendered in the top-left of the bordered box.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Subtitle rendered after `·` next to the title.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = subtitle;
        self
    }

    /// Override the border style (default `BorderType::Rounded`).
    pub fn border_type(mut self, bt: BorderType) -> Self {
        self.border_type = bt;
        self
    }
}

impl Widget for WorldMap<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let palette = self.palette;
        let state = self.state;
        let now = self.now;
        let t = state.elapsed(now).as_secs_f32();

        Block::default()
            .style(Style::default().bg(palette.bg))
            .render(area, buf);

        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_type)
            .border_style(Style::default().fg(palette.primary).bg(palette.bg))
            .title(Line::from(vec![
                Span::styled(
                    format!(" {} ", self.title),
                    Style::default()
                        .fg(palette.bright)
                        .bg(palette.bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("· {} ", self.subtitle),
                    Style::default().fg(palette.dim).bg(palette.bg),
                ),
            ]))
            .title_bottom(
                Line::from(vec![Span::styled(
                    " q to quit ",
                    Style::default().fg(palette.dim).bg(palette.bg),
                )])
                .alignment(Alignment::Left),
            )
            .style(Style::default().bg(palette.bg));
        let inner = outer.inner(area);
        outer.render(area, buf);

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

        let cities = &state.cities;
        let bright_rgb = color_rgb(palette.bright);
        let accent_rgb = color_rgb(palette.accent);

        let canvas = Canvas::default()
            .marker(Marker::Braille)
            .x_bounds(X_BOUNDS)
            .y_bounds(Y_BOUNDS)
            .background_color(palette.bg)
            .paint(move |ctx| {
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

                for city in cities {
                    let amp = (t * 1.4 + city.phase).sin() * 0.35 + 0.65;
                    let scaled = scale_rgb(bright_rgb, amp);
                    ctx.draw(&Points {
                        coords: &[(city.lon, city.lat)],
                        color: rgb(scaled),
                    });
                }

                for (lon, lat, frac) in &trace_dots {
                    let intensity = 1.0 - (frac - 0.5).abs() as f32 * 1.6;
                    let scaled = scale_rgb(accent_rgb, intensity.max(0.35));
                    ctx.draw(&Points {
                        coords: &[(*lon, *lat)],
                        color: rgb(scaled),
                    });
                }

                for city in cities {
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
                            city.code.to_string(),
                            Style::default().fg(palette.dim),
                        )),
                    );
                }
            });
        canvas.render(rows[0], buf);

        let active = state.traces.len();
        let packets_per_sec = state.total_packets as f64 / t.max(1.0) as f64;
        let avg_hop_ms = 42.0 + (t * 0.6).sin() * 12.0;

        let stats = Paragraph::new(Line::from(vec![
            Span::styled(
                "  active routes  ",
                Style::default().fg(palette.dim).bg(palette.bg),
            ),
            Span::styled(
                format!("{:<4}", active),
                Style::default()
                    .fg(palette.bright)
                    .bg(palette.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   throughput  ",
                Style::default().fg(palette.dim).bg(palette.bg),
            ),
            Span::styled(
                format!("{:>5.1} pkt/s", packets_per_sec),
                Style::default()
                    .fg(palette.bright)
                    .bg(palette.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   avg hop  ",
                Style::default().fg(palette.dim).bg(palette.bg),
            ),
            Span::styled(
                format!("{:>4.0}ms", avg_hop_ms),
                Style::default()
                    .fg(palette.primary)
                    .bg(palette.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   total sent  {:<6}", state.total_packets),
                Style::default().fg(palette.dim).bg(palette.bg),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(palette.border).bg(palette.bg))
                .style(Style::default().bg(palette.bg)),
        );
        stats.render(rows[1], buf);
    }
}
