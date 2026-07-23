# Architecture

## Overview

Portarium is organized as a Cargo workspace with four crates sharing a common core library:

| Crate | Description |
|---|---|
| `core/` | Business logic library — models, scanning, logging, graph, frameworks |
| `tui/` | Ratatui terminal application binary |
| `cli/` | Clap-based CLI binary |
| `src-tauri/` | Tauri v2 desktop application (thin commands over core) |

## Workspace Structure

```
portarium/
├── core/                  # Library — all business logic
│   └── src/
│       ├── models.rs      # PortInfo, PortEvent, TrafficSample, GraphNode/Edge, Framework
│       ├── error.rs       # Unified Error type (thiserror)
│       ├── config.rs      # ScannerConfig, LoggerConfig, GraphConfig, PortariumConfig
│       ├── scanner/       # PortScanner + ScannerBackend trait (Unix/Windows impl)
│       ├── logger/        # PortLogger — event + traffic tracking
│       ├── graph/         # PortGraph — connection graph via ss/lsof
│       ├── frameworks/    # FrameworkRegistry — TOML-extensible + built-in list
│       ├── service.rs     # PortariumService — main facade
│       └── lib.rs         # Public API via explicit re-exports
├── tui/                   # Ratatui TUI binary
├── src-tauri/             # Tauri v2 desktop app (thin wrappers over core)
├── cli/                   # Clap CLI binary
└── Cargo.toml             # Workspace root
```

## Key Interfaces

- **`PortariumService`** — main facade combining scanner, logger, and graph.
- **`ScannerBackend`** trait — platform-specific implementations (`UnixScanner`, `WindowsScanner`).
- **`FrameworkRegistry`** — detection of known frameworks (TOML config + built-in defaults).

## Data Flow

```
System Commands (lsof/ss/netstat)
        ↓
 PortScanner ──→ PortInfo[]
        ↓
 PortLogger ──→ PortEvent[], TrafficSample[]
        ↓
 PortGraph ──→ GraphNode[], GraphEdge[]
        ↓
 PortariumService (facade combining all three)
   ├── TUI (Ratatui) — in-process
   ├── CLI (clap) — in-process
   └── Desktop (Tauri) — in-process via commands
```

No external services, daemons, or network connections are required. All data is collected from local system commands and `/proc`.