# ono

Beautiful terminal UI components for Ratatui. Use as a library, or eject the source into your tree.

## Library usage (default)

```toml
[dependencies]
ono = "0.1"
ratatui = "0.30"
```

```rust
use ono::components::boot::Boot;
use ono::theme::Theme;

let theme = Theme::Forest;
// in your render loop:
Boot::new(theme.palette(), theme.knobs())
    .elapsed(elapsed)
    .render(area, buf);
```

Components are Ratatui widgets parameterized by typed builders and a `Theme` (palette + animation knobs). Covered by semver from v0.1.0: everything under `ono::components`, `ono::elements`, `ono::theme`.

## CLI usage (helper)

```sh
cargo install ono
ono list                    # browse the catalog
ono preview dashboard       # render live in your terminal
ono add splash              # eject source into ./src/ono/ (no runtime dep on ono after this)
```

`ono add` is the power-user eject path — use it when you want to rewrite rendering logic beyond what builder params allow. Ejected code imports only Ratatui and your own `theme.rs`.

## More

- Homepage + source: <https://github.com/nullorder/ono>
- Narrative docs (getting started, theming, components, ejecting):
  <https://github.com/nullorder/ono/tree/main/docs>

## License

[MIT](./LICENSE)
