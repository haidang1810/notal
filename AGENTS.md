# AGENTS.md

Context file for AI agents working on this codebase.

## Project Overview

**Notal** — Tauri v2 desktop app for smart note-taking with AI-powered memory layers.
Notes are captured, enriched by LLM, scored, and automatically promoted through cognitive memory layers (working → episodic → semantic).

## Tech Stack

| Layer | Tech |
|-------|------|
| Frontend | React 18, TypeScript, Tailwind CSS v4, Vite |
| Backend | Rust, Tauri v2 |
| Database | SQLite (rusqlite) with FTS5 full-text search |
| LLM | Google Gemini API (cloud), Ollama (local), fallback manager |
| Async | tokio runtime |

## Architecture

```
Frontend (React/TS)  ←→  Tauri IPC  ←→  Rust Backend  ←→  SQLite
                                              ↕
                                     LLM (Gemini/Ollama)
```

### Multi-Window System
The app uses URL-path routing in `App.tsx` to render different windows:
- `/` → Main app (sidebar + notes/ask/settings views)
- `/quick-capture` → Always-on-top floating capture window
- `/clipboard-toast` → Always-on-top clipboard notification
- `/screenshot-capture` → Always-on-top screenshot paste window

Each window is a separate Tauri `WebviewWindow` created from Rust side.

## Directory Map

```
src/                              # Frontend
├── App.tsx                       # Root — URL-path router for multi-window
├── components/
│   ├── layout/sidebar.tsx        # Obsidian-style collapsible sidebar
│   ├── note-zone/                # Note grid (note-list, note-card, note-zone)
│   ├── ask-zone/                 # RAG chat (ask-zone, chat messages)
│   ├── settings/                 # Settings page (llm-settings, memory-settings, hotkey-input)
│   ├── quick-capture/            # Quick capture window component
│   ├── clipboard-toast-window.tsx
│   └── screenshot-capture-window.tsx
├── hooks/                        # use-notes, use-settings, use-memory, use-ask
├── services/tauri-commands.ts    # All Tauri invoke() wrappers — single IPC bridge
└── types/index.ts                # Shared TS types mirroring Rust models

src-tauri/src/                    # Backend
├── lib.rs                        # App setup, state init, worker spawning, load_settings_from_db
├── commands/                     # Tauri #[command] handlers
│   ├── notes.rs                  # CRUD: create_note, get_notes, update_note, delete_note, ingest_file
│   ├── search.rs                 # search_notes (FTS5), ask_ai (RAG with citations)
│   ├── settings.rs               # get/update_settings, test_llm_connection, list_gemini_models
│   ├── capture.rs                # save_quick_capture, save_screenshot_note, close_* windows
│   ├── memory.rs                 # get_memory_stats
│   └── consolidation.rs          # trigger_consolidation
├── models/
│   ├── note.rs                   # Note struct (id, raw_text, summary, entities, topics, layer, scores...)
│   ├── settings.rs               # AppSettings struct, LlmProviderType enum
│   └── memory_layer.rs           # MemoryLayer enum
├── db/
│   ├── connection.rs             # init_db, CRUD fns, FTS5, settings KV store
│   ├── schema.rs                 # CREATE TABLE statements
│   └── migrations.rs             # Schema migrations
├── llm/
│   ├── provider.rs               # LLMProvider trait (generate_completion, generate_structured, is_available)
│   ├── gemini.rs                 # GeminiProvider — REST API client
│   ├── ollama.rs                 # OllamaProvider — local Ollama client
│   ├── fallback.rs               # FallbackManager — tries primary, falls back to secondary
│   ├── types.rs                  # CompletionRequest/Response, StructuredRequest, LLMError
│   └── mod.rs                    # create_provider() factory from AppSettings
├── services/
│   ├── enrichment/mod.rs         # Background worker: enrich unenriched notes via LLM
│   ├── enrichment/rules.rs       # Rule-based importance boosts
│   ├── clipboard.rs              # Clipboard watcher — polls, shows toast window
│   ├── hotkeys.rs                # Global hotkey registration + window toggling
│   ├── file_watcher.rs           # Inbox folder monitor (notify crate)
│   └── tray.rs                   # System tray setup
└── memory/
    ├── decay.rs                  # Score decay per hour per layer
    ├── promotion.rs              # Layer promotion logic (working → episodic → semantic)
    └── consolidation.rs          # Merge related notes into higher-level knowledge
```

## Key Patterns

### State Management
- `DbState` = `Arc<Mutex<Connection>>` — shared SQLite connection
- `LlmState` = `Arc<tokio::Mutex<FallbackManager>>` — swappable LLM provider
- Both managed via `app.manage()` and accessed in commands via `State<'_>`

### Settings Pipeline
Settings stored as individual KV rows in SQLite `settings` table:
1. `load_settings_from_db()` in `lib.rs` — reads all keys, returns `AppSettings` with defaults
2. `get_settings` command — same logic, returns to frontend
3. `update_settings` command — writes all fields to KV store
4. `test_llm_connection` — accepts `AppSettings` from frontend draft (not from DB)

### LLM Provider Factory
`llm::create_provider(settings)` builds a `FallbackManager`:
- Auto: Ollama primary → Gemini fallback
- Ollama: Ollama primary → Gemini fallback
- Gemini: Gemini primary → Ollama fallback

### Enrichment Pipeline
Background worker in `services/enrichment/mod.rs`:
1. Polls for unenriched notes every 5s
2. Calls LLM with structured JSON schema → summary, entities, topics, importance
3. Applies rule-based importance boosts
4. Updates note in DB, emits `note_enriched` event to frontend

### Memory System
- **Decay**: Scores decrease per hour based on layer-specific rates
- **Promotion**: Notes that maintain high scores promote to deeper layers
- **Consolidation**: Periodic merge of related notes into semantic knowledge

### Ask AI (RAG)
In `commands/search.rs`:
1. FTS5 search for top 10 relevant notes
2. Build context with `raw_text` (3000 char limit) + creation dates
3. LLM generates answer with `[#ID]` citation markers
4. Citations parsed, cited notes get score boost

## Build Commands

```bash
npm run tauri dev          # Dev mode (hot-reload)
npm run tauri build        # Production build
cd src-tauri && cargo check   # Rust type-check only
npx tsc --noEmit           # TypeScript type-check only
```

## Common Gotchas

- `cargo check` must run from `src-tauri/`, not project root
- `npm run build` / `npx tsc` must run from `notal/` (project root)
- Always-on-top windows need `set_always_on_top(true)` on every `.show()`, not just creation
- Tauri clipboard plugin `readImage()` is async — `.size()` returns `{width, height}` via method, not properties
- Settings page uses draft state — changes aren't in DB until user clicks Save
- `test_llm_connection` receives settings from frontend to test unsaved draft values
- Relative dates in notes (e.g. "ngày mai") must be resolved to absolute dates during enrichment
- CSP in `tauri.conf.json` must whitelist `generativelanguage.googleapis.com` for Gemini API calls
