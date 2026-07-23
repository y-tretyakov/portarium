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
[![Crates.io](https://img.shields.io/crates/v/portarium-core?style=for-the-badge&logo=rust&label=crates.io)](https://crates.io/crates/portarium-core)
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
