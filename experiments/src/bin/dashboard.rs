use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style as TuiStyle};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Terminal;

use experiments::{Palette, Theme};

const FRAME_TARGET: Duration = Duration::from_millis(33);
const SPARK_LEN: usize = 120;
const EVENT_INTERVAL: Duration = Duration::from_millis(1800);
const EVENT_BUFFER: usize = 8;

const EVENT_POOL: &[(&str, EventKind)] = &[
    ("spec loaded · aurora-banner", EventKind::Ok),
    ("snapshot match · boot-sequence", EventKind::Ok),
    ("render cycle complete", EventKind::Ok),
    ("template cache warmed", EventKind::Ok),
    ("color tier fallback · 256 → 16", EventKind::Warn),
    ("spec loaded · dashboard", EventKind::Ok),
    ("gauge redrawn", EventKind::Ok),
    ("sparkline window advanced", EventKind::Ok),
    ("subscriber connected", EventKind::Ok),
];

#[derive(Clone, Copy)]
enum EventKind {
    Ok,
    Warn,
}

struct EventEntry {
    stamp: String,
    text: &'static str,
    kind: EventKind,
}

struct State {
    spark: VecDeque<u64>,
    events: VecDeque<EventEntry>,
    last_event_at: Instant,
    clock_base: Instant,
    next_event: usize,
}

impl State {
    fn new(now: Instant) -> Self {
        let mut spark = VecDeque::with_capacity(SPARK_LEN);
        for _ in 0..SPARK_LEN {
            spark.push_back(0);
        }
        Self {
            spark,
            events: VecDeque::with_capacity(EVENT_BUFFER),
            last_event_at: now - EVENT_INTERVAL,
            clock_base: now,
            next_event: 0,
        }
    }

