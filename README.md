# Ono

Beautiful terminal UI components for Ratatui and (soon) Textual / Bubble Tea / Ink.

> Early work in progress. The CLI isn't published yet. Prototypes run locally via the `experiments/` crate; the shipped library and CLI are in active development under `ono/`.

## Usage — library (default)

```toml
[dependencies]
ono = "0.1"
ratatui = "0.30"
```

```rust
use ono::components::splash::{Banner, Splash};
use ono::theme::Theme;

let theme = Theme::Forest;
let banner = Banner::from_text("ono", &figlet_rs::FIGlet::standard().unwrap());

// In your render loop:
Splash::new(&banner, theme.palette(), theme.knobs())
    .elapsed(elapsed)
    .render(area, buf);
```

Components are themeable, parameterized Ratatui widgets. Customize through typed builder methods (`.tagline()`, `.pulse(false)`, ...) and the `Palette` / `Knobs` on a `Theme`.

A runnable example lives in [`examples/ratatui-library/`](examples/ratatui-library/).

## Usage — eject (power path)

When typed params aren't enough and you want to rewrite rendering logic, pull the source into your tree:

```sh
cargo install ono
cd your-project
ono add splash
ono add dashboard
```

This copies the component's `.rs` file (and any transitive deps) into `./src/ono/`, plus a `theme.rs` you own. Ejected code imports only Ratatui and your own `theme.rs` — no runtime dependency on `ono`.

A runnable example lives in [`examples/ratatui-demo/`](examples/ratatui-demo/).

## CLI reference

```sh
ono list                            # browse the catalog
ono preview splash                  # live render in your terminal
ono add splash                      # eject source into ./src/ono/
ono add splash --theme cyber        # bake a different default palette into theme.rs
```

## Try the prototypes

Requires a true-color terminal. `q`, `Esc`, or `Ctrl+C` to exit.

```sh
cargo experiments splash
cargo experiments boot
cargo experiments dashboard
cargo experiments map
```

Runs in the canonical **forest** theme by default. Other themes are available behind cargo features for dev comparison:

```sh
just theme splash retro
just theme dashboard minimal
just theme boot cyber
```

## Model

- **Components** — themeable Ratatui widgets. Split into **elements** (atomic: `progress`, `spinner`, `box`, ...) and **components** (composite: `splash`, `boot`, `dashboard`, `statusbar`, ...).
- **Themes** — palette of semantic roles + animation knobs. Forest is canonical; others behind cargo features.

Ratatui is the first target. Textual, Bubble Tea, and Ink are on the way.

## Documentation

Narrative docs in [`docs/`](./docs/):

- [Getting started](./docs/getting-started.md) — five-minute library walkthrough.
- [Theming](./docs/theming.md) — palette roles, knobs, writing a custom theme.
- [Components](./docs/components.md) — catalog with a snippet per widget.
- [Ejecting](./docs/eject.md) — when to use `ono add`, and the tradeoffs.

Reference rustdoc lands on [docs.rs/ono](https://docs.rs/ono) once v0.1.0 publishes.

## License

[MIT](./LICENSE) — NullOrder.
