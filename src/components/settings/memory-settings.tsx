// Memory settings section — decay rates, consolidation interval, inbox folder path

import type { AppSettings } from '../../types';

interface MemorySettingsProps {
  settings: AppSettings;
  onChange: (updates: Partial<AppSettings>) => void;
}

interface DecayRowProps {
  label: string;
  value: number;
  color: string;
  onUpdate: (v: number) => void;
}

function DecayRow({ label, value, color, onUpdate }: DecayRowProps) {
  return (
    <div className="flex items-center gap-3">
      <span className={`text-xs w-20 ${color}`}>{label}</span>
      <input
        type="number"
        min="0"
        max="1"
        step="0.01"
        value={value}
        onChange={(e) => onUpdate(Number(e.target.value))}
        className="w-24 bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-1.5 outline-none focus:border-purple-500"
      />
      <span className="text-xs text-gray-500">per cycle</span>
    </div>
  );
}

export function MemorySettings({ settings, onChange }: MemorySettingsProps) {
  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-gray-300">Memory Settings</h3>

      <div className="flex flex-col gap-1">
        <label className="text-xs text-gray-400 mb-2">Decay Rates</label>
        <DecayRow
          label="Working"
          value={settings.decay_rate_working}
          color="text-amber-400"
          onUpdate={(v) => onChange({ decay_rate_working: v })}
        />
        <DecayRow
          label="Episodic"
          value={settings.decay_rate_episodic}
          color="text-blue-400"
          onUpdate={(v) => onChange({ decay_rate_episodic: v })}
        />
        <DecayRow
          label="Semantic"
          value={settings.decay_rate_semantic}
          color="text-purple-400"
          onUpdate={(v) => onChange({ decay_rate_semantic: v })}
        />
      </div>

      <div className="flex flex-col gap-1">
        <label className="text-xs text-gray-400">Consolidation Interval (minutes)</label>
        <input
          type="number"
          min="1"
          max="10080"
          value={settings.consolidation_interval_minutes}
          onChange={(e) => onChange({ consolidation_interval_minutes: Number(e.target.value) })}
          className="w-32 bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
        />
      </div>

      <div className="flex flex-col gap-1">
        <label className="text-xs text-gray-400">Inbox Folder Path</label>
        <input
          type="text"
          value={settings.inbox_folder_path}
          onChange={(e) => onChange({ inbox_folder_path: e.target.value })}
          placeholder="~/Notal/inbox (leave empty for default)"
          className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500"
        />
        <p className="text-xs text-gray-500">
          Default: ~/Notal/inbox. Drop files here to auto-ingest as notes.
        </p>
      </div>
    </div>
  );
}
