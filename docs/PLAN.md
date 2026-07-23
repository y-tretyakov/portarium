# Portarium — Архитектурный План

**Дата:** 22 июля 2026  
**Автор:** Главный Архитектор

## Цель

Устранить дублирование кода и перейти на единую Rust-базу через `portarium-core` + Ratatui.

## Архитектурные Решения (ADRs)

- **[ADR-001: Core Extraction](./ADR-001-Core-Extraction.md)** — ✅
- **[ADR-002: Ratatui Migration](./ADR-002-Ratatui-Migration.md)** — ✅
- ADR-003: CLI Design (в разработке)

## Целевая архитектура

```
portarium/
├── core/                  # Rust library — вся бизнес-логика [✅]
├── tui/                   # Ratatui TUI [в разработке]
├── src-tauri/             # Tauri Desktop (использует core) [✅]
├── cli/                   # CLI [ожидает]
└── Cargo.toml             # workspace
```


## Этапы

**Этап 0 — Инфраструктура** (завершён)
- Cargo workspace, Justfile, lint-конфиги

**Этап 1 — portarium-core** (завершён)
- Модульная структура, модели, scanner, logger, graph, frameworks, service

**Этап 2 — portarium-tui (Ratatui)** (текущий)
- Реализация event loop, экранов, интеграция с core

**Этап 3 — Tauri Desktop** (завершён)
- Тонкие команды над `portarium-core`

**Этап 4 — CLI**
- clap-based интерфейс

**Этап 5 — Тесты, CI/CD, Документация**
- Полное покрытие, GitHub Actions

**Этап 6 — Backward Compatibility**
- Поддержка `npx portarium`

## Риски и Митigation

| Риск                        | Вероятность | Митigation                     |
|----------------------------|-------------|--------------------------------|
| Поломка Tauri              | Низкая      | Этап 3 уже завершён            |
| Потеря UX в TUI            | Средняя     | Поэкранная миграция + тесты    |
| Platform-specific баги     | Высокая     | CI на всех ОС                  |

