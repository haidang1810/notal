// LLM settings section — provider selector, endpoint, API key, clipboard watcher toggle, hotkeys

import { useState, useEffect } from 'react';
import type { AppSettings } from '../../types';
import { testLlmConnection, reregisterHotkeys, listGeminiModels } from '../../services/tauri-commands';
import type { GeminiModelInfo } from '../../services/tauri-commands';
import { HotkeyInput } from './hotkey-input';

interface LlmSettingsProps {
  settings: AppSettings;
  onChange: (updates: Partial<AppSettings>) => void;
}

export function LlmSettings({ settings, onChange }: LlmSettingsProps) {
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [hotkeyStatus, setHotkeyStatus] = useState<string | null>(null);
  const [geminiModels, setGeminiModels] = useState<GeminiModelInfo[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);

  // Fetch Gemini models when API key changes (debounced)
  useEffect(() => {
    if (!settings.gemini_api_key || settings.gemini_api_key.length < 10) {
      setGeminiModels([]);
      return;
    }
    const timer = setTimeout(async () => {
      setLoadingModels(true);
      try {
        const models = await listGeminiModels(settings.gemini_api_key);
        setGeminiModels(models);
      } catch {
        setGeminiModels([]);
      }
      setLoadingModels(false);
    }, 500);
    return () => clearTimeout(timer);
  }, [settings.gemini_api_key]);

  const handleReregisterHotkeys = async () => {
    setHotkeyStatus('applying…');
    try {
      await reregisterHotkeys();
      setHotkeyStatus('applied');
    } catch (err) {
      setHotkeyStatus(`error: ${err}`);
    }
    setTimeout(() => setHotkeyStatus(null), 3000);
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await testLlmConnection(settings);
      setTestResult(ok);
    } catch {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-gray-300">LLM Provider</h3>

      <div className="flex flex-col gap-1">
        <label className="text-xs text-gray-400">Provider</label>
        <select
          value={settings.llm_provider}
          onChange={(e) => onChange({ llm_provider: e.target.value as AppSettings['llm_provider'] })}
          className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
        >
          <option value="auto">Auto (try Ollama, fallback Gemini)</option>
          <option value="ollama">Ollama (local)</option>
          <option value="gemini">Gemini (cloud)</option>
        </select>
      </div>

      {settings.llm_provider !== 'gemini' && (
        <div className="flex flex-col gap-1">
          <label className="text-xs text-gray-400">Ollama Endpoint</label>
          <input
            type="url"
            value={settings.ollama_endpoint}
            onChange={(e) => onChange({ ollama_endpoint: e.target.value })}
            placeholder="http://localhost:11434"
            className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
          />
        </div>
      )}

      {settings.llm_provider !== 'ollama' && (
        <div className="flex flex-col gap-1">
          <label className="text-xs text-gray-400">Gemini API Key</label>
          <input
            type="password"
            value={settings.gemini_api_key}
            onChange={(e) => onChange({ gemini_api_key: e.target.value })}
            placeholder="AIza…"
            className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
          />
        </div>
      )}

      {/* Gemini model selector — shown when Gemini is active and API key is set */}
      {settings.llm_provider !== 'ollama' && settings.gemini_api_key.length >= 10 && (
        <div className="flex flex-col gap-1">
          <label className="text-xs text-gray-400">Gemini Model</label>
          {loadingModels ? (
            <span className="text-xs text-gray-500 py-2">Loading models…</span>
          ) : geminiModels.length > 0 ? (
            <select
              value={settings.gemini_model}
              onChange={(e) => onChange({ gemini_model: e.target.value })}
              className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
            >
              {geminiModels.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.display_name}
                </option>
              ))}
            </select>
          ) : (
            <input
              type="text"
              value={settings.gemini_model}
              onChange={(e) => onChange({ gemini_model: e.target.value })}
              placeholder="gemini-2.0-flash-lite"
              className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
            />
          )}
        </div>
      )}

      {/* Test connection */}
      <div className="flex items-center gap-3">
        <button
          onClick={handleTestConnection}
          disabled={testing}
          className="px-4 py-1.5 bg-gray-800 hover:bg-gray-700 text-gray-200 text-sm rounded-lg transition-colors disabled:opacity-50"
        >
          {testing ? 'Testing…' : 'Test Connection'}
        </button>
        {testResult === true && (
          <span className="text-xs text-green-400">Connected</span>
        )}
        {testResult === false && (
          <span className="text-xs text-red-400">Unavailable</span>
        )}
      </div>

      {/* Clipboard watcher toggle — off by default for privacy */}
      <div className="border-t border-gray-800 pt-4">
        <h3 className="text-sm font-semibold text-gray-300 mb-3">Clipboard Watcher</h3>
        <div className="flex items-center gap-3">
          <input
            type="checkbox"
            id="clipboard-watcher"
            checked={settings.clipboard_watcher_enabled}
            onChange={(e) => onChange({ clipboard_watcher_enabled: e.target.checked })}
            className="accent-purple-500"
          />
          <label htmlFor="clipboard-watcher" className="text-sm text-gray-300 cursor-pointer">
            Monitor clipboard for text changes
          </label>
        </div>
        <p className="text-xs text-gray-500 mt-1 ml-6">
          Disabled by default. Enable to get "Save to Notal?" prompts when you copy text.
        </p>
      </div>

      {/* Global hotkey configuration */}
      <div className="border-t border-gray-800 pt-4">
        <h3 className="text-sm font-semibold text-gray-300 mb-3">Global Hotkeys</h3>
        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-400">Quick Capture</label>
            <HotkeyInput
              value={settings.hotkey_capture}
              onChange={(v) => onChange({ hotkey_capture: v })}
              placeholder="Click and press keys… (default: Ctrl+Shift+N)"
            />
          </div>
          <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-400">Screenshot Capture</label>
            <HotkeyInput
              value={settings.hotkey_open}
              onChange={(v) => onChange({ hotkey_open: v })}
              placeholder="Click and press keys… (default: Ctrl+Shift+S)"
            />
          </div>
          <p className="text-xs text-gray-500">
            Click the input and press your desired key combination. Save settings then click Apply.
          </p>
          <div className="flex items-center gap-3">
            <button
              onClick={handleReregisterHotkeys}
              className="px-4 py-1.5 bg-gray-800 hover:bg-gray-700 text-gray-200 text-sm rounded-lg transition-colors"
            >
              Apply Hotkeys
            </button>
            {hotkeyStatus && (
              <span className={`text-xs ${hotkeyStatus.startsWith('error') ? 'text-red-400' : 'text-green-400'}`}>
                {hotkeyStatus}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
