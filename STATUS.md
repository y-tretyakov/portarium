# Portarium — Состояние проекта

**Последнее обновление:** 22 июля 2026

## Текущая архитектура (до рефакторинга)

```
portarium/
├── src/                   # React + Vite фронтенд (Tauri Desktop)
├── src-tauri/src/         # Rust бэкенд (scanner, logger, connections, tray)
├── src-tui/               # TUI на OpenTUI (Bun + React) — дублирует scanner.ts
├── bin/cli.tsx            # CLI точка входа для Bun
└── docs/                  # Документация
```

## Проблемы
- Дублирование scanner.rs ↔ scanner.ts (разная реализация одной логики)
- Дублирование frameworks.ts (вручную поддерживать два списка)
- OpenTUI зависит от Bun — лишняя external dependency
- Нет тестов
- Нет обработки ошибок (unwrap/expect)

## Целевая архитектура (WIP)

```
portarium/
├── core/                  # Rust library — вся бизнес-логика [✅ готово]
│   ├── src/
│   │   ├── lib.rs         # Публичное API (pub use всех типов)
│   │   ├── config.rs      # ScannerConfig, LoggerConfig, PortariumConfig
│   │   ├── error.rs       # Error enum (thiserror) + Result
│   │   ├── models.rs      # PortInfo, PortEvent, GraphNode, Framework...
│   │   ├── scanner/       # PortScanner — lsof/ss/netstat, kill, restart
│   │   ├── logger/        # PortLogger — события + трафик
│   │   ├── graph/         # PortGraph — построение графа соединений
│   │   ├── frameworks/    # Registry: Vite, React, Postgres...
│   │   └── service.rs     # PortariumService — главный фасад
│   └── Cargo.toml
├── tui/                   # Ratatui TUI [в разработке]
├── src-tauri/             # Tauri Desktop (использует core) [ожидает]
├── cli/                   # CLI [ожидает]
└── Cargo.toml             # workspace [готово]
```

## Этапы

| Этап | Статус |
|------|--------|
| 0 — Инфраструктура | ✅ |
| 1 — portarium-core | ✅ |
| 2 — portarium-tui (Ratatui) | ⬜ |
| 3 — Tauri Desktop → core | ⬜ |
| 4 — CLI | ⬜ |
| 5 — Тесты и CI | ⬜ |
