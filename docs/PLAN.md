# Portarium — Архитектурный План

**Дата:** 22 июля 2026  
**Автор:** Главный Архитектор

## Цель

Устранить дублирование кода (Rust ↔ TypeScript), избавиться от зависимости OpenTUI, перейти на единую кодовую базу: `portarium-core` + Ratatui.

## Целевая архитектура

```
portarium/
├── core/                  # Rust library — вся бизнес-логика
│   ├── src/
│   │   ├── scanner.rs     # Сканирование портов (lsof, netstat, ss)
│   │   ├── logger.rs      # Логирование событий и трафика
│   │   ├── graph.rs       # Построение графа соединений
│   │   ├── frameworks.rs  # Единый список известных портов/фреймворков
│   │   └── lib.rs         # Публичное API
│   └── Cargo.toml
├── tui/                   # Ratatui TUI-приложение
├── src-tauri/             # Tauri Desktop (использует core)
├── cli/                   # Простой CLI-бинарник
└── Cargo.toml             # workspace
```

## Этапы

**Этап 0 — Инфраструктура** (хорошо, добавить):
- Создать `Cargo.toml` workspace
- Добавить `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml`
- Настроить `justfile` или Makefile для удобных команд (`just core`, `just tui`, `just tauri`)

**Этап 1 — portarium-core** (самый важный, улучшить):
- Создать модульную структуру:
  ```rust
  core/src/
  ├── lib.rs
  ├── config.rs
  ├── scanner/
  ├── logger/
  ├── graph/
  ├── frameworks/
  ├── models.rs      # PortInfo, PortEvent и т.д.
  ├── error.rs
  └── service.rs     # главный фасад PortariumService
  ```
- Сделать `pub use` удобного API
- Добавить `#[cfg(test)]` + unit-тесты + property-based тесты (где уместно)

**Этап 2 — portarium-tui (Ratatui)**
- Добавить `color-eyre` или `tracing` для красивых ошибок
- Реализовать `AppState` + event loop
- Сделать возможность запуска в "headless" режиме (для тестирования)

**Этап 3 — Tauri Desktop**
- Обновить зависимости
- Удалить дублированный код
- Сделать Tauri-команды тонкими обёртками над core

**Этап 4 — CLI**
- Сделать с помощью `clap`
- Поддержка субкоманд: `portarium`, `portarium watch`, `portarium kill`, `portarium graph` и т.д.

**Этап 5 — Тесты, CI/CD, Документация**
- Добавить `cargo test`, `cargo clippy --all-targets -- -D warnings`
- Coverage
- Обновить README + архитектурную диаграмму (mermaid)

**Дополнительный Этап 6 (рекомендую):**
- **Backward Compatibility** — убедиться, что `npx portarium` и старые бинарники продолжают работать (можно через symlink или обёртку).

## Риски

| Риск | Вероятность | Снижение |
|------|-------------|----------|
| Поломка Tauri Desktop | Средняя | Этап 3 после core + тесты |
| Потеря фич TUI | Средняя | Делать экраны по одному |
| Platform-specific баги | Высокая | Тесты на CI под все ОС |
