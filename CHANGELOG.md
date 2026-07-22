# Changelog

## [0.5.0] — 2026-07-22 — Core hardening + Tauri integration

### Added (Этап 0)
- Cargo workspace в корне проекта (core + tui + src-tauri + cli)
- `core/` — новый crate `portarium-core` с базовой структурой
- `tui/` — новый crate `portarium-tui` (заготовка для Ratatui)
- `cli/` — новый crate `portarium-cli` (заготовка для CLI)
- `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml`
- `Justfile` с командами: build, test, lint, fmt, ci
- `AGENTS.md` — памятка для AI-агента

### Added (Этап 1 — portarium-core)
- **`core/src/models.rs`** — PortInfo, PortEvent, EventType, TrafficSample, GraphNode, GraphEdge, PortGraph, Framework, Protocol
- **`core/src/error.rs`** — Базовый тип Error (thiserror) + type Result
- **`core/src/config.rs`** — ScannerConfig, LoggerConfig, GraphConfig, PortariumConfig
- **`core/src/scanner/mod.rs`** — PortScanner (lsof/ss/netstat), kill/restart, find_project_root, extract_project_name
- **`core/src/logger/mod.rs`** — PortLogger: трекинг событий (started/stopped/conflict), трафик, first_seen
- **`core/src/graph/mod.rs`** — build_port_graph + get_active_connections (platform-specific)

### Changed (core — quality pass)
- **Scanner**: Выделен `ScannerBackend` trait + `UnixScanner`/`WindowsScanner` impl'ы
- **Scanner**: Убраны `eprintln!` — все ошибки возвращаются через `Result`
- **Frameworks**: Разбит на `builtin.rs` + `registry.rs`. Добавлен `FrameworkRegistry` с TOML-расширяемостью
- **lib.rs**: Явный экспорт моделей вместо `pub use models::*`, убран `pub use error::Result`
- **Deps**: chrono, once_cell, toml, sysinfo, libc вынесены в `[workspace.dependencies]`

### Changed (Этап 3 — Tauri Desktop → core)
- **src-tauri**: Удалены файлы-дубликаты `scanner.rs`, `logger.rs`, `connections.rs` (~790 строк)
- **src-tauri**: Подключён `portarium-core` как единственный бэкенд
- **src-tauri/lib.rs**: `AppState` с `Arc<Mutex<PortariumService>>`, 6 Tauri команд — тонкие обёртки над core
- **src-tauri/tray.rs**: Использует `core::frameworks::is_dev_port()`, получает service через `Arc<Mutex<...>>`
- **src-tauri/Cargo.toml**: Убраны sysinfo, libc, lazy_static (они в core)
- **core/scanner**: Добавлен `Send` bound на `trait ScannerBackend` (нужен для Arc<Mutex<...>>)
- **`core/src/frameworks/mod.rs`** — FrameworkRegistry: detection dev-портов (Vite, React, etc.)
- **`core/src/service.rs`** — PortariumService facade (scan → log → graph)
- **`core/src/lib.rs`** — Публичное API через `pub use`
- 30 unit-тестов (включая proptest) на все модули
- Правильная обработка ошибок: никаких unwrap/expect в production-коде

### Changed
- `src-tauri/Cargo.toml` — включён в workspace, использует workspace version/edition
- `core/Cargo.toml` — добавлены зависимости: sysinfo, chrono, libc (unix), proptest + tempfile (dev)

### Fixed
- Clippy warnings в src-tauri (unused imports, dead_code, for_kv_map)
- Предыдущие версии без изменений (код не трогали)

## [0.3.2] — Предыдущий релиз

- Tauri v2 desktop приложение
- TUI на OpenTUI (Bun + React)
- D3.js граф портов
- Системный трей
- Кроссплатформенная поддержка
