# Portarium — Состояние проекта

**Последнее обновление:** 22 июля 2026

## Целевая архитектура (WIP)

```
portarium/
├── core/                  # Rust library — вся бизнес-логика [✅ готово]
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs         # Публичное API (явный экспорт)
│       ├── config.rs      # ScannerConfig, LoggerConfig, PortariumConfig
│       ├── error.rs       # Error enum (thiserror)
│       ├── models.rs      # PortInfo, PortEvent, GraphNode, Framework...
│       ├── scanner/       # PortScanner + ScannerBackend trait + impl'ы
│       ├── logger/        # PortLogger — события + трафик
│       ├── graph/         # PortGraph — построение графа соединений
│       ├── frameworks/    # FrameworkRegistry: TOML-расширяемый + builtin
│       └── service.rs     # PortariumService — главный фасад (Result-based)
├── tui/                   # Ratatui TUI [✅ готово]
├── src-tauri/             # Tauri Desktop (использует core) [✅ готово]
├── cli/                   # CLI [✅ готово]
└── Cargo.toml             # workspace
```

## Что сделано

### core — production-ready
- `ScannerBackend` trait + `UnixScanner` / `WindowsScanner` impl'ы
- Отсутствие `eprintln!` в lib-коде (все ошибки через `Result`)
- `FrameworkRegistry` с TOML-расширяемостью + builtin-списком
- Чистый pub API (явные экспорты, без wildcard)
- Workspace dependencies для chrono, sysinfo, libc, once_cell, toml
- 37 unit-тестов (включая proptest для frameworks)
- Clippy-clean (0 warnings)

### src-tauri — интегрирован с core
- scanner.rs, logger.rs, connections.rs — удалены (были дубликатами core)
- Использует `portarium_core::PortariumService` через `tauri::State<AppState>`
- `tray.rs` использует `core::frameworks::is_dev_port()` вместо inline-списка
- `AppState` — `Arc<Mutex<PortariumService>>`, проброшен в tray thread

## Этапы

| Этап | Статус |
|------|--------|
| 0 — Инфраструктура | ✅ |
| 1 — portarium-core | ✅ |
| 3 — Tauri Desktop → core | ✅ |
| 2 — portarium-tui (Ratatui) | ✅ базовая реализация |
|   ├── Terminal setup (tui.rs) | ✅ |
|   ├── Async event loop (main.rs) | ✅ |
|   ├── UI rendering (Dashboard, Detail, Graph, Help) | ✅ |
|   └── Background scan task | ✅ |
| 4 — CLI | ✅ clap-based |
|   ├── list (table/JSON) | ✅ |
|   ├── watch (live polling) | ✅ |
|   ├── events (filter by port) | ✅ |
|   ├── graph (nodes+edges) | ✅ |
|   ├── traffic (per port) | ✅ |
|   ├── kill (by PID) | ✅ |
|   └── restart (by PID+cmd+cwd) | ✅ |
| 5 — Тесты и CI | ⬜ |
