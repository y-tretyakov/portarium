# Development Flow

## 1. Перед началом работы
1. `git pull`
2. Прочитать `PLAN.md` + `STATUS.md`
3. `just ci`

## 2. Типичный цикл (Core-first)

1. **Core Engineer** реализует/улучшает модуль.
2. Главный Архитектор → ревью.
3. **TUI Engineer** / **Tauri Integration** адаптируют.
4. **Testing & Polish** добавляет тесты + обновляет документацию.
5. Обновить `STATUS.md` + `CHANGELOG.md`.
6. `git commit -m "feat(core): ..."`

## 3. Релизный процесс
- Обновить версию в `Cargo.toml` workspace.
- `just ci`
- Тег + GitHub Release.