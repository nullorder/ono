default:
    @just --list

# Run an experiment in the default (cyber) theme
experiment NAME:
    cargo experiments {{ NAME }}

# Run an experiment in a specific theme (cyber|retro|minimal|forest)
# Non-cyber themes are gated behind cargo features.
theme NAME THEME:
    @if [ "{{ THEME }}" = "cyber" ]; then \
        cargo run -p experiments --release --bin {{ NAME }} -- --theme cyber; \
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

# Dry-run a publish: shows what would go to crates.io without uploading
publish-check CRATE:
    cargo publish -p {{ CRATE }} --dry-run

# Publish a workspace crate to crates.io. Requires a clean tree on a tagged commit.
publish CRATE:
    @git diff --quiet || (echo "tree dirty — commit or stash first" && exit 1)
    @git describe --exact-match --tags HEAD >/dev/null 2>&1 || (echo "HEAD is not on a tag" && exit 1)
    cargo publish -p {{ CRATE }} --dry-run
    @echo "dry-run ok. publishing {{ CRATE }} in 5s — ctrl-c to abort."
    @sleep 5
    cargo publish -p {{ CRATE }}
