# AGENTS.md

Working notes for AI coding agents (Claude Code, Cursor, Pi, etc.) contributing to Ono. Read this before touching code.

## What Ono is

A framework-agnostic library of beautiful terminal UI components. Think shadcn / Aceternity, but for TUIs.

Two usage modes, both first-class:

- **Library (default):** `cargo add ono`, `use ono::components::splash::Splash;`. Themeable Ratatui widgets driven by typed builders.
- **Eject (power path):** `ono add <name>` copies the component's source into the user's tree. Use when the typed params aren't enough and you want to rewrite rendering. Ejected code has no runtime dependency on `ono`.

**Status:** early work in progress. No public release yet.

## Authoritative context

Internal planning — goals, roadmap, sprint tasks, aesthetic decisions, theming rules — lives under `plan/`. That directory is gitignored (public contributors don't see it, but agents working on this repo do). If anything below conflicts with a file in `plan/`, **the file in `plan/` wins**.

When in doubt about what to build next, or why a decision was made, read `plan/`. Start with `plan/ono-plan.md`.

## Model

Two top-level concerns in the shipped crate (`ono/src/`):

- **Components** — what the user copies. Two kinds:
  - **Elements** (`ono/src/elements/`) — atomic: `box`, `progress`, `spinner`, `percentage`, `sparkline`, `typewriter`. Cannot compose other components.
  - **Components** (`ono/src/components/`) — composite: `splash`, `boot`, `dashboard`, `statusbar`, `map`. Compose elements and/or other components.
- **Themes** (`ono/src/theme/`) — `Theme` enum + `Palette` (9 semantic roles) + `Knobs` (animation/behavior). Components refer to palette roles by name, never hex.

**Composition rule:** components compose elements and other components. Elements are atomic. Prevents cycles.

**Spec files** (`ono/specs/`) are the source of truth for each component's params, defaults, composition, and class→role mapping. Consumed by the engine for preview and, later, by codegen. They live inside the `ono` crate so they ship in the published binary (embedded at compile time).

## Repo layout

```
ono/
├── Cargo.toml              workspace; rust-version = 1.85
├── ono/                    the shipped crate (CLI + engine + component source)
│   ├── Cargo.toml          features: theme-retro, theme-minimal, theme-cyber, all-themes
│   └── src/
│       ├── main.rs         CLI entry (list, add, preview)
│       ├── theme/          Theme + Palette + Knobs; forest always-built, others feature-gated
│       ├── elements/       atomic components (Ratatui source)
│       ├── components/     composite components (Ratatui source)
│       ├── spec/           TOML parse + composition resolver
│       └── cli/            subcommand implementations
│   └── specs/                TOML source of truth (embedded at build time)
│       ├── elements/*.toml
│       └── components/*.toml
├── experiments/            prototype crate — scratchpad for new work before it graduates into ono/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          original Theme primitives (graduate into ono/src/theme/)
│       └── bin/            one binary per prototype
├── examples/
│   └── ratatui-demo/       example project consuming several components
├── site/                   Astro showcase
├── plan/                   internal planning (gitignored) — follow along there
├── .cargo/config.toml      `cargo experiments <name>` alias
├── justfile                dev recipes
├── README.md               public
├── LICENSE                 MIT, NullOrder
└── AGENTS.md               this file
```

The `experiments/` crate stays in the repo as a scratchpad — new explorations land there before graduating into `ono/`. Don't delete it. Don't refactor it into permanent infrastructure either.

## Commands

```sh
just experiment <name>          # run a prototype in forest (default)
just theme <name> <theme>       # run in forest|retro|minimal|cyber
just experiments                # list available
just all-themes                 # build with every theme enabled
just check                      # cargo check --workspace --features all-themes
just format                     # cargo fmt --all

cargo experiments <name>        # raw alias, same as `just experiment`
```

Always `just check` (or build with `--features all-themes`) before declaring a change done — the default build only compiles forest, so theme-gated regressions can hide.

## Coding rules

These are non-negotiable. Reviewers will reject code that violates them.

1. **No comments unless the WHY is non-obvious.** Don't explain WHAT the code does — names already do that. Don't reference the current task or fix.
2. **No emojis in code or files** unless explicitly requested.
3. **No backwards-compat shims for things you're removing.** Just delete.
4. **No new abstractions beyond what the task requires.** Three similar lines beats a premature trait.
5. **No hardcoded hex colors in component code.** Always go through `theme.palette().<role>`. Hardcoded hex in theme palette definitions is fine (that's where palettes live); anywhere else is a bug.
6. **No branching on theme identity for visual logic.** `if theme == Theme::Forest` is a code smell. Branch on knobs or palette roles: `if theme.knobs().scanline { ... }`. Branching on theme for non-visual structural choices (e.g., border type) is tolerable but prefer knobs when possible.
7. **No `unwrap()` on user-reachable code paths.** OK in `main` setup and prototype glue; not OK in render loops.
8. **Match Rust idioms.** snake_case for fns/vars, PascalCase for types, SCREAMING for consts. `cargo fmt` before committing.
9. **Concrete types over generics** when only one concrete type is used. Example: `Terminal<CrosstermBackend<io::Stdout>>` not `Terminal<B: Backend>`.
10. **Component source must work identically in library mode and eject mode.** Use relative `super::super::theme::...` imports so the same `.rs` file compiles both in the `ono` crate (library users) and under a vendored `src/ono/` tree (eject users). Do not `use ono::...` inside `ono/src/components/` or `ono/src/elements/` — that would break the eject path. Library users get the components via `use ono::components::...` externally; that's fine.

## Theme rules

Details in `plan/theming.md`. Quick reference:

- **Forest is canonical.** It's the only theme built and shipped by default. Retro, Minimal, and Cyber exist as feature-gated dev tools.
- **Adding a theme** requires: a new variant in `Theme`, a `Palette` constant, a `Knobs` constant, a `gradient()` arm, a `name()` arm — all behind `#[cfg(feature = "theme-<name>")]`.
- **Every theme must fill all 9 Palette roles and all Knob fields.** No `Option<Color>`. No per-component fallbacks.
- **Themes are not advertised** as a user-selectable feature in public messaging. Single-theme (forest) story.
- **`ono add` emits `theme.rs`** into the user's project with concrete hex for the chosen theme. First add writes it; subsequent adds reuse.

## Aesthetic constraints (forest)

Details in `plan/aesthetic-decision.md`. Quick reference:

- Palette: only the 9 canonical roles. No new hues without updating `Palette` for every theme.
- Animation: slow and breathing. No bounce/elastic easing. No scanline on forest.
- Two O's in "ono" = eyes. Keep them visually prominent in hero pieces. Subtle async pulse.
- Tagline "beautiful terminal UI components" pairs with the wordmark until the brand is established.

## Release discipline

- **Don't announce a release until it ships.** Repo can be public earlier; marketing waits for working code.
- **Ejected code must not import from `ono`.** Component source imports only the target framework and the user's own `theme.rs`. Library users get the same files via `use ono::components::...` — that's the library path and it's fine. Don't conflate the two.
- **Don't let generated code look generated.** When codegen lands, template output must read as idiomatic. Stop and fix templates if it doesn't.
- **Don't force cross-target universality.** Some components are Ratatui-only, some Textual-only. Document divergence honestly.

## Adding work

**A new experiment binary:**
1. Create `experiments/src/bin/<name>.rs`.
2. Use `Theme::parse_from_args()` to read `--theme`.
3. Pull colors from `theme.palette()`, behavior from `theme.knobs()`.
4. Conform to forest for default appearance; verify retro + minimal + cyber don't crash with `just all-themes`.
5. No new shared abstractions in `lib.rs` unless every existing experiment needs them.

**A new element or component in the real crate:**
1. Draft the spec: `ono/specs/elements/<name>.toml` or `ono/specs/components/<name>.toml`.
2. Hand-write the Ratatui source under `ono/src/elements/` or `ono/src/components/`.
3. Register in the component catalog.
4. Ensure it uses palette roles (not hex) and relative `super::super::theme` imports (so the same file compiles both under the `ono` crate and under an ejected `src/ono/` tree — no `use ono::...` inside the component file).
5. Verify `ono list` shows it; `ono preview <name>` renders it; `ono add <name>` copies a clean file.

**A new theme (rare — see `plan/theming.md` first):**
1. Add `theme-<name>` to the relevant `Cargo.toml` `[features]`.
2. Add the `Theme` variant + `Palette` const + `Knobs` const + match arms, all `#[cfg]`-gated.
3. Add `SPINNER_<NAME>` if needed.
4. Update `plan/theming.md` with the rationale.

**A new dev recipe:** add to `justfile`. Keep names lowercase, hyphenated. Brief comment line above each.

## Verification before declaring done

- `just check` passes (with `all-themes`).
- `cargo build --workspace --release` passes (default features).
- For UI changes: actually run the binary in a real terminal. Type-check passes ≠ visual correctness. If you can't run it (no TTY), say so explicitly — don't claim success.
- For new themes: run every experiment/component under that theme. None should panic.

## What NOT to do

- Don't create `*.md` planning docs outside `plan/` without asking.
- Don't add dependencies without checking if the existing ones cover the need.
- Don't introduce `clap` to experiments (`std::env::args()` is enough). The real CLI in `ono/` can use clap.
- Don't add `tokio` or async runtimes. Rendering is sync.
- Don't run `cargo update` casually — it churns `Cargo.lock`. Update deliberately, in its own commit.
- Don't push to `main` of `github.com/nullorder/ono` without explicit human ask.
- Don't post anything publicly on the project's behalf. Release posts go through Ani.

## When in doubt

Re-read `plan/ono-plan.md`. It's the brief. Most "should I…?" questions are answered there.
