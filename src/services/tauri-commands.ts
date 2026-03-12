// Typed wrappers around Tauri invoke() calls — aligned with Rust backend commands

import { invoke } from '@tauri-apps/api/core';
import type { Note, MemoryStats, SearchResult, AskResponse, AppSettings } from '../types';

// ─── Notes ──────────────────────────────────────────────────────────────────

export function createNote(rawText: string, source?: string): Promise<Note> {
  return invoke('create_note', { rawText, source });
}

export function getNotes(layer?: string, includeArchived?: boolean): Promise<Note[]> {
  return invoke('get_notes', { layer, includeArchived });
}

export function getNoteById(id: number): Promise<Note> {
  return invoke('get_note_by_id', { id });
}

export function updateNote(
  id: number,
  rawText?: string,
  pinned?: boolean,
  archived?: boolean,
): Promise<void> {
  return invoke('update_note', { id, rawText, pinned, archived });
}

export function deleteNote(id: number): Promise<void> {
  return invoke('delete_note', { id });
}

export function ingestFile(filePath: string): Promise<number[]> {
  return invoke('ingest_file', { filePath });
}

// ─── Search & AI ─────────────────────────────────────────────────────────────

export function searchNotes(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke('search_notes', { query, limit });
}

export function askAi(question: string): Promise<AskResponse> {
  return invoke('ask_ai', { question });
}

// ─── Memory ──────────────────────────────────────────────────────────────────

export function getMemoryStats(): Promise<MemoryStats> {
  return invoke('get_memory_stats');
}

// ─── Settings ─────────────────────────────────────────────────────────────────

export function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export function updateSettings(settings: AppSettings): Promise<void> {
  return invoke('update_settings', { settings });
}

export function testLlmConnection(settings: AppSettings): Promise<boolean> {
  return invoke('test_llm_connection', { settings });
}

export function reregisterHotkeys(): Promise<void> {
  return invoke('reregister_hotkeys');
}

export interface GeminiModelInfo {
  id: string;
  display_name: string;
}

export function listGeminiModels(apiKey: string): Promise<GeminiModelInfo[]> {
  return invoke('list_gemini_models', { apiKey });
}
