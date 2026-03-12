# Notal

Smart note-taking desktop app with AI-powered memory. Notes are automatically enriched, scored, and organized into memory layers (working → episodic → semantic) inspired by human cognitive memory.

Built with **Tauri v2** (Rust backend) + **React 18** (TypeScript frontend).

## Features

- **Quick Capture** — global hotkey to instantly capture a thought (default: `Ctrl+Shift+N`)
- **Screenshot Capture** — paste screenshots from clipboard with optional caption (default: `Ctrl+Shift+S`)
- **Clipboard Watcher** — optional always-on-top toast to save copied text (off by default for privacy)
- **File Inbox** — drop files (PDF, TXT, MD) into a watched folder for automatic ingestion
- **AI Enrichment** — background worker summarizes notes, extracts entities/topics, scores importance
- **Ask AI (RAG)** — ask questions against your notes with citation-backed answers
- **FTS5 Search** — full-text keyword search across all notes
- **Memory Layers** — notes decay and promote through working → episodic → semantic layers
- **Consolidation** — periodic background merging of related notes into higher-level knowledge
- **Multi-LLM Support** — Gemini (cloud) and Ollama (local), with automatic fallback
- **System Tray** — runs in background, accessible from tray icon

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) (stable toolchain)
- [Tauri v2 CLI](https://v2.tauri.app/start/prerequisites/)
- **Linux only:** system dependencies for Tauri

### Linux system dependencies (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

## Setup

```bash
# Clone and enter project
cd notal

# Install frontend dependencies
npm install

# Verify Rust toolchain
rustup update stable
```

## Development

```bash
# Start dev mode (hot-reload frontend + Rust backend)
npm run tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420` (frontend)
- Tauri Rust backend with hot-reload on code changes

### Useful dev commands

```bash
# Frontend only — type-check
npx tsc --noEmit

# Frontend only — dev server (no Tauri)
npm run dev

# Backend only — check compilation
cd src-tauri && cargo check

# Backend only — check with all warnings
cd src-tauri && cargo clippy
```

## Build

### Production build (bundled app)

```bash
# Full build: compiles frontend + Rust backend + creates installer
npm run tauri build
```

Output location by platform:
- **Linux:** `src-tauri/target/release/bundle/deb/` (.deb) and `src-tauri/target/release/bundle/appimage/` (.AppImage)
- **macOS:** `src-tauri/target/release/bundle/dmg/` (.dmg)
- **Windows:** `src-tauri/target/release/bundle/msi/` (.msi) and `src-tauri/target/release/bundle/nsis/` (.exe)

### Build steps breakdown

```bash
# 1. Build frontend only
npm run build
# Output: dist/

# 2. Build Rust backend only (debug)
cd src-tauri && cargo build

# 3. Build Rust backend only (release, optimized)
cd src-tauri && cargo build --release
```

### Debug build (faster compile, larger binary)

```bash
npm run tauri build -- --debug
```

## Project Structure

```
notal/
├── src/                          # Frontend (React + TypeScript)
│   ├── components/
│   │   ├── settings/             # LLM, memory, hotkey settings
│   │   ├── note-zone/            # Note list & card grid
│   │   ├── ask-zone/             # Ask AI chat interface
│   │   ├── quick-capture/        # Quick capture window
│   │   └── layout/               # Sidebar
│   ├── hooks/                    # React hooks (use-notes, use-settings, etc.)
│   ├── services/                 # Tauri IPC bridge (tauri-commands.ts)
│   ├── types/                    # TypeScript type definitions
│   ├── App.tsx                   # Root with URL-path routing for multi-window
│   └── main.tsx                  # Entry point
├── src-tauri/                    # Backend (Rust + Tauri v2)
│   ├── src/
│   │   ├── commands/             # Tauri command handlers
│   │   ├── models/               # Data models (Note, AppSettings)
│   │   ├── db/                   # SQLite database layer + FTS5
│   │   ├── services/             # Enrichment, clipboard, hotkeys, file watcher, tray
│   │   ├── memory/               # Decay, promotion, consolidation workers
│   │   ├── llm/                  # Gemini & Ollama providers + fallback manager
│   │   └── lib.rs                # App setup, state init, worker spawning
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Configuration

All settings are persisted in SQLite and configurable from the in-app Settings page:

| Setting | Default | Description |
|---------|---------|-------------|
| LLM Provider | `auto` | `auto` (Ollama → Gemini fallback), `ollama`, or `gemini` |
| Ollama Endpoint | `http://localhost:11434` | Local Ollama server URL |
| Gemini API Key | — | Google AI Studio API key |
| Gemini Model | `gemini-2.0-flash-lite` | Model used for enrichment & RAG |
| Quick Capture Hotkey | `Ctrl+Shift+N` | Global hotkey for quick note capture |
| Screenshot Hotkey | `Ctrl+Shift+S` | Global hotkey for screenshot capture |
| Clipboard Watcher | `off` | Monitor clipboard for text (privacy-sensitive) |
| Inbox Folder | `~/Notal/inbox` | Folder monitored for file ingestion |
| Decay Rates | 0.1 / 0.05 / 0.01 | Per-hour decay for working/episodic/semantic |
| Consolidation Interval | 60 min | How often to run memory consolidation |

## Tech Stack

- **Frontend:** React 18, TypeScript, Tailwind CSS v4, Vite
- **Backend:** Rust, Tauri v2, SQLite (rusqlite + FTS5), tokio
- **LLM:** Google Gemini API, Ollama (local)
- **Plugins:** tauri-plugin-clipboard-manager, tauri-plugin-global-shortcut, tauri-plugin-fs, tauri-plugin-dialog, tauri-plugin-shell

## License

Private — all rights reserved.
