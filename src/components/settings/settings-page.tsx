// Settings view — inline page (not modal) with LLM and Memory sections

import { useState } from 'react';
import type { AppSettings } from '../../types';
import { LlmSettings } from './llm-settings';
import { MemorySettings } from './memory-settings';

type SettingsTab = 'llm' | 'memory';

interface SettingsPageProps {
  settings: AppSettings;
  onSave: (updates: Partial<AppSettings>) => void;
  onClose: () => void;
}

export function SettingsPage({ settings, onSave, onClose }: SettingsPageProps) {
  const [tab, setTab] = useState<SettingsTab>('llm');
  const [draft, setDraft] = useState<AppSettings>({ ...settings });

  const handleChange = (updates: Partial<AppSettings>) => {
    setDraft((prev) => ({ ...prev, ...updates }));
  };

  const handleSave = () => {
    onSave(draft);
    onClose();
  };

  return (
    <section className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-gray-800 flex-shrink-0">
        <h1 className="text-sm font-semibold text-gray-200">Settings</h1>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 px-5 pt-3 flex-shrink-0 border-b border-gray-800">
        {(['llm', 'memory'] as SettingsTab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 text-sm font-medium transition-colors cursor-pointer ${
              tab === t
                ? 'text-purple-400 border-b-2 border-purple-500'
                : 'text-gray-500 hover:text-gray-300'
            }`}
          >
            {t === 'llm' ? 'LLM' : 'Memory'}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-5 py-4">
        <div className="max-w-lg">
          {tab === 'llm' && (
            <LlmSettings settings={draft} onChange={handleChange} />
          )}
          {tab === 'memory' && (
            <MemorySettings settings={draft} onChange={handleChange} />
          )}
        </div>
      </div>

      {/* Footer */}
      <div className="flex justify-end gap-2 px-5 py-3 border-t border-gray-800 flex-shrink-0">
        <button
          onClick={onClose}
          className="px-4 py-1.5 text-sm text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
        >
          Cancel
        </button>
        <button
          onClick={handleSave}
          className="px-5 py-1.5 bg-purple-600 hover:bg-purple-700 text-white text-sm rounded-lg transition-colors cursor-pointer"
        >
          Save
        </button>
      </div>
    </section>
  );
}
