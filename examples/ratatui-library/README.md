# ratatui-library

Same scenes as `examples/ratatui-demo/`, but integrated the other way:
`ono` is a normal `cargo` dependency and components are imported from
the crate. Nothing is vendored.

## The difference

| | `ratatui-library/` (this crate) | `ratatui-demo/` |
|---|---|---|
| Integration | `ono = { path = "../../ono" }` | `ono add` → vendored `src/ono/` |
| Imports | `use ono::components::splash::*;` | `mod ono;` then `use ono::...` |
| Theme | `ono::theme::Theme::Forest` | local `theme.rs` owned by the user |
| Updates | bump the `ono` version | re-run `ono add` or edit in place |
| Customization | swap builder params, supply a custom `Palette` | rewrite the source |

Pick library mode when the builder surface covers your needs. Eject
when you want to own the rendering code.

## Run it

```
cargo run -p ratatui-library
```

Splash plays for 3 s, boot for ~7 s, then a live dashboard runs until
you press `q`, `Esc`, or `Ctrl+C`.
