# AGENTS.md

Working notes for AI coding agents (Claude Code, Cursor, Pi, etc.) contributing to Ono. Read this before touching code.

## What Ono is

A framework-agnostic registry of beautiful, copy-paste terminal UI components. Think shadcn / Aceternity, but for TUIs. Users run a CLI, pick a component, get idiomatic source dropped into their project — code they own, not a runtime dependency.

**Status:** Phase 0 (prototype validation). No public release yet.

## Authoritative context

If anything below conflicts with these files, **the files win**:

| File | Purpose |
|---|---|
| `plan/ono-plan.md` | The original brief. Source of truth for goals, phases, hard rules. |
| `plan/roadmap.md` | Version-by-version plan. Where we are, what ships when. |
| `plan/aesthetic-decision.md` | Locked palette + animation parameters. Do not deviate. |
| `plan/theming.md` | Theme system rules + roadmap position. |
| `plan/v0.0.4-tasks.md` | Current sprint task list. |

The `plan/` directory is gitignored — public contributors don't see it, but agents working on the repo do. Treat it as load-bearing.

## Repo layout

```
ono/
├── Cargo.toml              workspace; rust-version = 1.85
├── experiments/            Phase 0 prototype crate
│   ├── Cargo.toml          features: theme-retro, theme-minimal, theme-forest, all-themes
│   ├── src/lib.rs          Theme + Palette + Knobs (the theming primitive)
│   └── src/bin/            one binary per prototype
├── plan/                   internal planning (gitignored)
├── .cargo/config.toml      `cargo experiments <name>` alias
├── justfile                dev recipes
├── README.md               public stub
├── LICENSE                 MIT, NullOrder
└── AGENTS.md               this file
```

The `experiments/` crate is **deliberately ephemeral** — it gets deleted when Phase 1 (v0.1.0) starts and the real engine + spec system replaces it. Don't refactor experiments into permanent infrastructure.

## Commands

```sh
just experiment <name>          # run a prototype in cyber (default)
just theme <name> <theme>       # run in cyber|retro|minimal|forest
just experiments                # list available
just all-themes                 # build with every theme enabled
just check                      # cargo check --workspace --features all-themes
just format                     # cargo fmt --all

cargo experiments <name>        # raw alias, same as `just experiment`
```

Always `just check` (or build with `--features all-themes`) before declaring a change done — the default build only compiles the cyber theme, so theme-gated regressions can hide.

## Coding rules

These are non-negotiable. Reviewers will reject code that violates them.

1. **No comments unless the WHY is non-obvious.** Don't explain WHAT the code does — names already do that. Don't reference the current task or fix.
2. **No emojis in code or files** unless explicitly requested.
3. **No backwards-compat shims for things you're removing.** Just delete.
4. **No new abstractions beyond what the task requires.** Three similar lines beats a premature trait.
5. **No hardcoded hex colors in component code.** Always go through `theme.palette().<role>`. Hardcoded hex in `experiments/src/lib.rs` is fine (that's where palettes live); anywhere else is a bug.
6. **No branching on theme identity for visual logic.** `if theme == Theme::Cyber` is a code smell. Branch on knobs or palette roles: `if theme.knobs().scanline { ... }`.
7. **No `unwrap()` on user-reachable code paths.** OK in `main` setup and prototype glue; not OK in render loops.
8. **Match Rust idioms.** snake_case for fns/vars, PascalCase for types, SCREAMING for consts. `cargo fmt` before committing.
9. **Concrete types over generics** when only one concrete type is used. Example: `Terminal<CrosstermBackend<io::Stdout>>` not `Terminal<B: Backend>`.

## Theme rules

See `plan/theming.md` for the full decision. Quick reference:

- **Cyber is canonical.** It's the only theme enabled in default builds. Retro, Minimal, and Forest exist as feature-gated dev tools.
- **Adding a theme** requires: a new variant in `Theme`, a `Palette` constant, a `Knobs` constant, a `gradient()` arm, a `name()` arm — all behind `#[cfg(feature = "theme-<name>")]`. Update `plan/theming.md`.
- **Every theme must fill all 9 Palette roles and all Knob fields.** No `Option<Color>`. No per-component fallbacks.
- **Themes are not advertised** as a user feature in v1.0. Public messaging stays single-theme.

## Aesthetic constraints (retro)

When working on retro components, conform to `plan/aesthetic-decision.md`:

- Palette: only the 6 canonical colors. No new hues.
- Animation: 30 FPS, 4–6s gradient sweeps, no bounce/elastic easing.
- Two O's in "ono" = eyes. Keep them visually prominent in hero pieces.
- Tagline "beautiful terminal UI components" pairs with the wordmark.

## Phase discipline

From `plan/ono-plan.md` — these are hard rules to enforce against future selves:

1. **Don't skip Phase 0.** No infrastructure (specs, engine, CLI, site, codegen) until v0.0.4 ships and the validation post hits its kill criterion.
2. **Don't announce until v0.1.0.** Repo can be public earlier, but no marketing push until the MVP works.
3. **Don't ship `ono` as a runtime import in generated code.** Generated component code imports only the target framework + the user's utils.
4. **Don't let generated code look generated.** When codegen lands (v0.2.0), template output must read as idiomatic. Stop and fix templates if it doesn't.
5. **Don't force cross-target universality.** Some components are Ratatui-only, some Textual-only. That's fine. Don't bend specs to make every component everywhere.

## Adding work

**A new experiment binary:**
1. Create `experiments/src/bin/<name>.rs`.
2. Use `Theme::parse_from_args()` to read `--theme`.
3. Pull colors from `theme.palette()`, behavior from `theme.knobs()`.
4. Conform to cyber for default appearance; verify retro + minimal + forest don't crash with `just all-themes`.
5. No new shared abstractions in `lib.rs` unless every existing experiment needs them.

**A new theme (rare — see `plan/theming.md` first):**
1. Add `theme-<name>` to `experiments/Cargo.toml` `[features]`.
2. Add the `Theme` variant + `Palette` const + `Knobs` const + match arms in `lib.rs`, all `#[cfg]`-gated.
3. Add `SPINNER_<NAME>` if needed.
4. Update `plan/theming.md` with the rationale.

**A new dev recipe:** add to `justfile`. Keep names lowercase, hyphenated. Brief comment line above each.

## Verification before declaring done

- `just check` passes (with `all-themes`).
- `cargo build --workspace --release` passes (default features).
- For UI changes: actually run the binary in a real terminal. Type-check passes ≠ visual correctness. If you can't run it (no TTY), say so explicitly — don't claim success.
- For new themes: run all three experiments under that theme. None should panic.

## What NOT to do

- Don't create `*.md` planning docs outside `plan/` without asking.
- Don't add dependencies without checking if the existing ones cover the need.
- Don't introduce `clap` or any CLI parser to experiments (`std::env::args()` is enough; the real CLI is a Phase 1 concern).
- Don't add `tokio` or async runtimes. Phase 0 is sync only.
- Don't run `cargo update` casually — it churns `Cargo.lock`. Update deliberately, in its own commit.
- Don't write a CHANGELOG yet. That starts at v0.1.0.
- Don't push to `main` of `github.com/nullorder/ono` without explicit human ask.
- Don't post anything publicly on the project's behalf. Validation posts go through Ani.

## When in doubt

Re-read `plan/ono-plan.md`. It's the brief. Most "should I…?" questions are answered there.
