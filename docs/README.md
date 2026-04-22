# Ono docs

Library-first documentation for [`ono`](https://github.com/nullorder/ono)
— beautiful terminal UI components for Ratatui.

| Page | For when you want to … |
|---|---|
| [Getting started](./getting-started.md) | add `ono` to a project and render your first component in five minutes. |
| [Theming](./theming.md) | understand `Theme` / `Palette` / `Knobs`, the nine palette roles, and how to build a custom theme. |
| [Components](./components.md) | browse every element and component with a minimal usage snippet. |
| [Ejecting](./eject.md) | understand when to reach for `ono add`, how the dual-tree design works, and the tradeoffs vs staying on the library. |

## Rustdoc

Every public item on the library surface has rustdoc with a minimal
usage example. The rustdoc catalog is the reference; these pages are
the narrative.

- Online (after v0.1.0 publishes): [docs.rs/ono](https://docs.rs/ono)
- Locally: `cargo doc -p ono --open`

## Reading order

If you're new: **getting-started → components → theming → eject**.

If you already know Ratatui and just want to add a widget: jump to
[components](./components.md), copy the snippet, adjust.

If you're evaluating whether to take the eject path: read
[ejecting](./eject.md) first.

## See also

- [Root README](../README.md) — project pitch and cross-framework roadmap.
- [`examples/ratatui-library/`](../examples/ratatui-library/) —
  library-mode integration.
- [`examples/ratatui-demo/`](../examples/ratatui-demo/) — eject-mode
  integration.
- [CHANGELOG](../CHANGELOG.md) — what's covered by semver starting at
  v0.1.0.
