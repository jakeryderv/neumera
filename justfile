# --- General --------------------------------------------------

# List available commands
default:
    @just --list

# One-time setup: install dependencies, build the extension, enable git hooks
setup: hooks dev

# Sync Python dependencies
sync:
    uv sync

# Enable the repo's git hooks (pre-commit runs fmt-check and lint)
hooks:
    git config core.hooksPath .githooks

# Rebuild and reinstall the Rust extension, regenerating type stubs
dev:
    uv sync --no-install-project
    uv run --no-sync maturin develop --uv --generate-stubs


# --- Testing --------------------------------------------------

# Run Python and Rust tests
test: dev
    uv run pytest
    cargo test --manifest-path rust/Cargo.toml


# --- Code quality ---------------------------------------------

# Lint Python and Rust
lint:
    uv lock --check
    uv run ruff check .
    uv run ty check
    cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings

# Check formatting without modifying files
fmt-check:
    uv run ruff format --check .
    cargo fmt --manifest-path rust/Cargo.toml --all -- --check

# Format Python and Rust
fmt:
    uv run ruff format .
    cargo fmt --manifest-path rust/Cargo.toml --all

# Run all checks
check: dev fmt-check lint test


# --- Building -------------------------------------------------

# Build Python source and wheel distributions
build:
    uv build


# --- Releasing ------------------------------------------------

# Bump the version, run checks, then commit and tag locally (pushing is manual)
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -n "$(git status --porcelain)" ]]; then echo "error: working tree is not clean" >&2; exit 1; fi
    if [[ "$(git branch --show-current)" != main ]]; then echo "error: not on main" >&2; exit 1; fi
    if git rev-parse -q --verify "refs/tags/v{{version}}" >/dev/null; then echo "error: tag v{{version}} already exists" >&2; exit 1; fi
    sed -i '0,/^version = ".*"$/s//version = "{{version}}"/' rust/Cargo.toml
    just check
    git add rust/Cargo.toml rust/Cargo.lock
    git commit -m "chore: release {{version}}"
    git tag "v{{version}}"
    echo "Tagged v{{version}}. To publish: git push origin main && git push origin v{{version}}"
