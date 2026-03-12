// Quick capture floating window — minimal textarea, Enter saves, Escape closes

import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function QuickCaptureWindow() {
  const [text, setText] = useState('');
  const [saving, setSaving] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-focus textarea when window mounts
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  async function handleSave() {
    const trimmed = text.trim();
    if (!trimmed || saving) return;
    setSaving(true);
    try {
      await invoke('save_quick_capture', { rawText: trimmed });
      await invoke('close_quick_capture');
    } catch (err) {
      console.error('Quick capture save failed:', err);
      setSaving(false);
    }
  }

  async function handleClose() {
    await invoke('close_quick_capture');
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSave();
    }
    if (e.key === 'Escape') {
      handleClose();
    }
  }

  return (
    <div className="h-screen flex items-center justify-center bg-transparent">
      <div className="w-full h-full bg-gray-900 rounded-lg shadow-2xl border border-gray-700 flex flex-col p-3 gap-2">
        <textarea
          ref={textareaRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Capture a thought… (Enter to save, Esc to cancel)"
          disabled={saving}
          className="
            flex-1 bg-transparent text-gray-100 text-sm
            placeholder-gray-500 resize-none outline-none
            leading-relaxed
          "
        />
        <div className="flex justify-between items-center text-xs text-gray-600 select-none">
          <span>Hotkey to toggle</span>
          <span>{saving ? 'Saving…' : 'Enter ↵ save · Esc cancel'}</span>
        </div>
      </div>
    </div>
  );
}