    fn tick(&mut self, now: Instant) {
        let t = now.duration_since(self.clock_base).as_secs_f32();
        let base = (t * 1.3).sin() * 0.5 + 0.5;
        let jitter = (t * 7.9).sin() * 0.12;
        let value = ((base + jitter).clamp(0.05, 0.98) * 100.0) as u64;
        self.spark.pop_front();
        self.spark.push_back(value);

        if now.duration_since(self.last_event_at) >= EVENT_INTERVAL {
            let (text, kind) = EVENT_POOL[self.next_event % EVENT_POOL.len()];
            self.next_event += 1;
            let stamp = format_clock(now.duration_since(self.clock_base));
            self.events.push_back(EventEntry { stamp, text, kind });
            while self.events.len() > EVENT_BUFFER {
                self.events.pop_front();
            }
            self.last_event_at = now;
        }
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

fn fg_bg(fg: Color, bg: Color) -> TuiStyle {
    TuiStyle::default().fg(fg).bg(bg)
}

fn panel(title: &str, palette: &Palette, border_type: BorderType) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(fg_bg(palette.border, palette.bg))
        .title(Span::styled(
            format!(" {} ", title),
            fg_bg(palette.primary, palette.bg),
        ))
        .style(TuiStyle::default().bg(palette.bg))
}

fn signal_tile(now: Instant, base: Instant, palette: &Palette, style: Theme) -> Paragraph<'static> {
    let t = now.duration_since(base).as_secs_f32();
    let rate = 1240.0 + (t * 0.7).sin() * 80.0 + (t * 3.1).sin() * 20.0;
    let trend = match style {
        #[cfg(feature = "theme-minimal")]
        Theme::Minimal => "up  2.4%  5m",
        _ => "▲ 2.4%  last 5m",
    };
    let lines = vec![
        Line::from(Span::raw(" ")),
        Line::from(Span::styled(
            format!("{:.0}", rate),
            fg_bg(palette.bright, palette.bg).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(Span::styled("req / min", fg_bg(palette.dim, palette.bg)))
            .alignment(Alignment::Center),
        Line::from(Span::raw(" ")),
        Line::from(Span::styled(trend, fg_bg(palette.accent, palette.bg)))
            .alignment(Alignment::Center),
    ];
    Paragraph::new(lines).block(panel("signal", palette, BorderType::Plain))
}

fn events_widget(state: &State, palette: &Palette, style: Theme) -> Paragraph<'static> {
    let lines: Vec<Line<'static>> = state
        .events
        .iter()
        .rev()
        .map(|e| {
            let (prefix, color) = match e.kind {
                EventKind::Ok => ("[ok]", palette.bright),
                EventKind::Warn => ("[!!]", palette.warn),
            };
            Line::from(vec![
                Span::styled(e.stamp.clone(), fg_bg(palette.dim, palette.bg)),
                Span::raw("  "),
                Span::styled(prefix, fg_bg(color, palette.bg)),
                Span::raw("  "),
                Span::styled(e.text, fg_bg(palette.primary, palette.bg)),
            ])
        })
        .collect();
    let border = match style {
        #[cfg(feature = "theme-cyber")]
        Theme::Cyber => BorderType::Double,
        _ => BorderType::Plain,
    };
    Paragraph::new(lines).block(panel("events", palette, border))
}

fn gauge<'a>(label: &'a str, ratio: f64, color: Color, palette: &'a Palette, unicode: bool) -> Gauge<'a> {
    Gauge::default()
        .block(Block::default().borders(Borders::NONE).style(TuiStyle::default().bg(palette.bg)))
        .gauge_style(TuiStyle::default().fg(color).bg(palette.border))
        .ratio(ratio)
        .label(Span::styled(
            format!("{}  {:>3.0}%", label, ratio * 100.0),
            fg_bg(palette.bright, palette.bg),
        ))
        .use_unicode(unicode)
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
    let mut state = State::new(start);
    let mut last_frame = start;
    let palette = style.palette();
    let knobs = style.knobs();

    let outer_border = match style {
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
        let clock = format_clock(now.duration_since(start));

        let spark_vec: Vec<u64> = state.spark.iter().copied().collect();
        let mem_ratio = (0.52 + (t * 0.5).sin() as f64 * 0.08).clamp(0.0, 1.0);
        let render_ratio = (0.68 + (t * 0.9).sin() as f64 * 0.10).clamp(0.0, 1.0);

        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(Block::default().style(TuiStyle::default().bg(palette.bg)), area);

            let outer = Block::default()
                .borders(Borders::ALL)
                .border_type(outer_border)
                .border_style(fg_bg(palette.primary, palette.bg))
                .title(Line::from(vec![
                    Span::styled(" ono ", fg_bg(palette.bright, palette.bg).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("· dashboard · {} ", style.name()), fg_bg(palette.dim, palette.bg)),
                ]))
                .title_bottom(
                    Line::from(vec![Span::styled(" q to quit ", fg_bg(palette.dim, palette.bg))])
                        .alignment(Alignment::Left),
                )
                .title_bottom(
                    Line::from(vec![Span::styled(
                        format!(" {} · v0.0.4 ", clock),
                        fg_bg(palette.dim, palette.bg),
                    )])
                    .alignment(Alignment::Right),
                )
                .style(TuiStyle::default().bg(palette.bg));
            let inner = outer.inner(area);
            f.render_widget(outer, area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(8), Constraint::Min(6), Constraint::Length(5)])
                .split(inner);

            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24), Constraint::Min(30)])
                .spacing(2)
                .split(rows[0]);

            f.render_widget(signal_tile(now, start, palette, style), top[0]);

            let spark = Sparkline::default()
                .block(panel("throughput", palette, BorderType::Plain))
                .data(&spark_vec)
                .max(100)
                .style(fg_bg(palette.accent, palette.bg))
                .bar_set(symbols::bar::NINE_LEVELS);
            f.render_widget(spark, top[1]);

            f.render_widget(events_widget(&state, palette, style), rows[1]);

            let gauges_outer = panel("usage", palette, BorderType::Plain);
            let gauges_inner = gauges_outer.inner(rows[2]);
            f.render_widget(gauges_outer, rows[2]);

            let gauges = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .spacing(0)
                .split(gauges_inner);

            f.render_widget(
                gauge("memory", mem_ratio, palette.primary, palette, knobs.gauge_unicode),
                gauges[0],
            );
            f.render_widget(
                gauge("render", render_ratio, palette.accent, palette, knobs.gauge_unicode),
                gauges[1],
            );
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
