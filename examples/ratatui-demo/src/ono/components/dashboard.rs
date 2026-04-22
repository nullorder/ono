//! Component: dashboard — command-center style monitoring view.
//!
//! Stat tiles, a throughput sparkline, a service list, region load bars,
//! resource gauges, and an event log stacked in a bordered frame. State
//! evolves over time (event queue, service status flips); construct a
//! `DashboardState` once and call `tick(now)` each frame.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline, Widget};

use super::super::theme::{Knobs, Palette};

const SPARK_LEN: usize = 140;
const EVENT_INTERVAL: Duration = Duration::from_millis(1600);
const EVENT_BUFFER: usize = 6;
const REGION_BAR_STEPS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[derive(Clone, Copy)]
pub(crate) enum EventKind {
    Ok,
    Warn,
    Info,
}

struct EventEntry {
    stamp: String,
    text: String,
    kind: EventKind,
}

const EVENT_POOL: &[(&str, EventKind)] = &[
    ("deploy · api-gateway v2.4.1 → 8 pods ready", EventKind::Ok),
    ("spec loaded · splash", EventKind::Info),
    ("billing-svc latency p99 > 200ms threshold", EventKind::Warn),
    ("snapshot match · boot", EventKind::Ok),
    ("template cache warmed · 14 entries", EventKind::Info),
    ("scaler · worker-pool +2 replicas", EventKind::Ok),
    ("color tier fallback · 256 → 16", EventKind::Warn),
    ("deploy · auth-service v1.12.0 canary 10%", EventKind::Info),
    ("render cycle complete · 7.2ms avg", EventKind::Ok),
    ("subscriber connected · region:eu-west-1", EventKind::Info),
    ("alert resolved · billing-svc p99 normal", EventKind::Ok),
    ("gc pause · 42ms on worker-03", EventKind::Info),
];

#[derive(Clone, Copy, PartialEq)]
enum ServiceStatus {
    Healthy,
    Degraded,
}

struct Service {
    name: &'static str,
    base_latency_ms: f32,
    status: ServiceStatus,
    phase: f32,
    last_flip: Instant,
}

const SERVICE_DEFS: &[(&str, f32)] = &[
    ("api-gateway", 12.0),
    ("auth-service", 8.0),
    ("billing-svc", 180.0),
    ("worker-pool", 45.0),
    ("queue-consumer", 23.0),
    ("feature-store", 18.0),
];

struct Region {
    code: &'static str,
    phase: f32,
    frequency: f32,
}

const REGION_DEFS: &[(&str, f32)] = &[
    ("us", 0.41),
    ("eu", 0.63),
    ("as", 0.29),
    ("sa", 0.77),
    ("au", 0.52),
];

/// Mutable animation state for [`Dashboard`]. Call [`DashboardState::tick`]
/// each frame to advance the simulated metrics and event log.
pub struct DashboardState {
    spark: VecDeque<u64>,
    events: VecDeque<EventEntry>,
    services: Vec<Service>,
    regions: Vec<Region>,
    last_event_at: Instant,
    clock_base: Instant,
    next_event: usize,
}

impl DashboardState {
    /// Initialize state with `now` as the clock base.
    pub fn new(now: Instant) -> Self {
        let mut spark = VecDeque::with_capacity(SPARK_LEN);
        for _ in 0..SPARK_LEN {
            spark.push_back(0);
        }
        let services = SERVICE_DEFS
            .iter()
            .enumerate()
            .map(|(i, (name, base))| Service {
                name,
                base_latency_ms: *base,
                status: if *base > 150.0 { ServiceStatus::Degraded } else { ServiceStatus::Healthy },
                phase: i as f32 * 0.87,
                last_flip: now,
            })
            .collect();
        let regions = REGION_DEFS
            .iter()
            .map(|(code, phase)| Region { code, phase: *phase, frequency: 0.15 + phase * 0.1 })
            .collect();
        Self {
            spark,
            events: VecDeque::with_capacity(EVENT_BUFFER),
            services,
            regions,
            last_event_at: now.checked_sub(EVENT_INTERVAL).unwrap_or(now),
            clock_base: now,
            next_event: 0,
        }
    }

