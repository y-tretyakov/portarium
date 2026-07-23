Архитектурный План — Этап 2: portarium-tui (Ratatui)
Цель
Полностью переписать TUI с OpenTUI (Bun + React) на Rust + Ratatui. Использовать async-app паттерн с Tokio + crossterm EventStream, т.к. TUI должен параллельно сканировать порты через core::PortariumService.
Архитектура
tui/src/
├── main.rs          # Entry point, panic hook, terminal init
├── app.rs           # App state + update (TEA Model/Update)
├── ui.rs            # View — все рендер-функции
├── screens/         # Экраны (модуль для каждого экрана)
│   ├── mod.rs
│   ├── dashboard.rs # Главный экран: таблица портов + статус-бар
│   ├── detail.rs    # Детали порта: трафик, события, инфо
│   └── graph.rs     # Граф соединений
├── components/      # Переиспользуемые компоненты
│   ├── mod.rs
│   ├── port_table.rs # Таблица портов с сортировкой
│   ├── status_bar.rs # Статус-бар (poll interval, кол-во портов, help)
│   ├── help_popup.rs # Help-поповер (key bindings)
│   └── event_list.rs # Список событий (scrollable)
├── action.rs        # Action enum (Message)
├── event.rs         # EventHandler — crossterm events + tick + background
├── tui.rs           # Terminal init/restore, panic guard
├── config.rs        # Загрузка TUI-специфичной конфигурации
└── logging.rs       # tracing-subscriber setup
Схема данных (AppState)
struct App {
    service: PortariumService,    // из portarium-core
    ports: Vec<PortInfo>,         // кеш результатов скана
    events: Vec<PortEvent>,       // из логгера
    traffic: HashMap<u16, Vec<TrafficSample>>,
    selected_port: Option<u16>,
    screen: Screen,               // Dashboard | Detail(u16) | Graph
    status: Status,               // Idle | Scanning | Error(String)
    should_quit: bool,
}
Action Enum
enum Action {
    Tick,
    Scan,
    ScanComplete(Vec<PortEvent>),
    SelectNext,
    SelectPrev,
    Enter(PortInfo),       // открыть детали порта
    Kill(PortInfo),
    Restart(PortInfo),
    Back,                  // назад на Dashboard
    ToggleHelp,
    ChangeScreen(Screen),
    Quit,
}
Event Loop (async, select!)
select! {
    Some(Ok(event)) = crossterm_events.next() => { /* key → Action */ }
    _ = tick.tick() => { /* Action::Tick */ }
    Some(result) = scan_rx.recv() => { /* Action::ScanComplete */ }
    Some(action) = action_rx.recv() => { /* process action */ }
}
Экраны
Screen	Описание
Dashboard	Таблица портов (port, pid, process, project, framework). Сортировка по колонкам. Status bar снизу: количество портов, poll interval, dev ports
Detail(port)	Детальная информация о порте: события, трафик, команда запуска, проект. Кнопки Kill / Restart
Graph	Текстовое представление графа соединений (список нод и связей; визуальный граф позже)
UI компоненты
- PortTable — Table widget с TableState, highlight, scroll, сортировка по клику на заголовок
- StatusBar — закреплённая строка снизу
- HelpPopup — модальный popup с клавишами (по ?)
- EventList — List с ListState для скролла событий
План работ (подзадачи для субагентов)
№	Задача	Описание
2.1	Настройка проекта	tui/Cargo.toml: ratatui 0.30, crossterm 0.29 (event-stream), futures, tokio, color-eyre, tracing
2.2	tui.rs + main.rs	Terminal init/restore, panic hook, color-eyre install, #[tokio::main]
2.3	Action enum + AppState	TEA: Action enum, App struct с service, ports, events, screen, status
2.4	event.rs	EventHandler: crossterm EventStream, tick interval, mpsc channel для ScanComplete
2.5	Dashboard screen	PortTable: Table widget, сортировка, highlight, статус-бар
2.6	Detail screen	PortInfo details, события, трафик, Kill/Restart
2.7	Graph screen	Текстовый граф (список нод и связей)
2.8	HelpPopup	Модальный popup с клавишами
2.9	ui.rs	Маршрутизация рендера по экрану + layout
2.10	app.rs — update	Обработка всех Action: scan, select, enter, kill, restart, quit
2.11	Кнопки Kill/Restart	Отправка Action::Kill/Restart в core, обработка результата
2.12	Сборка и интеграция	cargo build -p portarium-tui, исправление ошибок, clippy
2.13	Тесты	Unit-тесты на AppState.update(), интеграционный тест headless
2.14	Обновление статуса	STATUS.md, CHANGELOG.md
Headless режим
Для тестирования: запуск с --headless флагом (clap), который выполняет один scan и выводит результат в JSON, без входа в TUI.
Ключевые решения
1. Async, не sync — PortScanner::scan() блокирующий, но его вызов обёрнут в tokio::task::spawn_blocking
2. Tick rate — 250ms (совпадает с poll_interval_secs из config)
3. mpsc каналы — scan_rx для результатов скана, action_tx для dispatch
4. Никаких unwrap() — все ошибки через Result / color-eyre
5. TEA паттерн — чистые update(), рендер без side-эффектов
Проверка качества
cargo fmt -p portarium-tui
cargo clippy -p portarium-tui -- -D warnings
cargo test -p portarium-tui
cargo build -p portarium-tui
Хочешь, я подготовлю команды для делегирования каждой подзадачи субагентам, или сначала утвердим этот план?
▣  Build · DeepSeek V4 Flash Free · 1m 5s
Утверждаю.
Отлично, план утверждён. Начинаю делегирование подзадач. Запускаю параллельно:
- 2.1 — Настройка Cargo.toml
- 2.2 — tui.rs + main.rs
- 2.3 — Action enum + AppState
# Todos
[•] 2.1 — Настройка tui/Cargo.toml (ratatui, crossterm, tokio, futures, tracing, clap)
[•] 2.2 — tui.rs (Terminal init/restore, panic hook) + main.rs (#[tokio::main])
[•] 2.3 — Action enum + AppState (TEA Model)
[ ] 2.4 — event.rs (EventHandler: EventStream + tick + mpsc)
[ ] 2.5 — Dashboard screen (PortTable + StatusBar)
[ ] 2.6 — Detail screen (Port info + events + traffic + Kill/Restart)
[ ] 2.7 — Graph screen (text-based port graph)
[ ] 2.8 — HelpPopup (modal with key bindings)
[ ] 2.9 — ui.rs (render routing + layout)
[ ] 2.10 — app.rs update (all Action handling)
[ ] 2.11 — Kill/Restart integration
[ ] 2.12 — Build + integration + clippy
[ ] 2.13 — Tests (unit + headless integration)
[ ] 2.14 — Обновление STATUS.md + CHANGELOG.md