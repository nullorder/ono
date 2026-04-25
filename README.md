# Ono

[![Crates.io](https://img.shields.io/crates/v/ono.svg)](https://crates.io/crates/ono)
[![Docs.rs](https://img.shields.io/docsrs/ono)](https://docs.rs/ono)
[![Lib.rs](https://img.shields.io/badge/lib.rs-ono-blue?logo=rust)](https://lib.rs/crates/ono)
[![License: MIT](https://img.shields.io/crates/l/ono.svg)](./LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.85%2B-blue.svg)](https://www.rust-lang.org/)
[![Downloads](https://img.shields.io/crates/d/ono.svg)](https://crates.io/crates/ono)
[![Open Source](https://img.shields.io/badge/open%20source-%E2%9D%A4-red)](https://github.com/nullorder/ono)

Beautiful terminal UI components for [Ratatui](https://github.com/ratatui/ratatui). Themeable widgets you can use as a library or eject into your tree as plain source.

The shipped Rust crate lives at [`ono/`](./ono/) — see [`ono/README.md`](./ono/README.md) for install + usage, or [crates.io/crates/ono](https://crates.io/crates/ono).

## Two ways to use it

- **Library** (default) — add the crate, `use ono::components::...`, drive components with typed builders and a `Theme`.
- **Eject** (power path) — `ono add <name>` copies the component's source into your tree, no runtime dependency on ono after that. Use it when you want to rewrite rendering beyond what builder params expose.

## Model

- **Components** — themeable widgets. Split into **elements** (atomic: `progress`, `spinner`, `box`, ...) and **components** (composite: `splash`, `boot`, `dashboard`, `statusbar`, ...).
- **Themes** — palette of nine semantic roles + animation knobs. Ono ships four built-in themes (Forest, Retro, Minimal, Cyber); Forest is the default. Custom themes are just a `Palette` + `Knobs` you construct yourself.

## Documentation

- [Getting started](./docs/getting-started.md) — five-minute walkthrough.
- [Theming](./docs/theming.md) — palette roles, knobs, custom themes.
- [Components](./docs/components.md) — catalog with a snippet per widget.
- [Ejecting](./docs/eject.md) — when to use the eject path, and the tradeoffs.

API reference: [docs.rs/ono](https://docs.rs/ono).

## Contributing

Bug reports, feature requests, and patches welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md) for dev setup and how new components graduate from the experiments scratchpad into the library.

## License

[MIT](./LICENSE) — NullOrder.
