# Theming

Themes are a first-class architectural primitive, not decoration. Every
component pulls its colors from a `Palette` and its animation behavior
from a `Knobs` struct — **never** from hard-coded literals. Swapping
themes means swapping those two inputs; no component code changes.

## The three types

```rust
use ono::theme::{Theme, Palette, Knobs};
```

- **`Theme`** — an enum bundling a palette + knobs preset. Always
  includes `Forest`; the other variants (`Retro`, `Minimal`, `Cyber`) are
  feature-gated.
- **`Palette`** — nine color roles. Semver-locked at v0.1.0.
- **`Knobs`** — animation and behavior parameters (gradient period,
  scanline speed, spinner frame set, …).

A component typically takes `&Palette` (and sometimes `&Knobs`). Pull
them off a `Theme`:

```rust
let theme = Theme::Forest;
let palette = theme.palette();
let knobs = theme.knobs();
```

## Palette roles

| Role | Intended use |
|---|---|
| `bg` | Terminal background. Rarely painted — let the user's terminal show through. |
| `surface` | Panel / card background, one shade above `bg`. |
| `border` | Box borders, dividers. |
| `dim` | Secondary labels, timestamps, metadata. |
| `primary` | Body text, the dominant readable color. |
| `bright` | Highlights, stat values, status dots, "OK" markers. |
| `accent` | Gradient endpoint, progress fill, selection — the hero accent. |
| `secondary` | Second accent for two-hue compositions. |
| `warn` | Error and warn states only. Not a generic accent. |

Adding a role is a breaking change — `Palette` is not
`#[non_exhaustive]` because ejected `theme.rs` files construct it with
struct literals.

## Built-in themes

| Theme | Feature flag | Character |
|---|---|---|
| `Forest` | _(always on)_ | Calm greens, warm gold, slow breathing. Canonical. |
| `Retro` | `theme-retro` | Amber phosphor, scanline on, idle flicker. |
| `Minimal` | `theme-minimal` | Quiet monochrome with a single violet accent. |
| `Cyber` | `theme-cyber` | Neon, scanline + glitch on, high contrast. |

Enable one or all:

```toml
# Cargo.toml
[dependencies]
ono = { version = "0.1", features = ["theme-cyber"] }
# or all at once
ono = { version = "0.1", features = ["all-themes"] }
```

The feature-gated themes exist primarily as dev tools — they keep
component code honest (if a component only looks right under one
palette, that's a leak). The shipped story is single-theme Forest; other
themes are not advertised to end users.

## Knobs

`Knobs` tune per-theme behavior without requiring component code to
branch on theme identity. Fields (all `pub`):

```rust
pub struct Knobs {
    pub gradient_period_secs: f32,
    pub pulse_amplitude: f32,
    pub scanline: bool,
    pub scanline_speed_rows_per_sec: f32,
    pub scanline_boost: f32,
    pub glitch: bool,
    pub glitch_interval_min: f32,
    pub glitch_interval_max: f32,
    pub glitch_duration_ms: u64,
    pub idle_flicker: bool,
    pub reveal_ms_per_char: u64,
    pub spinner: &'static [char],
    pub cursor_blink_hz: f32,
    pub gauge_unicode: bool,
}
```

Components branch on knobs, not themes:

```rust
// good — no coupling to Theme variants
if knobs.scanline { /* draw scanline */ }
if knobs.gauge_unicode { /* block glyphs */ } else { /* ASCII */ }

// smell — ties rendering to a specific theme
// if theme == Theme::Retro { … }
```

## Writing a custom theme

`Palette` and `Knobs` have `pub` fields — construct your own and pass
`&Palette` / `&Knobs` wherever a component expects them. No `Theme`
enum involvement required.

```rust
use ono::theme::{Palette, Knobs};
use ratatui::style::Color;

const MY_PALETTE: Palette = Palette {
    bg:        Color::Rgb(0x10, 0x12, 0x18),
    surface:   Color::Rgb(0x18, 0x1B, 0x22),
    border:    Color::Rgb(0x2A, 0x2F, 0x3A),
    dim:       Color::Rgb(0x6A, 0x70, 0x80),
    primary:   Color::Rgb(0xC8, 0xCC, 0xD4),
    bright:    Color::Rgb(0xF0, 0xE8, 0xA0),
    accent:    Color::Rgb(0xE8, 0x8A, 0x30),
    secondary: Color::Rgb(0x50, 0xA0, 0xC8),
    warn:      Color::Rgb(0xE0, 0x40, 0x40),
};

const MY_KNOBS: Knobs = Knobs {
    gradient_period_secs: 6.0,
    pulse_amplitude: 0.04,
    scanline: false,
    scanline_speed_rows_per_sec: 0.0,
    scanline_boost: 1.0,
    glitch: false,
    glitch_interval_min: 0.0,
    glitch_interval_max: 0.0,
    glitch_duration_ms: 0,
    idle_flicker: true,
    reveal_ms_per_char: 24,
    spinner: &['◐', '◓', '◑', '◒'],
    cursor_blink_hz: 1.0,
    gauge_unicode: true,
};

// Anywhere a component asks for a Theme:
// Splash::new(&banner, &MY_PALETTE, &MY_KNOBS) …
```

Every role must be filled. Every knob must be set. No `Option`, no
fallbacks.

## Theme in ejected code

When you run `ono add <name>`, a **full `theme.rs`** is written into
your project alongside the component source. It contains the `Theme`
enum, every `Palette` constant (forest by default), every `Knobs`
constant, and the role-lookup table.

- On the first `ono add`, `theme.rs` is written.
- On subsequent adds, the CLI keeps your existing `theme.rs` — delete
  the file to regenerate.
- You own it. Recoloring is just editing hex values in a struct literal.
- It has no runtime dependency on the `ono` crate.

See [ejecting](./eject.md) for the full picture.

## Rules

1. **Never hardcode hex in component code.** Go through the palette.
2. **Never branch on `Theme` identity for visual logic.** Branch on
   knobs or palette roles.
3. **Never add per-component color fallbacks.** If a role is missing
   you're designing a new role — update `Palette` everywhere.
