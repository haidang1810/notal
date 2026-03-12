// Note input — textarea with Ctrl+Enter submit, drag-drop, and file picker

import { useState, useRef, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { ingestFile } from '../../services/tauri-commands';

interface NoteInputProps {
  onSubmit: (text: string) => Promise<void>;
  disabled?: boolean;
}

export function NoteInput({ onSubmit, disabled }: NoteInputProps) {
  const [text, setText] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [dropStatus, setDropStatus] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = useCallback(async () => {
    if (!text.trim() || submitting) return;
    setSubmitting(true);
    try {
      await onSubmit(text.trim());
      setText('');
      textareaRef.current?.focus();
    } catch {
      // error handled by parent
    } finally {
      setSubmitting(false);
    }
  }, [text, onSubmit, submitting]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  };

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);

    const items = Array.from(e.dataTransfer.items);
    const filePaths: string[] = [];
    for (const item of items) {
      if (item.kind === 'file') {
        const f = item.getAsFile();
        const tauriPath = (f as unknown as { path?: string })?.path;
        if (tauriPath) filePaths.push(tauriPath);
      }
    }
    if (filePaths.length === 0) return;

    setDropStatus(`Ingesting ${filePaths.length} file(s)...`);
    try {
      for (const p of filePaths) { await ingestFile(p); }
      setDropStatus(`Ingested ${filePaths.length} file(s)`);
    } catch (err) {
      setDropStatus(`Drop failed: ${String(err)}`);
    }
    setTimeout(() => setDropStatus(null), 3000);
  }, []);

  const handleFilePicker = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        title: 'Select files to ingest as notes',
      });
      if (!selected) return;

      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) return;

      setDropStatus(`Ingesting ${paths.length} file(s)...`);
      for (const p of paths) { await ingestFile(p); }
      setDropStatus(`Ingested ${paths.length} file(s)`);
    } catch (err) {
      setDropStatus(`File ingest failed: ${String(err)}`);
    }
    setTimeout(() => setDropStatus(null), 3000);
  }, []);

  return (
    <div
      className="flex flex-col gap-2 p-3 flex-shrink-0"
      onDragEnter={handleDragEnter}
      onDragOver={(e) => e.preventDefault()}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <div className={`relative rounded-lg border transition-colors ${
        dragOver
          ? 'border-dashed border-purple-400 bg-purple-900/10'
          : 'border-gray-700'
      }`}>
        <textarea
          ref={textareaRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled || submitting}
          placeholder="Capture a thought… (Ctrl+Enter to save, or drag & drop a file)"
          rows={3}
          className="w-full bg-transparent text-gray-100 text-sm placeholder-gray-600 p-3 resize-none outline-none rounded-lg"
        />

        {dragOver && (
          <div className="absolute inset-0 flex items-center justify-center rounded-lg pointer-events-none">
            <span className="text-purple-400 text-sm font-medium">Drop file to ingest</span>
          </div>
        )}
      </div>

      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-600">
          {dropStatus ?? 'Ctrl+Enter to save'}
        </span>
        <div className="flex items-center gap-2">
          <button
            onClick={handleFilePicker}
            disabled={submitting || disabled}
            className="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 disabled:opacity-40 text-gray-300 text-sm rounded-lg transition-colors"
            title="Choose file to ingest as note"
          >
            📎 File
          </button>
          <button
            onClick={handleSubmit}
            disabled={!text.trim() || submitting || disabled}
            className="px-4 py-1.5 bg-purple-600 hover:bg-purple-700 disabled:opacity-40 disabled:cursor-not-allowed text-white text-sm rounded-lg transition-colors"
          >
            {submitting ? 'Saving…' : 'Add Note'}
          </button>
        </div>
      </div>
    </div>
  );
}
