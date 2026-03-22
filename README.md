# Aide

A personal assistant task manager for macOS, built with Tauri v2. Type anything in natural language — new tasks, progress updates, scope changes, completions — and Claude interprets them, updates your task list, and responds conversationally.

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat&logo=tauri) ![macOS](https://img.shields.io/badge/macOS-only-000000?style=flat&logo=apple)

## Features

- **Natural language input** — add tasks, give updates, mark things done, all in plain English
- **Claude-powered reasoning** — uses `claude-sonnet-4-20250514` to interpret intent, extract due dates, infer priorities, and link related tasks to projects
- **Drift detection** — automatically flags tasks that are falling behind based on elapsed time vs. reported progress
- **Daily briefing** — generates a personalised morning briefing based on your task list and how busy your day is
- **Projects** — tasks are grouped into projects, inferred automatically from context
- **Frameless window** — custom title bar, no OS chrome

## Architecture

| Layer | Tech |
|-------|------|
| Desktop shell | Tauri v2 |
| Frontend | Single `src/index.html` — vanilla HTML, CSS, JS, no bundler |
| Backend | Rust — all file I/O and Claude API calls |
| AI | Anthropic Claude API (`claude-sonnet-4-20250514`) |
| Storage | `~/Library/Application Support/aide/tasks.json` |
| Config | `~/Library/Application Support/aide/config.json` (API key) |

The frontend never calls the Anthropic API directly. All Claude calls are proxied through a Rust Tauri command.

## Prerequisites

- macOS
- [Rust](https://rustup.rs)
- [Node.js](https://nodejs.org) (for the Tauri CLI)
- Xcode Command Line Tools (`xcode-select --install`)
- An [Anthropic API key](https://console.anthropic.com)

## Getting started

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build a release .app
npm run tauri build
```

On first launch, Aide will prompt you for your Anthropic API key. It is written to `~/Library/Application Support/aide/config.json` and never committed to the repo.

## Project structure

```
aide/
├── src/
│   └── index.html          # Entire frontend — HTML, CSS, JS
└── src-tauri/
    ├── src/
    │   ├── main.rs         # Entry point
    │   ├── lib.rs          # Tauri app builder
    │   ├── commands.rs     # IPC commands (load/save tasks, API key, Claude)
    │   └── claude.rs       # Anthropic API client
    ├── capabilities/
    │   └── default.json    # Tauri IPC permissions
    ├── icons/              # App icons
    ├── Cargo.toml
    └── tauri.conf.json
```

## Data & privacy

- Tasks and config are stored locally in `~/Library/Application Support/aide/`
- The API key is never logged or transmitted anywhere other than the Anthropic API
- Nothing is stored in the cloud
