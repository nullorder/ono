default:
    @just --list

# Run an experiment in the default (retro) theme
experiment NAME:
    cargo experiments {{ NAME }}

# Run an experiment in a specific theme (retro|minimal|cyber)
# Non-retro themes are gated behind cargo features.
theme NAME THEME:
    @if [ "{{ THEME }}" = "retro" ]; then \
        cargo run -p experiments --release --bin {{ NAME }} -- --theme retro; \
    else \
        cargo run -p experiments --release --features theme-{{ THEME }} --bin {{ NAME }} -- --theme {{ THEME }}; \
    fi

# Build with every theme enabled (dev comparison)
all-themes:
    cargo build --workspace --features all-themes --release

# List available experiments
experiments:
    @ls experiments/src/bin/ | sed 's/\.rs$//'

check:
    cargo check --workspace --features all-themes

format:
    cargo fmt --all
