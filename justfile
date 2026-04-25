default:
    @just --list

# Run an experiment in the default (forest) theme
experiment NAME:
    cargo experiments {{ NAME }}

# Run an experiment in a specific theme (forest|retro|minimal|cyber)
# Non-forest themes are gated behind cargo features.
theme NAME THEME:
    @if [ "{{ THEME }}" = "forest" ]; then \
        cargo run -p experiments --release --bin {{ NAME }} -- --theme forest; \
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

# Override the port with `just docs 8080` if 4590 is taken.
# Build rustdoc for `ono` and serve it locally. Ctrl-C to stop.
docs PORT='4590':
    cargo doc -p ono --features all-themes --no-deps
    @echo ""
    @echo "→ http://localhost:{{ PORT }}/ono/"
    @echo "  (Ctrl-C to stop)"
    @echo ""
    python3 -m http.server {{ PORT }} -d target/doc --bind 127.0.0.1

# Dry-run a publish: shows what would go to crates.io without uploading
publish-check:
    cargo publish -p ono --dry-run

# Tag v<workspace-version> and push to origin (idempotent).
gh-tag:
    #!/usr/bin/env bash
    set -euo pipefail
    git diff --quiet || { echo "tree dirty — commit or stash first"; exit 1; }
    VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed -E 's/version = "(.+)"/\1/')
    TAG="v$VERSION"
    if git rev-parse "$TAG" >/dev/null 2>&1; then
        echo "tag $TAG already exists locally"
    else
        git tag -a "$TAG" -m "Release $TAG"
        echo "tagged $TAG"
    fi
    git push origin "$TAG"

# Cut a GitHub release for the current version (prompts for title).
gh-publish:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed -E 's/version = "(.+)"/\1/')
    TAG="v$VERSION"
    printf "Release title (default: %s): " "$TAG" > /dev/tty
    read -r TITLE < /dev/tty
    TITLE="${TITLE:-$TAG}"
    gh release create "$TAG" --title "$TITLE" --generate-notes

# Upload ono to crates.io (5s abort window after dry-run).
crate-publish:
    @git describe --exact-match --tags HEAD >/dev/null 2>&1 || (echo "HEAD is not on a tag" && exit 1)
    cargo publish -p ono --dry-run
    @echo "dry-run ok. publishing ono in 5s — ctrl-c to abort."
    @sleep 5
    cargo publish -p ono

# Full release flow: tag, crates.io, GitHub release.
publish: gh-tag crate-publish gh-publish
