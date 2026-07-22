# Portarium — just command runner
# Use: `just <command>`

# ── Build ──────────────────────────────────────────────────────────────────────

# Build all workspace crates
build:
    cargo build --workspace

# Build only core library
core:
    cargo build -p portarium-core

# Build TUI binary
tui:
    cargo build -p portarium-tui

# Build CLI binary
cli:
    cargo build -p portarium

# Build Tauri desktop (requires npm deps)
tauri:
    cargo build -p portarium --manifest-path src-tauri/Cargo.toml

# ── Test ───────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test --workspace

# Test only core
test-core:
    cargo test -p portarium-core

# ── Lint / Format ──────────────────────────────────────────────────────────────

# Run clippy on all targets
lint:
    cargo clippy --all-targets -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting without changes
fmt-check:
    cargo fmt --all -- --check

# ── Full CI check ──────────────────────────────────────────────────────────────

# Run format check + lint + test
ci: fmt-check lint test

# ── Help ───────────────────────────────────────────────────────────────────────

# Show available commands
default:
    @just --list
