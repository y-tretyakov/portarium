<div align="center">

![Portarium — Developer Port Manager](https://raw.githubusercontent.com/y-tretyakov/portarium/main/social-preview.jpg)

# ⚡ Portarium

**Know what's running. Kill what's blocking. See how it's connected.**

A blazing-fast, native developer port manager. Stop playing detective with `netstat` and `lsof`. Portarium watches your ports, tracks traffic, and visualizes network topology — in your terminal, CLI, or desktop.

[![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/y-tretyakov/portarium/releases)
[![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/y-tretyakov/portarium/releases)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/y-tretyakov/portarium/releases)
[![Built with Tauri](https://img.shields.io/badge/Desktop-Tauri_2-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app)
[![Terminal UI](https://img.shields.io/badge/Terminal-Ratatui-7c6fff?style=for-the-badge)](https://ratatui.rs)
[![npm](https://img.shields.io/npm/v/portarium?style=for-the-badge&logo=npm&label=npm)](https://www.npmjs.com/package/portarium)
[![Crates.io](https://img.shields.io/crates/v/portarium-core?style=for-the-badge&logo=rust&label=crates.io&cacheSeconds=0)](https://crates.io/crates/portarium-core)
[![CI](https://img.shields.io/github/actions/workflow/status/y-tretyakov/portarium/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=CI)](https://github.com/y-tretyakov/portarium/actions/workflows/ci.yml)
[![Stars](https://img.shields.io/github/stars/y-tretyakov/portarium?style=social)](https://github.com/y-tretyakov/portarium)

</div>

---

## The Problem

```text
Error: listen EADDRINUSE: address already in use :::3000
```

Something is already squatting on your port. You hunt for PIDs, copy-paste kill commands, and lose focus. Every. Single. Time.

**Portarium ends that.**

---

## Features

### Real-Time Port Dashboard
Every listening port at a glance — process name, PID, connections, framework detection, and project identification.

### One-Click Control
Kill any process instantly. If the start command is known, restart it directly in a new terminal. Dead processes show a stopped badge with a persistent restart button.

### Terminal UI (Beta)
A full keyboard-driven Ratatui interface — no desktop shell required. Perfect for SSH sessions and developers who live in the terminal.

### Interactive Port Map (Desktop)
A D3.js-powered network topology visualization showing how your services communicate. Drag, zoom, and explore.

### System Tray Intelligence (Desktop)
Lives quietly in your system tray. A traffic light icon alerts you to conflicts. Background logging keeps working when the window is closed.

### Smart Detection Engine
- **Framework Detection** — Recognizes React, Vite, Angular, Django, Node, and more via default ports.
- **Project Context** — Crawls for `package.json`, `Cargo.tomr`, or `go.mod` to name your running servers.

### CLI
Full-featured command-line interface for scripts, pipelines, and headless environments.

---

## Terminal UI (Beta)

Portarium ships with a keyboard-driven terminal interface built on [Ratatui](https://ratatui.rs).

> **Status:** Beta — actively developed. Core functionality is stable, some visual polish is ongoing.

### Pages

| Page | Key | Description |
|------|-----|-------------|
| **Ports** | `1` | Full port list with framework colors, search, and filter |
| **Dashboard** | `2` | Summary stats, active services, and recent events |
| **Services** | `3` | Ports grouped by project or framework |
| **Logs** | `4` | Chronological event log of port starts and stops |

### Keybindings

| Key | Action |
|-----|--------|
| `↑` `↓` / `j` `k` | Navigate port list |
| `k` | Kill selected process |
| `K` | Kill all filtered processes |
| `r` | Restart selected (when start command is known) |
| `/` | Search / filter ports |
| `f` | Cycle filter: all → dev → other |
| `1`–`4` / `Tab` | Switch pages |
| `u` | Refresh now |
| `q` / `Ctrl+C` | Quit |

---

## Installation

### From Source

#### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| **Rust** | ≥ 1.75 | [rustup.rs](https://rustup.rs) |

#### Terminal UI

```bash
cargo run -p portarium-tui
```

#### CLI

```bash
cargo run -p portarium-cli -- list
cargo run -p portarium-cli -- watch
cargo run -p portarium-cli -- events
cargo run -p portarium-cli -- graph
cargo run -p portarium-cli -- traffic
cargo run -p portarium-cli -- kill <pid>
cargo run -p portarium-cli -- restart <pid>
```

#### Desktop App

```bash
git clone https://github.com/y-tretyakov/portarium.git
cd portarium
npm install
npm run tauri dev
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/y-tretyakov/portarium/releases).

---

## Supported Frameworks

Portarium auto-detects these frameworks out-of-the-box:

| Port | Framework | Color |
|------|-----------|-------|
| 3000, 3001 | React | `#61dafb` |
| 4200 | Angular | `#dd0031` |
| 5173, 5174, 4173 | Vite | `#646cff` |
| 4000, 2000 | Node.js | `#68a063` |
| 8000 | Django | `#2bbc8a` |
| 8080, 80 | HTTP | `#f0a500` |
| 8888 | Jupyter | `#f37626` |
| 5432 | Postgres | `#336791` |
| 6379 | Redis | `#dc382d` |
| 3306 | MySQL | `#4479a1` |
| 27017 | MongoDB | `#4db33d` |
| 1420 | Tauri | `#ffc131` |
| 22 | SSH | `#6e7681` |
| 443, 8443 | HTTPS | `#22c55e` |
| 9000 | PHP | `#8892bf` |

---

## Diagnostic & Testing Guide

### Spawning Test Services

Quickly populate ports to exercise Portarium:

```bash
# Python — one-shot HTTP servers on popular ports
python3 -m http.server 3000  --bind 127.0.0.1 &   # React dev port
python3 -m http.server 4200  --bind 127.0.0.1 &   # Angular
python3 -m http.server 5173  --bind 127.0.0.1 &   # Vite
python3 -m http.server 8000  --bind 127.0.0.1 &   # Django
python3 -m http.server 8080  --bind 127.0.0.1 &   # HTTP
python3 -m http.server 5432  --bind 127.0.0.1 &   # Postgres (simulated)
python3 -m http.server 6379  --bind 127.0.0.1 &   # Redis (simulated)
python3 -m http.server 3306  --bind 127.0.0.1 &   # MySQL (simulated)
python3 -m http.server 27017 --bind 127.0.0.1 &   # MongoDB (simulated)
python3 -m http.server 9000  --bind 127.0.0.1 &   # PHP
python3 -m http.server 1420  --bind 127.0.0.1 &   # Tauri
python3 -m http.server 8888  --bind 127.0.0.1 &   # Jupyter

# Node.js (if installed)
npx serve -l 3000 &
npx serve -l 4000 &

# Netcat — lightweight listeners
nc -lk 3000 &
nc -lk 5000 &
nc -lk 6000 &

# macOS — use any port with the built-in Python or:
socat TCP-LISTEN:3000,fork,reuseaddr EXEC:cat &
```

### Stopping Test Services

```bash
# Kill by PID (find with Portarium or ps)
kill -9 <PID>

# Kill all test Python servers
pkill -f "python3 -m http.server"

# Kill all netcat listeners
pkill nc

# Kill everything on common dev ports (macOS/Linux)
lsof -ti:3000,4200,5173,8000,8080,5432,6379,3306,27017,9000,1420,8888 | xargs kill -9

# Kill a range of ports
lsof -ti:3000-9000 | xargs kill -9
```

### Diagnostic Checklist

Walk through these steps to verify every Portarium feature:

| # | Test | CLI | TUI | Desktop |
|---|------|-----|-----|---------|
| 1 | List all listening ports | `cargo run -p portarium-cli -- list` | Launch TUI, verify Ports page (1) | Open app → Ports page |
| 2 | List as JSON | `cargo run -p portarium-cli -- list --json` | — | — |
| 3 | Watch live updates | `cargo run -p portarium-cli -- watch` | Ports auto-refresh every 5s | Background thread polls every 2s |
| 4 | Watch with custom interval | `cargo run -p portarium-cli -- watch --interval 5` | — | — |
| 5 | View event log | `cargo run -p portarium-cli -- events` | Switch to Logs page (4) | Navigate to Logs page |
| 6 | Filter events by port | `cargo run -p portarium-cli -- events --port 3000` | — | — |
| 7 | View connection graph | `cargo run -p portarium-cli -- graph` | Switch to Graph screen (3) | Navigate to Port Map page |
| 8 | View traffic for a port | `cargo run -p portarium-cli -- traffic 3000` | Select a port → Enter → Detail (2) | Navigate to Traffic page |
| 9 | Kill a process by PID | `cargo run -p portarium-cli -- kill <pid>` | Select port → press `k` | Click kill button on port row |
| 10 | Kill all filtered | — | Press `K` | Click "Kill All" button |
| 11 | Restart a process | `cargo run -p portarium-cli -- restart <pid> --cmd "python3 -m http.server 3000" --cwd /tmp` | Select port → press `r` | Click restart button |
| 12 | Filter dev ports only | — | Press `f` to cycle filter | Click "Dev" filter tab |
| 13 | Search by name/port | — | Press `/` to enter search | Type in search bar |
| 14 | Force refresh | — | Press `u` | Click refresh button |
| 15 | Tray icon states | — | — | Minimize window; tray icon shows green/yellow/red |
| 16 | Port map visualization | — | — | Navigate to Port Map page |
| 17 | Service grouping | — | Switch to Services page (3) | Navigate to Services page |

### Debugging & Logging

```bash
# Enable debug-level tracing (TUI only)
RUST_LOG=debug cargo run -p portarium-tui

# Module-specific tracing
RUST_LOG=portarium_tui=debug cargo run -p portarium-tui

# Full backtrace on panic
RUST_BACKTRACE=1 cargo run -p portarium-tui

# Combine tracing + backtrace
RUST_LOG=debug RUST_BACKTRACE=full cargo run -p portarium-tui

# CLI with color-eyre error reporting (always enabled)
cargo run -p portarium-cli -- list

# Pipe JSON output for scripting
cargo run -p portarium-cli -- list --json | jq '.'
cargo run -p portarium-cli -- events --json | jq '.[] | select(.port == 3000)'
cargo run -p portarium-cli -- graph --json | jq '.nodes[] | {pid, process, port}'
```

### Platform-Specific Notes

| Platform | Scanner | Kill | Restart |
|----------|---------|------|---------|
| **Linux** | `lsof -iTCP -sTCP:LISTEN -n -P` | `SIGTERM` → 2s → `SIGKILL` | `command &` (detached) |
| **macOS** | `lsof -iTCP -sTCP:LISTEN -n -P` | `SIGTERM` → 2s → `SIGKILL` | `command &` (detached) |
| **Windows** | `netstat -ano` | `taskkill /PID <pid> /F` | `cmd /C start cmd /K <cmd>` |

### All Interface Commands Reference

#### CLI (`portarium-cli`)

```bash
# Build
cargo build -p portarium-cli

# Run directly
cargo run -p portarium-cli -- <subcommand> [flags]

# Help
cargo run -p portarium-cli -- --help
cargo run -p portarium-cli -- <subcommand> --help

# Available subcommands:
#   list       List all listening ports
#   watch      Poll and display ports in real-time
#   events     Show port event log
#   graph      Show connection graph
#   traffic    Show traffic for a specific port
#   kill       Kill a process by PID
#   restart    Kill and restart a process
```

| Subcommand | Arguments | Flags | Description |
|------------|-----------|-------|-------------|
| `list` | — | `--json` | Table or JSON of all open ports |
| `watch` | — | `--interval <sec>` (default: 2), `--json` | Live polling port list |
| `events` | — | `--port <u16>`, `--json` | Event log, optional port filter |
| `graph` | — | `--json` | Connection graph nodes/edges |
| `traffic` | `<port>` | `--json` | Traffic samples for a port |
| `kill` | `<pid>` | — | Terminate a process |
| `restart` | `<pid>` | `--cmd <str>` `--cwd <str>` | Kill then spawn replacement |

#### TUI (`portarium-tui`)

```bash
# Build & run
cargo build -p portarium-tui
cargo run -p portarium-tui

# With debug logging
RUST_LOG=debug cargo run -p portarium-tui

# Help overlay inside TUI: press ?
```

| Key | Action |
|-----|--------|
| `↑` / `k` | Navigate up |
| `↓` / `j` | Navigate down |
| `Enter` | Select port (go to Detail screen) |
| `Esc` / `Backspace` | Back to Dashboard |
| `1` | Dashboard (port list) screen |
| `2` | Detail screen (port info + timeline) |
| `3` | Graph screen (connection graph) |
| `r` | Trigger manual scan |
| `?` | Toggle help overlay |
| `q` / `Ctrl+C` | Quit |

#### JS TUI (`npx portarium` / `src-tui`)

```bash
# Via npm
npx portarium

# From source
bun run src-tui/index.tsx

# Or via the bin alias
portarium
```

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate port list |
| `j` | Navigate down |
| `k` | Kill selected process |
| `K` | Kill all filtered processes |
| `r` | Restart selected (if start_cmd known) |
| `/` | Enter search mode |
| `Esc` | Clear / exit search |
| `f` | Cycle filter: all → dev → other |
| `1` | Ports page |
| `2` | Dashboard page |
| `3` | Services page |
| `4` | Logs page |
| `Tab` | Cycle pages forward |
| `Shift+Tab` | Cycle pages backward |
| `u` | Force refresh |
| `q` / `Ctrl+C` | Quit |

#### Desktop App (Tauri)

```bash
# Development mode with hot reload
npm install
npm run tauri dev

# Production build
npm run tauri build

# Frontend dev server only (browser)
npm run dev
```

| Interface | Feature | How to access |
|-----------|---------|---------------|
| **Sidebar nav** | Dashboard / Ports / Traffic / Port Map / Services / Settings / Logs | Click sidebar icons |
| **Ports table** | Search bar, filter tabs (All/Dev/Other), kill/restart buttons, sparklines | Ports page |
| **Port Map** | D3.js force-directed graph with animated particles, zoom, drag, node inspector | Port Map page |
| **Traffic monitor** | Per-port sparkline charts, current/peak connections | Traffic page |
| **Logs** | Chronological event log viewer with refresh | Logs page |
| **System tray** | Green/yellow/red traffic light icon, Open / Quit menu | Window close → hides to tray |
| **Notifications** | Toast popups for kill/restart results | Auto-shown on action |
| **Custom titlebar** | Minimize / maximize / close | Top of window (decorations disabled) |

---

## Architecture

```
portarium/
├── core/       # Rust library — all business logic (models, scanner, logger, graph, frameworks)
├── tui/        # Ratatui terminal application
├── cli/        # Clap CLI binary
├── src-tauri/  # Tauri v2 desktop application
└── src/        # React 19 + Vite 7 frontend (desktop only)
```

All data is collected in-process via system commands (`lsof`/`ss`/`netstat`) and `/proc`. No external services, daemons, or network connections required.

---

## Development

```bash
# Build everything
cargo build

# Run tests
cargo test

# Lint
cargo clippy && cargo fmt --check

# Full CI pipeline
just ci
```

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

---

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

**If Portarium saved you from one more `EADDRINUSE`, give it a ⭐**

Made with Rust + React + Ratatui by [y-tretyakov](https://github.com/y-tretyakov)

</div>
