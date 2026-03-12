# Phase Implementation Report

## Executed Phase
- Phase: phase-08-advanced-features
- Plan: /home/danghuynh/dev/be/agent-note-workspace/plans/260311-1526-recall-smart-note-app
- Status: completed

## Files Modified

### Created / rewritten
| File | Lines | Notes |
|------|-------|-------|
| `src-tauri/src/services/clipboard.rs` | 35 | Clipboard monitor — polls every 1s via `ClipboardExt`, emits `clipboard_changed` |
| `src-tauri/src/services/file_watcher.rs` | 165 | notify + debouncer inbox watcher, text/media file ingestion, processed_files dedup |
| `src-tauri/src/commands/settings.rs` | 145 | `get_settings`, `update_settings`, `test_llm_connection` commands |
| `src/hooks/use-settings.ts` | 55 | Load/save via Tauri commands (replaces localStorage) |
| `src/components/settings/llm-settings.tsx` | 100 | Updated to snake_case fields + test connection button + clipboard toggle |
| `src/components/settings/memory-settings.tsx` | 75 | Updated to snake_case fields, removed non-backend fields |

### Modified
| File | Change |
|------|--------|
| `src-tauri/src/models/settings.rs` | Added `clipboard_watcher_enabled: bool` field |
| `src-tauri/src/db/connection.rs` | Added `get_setting`, `set_setting` helpers |
| `src-tauri/src/services/enrichment/mod.rs` | Added LLM health check loop, emits `llm_status` online/offline |
| `src-tauri/src/lib.rs` | Loads settings from DB, spawns clipboard/file watchers, registers settings commands |
| `src-tauri/Cargo.toml` | Added `dirs = "5"` |
| `src/types/index.ts` | `AppSettings` rewritten to snake_case matching Rust backend |
| `src/hooks/use-memory.ts` | Added `llm_status` event listener, exposes `isOnline` |
| `src/services/tauri-commands.ts` | Added `getSettings`, `updateSettings`, `testLlmConnection` |
| `src/App.tsx` | Passes `isOnline` from `useMemory` to `StatusBar` |

## Tasks Completed
- [x] Create services/clipboard.rs with polling monitor
- [x] Create services/file_watcher.rs with notify-based watcher
- [x] Create commands/settings.rs with get/update/test commands
- [x] Add offline detection to enrichment worker
- [x] Update frontend status bar with LLM online/offline indicator
- [x] Implement settings persistence in SQLite (individual key-value rows)
- [x] Wire up clipboard and file watchers in lib.rs setup
- [x] Use-settings now loads/saves via Tauri commands
- [x] Add "pending enrichment" badge to note cards (was already present in note-card.tsx)

## Tests Status
- Type check (tsc --noEmit): pass
- Cargo check: pass
- Unit tests: not run (no test suite in codebase yet — Phase 9)

## Key Design Decisions
- Settings stored as individual key-value rows (not a single JSON blob) for easier partial reads/writes
- `load_settings_from_db` in `lib.rs` reuses the same logic as `get_settings` command without the Tauri `State<>` wrapper — avoids duplication via a private fn
- `AppSettings` TS type now uses snake_case to match Rust serde output directly — no camelCase adapter needed
- Clipboard watcher: `false` by default, guarded at startup and in the command — privacy-safe
- File watcher: debouncer kept alive via `let _debouncer = debouncer` inside the spawned async task so it isn't dropped when `start_file_watcher` returns

## Issues Encountered
1. `notify-debouncer-full` API: `.watch()` must be called via `debouncer.watcher().watch(...)` not `debouncer.watch(...)` — fixed
2. `LLMProvider` trait not in scope in `commands/settings.rs` — added explicit import
3. `event.paths` owned by `DebouncedEvent` — iterated by reference `&event.paths`
4. Old `AppSettings` TS type used camelCase (ollamaUrl, geminiApiKey, etc.) — fully updated to snake_case; removed fields not in backend (`ollamaModel`, `geminiModel`, `autoEnrich`, `decayEnabled`)

## Unresolved Questions
- Clipboard watcher restart: changing `clipboard_watcher_enabled` via settings currently requires app restart to take effect. A runtime toggle would need a cancellation token or shutdown channel — deferred to Phase 9.
- File watcher inbox path change: same issue — watcher is started once at launch with the configured path. Runtime re-configuration deferred.
- `tauri_plugin_clipboard_manager::ClipboardExt` import was used directly — verify it compiles with the plugin version in Cargo.lock if plugin API changes.
