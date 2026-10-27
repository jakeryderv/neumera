# --- General --------------------------------------------------

# List available commands
default:
    @just --list

# Sync Python dependencies
sync:
    uv sync

# Rebuild and reinstall the Rust extension
dev:
    uv sync --reinstall-package neumera


# --- Testing --------------------------------------------------

# Run Python and Rust tests
test: dev
    uv run pytest
    cargo test --manifest-path rust/Cargo.toml


# --- Code quality ---------------------------------------------

# Lint Python and Rust
lint:
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
check: fmt-check lint test


# --- Building -------------------------------------------------

# Build Python source and wheel distributions
build:
    uv build
