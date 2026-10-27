# Show available commands
default:
    @just --list

# Install/sync Python environment
sync:
    uv sync

# Run Python and Rust tests
test:
    uv run pytest
    cargo test --manifest-path rust/Cargo.toml

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

# Build Python distributions
build:
    uv build
