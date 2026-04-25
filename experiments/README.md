# experiments

Scratchpad crate for prototyping new ono components.

This is where new visual and behavioural ideas get tried out before they earn a spot in the shipped `ono` crate. It exists for one reason: low-friction iteration without dragging the public API along for the ride.

> Not published to crates.io. Not part of ono's semver contract. Don't depend on it from outside this repo.

## Running an experiment

Requires Rust 1.85+ and [`just`](https://github.com/casey/just).

```sh
just experiments                 # list available experiments
just experiment <name>           # run in forest (default theme)
just theme <name> <theme>        # run in forest|retro|minimal|cyber
just all-themes                  # build every experiment with every theme enabled
```

`q`, `Esc`, or `Ctrl+C` to exit. Requires a true-color terminal.

The raw alias `cargo experiments <name>` is the same as `just experiment <name>`.

## Adding a new experiment

1. Create `src/bin/<name>.rs`. One binary per prototype — keep them self-contained.
2. Use `Theme::parse_from_args()` to read `--theme`.
3. Pull colors from `theme.palette()`, behavior from `theme.knobs()`. No hardcoded hex.
4. Make it look right under forest first; verify retro + minimal + cyber don't crash with `just all-themes`.
5. Don't add shared abstractions to `src/lib.rs` unless every existing experiment needs them. The scratchpad earns its keep by staying lightweight — duplication beats premature factoring here.

## Graduating an experiment into the library

Once an experiment proves its value visually and the API shape is clear:

1. Draft a spec under `ono/specs/elements/<name>.toml` or `ono/specs/components/<name>.toml`.
2. Hand-write the Ratatui source under `ono/src/elements/<name>.rs` or `ono/src/components/<name>.rs`. Use **relative imports** (`super::super::theme::...`) so the same `.rs` file works both as a library module in the `ono` crate and as a vendored copy under a user's `src/ono/` tree. Do **not** `use ono::...` inside component files.
3. Use palette roles, never hex literals.
4. Register the component in the catalog so `ono list` / `ono preview` / `ono add` see it.
5. Add rustdoc with a minimal usage example. The crate runs `#![warn(missing_docs)]`.
6. Verify: `ono list` shows it, `ono preview <name>` renders it, `ono add <name>` copies a clean file.
7. Add a `CHANGELOG.md` entry under the next version.

Leave the experiment binary in place after graduation — it's a useful regression check.

## Why a scratchpad?

The shipped `ono` crate is bound by semver from v0.1.0 onward. Adding a palette role, renaming a builder, changing a default — those are breaking changes. The experiments crate has none of that overhead: try a thing, throw it away, try a different thing. By the time something graduates into `ono/`, the shape has settled.
