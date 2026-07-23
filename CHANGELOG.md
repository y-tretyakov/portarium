# Changelog

## [0.6.0] — 2026-07-23 — Stage 2, 4, 5: TUI + CLI + Tests & CI

### Added (Stage 2 — portarium-tui Ratatui)
- `tui/src/` — Full Ratatui TUI application
- `action.rs` — Action enum + Screen enum (TEA pattern)
- `app.rs` — App state with PortariumService, port cache, events, traffic
- `ui.rs` — Screen rendering: Dashboard, Detail, Graph, HelpPopup
- `main.rs` — Async event loop (tokio + crossterm EventStream), map_key
- `tui.rs` — Terminal init/restore, panic hook

### Added (Stage 4 — CLI)
- `cli/src/` — Full CLI using clap
- `commands.rs` — list, watch, events, graph, traffic, kill, restart
- Formatted table output + JSON mode for all commands

### Added (Stage 5 — Tests, CI/CD, Documentation)
- **Core tests** (142 total):
  - `config.rs` — 7 tests: defaults, serialization, deserialization, roundtrip
  - `models.rs` — 15 tests: Protocol, PortInfo, PortEvent, EventType, GraphNode, GraphEdge, PortGraph, TrafficSample, Framework
  - `error.rs` — 8 tests: all Error variants (Display, Debug, From, Send, Sync)
  - `service.rs` — 7 tests: default, events, traffic, conflicts, config, ordering
  - `frameworks/` — 5 tests (including proptest)
  - `graph/` — 8 tests (including proptest for unique ports)
  - `logger/` — 12 tests (including proptest for arbitrary ports)
  - `scanner/` — 6 tests: find_project_root, extract_project_name
  - Integration tests — 7 tests: config_json_roundtrip, service_integration
- **TUI tests** (41 total):
  - `app.rs` — 21 tests: navigation, screens, enter/back, scan, error, selected
  - `main.rs` — 16 tests: map_key for all keys + unknown
  - `ui.rs` — 4 tests: centered_rect
- **CLI tests** (11 unit + 7 integration):
  - `commands.rs` — 9 tests: print tables with data/empty
  - `cli/tests/cli_integration.rs` — 7 tests: PortInfo, Event, Graph, Traffic
- **Property-based**: proptest for frameworks, logger, graph
- **CI/CD**: `ci.yml` — fmt + clippy + test (3 OS) + coverage
- **Release**: `release.yml` — build CLI + TUI on 3 platforms
- **docs/CODING_STANDARDS.md** — coding standards documented

### Changed
- `Justfile` — added test-tui, test-cli, test-verbose, cov-* commands
- `tui/Cargo.toml` — fixed dependencies

## [0.5.0] — 2026-07-22 — Core hardening + Tauri integration

### Added (Stage 0)
- Cargo workspace at project root (core + tui + src-tauri + cli)
- `core/` — new `portarium-core` crate with base structure
- `tui/` — new `portarium-tui` crate (scaffold for Ratatui)
- `cli/` — new `portarium-cli` crate (scaffold for CLI)
- `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml`
- `Justfile` with commands: build, test, lint, fmt, ci
- `AGENTS.md` — cheat-sheet for AI agents

### Added (Stage 1 — portarium-core)
- **`core/src/models.rs`** — PortInfo, PortEvent, EventType, TrafficSample, GraphNode, GraphEdge, PortGraph, Framework, Protocol
- **`core/src/error.rs`** — Base Error type (thiserror) + type Result
- **`core/src/config.rs`** — ScannerConfig, LoggerConfig, GraphConfig, PortariumConfig
- **`core/src/scanner/mod.rs`** — PortScanner (lsof/ss/netstat), kill/restart, find_project_root, extract_project_name
- **`core/src/logger/mod.rs`** — PortLogger: event tracking (started/stopped/conflict), traffic, first_seen
- **`core/src/graph/mod.rs`** — build_port_graph + get_active_connections (platform-specific)

### Changed (core — quality pass)
- **Scanner**: Extracted `ScannerBackend` trait + `UnixScanner`/`WindowsScanner` impls
- **Scanner**: Removed `eprintln!` — all errors returned via `Result`
- **Frameworks**: Split into `builtin.rs` + `registry.rs`. Added `FrameworkRegistry` with TOML extensibility
- **lib.rs**: Explicit model exports instead of `pub use models::*`, removed `pub use error::Result`
- **Deps**: chrono, once_cell, toml, sysinfo, libc moved to `[workspace.dependencies]`

### Changed (Stage 3 — Tauri Desktop → core)
- **src-tauri**: Removed duplicate files `scanner.rs`, `logger.rs`, `connections.rs` (~790 lines)
- **src-tauri**: Connected `portarium-core` as the sole backend
- **src-tauri/lib.rs**: `AppState` with `Arc<Mutex<PortariumService>>`, 6 Tauri commands — thin wrappers over core
- **src-tauri/tray.rs**: Uses `core::frameworks::is_dev_port()`, gets service via `Arc<Mutex<...>>`
- **src-tauri/Cargo.toml**: Removed sysinfo, libc, lazy_static (they are in core)
- **core/scanner**: Added `Send` bound on `trait ScannerBackend` (needed for Arc<Mutex<...>>)
- **`core/src/frameworks/mod.rs`** — FrameworkRegistry: dev port detection (Vite, React, etc.)
- **`core/src/service.rs`** — PortariumService facade (scan → log → graph)
- **`core/src/lib.rs`** — Public API via `pub use`
- 30 unit tests (including proptest) across all modules
- Proper error handling: no unwrap/expect in production code

### Changed
- `src-tauri/Cargo.toml` — included in workspace, uses workspace version/edition
- `core/Cargo.toml` — added dependencies: sysinfo, chrono, libc (unix), proptest + tempfile (dev)

### Fixed
- Clippy warnings in src-tauri (unused imports, dead_code, for_kv_map)
- Previous versions unchanged (code not modified)

## [0.3.2] — Previous release

- Tauri v2 desktop application
- TUI on OpenTUI (Bun + React)
- D3.js port graph
- System tray
- Cross-platform support
