# Components catalog

Every widget the library ships, grouped by tier.

> **Heads up — screenshots / recordings are not yet inline on this page.**
> Visual captures land on the deferred *recordings* track alongside the
> showcase site. Until then, the fastest way to see a component is:
>
> ```sh
> cargo install ono
> ono preview <name>   # q / Esc / Ctrl+C to exit
> ```
>
> Or run a working example locally:
>
> ```sh
> cargo run -p ratatui-library   # library integration
> cargo run -p ratatui-demo      # eject-mode integration
> ```

Every snippet below assumes:

```rust
use ono::theme::Theme;
let theme = Theme::Forest;
let palette = theme.palette();
let knobs = theme.knobs();
```

## Model

- **Elements** are atomic — they do not compose other `ono` widgets.
  `box`, `progress`, `spinner`, `percentage`, `sparkline`, `typewriter`.
- **Components** compose elements and other components. `splash`,
  `boot`, `dashboard`, `statusbar`, `map`.

`ono add <name>` copies the source file plus every transitive dep.

---

## Elements

### `box` (module `ono::elements::boxed`)

Bordered panel with optional title. Thin wrapper over
`ratatui::widgets::Block` that pulls its border and title colors from
the palette.

```rust
use ono::elements::boxed::BoxFrame;
use ratatui::widgets::{Borders, Widget};

BoxFrame::new(palette)
    .title("metrics")
    .borders(Borders::ALL)
    .render(area, buf);
```

### `percentage`

Inline percentage readout. Clamps to `0.0..=1.0`.

```rust
use ono::elements::percentage::Percentage;
use ratatui::widgets::Widget;

Percentage::new(0.72, palette).decimals(1).render(area, buf); // "72.0%"
```

### `progress`

Horizontal progress bar. Unicode sub-cell fills or ASCII fallback.

```rust
use ono::elements::progress::{Progress, ProgressStyle};
use ratatui::widgets::Widget;

Progress::new(0.42, palette)
    .width(30)
    .label("install")
    .show_percent(true)
    .style(ProgressStyle::Unicode)
    .render(area, buf);
```

### `sparkline`

Block-glyph mini-chart. Auto-scaled to the min/max of the visible
window. Newest value on the right.

```rust
use ono::elements::sparkline::Sparkline;
use ratatui::widgets::Widget;

let samples: Vec<f32> = (0..40).map(|i| (i as f32 * 0.3).sin()).collect();
Sparkline::new(&samples, palette).width(20).render(area, buf);
```

### `spinner`

Stateless single-cell spinner. Caller advances `tick` each frame.

```rust
use ono::elements::spinner::Spinner;
use ratatui::widgets::Widget;

let tick = 7; // your frame counter
Spinner::new(palette)
    .frames(knobs.spinner)
    .tick(tick)
    .label("loading")
    .render(area, buf);
```

### `typewriter`

Progressive text reveal with an optional blinking caret.

```rust
use ono::elements::typewriter::Typewriter;
use ratatui::widgets::Widget;

Typewriter::new("boot sequence complete", palette)
    .progress(0.5)
    .cursor(true)
    .cursor_blink(3)
    .render(area, buf);
```

---

## Components

### `splash`

Animated FIGlet wordmark. Gradient sweep, optional per-eye pulse,
optional scanline, optional idle flicker. Stateless — caller owns the
clock.

```rust
use std::time::Duration;
use figlet_rs::FIGlet;
use ono::components::splash::{Banner, Splash};
use ratatui::widgets::Widget;

let font = FIGlet::standard().unwrap();
let banner = Banner::from_text("my-app", &font); // build ONCE, reuse

// in your render loop:
Splash::new(&banner, palette, knobs)
    .tagline("beautiful terminal UI components")
    .elapsed(Duration::from_millis(1200))
    .render(area, buf);
```

`Banner::from_text` is expensive; build once at startup.

### `boot`

Animated boot log. Steps typewriter-reveal, a spinner runs for each
step's `pending_ms`, and an `Ok` / `Warn` outcome marker is drawn.
After the final step the sequence pauses and loops.

```rust
use std::time::Duration;
use ono::components::boot::Boot;
use ratatui::widgets::Widget;

Boot::new(palette, knobs)
    .header("> booting my-app")
    .elapsed(Duration::from_millis(2400))
    .render(area, buf);
```

Supply a custom step list with `.steps(&[Step { … }, …])`. See
`DEFAULT_STEPS` in the rustdoc for a template.

### `dashboard`

Composite hero view: stat tiles, services table, region bars, event
log, sparkline. State lives in a `DashboardState` the caller ticks each
frame.

```rust
use std::time::Instant;
use ono::components::dashboard::{Dashboard, DashboardState};
use ratatui::widgets::Widget;

let now = Instant::now();
let mut state = DashboardState::new(now);

// each frame:
state.tick(now);
Dashboard::new(&state, now, palette, knobs)
    .title("my-app")
    .subtitle("staging")
    .render(area, buf);
```

### `statusbar`

One-line status bar composing `spinner`, `progress`, and `percentage`.
Toggle parts with `show_*` builder methods; the layout collapses around
what's shown.

```rust
use ono::components::statusbar::Statusbar;
use ratatui::widgets::Widget;

Statusbar::new(palette)
    .label("Compiling")
    .percent(0.35)
    .show_spinner(true)
    .tick(12)
    .render(area, buf);
```

### `map`

Stylized world map with animated packet traces between cities. State
lives in a `MapState` the caller ticks each frame.

```rust
use std::time::Instant;
use ono::components::map::{MapState, WorldMap};
use ratatui::widgets::Widget;

let now = Instant::now();
let mut state = MapState::new(now);

// each frame:
state.tick(now);
WorldMap::new(&state, now, palette)
    .title("my-app")
    .subtitle("global traffic")
    .render(area, buf);
```

---

## Rustdoc

Every public item has rustdoc with a minimal usage example. Browse at
[docs.rs/ono](https://docs.rs/ono) once v0.1.0 ships, or run locally:

```sh
cargo doc -p ono --open
```

## Ratatui target

All components in v0.1.0 target Ratatui. Textual, Bubble Tea, and Ink
generators ship later — some components may not translate to every
target. When that happens it's flagged honestly on the per-component
page rather than faked behind a lowest-common-denominator abstraction.