    /// Advance metrics, services, and the event log to `now`. Call once per
    /// frame before rendering.
    pub fn tick(&mut self, now: Instant) {
        let t = now.duration_since(self.clock_base).as_secs_f32();
        let base = (t * 1.3).sin() * 0.5 + 0.5;
        let jitter = (t * 7.9).sin() * 0.12;
        let value = ((base + jitter).clamp(0.05, 0.98) * 100.0) as u64;
        self.spark.pop_front();
        self.spark.push_back(value);

        for svc in &mut self.services {
            if svc.name == "billing-svc" && now.duration_since(svc.last_flip).as_secs_f32() > 22.0 {
                svc.status = match svc.status {
                    ServiceStatus::Healthy => ServiceStatus::Degraded,
                    ServiceStatus::Degraded => ServiceStatus::Healthy,
                };
                svc.last_flip = now;
            }
        }

        if now.duration_since(self.last_event_at) >= EVENT_INTERVAL {
            let (text, kind) = EVENT_POOL[self.next_event % EVENT_POOL.len()];
            self.next_event += 1;
            let stamp = format_clock(now.duration_since(self.clock_base));
            self.events.push_back(EventEntry {
                stamp,
                text: text.to_string(),
                kind,
            });
            while self.events.len() > EVENT_BUFFER {
                self.events.pop_front();
            }
            self.last_event_at = now;
        }
    }

    fn elapsed(&self, now: Instant) -> Duration {
        now.duration_since(self.clock_base)
    }
}

fn format_clock(d: Duration) -> String {
    let base = 14 * 3600 + 30 * 60;
    let total = base + d.as_secs();
    let h = (total / 3600) % 24;
    let m = (total / 60) % 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Composite hero dashboard: stat tiles, service table, region bars, event
/// log, and sparkline, all driven by a [`DashboardState`].
///
/// ```no_run
/// use std::time::Instant;
/// use ono::components::dashboard::{Dashboard, DashboardState};
/// use ono::theme::Theme;
/// use ratatui::widgets::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
/// # let area = buf.area;
///
/// let theme = Theme::Forest;
/// let now = Instant::now();
/// let mut state = DashboardState::new(now);
/// state.tick(now);
///
/// Dashboard::new(&state, now, theme.palette(), theme.knobs())
///     .title("my-app")
///     .subtitle("staging")
///     .render(area, &mut buf);
/// ```
pub struct Dashboard<'a> {
    state: &'a DashboardState,
    now: Instant,
    title: &'a str,
    subtitle: &'a str,
    border_type: BorderType,
    palette: &'a Palette,
    knobs: &'a Knobs,
}

impl<'a> Dashboard<'a> {
    /// Construct a dashboard view over `state`. `now` is the frame timestamp.
    pub fn new(
        state: &'a DashboardState,
        now: Instant,
        palette: &'a Palette,
        knobs: &'a Knobs,
    ) -> Self {
        Self {
            state,
            now,
            title: "ono",
            subtitle: "command center",
            border_type: BorderType::Rounded,
            palette,
            knobs,
        }
    }

    /// Title rendered top-left of the outer border.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Subtitle rendered after `·` next to the title.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = subtitle;
        self
    }

    /// Outer border style (default `BorderType::Rounded`).
    pub fn border_type(mut self, bt: BorderType) -> Self {
        self.border_type = bt;
        self
    }
}

fn fg_bg(fg: Color, bg: Color) -> Style {
    Style::default().fg(fg).bg(bg)
}

fn panel(title: &str, palette: &Palette) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(fg_bg(palette.border, palette.bg))
        .title(Span::styled(
            format!(" {} ", title),
            fg_bg(palette.primary, palette.bg),
        ))
        .style(Style::default().bg(palette.bg))
}

fn stat_tile(
    title: &'static str,
    value: String,
    subtitle: String,
    trend: Option<(String, Color)>,
    palette: &Palette,
) -> Paragraph<'static> {
    let mut lines = vec![
        Line::from(Span::raw(" ")),
        Line::from(Span::styled(
            value,
            fg_bg(palette.bright, palette.bg).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(Span::styled(subtitle, fg_bg(palette.dim, palette.bg)))
            .alignment(Alignment::Center),
    ];
    if let Some((t, c)) = trend {
        lines.push(Line::from(Span::raw(" ")));
        lines.push(
            Line::from(Span::styled(t, fg_bg(c, palette.bg))).alignment(Alignment::Center),
        );
    }
    Paragraph::new(lines).block(panel(title, palette))
}

fn services_widget(state: &DashboardState, palette: &Palette, t: f32) -> Paragraph<'static> {
    let lines: Vec<Line<'static>> = state
        .services
        .iter()
        .map(|svc| {
            let wobble = (t * 0.8 + svc.phase).sin() * 0.15 + 1.0;
            let latency = (svc.base_latency_ms * wobble).max(1.0);
            let latency_str = format!("{:>5.0}ms", latency);
            let (dot, status_label, status_color) = match svc.status {
                ServiceStatus::Healthy => ("●", "healthy ", palette.bright),
                ServiceStatus::Degraded => ("○", "degraded", palette.warn),
            };
            let latency_color = if latency < 50.0 {
                palette.dim
            } else if latency < 100.0 {
                palette.primary
            } else {
                palette.accent
            };
            Line::from(vec![
                Span::styled(dot.to_string(), fg_bg(status_color, palette.bg)),
                Span::raw("  "),
                Span::styled(
                    format!("{:<16}", svc.name),
                    fg_bg(palette.primary, palette.bg),
                ),
                Span::styled(status_label.to_string(), fg_bg(palette.dim, palette.bg)),
                Span::raw("  "),
                Span::styled(latency_str, fg_bg(latency_color, palette.bg)),
            ])
        })
        .collect();
    Paragraph::new(lines).block(panel("services", palette))
}

