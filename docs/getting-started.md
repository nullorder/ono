# Getting started

Five-minute walkthrough: add `ono` to a Ratatui app and render your first
component.

## 1. Create a project

```sh
cargo new my-tui && cd my-tui
cargo add ono ratatui crossterm figlet-rs
```

## 2. Pick a theme

Every component takes a `&Palette` and usually a `&Knobs`. Grab both from a
`Theme`:

```rust
use ono::theme::Theme;

let theme = Theme::Forest;
let palette = theme.palette();
let knobs = theme.knobs();
```

Forest is the canonical theme and the only one built by default. See
[theming](./theming.md) for Retro / Minimal / Cyber and how to write your
own.

## 3. Render a component

Components are standard Ratatui `Widget`s. Build a `Splash` on top of a
prebuilt `Banner`:

```rust
use std::time::Duration;
use figlet_rs::FIGlet;
use ono::components::splash::{Banner, Splash};
use ono::theme::Theme;
use ratatui::widgets::Widget;

let theme = Theme::Forest;
let font = FIGlet::standard().expect("figlet standard font");
let banner = Banner::from_text("my-app", &font);

// Inside your render loop:
// Splash::new(&banner, theme.palette(), theme.knobs())
//     .tagline("boot sequence engaged")
//     .elapsed(elapsed)
//     .render(area, buf);
```

`Banner::from_text` is expensive — build it **once** and reuse across
frames. `Splash` itself is cheap to construct per frame.

## 4. Drive the clock

Almost every animated component expects a monotonically-increasing
`Duration` via an `.elapsed(...)` builder. Own the clock in your app:

```rust
use std::time::Instant;

let start = Instant::now();

loop {
    let elapsed = start.elapsed();
    // … render using `elapsed` …
}
```

Stateful components like `Dashboard` / `WorldMap` also have a `State`
companion that you mutate each frame:

```rust
use ono::components::dashboard::{Dashboard, DashboardState};
use std::time::Instant;

let now = Instant::now();
let mut state = DashboardState::new(now);

loop {
    let now = Instant::now();
    state.tick(now); // advance simulation

    // … in your render loop:
    // Dashboard::new(&state, now, theme.palette(), theme.knobs())
    //     .render(area, buf);
}
```

## 5. Run a complete example

A working boot → splash → dashboard app lives at
[`examples/ratatui-library/`](../examples/ratatui-library/). Clone the
repo and:

```sh
cargo run -p ratatui-library
```

Press `q`, `Esc`, or `Ctrl+C` to exit.

## Where to go next

- [Theming](./theming.md) — palette roles, knobs, writing a custom theme.
- [Components](./components.md) — catalog of every element and component
  with usage snippets.
- [Ejecting](./eject.md) — when (and why) to copy component source into
  your tree with `ono add`.
