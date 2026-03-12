// Hook for app settings — loads from Tauri backend (SQLite), saves via update_settings command.
// Falls back to in-memory defaults if the Tauri command fails (e.g. during development).

import { useState, useEffect, useCallback } from 'react';
import type { AppSettings } from '../types';
import { getSettings, updateSettings } from '../services/tauri-commands';

const DEFAULT_SETTINGS: AppSettings = {
  llm_provider: 'auto',
  ollama_endpoint: 'http://localhost:11434',
  gemini_api_key: '',
  gemini_model: 'gemini-2.0-flash-lite',
  decay_rate_working: 0.1,
  decay_rate_episodic: 0.05,
  decay_rate_semantic: 0.01,
  consolidation_interval_minutes: 60,
  inbox_folder_path: '',
  hotkey_capture: 'Ctrl+Shift+N',
  hotkey_open: 'Ctrl+Shift+S',
  clipboard_watcher_enabled: false,
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [saving, setSaving] = useState(false);

  // Load settings from backend on mount
  useEffect(() => {
    getSettings()
      .then((loaded) => setSettings(loaded))
      .catch(() => {
        // Non-fatal — keep defaults (e.g. during browser-based development)
      });
  }, []);

  // Persist a partial or full settings update to the backend
  const saveSettings = useCallback(async (updates: Partial<AppSettings>) => {
    setSettings((prev) => {
      const next = { ...prev, ...updates };
      setSaving(true);
      updateSettings(next)
        .catch(() => {
          // Non-fatal — settings stay in local state even if persist fails
        })
        .finally(() => setSaving(false));
      return next;
    });
  }, []);

  const resetSettings = useCallback(async () => {
    setSettings(DEFAULT_SETTINGS);
    setSaving(true);
    updateSettings(DEFAULT_SETTINGS)
      .catch(() => {})
      .finally(() => setSaving(false));
  }, []);

  return { settings, saveSettings, resetSettings, saving };
}
