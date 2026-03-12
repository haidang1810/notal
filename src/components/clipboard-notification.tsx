// Clipboard notification — listens for "clipboard_changed" Tauri event and shows a toast.
// Offers "Save" (creates a note) and "Dismiss" buttons. Auto-dismisses after 5 seconds.

import { useEffect, useState, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { createNote } from '../services/tauri-commands';

interface ClipboardToast {
  id: number;
  text: string;
}

const PREVIEW_MAX_CHARS = 100;
const AUTO_DISMISS_MS = 5000;

function truncate(text: string, max: number): string {
  return text.length > max ? text.slice(0, max) + '…' : text;
}

export function ClipboardNotification() {
  const [toasts, setToasts] = useState<ClipboardToast[]>([]);
  const nextId = useRef(0);

  const dismiss = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const handleSave = useCallback(async (toast: ClipboardToast) => {
    try {
      await createNote(toast.text, 'clipboard');
    } catch (err) {
      console.error('[clipboard] Failed to save note:', err);
    }
    dismiss(toast.id);
  }, [dismiss]);

  useEffect(() => {
    // Listen for clipboard change events emitted by the Rust clipboard watcher
    const unlisten = listen<string>('clipboard_changed', (event) => {
      const text = event.payload;
      if (!text?.trim()) return;

      const id = nextId.current++;
      const toast: ClipboardToast = { id, text };
      setToasts((prev) => [...prev.slice(-4), toast]); // cap at 5 toasts

      // Auto-dismiss after timeout
      setTimeout(() => dismiss(id), AUTO_DISMISS_MS);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [dismiss]);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 flex flex-col gap-2 z-50 max-w-sm">
      {toasts.map((toast) => (
        <ToastCard
          key={toast.id}
          toast={toast}
          onSave={() => handleSave(toast)}
          onDismiss={() => dismiss(toast.id)}
        />
      ))}
    </div>
  );
}

interface ToastCardProps {
  toast: ClipboardToast;
  onSave: () => void;
  onDismiss: () => void;
}

function ToastCard({ toast, onSave, onDismiss }: ToastCardProps) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg shadow-xl p-3 flex flex-col gap-2 animate-in slide-in-from-bottom-2">
      <div className="flex items-start justify-between gap-2">
        <span className="text-xs text-gray-400 font-medium">Clipboard</span>
        <button
          onClick={onDismiss}
          className="text-gray-500 hover:text-gray-300 text-xs leading-none"
          aria-label="Dismiss"
        >
          ✕
        </button>
      </div>
      <p className="text-sm text-gray-200 leading-snug break-words">
        {truncate(toast.text, PREVIEW_MAX_CHARS)}
      </p>
      <div className="flex gap-2 justify-end">
        <button
          onClick={onDismiss}
          className="px-3 py-1 text-xs text-gray-400 hover:text-gray-200 transition-colors"
        >
          Dismiss
        </button>
        <button
          onClick={onSave}
          className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white text-xs rounded-md transition-colors"
        >
          Save to Notal
        </button>
      </div>
    </div>
  );
}
