// Screenshot capture overlay — triggered by "screenshot-trigger" Tauri event.
// User pastes a screenshot from clipboard (Ctrl+V), adds optional caption, saves as note.
// The overlay instructs user to capture screen with OS tools first, then paste here.

import { useEffect, useState, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export function ScreenshotCaptureOverlay() {
  const [visible, setVisible] = useState(false);
  const [imageData, setImageData] = useState<string | null>(null);
  const [caption, setCaption] = useState('');
  const [saving, setSaving] = useState(false);
  const captionRef = useRef<HTMLTextAreaElement>(null);

  // Listen for screenshot-trigger event from backend hotkey
  useEffect(() => {
    const unlisten = listen('screenshot-trigger', () => {
      setVisible(true);
      setImageData(null);
      setCaption('');
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Handle paste — extract image from clipboard
  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
      if (item.type.startsWith('image/')) {
        e.preventDefault();
        const blob = item.getAsFile();
        if (!blob) continue;

        const reader = new FileReader();
        reader.onload = () => {
          setImageData(reader.result as string);
          // Focus caption after image loads
          setTimeout(() => captionRef.current?.focus(), 100);
        };
        reader.readAsDataURL(blob);
        return;
      }
    }
  }, []);

  // Global paste listener when overlay is visible
  useEffect(() => {
    if (!visible) return;

    const handler = (e: ClipboardEvent) => {
      const items = e.clipboardData?.items;
      if (!items) return;

      for (const item of items) {
        if (item.type.startsWith('image/')) {
          e.preventDefault();
          const blob = item.getAsFile();
          if (!blob) continue;

          const reader = new FileReader();
          reader.onload = () => {
            setImageData(reader.result as string);
            setTimeout(() => captionRef.current?.focus(), 100);
          };
          reader.readAsDataURL(blob);
          return;
        }
      }
    };

    window.addEventListener('paste', handler);
    return () => window.removeEventListener('paste', handler);
  }, [visible]);

  const handleSave = useCallback(async () => {
    if (!imageData && !caption.trim()) return;
    setSaving(true);
    try {
      // Build note text: caption + image reference
      const parts: string[] = [];
      if (caption.trim()) parts.push(caption.trim());
      if (imageData) parts.push('[Screenshot captured]');
      const rawText = parts.join('\n\n');

      await invoke('save_screenshot_note', { rawText, source: 'screenshot' });
      setVisible(false);
    } catch (err) {
      console.error('[screenshot] Save failed:', err);
    } finally {
      setSaving(false);
    }
  }, [imageData, caption]);

  const handleClose = useCallback(() => {
    setVisible(false);
    setImageData(null);
    setCaption('');
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      handleClose();
    }
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleSave();
    }
  }, [handleClose, handleSave]);

  if (!visible) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70"
      onKeyDown={handleKeyDown}
      onPaste={handlePaste}
    >
      <div className="bg-gray-950 border border-gray-800 rounded-xl w-full max-w-lg mx-4 flex flex-col max-h-[80vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-gray-800 flex-shrink-0">
          <h2 className="text-sm font-semibold text-gray-200">Screenshot Capture</h2>
          <button
            onClick={handleClose}
            className="text-gray-500 hover:text-gray-200 text-lg leading-none transition-colors cursor-pointer"
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-5 py-4 flex flex-col gap-3">
          {!imageData ? (
            <div className="border-2 border-dashed border-gray-700 rounded-lg p-8 text-center">
              <ScreenshotIcon />
              <p className="text-gray-400 text-sm mt-3">
                Take a screenshot with your OS tool, then paste here
              </p>
              <p className="text-gray-600 text-xs mt-1">
                Linux: PrtSc / Shift+PrtSc &middot; Press Ctrl+V to paste
              </p>
            </div>
          ) : (
            <div className="rounded-lg overflow-hidden border border-gray-800 bg-gray-900">
              <img
                src={imageData}
                alt="Screenshot preview"
                className="max-h-60 w-full object-contain"
              />
            </div>
          )}

          {/* Caption input */}
          <textarea
            ref={captionRef}
            value={caption}
            onChange={(e) => setCaption(e.target.value)}
            placeholder="Add a caption… (optional)"
            rows={2}
            className="bg-gray-900 border border-gray-700 text-gray-100 text-sm rounded-lg px-3 py-2 outline-none focus:border-purple-500 resize-none"
          />
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-5 py-3 border-t border-gray-800 flex-shrink-0">
          <span className="text-[11px] text-gray-600">Ctrl+Enter to save &middot; Esc to cancel</span>
          <div className="flex gap-2">
            <button
              onClick={handleClose}
              className="px-4 py-1.5 text-sm text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={saving || (!imageData && !caption.trim())}
              className="px-5 py-1.5 bg-purple-600 hover:bg-purple-700 text-white text-sm rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
            >
              {saving ? 'Saving…' : 'Save'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function ScreenshotIcon() {
  return (
    <svg
      width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor"
      strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"
      className="mx-auto text-gray-600"
    >
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <circle cx="8.5" cy="8.5" r="1.5" />
      <path d="m21 15-5-5L5 21" />
    </svg>
  );
}
