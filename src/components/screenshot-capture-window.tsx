// Screenshot capture — always-on-top floating window.
// Reads image from clipboard via Tauri plugin. User takes screenshot with OS tool, then clicks Paste.

import { useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { readImage } from '@tauri-apps/plugin-clipboard-manager';

export function ScreenshotCaptureWindow() {
  const [imageData, setImageData] = useState<string | null>(null);
  const [caption, setCaption] = useState('');
  const [saving, setSaving] = useState(false);
  const [pasting, setPasting] = useState(false);
  const captionRef = useRef<HTMLTextAreaElement>(null);

  const handlePasteFromClipboard = useCallback(async () => {
    setPasting(true);
    try {
      const img = await readImage();
      const rgba = await img.rgba();
      const size = await img.size();
      const width = size?.width ?? 0;
      const height = size?.height ?? 0;
      if (width > 0 && height > 0) {
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d')!;
        const imageDataObj = new ImageData(new Uint8ClampedArray(rgba), width, height);
        ctx.putImageData(imageDataObj, 0, 0);
        const dataUrl = canvas.toDataURL('image/png');
        setImageData(dataUrl);
        setTimeout(() => captionRef.current?.focus(), 100);
      }
    } catch (err) {
      console.error('[screenshot] clipboard read failed:', err);
    }
    setPasting(false);
  }, []);

  const handleClose = useCallback(async () => {
    setImageData(null);
    setCaption('');
    await invoke('close_screenshot_capture').catch(() => {});
  }, []);

  const handleSave = useCallback(async () => {
    if (!imageData && !caption.trim()) return;
    setSaving(true);
    try {
      const parts: string[] = [];
      if (caption.trim()) parts.push(caption.trim());
      if (imageData) parts.push('[Screenshot captured]');
      await invoke('save_screenshot_note', { rawText: parts.join('\n\n'), source: 'screenshot' });
    } catch (err) {
      console.error('[screenshot] save failed:', err);
    }
    setSaving(false);
    handleClose();
  }, [imageData, caption, handleClose]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') handleClose();
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleSave();
    }
    // Ctrl+V also triggers paste
    if (e.key === 'v' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handlePasteFromClipboard();
    }
  }, [handleClose, handleSave, handlePasteFromClipboard]);

  return (
    <div
      className="h-screen bg-gray-900 flex flex-col select-none"
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-700 flex-shrink-0">
        <span className="text-xs text-purple-400 font-medium">Screenshot Capture</span>
        <button
          onClick={handleClose}
          className="text-gray-500 hover:text-gray-200 text-sm leading-none cursor-pointer"
        >
          ✕
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden p-3 flex flex-col gap-2 min-h-0">
        {!imageData ? (
          <div className="flex-1 border-2 border-dashed border-gray-700 rounded-lg flex flex-col items-center justify-center gap-3">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor"
              strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="text-gray-600">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <path d="m21 15-5-5L5 21" />
            </svg>
            <p className="text-gray-500 text-xs text-center px-4">
              1. Take screenshot (PrtSc)<br/>
              2. Click Paste or press Ctrl+V
            </p>
            <button
              onClick={handlePasteFromClipboard}
              disabled={pasting}
              className="px-4 py-1.5 bg-purple-600 hover:bg-purple-700 text-white text-xs rounded-md transition-colors disabled:opacity-50 cursor-pointer"
            >
              {pasting ? 'Reading…' : 'Paste from Clipboard'}
            </button>
          </div>
        ) : (
          <div className="flex-1 rounded-lg overflow-hidden border border-gray-700 min-h-0">
            <img src={imageData} alt="Preview" className="w-full h-full object-contain" />
          </div>
        )}

        <textarea
          ref={captionRef}
          value={caption}
          onChange={(e) => setCaption(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Caption… (optional)"
          rows={2}
          className="bg-gray-800 border border-gray-700 text-gray-100 text-xs rounded-lg px-3 py-2 outline-none focus:border-purple-500 resize-none flex-shrink-0"
        />
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-4 py-2 border-t border-gray-700 flex-shrink-0">
        <span className="text-[10px] text-gray-600">Ctrl+Enter save · Esc cancel</span>
        <div className="flex gap-2">
          <button onClick={handleClose}
            className="px-3 py-1 text-xs text-gray-400 hover:text-gray-200 transition-colors cursor-pointer">
            Cancel
          </button>
          <button onClick={handleSave}
            disabled={saving || (!imageData && !caption.trim())}
            className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white text-xs rounded-md transition-colors disabled:opacity-50 cursor-pointer">
            {saving ? 'Saving…' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}
