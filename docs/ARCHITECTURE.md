# Architecture Decision Record + Overview

## Context
Дублирование Rust ↔ TypeScript + зависимость от OpenTUI.

## Decision
Переход на `portarium-core` + Ratatui.

## Структура

```markdown
portarium/
├── core/                  # Library (бизнес-логика)
│   └── src/
│       ├── models.rs
│       ├── error.rs
│       ├── config.rs
│       ├── scanner/
│       ├── logger/
│       ├── graph/
│       ├── frameworks/
│       ├── service.rs     # Facade
│       └── lib.rs
├── tui/                   # Ratatui binary
├── src-tauri/             # Tauri (тонкие команды → core)
├── cli/                   # clap binary
└── Cargo.toml (workspace)
```

## Key Interfaces

- `PortariumService` — главный фасад.
- `ScannerBackend` trait (Unix/Windows).
- `FrameworkRegistry` (TOML + builtin).