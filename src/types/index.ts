// Shared TypeScript types for Notal app — aligned with Rust backend models

export type MemoryLayer = 'working' | 'episodic' | 'semantic';

export interface Note {
  id: number;
  raw_text: string;
  summary: string;
  importance: number;
  current_score: number;
  layer: string; // 'working' | 'episodic' | 'semantic'
  pinned: boolean;
  archived: boolean;
  created_at: string;
  last_accessed_at: string | null;
  last_updated_at: string | null;
  layer_promoted_at: string | null;
  access_count: number;
  access_count_since_promotion: number;
  entities: string[];
  topics: string[];
  connections: number[];
  source: string;
  enriched: boolean;
}

export interface MemoryStats {
  total: number;
  working: number;
  episodic: number;
  semantic: number;
  unenriched: number;
}

export interface SearchResult {
  note: Note;
  relevance_score: number;
}

export interface AskResponse {
  answer: string;
  citations: number[];
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  citations?: number[];
  timestamp: string;
}

/// AppSettings mirrors the Rust AppSettings struct — serialized as snake_case by Tauri.
export interface AppSettings {
  llm_provider: 'ollama' | 'gemini' | 'auto';
  ollama_endpoint: string;
  gemini_api_key: string;
  gemini_model: string;
  decay_rate_working: number;
  decay_rate_episodic: number;
  decay_rate_semantic: number;
  consolidation_interval_minutes: number;
  inbox_folder_path: string;
  hotkey_capture: string;
  hotkey_open: string;
  clipboard_watcher_enabled: boolean;
}

export interface LLMStatus {
  provider: string;
  available: boolean;
  model: string;
}
