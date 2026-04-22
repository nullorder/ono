# Ejecting to source

`ono add <name>` copies a component's source into your project. After
running it, your app imports the component from **your own tree**, not
from the `ono` crate. The library dependency is no longer needed at
runtime.

This page explains when to reach for that path, how the dual-tree design
works, and the tradeoffs.

## When to eject

**Stay on the library** if:

- The built-in builder methods cover your customization.
- You're fine receiving fixes + new features through `cargo update`.
- You want rustdoc and the compiler to guide you through the API.

**Eject** if:

- You want to rewrite the render logic beyond what parameters allow
  (change the layout algorithm, swap a sparkline for a custom chart,
  add domain-specific behavior).
- Your project has a policy against runtime dependencies on
  aspirational-aesthetic crates ("we want the code in our repo").
- You want to pin a specific visual exactly and never receive upstream
  changes to it.

Both modes are first-class. There is no "right" choice — they're
different tradeoffs. Many projects will mix them: stay on the library
for most things, eject the one component that needs heavy rework.

## How `ono add` works

```sh
cargo install ono            # one-time, to get the CLI
cd my-project
ono add splash               # or any element / component
```

What happens:

1. The CLI reads the embedded spec for `splash` and resolves its
   transitive dependencies (elements + sub-components it composes).
2. For each resolved item, it copies the source file (e.g.
   `components/splash.rs`, `elements/progress.rs`, …) into
   `./src/ono/` — creating `components/` and `elements/` subdirectories
   as needed.
3. On first add, it also writes `./src/ono/theme.rs` — a standalone copy
   of the `Palette` / `Knobs` / `Theme` module. You own this file.
4. It prints a `Next:` reminder to `mod ono;` from your crate root.

Subsequent `ono add` invocations reuse the existing `theme.rs`. Delete
it first if you want to regenerate with a different `--theme` flag.

```sh
ono add splash                  # default: forest
ono add splash --theme cyber    # only meaningful on first add
```

## The dual-tree design

The same `.rs` file lives in two different module trees without
modification:

- In the `ono` crate: `ono/src/components/splash.rs`
  imports `super::super::theme::Palette` → resolves to
  `ono::theme::Palette`.
- In an ejected project: `my-app/src/ono/components/splash.rs`
  imports `super::super::theme::Palette` → resolves to
  `my_app::ono::theme::Palette`.

Because the imports are **relative** (`super::super::…`) rather than
absolute (`use ono::theme::…`), they resolve correctly in whichever
tree the file sits in.

That's why `ono add` can be a straight file copy — no rewriting, no
patching. The consequence for contributors: component source files must
never say `use ono::…` internally. See rule 10 in `AGENTS.md`.

## What ends up in `src/ono/`

After `ono add splash` in a fresh project:

```
src/
├── main.rs
└── ono/
    ├── mod.rs
    ├── theme.rs
    └── components/
        └── splash.rs
```

After also adding `dashboard` and `statusbar`:

```
src/
├── main.rs
└── ono/
    ├── mod.rs
    ├── theme.rs
    ├── components/
    │   ├── mod.rs
    │   ├── dashboard.rs
    │   ├── splash.rs
    │   └── statusbar.rs
    └── elements/
        ├── mod.rs
        ├── percentage.rs    ← transitive dep of statusbar
        ├── progress.rs      ← transitive dep of statusbar
        └── spinner.rs       ← transitive dep of statusbar
```

Your crate root adds `mod ono;` and uses the components as modules:

```rust
mod ono;

use ono::components::splash::{Banner, Splash};
use ono::theme::Theme;
```

## Tradeoffs

| Concern | Library | Eject |
|---|---|---|
| Dependency footprint | `ono` as a crate dep | zero — after `ono add` |
| Get upstream fixes | `cargo update` | manual re-add and diff |
| Customize rendering | builder params only | edit the source |
| Rustdoc + IDE help | full | limited to your copy |
| Code review surface | small | whole component in your repo |
| Semver coverage | yes (from v0.1.0) | n/a — it's your code now |

## Working examples

- [`examples/ratatui-library/`](../examples/ratatui-library/) — library
  mode, no `src/ono/` tree.
- [`examples/ratatui-demo/`](../examples/ratatui-demo/) — eject mode,
  runs the exact same scenes from a vendored `src/ono/` tree.

The two examples render **the same components**; only the integration
style differs.
