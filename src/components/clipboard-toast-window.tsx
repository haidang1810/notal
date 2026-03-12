// Clipboard toast — always-on-top mini window shown when clipboard changes.
// Offers Save (creates note) and Dismiss. Auto-dismisses after 6 seconds.

import { useEffect, useState, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const PREVIEW_MAX = 120;
const AUTO_DISMISS_MS = 6000;

function truncate(text: string, max: number): string {
  return text.length > max ? text.slice(0, max) + '…' : text;
}

export function ClipboardToastWindow() {
  const [text, setText] = useState('');
  const [saving, setSaving] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  // Listen for clipboard_changed event
  useEffect(() => {
    const unlisten = listen<string>('clipboard_changed', (event) => {
      const content = event.payload?.trim();
      if (!content) return;
      setText(content);

      // Reset auto-dismiss timer on each new clipboard event
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => handleDismiss(), AUTO_DISMISS_MS);
    });

    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleDismiss = useCallback(async () => {
    if (timerRef.current) clearTimeout(timerRef.current);
    await invoke('close_clipboard_toast').catch(() => {});
  }, []);

  const handleSave = useCallback(async () => {
    if (!text.trim() || saving) return;
    setSaving(true);
    try {
      await invoke('create_note', { rawText: text, source: 'clipboard' });
    } catch (err) {
      console.error('[clipboard-toast] save failed:', err);
    }
    setSaving(false);
    handleDismiss();
  }, [text, saving, handleDismiss]);

  return (
    <div className="h-screen flex items-center justify-center bg-transparent select-none">
      <div className="w-full h-full bg-gray-900 rounded-lg shadow-2xl border border-gray-700 flex flex-col p-3 gap-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-purple-400 font-medium">Save to Notal?</span>
          <button
            onClick={handleDismiss}
            className="text-gray-500 hover:text-gray-200 text-xs leading-none cursor-pointer"
          >
            ✕
          </button>
        </div>

        <p className="text-[12px] text-gray-300 leading-relaxed flex-1 overflow-hidden line-clamp-3">
          {text ? truncate(text, PREVIEW_MAX) : 'Waiting for clipboard…'}
        </p>

        <div className="flex gap-2 justify-end">
          <button
            onClick={handleDismiss}
            className="px-3 py-1 text-xs text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
          >
            Dismiss
          </button>
          <button
            onClick={handleSave}
            disabled={saving || !text.trim()}
            className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white text-xs rounded-md transition-colors disabled:opacity-50 cursor-pointer"
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}
