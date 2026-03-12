// Keyboard shortcut recorder — click to focus, press key combo, displays formatted string.
// Outputs Tauri-compatible format: "Ctrl+Shift+N", "Alt+K", etc.

import { useState, useCallback, useRef } from 'react';

interface HotkeyInputProps {
  value: string;
  onChange: (hotkey: string) => void;
  placeholder?: string;
}

// Map browser key names to Tauri accelerator format
const KEY_MAP: Record<string, string> = {
  Control: 'Ctrl',
  Meta: 'Super',
  ' ': 'Space',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
};

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta']);

function formatKey(key: string): string {
  return KEY_MAP[key] ?? (key.length === 1 ? key.toUpperCase() : key);
}

export function HotkeyInput({ value, onChange, placeholder }: HotkeyInputProps) {
  const [recording, setRecording] = useState(false);
  const inputRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    // Ignore lone modifier presses — wait for a non-modifier key
    if (MODIFIER_KEYS.has(e.key)) return;

    // Build combo string
    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Super');

    parts.push(formatKey(e.key));
    const combo = parts.join('+');

    onChange(combo);
    setRecording(false);
    inputRef.current?.blur();
  }, [onChange]);

  const handleFocus = useCallback(() => setRecording(true), []);
  const handleBlur = useCallback(() => setRecording(false), []);

  return (
    <div
      ref={inputRef}
      tabIndex={0}
      onFocus={handleFocus}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      className={`bg-gray-900 border text-sm rounded-lg px-3 py-2 outline-none cursor-pointer select-none transition-colors ${
        recording
          ? 'border-purple-500 text-purple-300'
          : 'border-gray-700 text-gray-100 hover:border-gray-600'
      }`}
    >
      {recording ? (
        <span className="text-purple-400 animate-pulse">Press key combo…</span>
      ) : value ? (
        <span>{value}</span>
      ) : (
        <span className="text-gray-500">{placeholder ?? 'Click to record…'}</span>
      )}
    </div>
  );
}