fn regions_widget(state: &DashboardState, palette: &Palette, t: f32) -> Paragraph<'static> {
    let loads: Vec<f32> = state
        .regions
        .iter()
        .map(|r| {
            let v = (t * r.frequency + r.phase * std::f32::consts::TAU).sin() * 0.45 + 0.5;
            v.clamp(0.05, 0.98)
        })
        .collect();

    let mut bar_rows: Vec<Line<'static>> = Vec::new();
    for row in 0..3 {
        let mut spans = vec![Span::raw("  ")];
        for &load in &loads {
            let fill = (load * 3.0 + 0.5).floor() as usize;
            let idx = if row < (3 - fill) {
                0
            } else {
                (REGION_BAR_STEPS.len() - 1).min(((load * 8.0) as usize).max(1))
            };
            let color = if load > 0.75 {
                palette.accent
            } else if load > 0.4 {
                palette.primary
            } else {
                palette.dim
            };
            spans.push(Span::styled(
                format!(" {} ", REGION_BAR_STEPS[idx]),
                fg_bg(color, palette.bg),
            ));
            spans.push(Span::raw(" "));
        }
        bar_rows.push(Line::from(spans));
    }

    let mut label_spans = vec![Span::raw("  ")];
    for r in &state.regions {
        label_spans.push(Span::styled(
            format!(" {} ", r.code),
            fg_bg(palette.dim, palette.bg),
        ));
        label_spans.push(Span::raw(" "));
    }

    let healthy = loads.iter().filter(|&&l| l < 0.85).count();
    let degraded = loads.len() - healthy;

    let mut lines = bar_rows;
    lines.push(Line::from(label_spans));
    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(vec![
        Span::styled("  healthy: ", fg_bg(palette.dim, palette.bg)),
        Span::styled(
            format!("{}", healthy),
            fg_bg(palette.bright, palette.bg),
        ),
        Span::styled("   degraded: ", fg_bg(palette.dim, palette.bg)),
        Span::styled(
            format!("{}", degraded),
            fg_bg(
                if degraded > 0 { palette.accent } else { palette.dim },
                palette.bg,
            ),
        ),
    ]));

    Paragraph::new(lines).block(panel("regions", palette))
}

fn events_widget(state: &DashboardState, palette: &Palette) -> Paragraph<'static> {
    let lines: Vec<Line<'static>> = state
        .events
        .iter()
        .rev()
        .map(|e| {
            let (prefix, color) = match e.kind {
                EventKind::Ok => ("[ok]", palette.bright),
                EventKind::Warn => ("[!!]", palette.warn),
                EventKind::Info => ("[··]", palette.accent),
            };
            Line::from(vec![
                Span::styled(e.stamp.clone(), fg_bg(palette.dim, palette.bg)),
                Span::raw("  "),
                Span::styled(prefix.to_string(), fg_bg(color, palette.bg)),
                Span::raw("  "),
                Span::styled(e.text.clone(), fg_bg(palette.primary, palette.bg)),
            ])
        })
        .collect();
    Paragraph::new(lines).block(panel("events", palette))
}

fn gauge(
    label: &str,
    ratio: f64,
    color: Color,
    palette: &Palette,
    unicode: bool,
) -> Gauge<'static> {
    Gauge::default()
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(palette.bg)),
        )
        .gauge_style(Style::default().fg(color).bg(palette.border))
        .ratio(ratio)
        .label(Span::styled(
            format!("{:<8}  {:>3.0}%", label, ratio * 100.0),
            fg_bg(palette.bright, palette.bg),
        ))
        .use_unicode(unicode)
}

