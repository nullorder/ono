# Changelog

All notable changes to **ono** (the crate at `ono/`) are recorded here. The
format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
from v0.1.0 onward.

## [0.1.0] — 2026-04-25

First public release. Library surface and CLI behaviour below are the
semver-locked v0.1.0 contract.

### Added

- Library-first crate exposing themeable Ratatui widgets:
  - Elements: `BoxFrame`, `Percentage`, `Progress` (+ `ProgressStyle`),
    `Sparkline`, `Spinner`, `Typewriter`.
  - Components: `Boot` (+ `Step`, `StepOutcome`, `DEFAULT_STEPS`),
    `Dashboard` (+ `DashboardState`), `WorldMap` (+ `MapState`),
    `Splash` (+ `Banner`), `Statusbar`.
- `Theme` enum with `Forest` as the default, always built; `Retro`, `Minimal`, and
  `Cyber` behind `theme-retro` / `theme-minimal` / `theme-cyber` features
  (aggregate `all-themes`).
- Nine-role `Palette`: `bg`, `surface`, `border`, `dim`, `primary`, `bright`,
  `accent`, `secondary`, `warn`.
- `Knobs` animation + behavior struct fed into components alongside the palette.
- CLI (`ono`):
  - `ono list` — print the catalog grouped by ELEMENTS / COMPONENTS.
  - `ono preview <name>` — render a component into the current terminal.
  - `ono add <name> [--theme forest|retro|minimal|cyber]` — file-copy eject
    of a component's source plus its transitive deps and a `theme.rs` you own.
- Two examples:
  - `examples/ratatui-library/` — library-path integration.
  - `examples/ratatui-demo/` — ejected source tree integration.
- Rustdoc on every public item; crate-level landing doc with a quick-start
  snippet; `#![warn(missing_docs)]` enabled.

### Changed

- `Palette.accent2` → `Palette.secondary`. This rename lands *before* v0.1.0
  ships — once tagged, palette role names are semver-locked (users hand-edit
  `theme.rs` against them after ejecting).

### Removed

- Dead helpers leaked from the `experiments/` scratchpad: `Theme::gradient`,
  `GLITCH_CHARS`, `rgb`, `color_rgb`, `Xorshift::unit`, and duplicate spinner
  constants in `elements::spinner` (only the in-file default remains,
  private).

### Internal

Not covered by semver:

- `ono::spec` — spec parsing / resolver / validator. Now `pub(crate)`;
  reserved for the v0.2.0 codegen path.
- `ono::cli` — CLI entry points. Kept `pub` because the `ono` binary depends
  on the library, but `#[doc(hidden)]` so it doesn't appear on docs.rs.

### Semver coverage

Starting with v0.1.0, breaking changes to any `pub` item under
`ono::theme`, `ono::elements`, or `ono::components` require a minor-version
bump (major once we reach 1.0). Adding a palette role or a knob field is
considered a breaking change — `Palette` and `Knobs` are not
`#[non_exhaustive]` because ejected `theme.rs` files construct them with
struct literals.
