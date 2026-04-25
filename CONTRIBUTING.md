# Contributing to Ono

Thanks for considering a contribution. Ono is a library of themeable terminal UI components for Ratatui.

This doc covers everything you need to hack on the crate itself. If you just want to *use* ono, the [README](./README.md) is the right starting point.

## Ground rules

- **Open an issue before a large PR.** Small fixes (bugs, docs, typos) — send the PR. Anything that adds API surface, changes a palette role, or introduces a new component should start as an issue so we can align before code is written.
- **One change per PR.** Refactors, features, and unrelated cleanup go in separate PRs.

## Dev setup

Requires Rust **1.85+** and [`just`](https://github.com/casey/just) for the dev recipes.

```sh
git clone git@github.com:nullorder/ono.git
cd ono
just check          # cargo check --workspace --features all-themes
just format         # cargo fmt --all
```

Always run `just check` before opening a PR — the default build only compiles the forest theme, so theme-gated regressions can hide.

## Experiments — the scratchpad

New components start their life in [`experiments/`](./experiments/), not in the shipped crate. The scratchpad has no semver contract, so it's the right place to iterate on visual and behavioural ideas before they earn a spot in `ono/`.

See [`experiments/README.md`](./experiments/README.md) for how to run experiments, add a new one, and graduate it into the library.

## Adding a new theme

Ono supports multiple built-in themes. Forest is the default and is always built; Retro, Minimal, and Cyber live behind cargo features (`theme-retro`, `theme-minimal`, `theme-cyber`) so users only pay for what they use.

To add another:

1. Add `theme-<name>` to the relevant `Cargo.toml` `[features]`, and include it in the `all-themes` aggregate.
2. Add the `Theme` variant + `Palette` const + `Knobs` const + match arms, all `#[cfg(feature = "theme-<name>")]`-gated.
3. Every theme must fill **all 9 palette roles and all knob fields**. No `Option<Color>`. No per-component fallbacks.
4. Verify every experiment and component renders under the new theme: `just all-themes`.

Users who need a one-off look should write a custom `Palette` + `Knobs` in their own code rather than land it upstream — see [theming docs](./docs/theming.md). Upstream new themes when the look is broadly useful and the palette holds up across the full component catalog.

## Submitting a change

1. Fork and branch from `main`.
2. Run `just check` and `cargo fmt --all`. CI checks both.
3. For UI-affecting changes, *actually run* the affected experiment / preview in a real terminal. Type-check passes ≠ visual correctness.
4. Update [`CHANGELOG.md`](./CHANGELOG.md) under the next version's section. New components, palette role changes, and breaking API changes always go in the changelog.
5. Open a PR with a clear description of *what* changed and *why*. Reference any related issue.

### Commit style

- Imperative mood (`add splash component`, not `added` / `adds`).
- Conventional prefixes are welcome but not required: `feat(...)`, `fix(...)`, `docs(...)`, `refactor(...)`.
- Keep commits focused. Squash trivial fixups before opening the PR.

## Semver

Starting at v0.1.0, breaking changes to any `pub` item under `ono::theme`, `ono::elements`, or `ono::components` require a minor-version bump (major once we hit 1.0). Adding a palette role or a knob field counts as breaking — `Palette` and `Knobs` are not `#[non_exhaustive]` because ejected `theme.rs` files construct them with struct literals.

The internal engine (`ono::spec`, `ono::cli`) is **not** covered by semver.

## Reporting bugs

Open an issue at https://github.com/nullorder/ono/issues with:

- A minimal repro (ideally a small `cargo run` snippet).
- Terminal + OS + Rust version.
- What you expected vs what you got. Screenshots or `vhs` recordings help for visual bugs.

## License

By contributing, you agree your contributions will be licensed under the [MIT License](./LICENSE).
