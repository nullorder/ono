# Ono

Beautiful terminal UI components. Copy-paste, framework-agnostic, code you own.

> Early work in progress. The CLI isn't published yet. Prototypes run locally via the `experiments/` crate; the shipped CLI and component catalog are in active development under `ono/`.

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

Two concerns:

- **Components** — what users copy into their project. Split into **elements** (atomic: `progress`, `spinner`, `box`, ...) and **components** (composite: `splash`, `boot`, `dashboard`, `statusbar`, ...).
- **Themes** — palette of semantic roles + animation knobs. Forest is canonical; others behind cargo features.

When the CLI lands, `ono add <name>` will copy the component source plus a small `theme.rs` you own. No runtime dependency on `ono`.

Ratatui is the first target. Textual, Bubble Tea, and Ink are on the way.

## License

[MIT](./LICENSE) — NullOrder.