impl Widget for Dashboard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let palette = self.palette;
        let knobs = self.knobs;
        let state = self.state;
        let elapsed = state.elapsed(self.now);
        let t = elapsed.as_secs_f32();
        let clock = format_clock(elapsed);

        Block::default()
            .style(Style::default().bg(palette.bg))
            .render(area, buf);

        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(self.border_type)
            .border_style(fg_bg(palette.primary, palette.bg))
            .title(Line::from(vec![
                Span::styled(
                    format!(" {} ", self.title),
                    fg_bg(palette.bright, palette.bg).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("· {} ", self.subtitle),
                    fg_bg(palette.dim, palette.bg),
                ),
            ]))
            .title_bottom(
                Line::from(vec![Span::styled(
                    " q to quit ",
                    fg_bg(palette.dim, palette.bg),
                )])
                .alignment(Alignment::Left),
            )
            .title_bottom(
                Line::from(vec![Span::styled(
                    format!(" {} ", clock),
                    fg_bg(palette.dim, palette.bg),
                )])
                .alignment(Alignment::Right),
            )
            .style(Style::default().bg(palette.bg));
        let inner = outer.inner(area);
        outer.render(area, buf);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(10),
                Constraint::Length(7),
                Constraint::Min(4),
            ])
            .split(inner);

        let tiles = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(22),
                Constraint::Percentage(22),
                Constraint::Percentage(22),
                Constraint::Percentage(34),
            ])
            .spacing(1)
            .split(rows[0]);

        let rate = 1240.0 + (t * 0.7).sin() * 80.0 + (t * 3.1).sin() * 20.0;
        let p50 = 42.0 + (t * 0.9).sin() * 6.0;
        let p99 = 187.0 + (t * 0.6).sin() * 24.0;
        let error_rate = 0.12 + (t * 0.4).sin() * 0.05;

        stat_tile(
            "signal",
            format!("{:.0}", rate),
            "req / min".into(),
            Some(("▲ 2.4%  5m".into(), palette.accent)),
            palette,
        )
        .render(tiles[0], buf);
        stat_tile(
            "p50 latency",
            format!("{:.0}ms", p50),
            format!("p99: {:.0}ms", p99),
            Some(("▼ 1.1%  5m".into(), palette.bright)),
            palette,
        )
        .render(tiles[1], buf);
        stat_tile(
            "error rate",
            format!("{:.2}%", error_rate),
            "4xx + 5xx · 5m".into(),
            Some(("▲ 0.04%  5m".into(), palette.warn)),
            palette,
        )
        .render(tiles[2], buf);

        let spark_vec: Vec<u64> = state.spark.iter().copied().collect();
        let throughput_now = spark_vec.last().copied().unwrap_or(0);

        let throughput_block = panel("throughput", palette);
        let throughput_inner = throughput_block.inner(tiles[3]);
        throughput_block.render(tiles[3], buf);
        let throughput_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(throughput_inner);
        let header = Line::from(vec![
            Span::styled(
                format!("{:>4} req/s  ", throughput_now),
                fg_bg(palette.bright, palette.bg).add_modifier(Modifier::BOLD),
            ),
            Span::styled("rolling 140s", fg_bg(palette.dim, palette.bg)),
        ]);
        Paragraph::new(header).render(throughput_rows[0], buf);
        Sparkline::default()
            .data(&spark_vec)
            .max(100)
            .style(fg_bg(palette.accent, palette.bg))
            .bar_set(symbols::bar::NINE_LEVELS)
            .render(throughput_rows[1], buf);

        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .spacing(1)
            .split(rows[1]);

        services_widget(state, palette, t).render(mid[0], buf);
        regions_widget(state, palette, t).render(mid[1], buf);

        let cpu = (0.42 + (t * 0.6).sin() as f64 * 0.08).clamp(0.0, 1.0);
        let mem = (0.52 + (t * 0.5).sin() as f64 * 0.06).clamp(0.0, 1.0);
        let net = (0.38 + (t * 0.9).sin() as f64 * 0.12).clamp(0.0, 1.0);
        let disk = (0.33 + (t * 0.3).sin() as f64 * 0.04).clamp(0.0, 1.0);

        let gauges_outer = panel("resources", palette);
        let gauges_inner = gauges_outer.inner(rows[2]);
        gauges_outer.render(rows[2], buf);
        let gauge_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(gauges_inner);
        gauge("cpu", cpu, palette.primary, palette, knobs.gauge_unicode)
            .render(gauge_rows[0], buf);
        gauge("memory", mem, palette.primary, palette, knobs.gauge_unicode)
            .render(gauge_rows[1], buf);
        gauge("network", net, palette.accent, palette, knobs.gauge_unicode)
            .render(gauge_rows[2], buf);
        gauge("disk", disk, palette.dim, palette, knobs.gauge_unicode)
            .render(gauge_rows[3], buf);

        events_widget(state, palette).render(rows[3], buf);
    }
}
