// Scrollable note list with layer filter tabs

import type { Note } from '../../types';
import type { LayerFilter } from '../../hooks/use-notes';
import { NoteCard } from './note-card';

interface NoteListProps {
  notes: Note[];
  layerFilter: LayerFilter;
  onLayerFilter: (layer: LayerFilter) => void;
  onPin: (id: number, pinned: boolean) => void;
  onArchive: (id: number) => void;
  onDelete: (id: number) => void;
  loading?: boolean;
}

const TABS: { key: LayerFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'working', label: 'Working' },
  { key: 'episodic', label: 'Episodic' },
  { key: 'semantic', label: 'Semantic' },
];

const TAB_ACTIVE: Record<string, string> = {
  all: 'text-gray-100 border-b-2 border-gray-400',
  working: 'text-amber-400 border-b-2 border-amber-500',
  episodic: 'text-blue-400 border-b-2 border-blue-500',
  semantic: 'text-purple-400 border-b-2 border-purple-500',
};

export function NoteList({
  notes, layerFilter, onLayerFilter,
  onPin, onArchive, onDelete, loading,
}: NoteListProps) {
  return (
    <div className="flex flex-col flex-1 overflow-hidden">
      {/* Layer filter tabs */}
      <div className="flex gap-1 px-3 pt-1 border-b border-gray-800 flex-shrink-0">
        {TABS.map(({ key, label }) => (
          <button
            key={key}
            onClick={() => onLayerFilter(key)}
            className={`px-3 py-1.5 text-xs font-medium transition-colors pb-2 ${
              layerFilter === key
                ? TAB_ACTIVE[key]
                : 'text-gray-500 hover:text-gray-300'
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Scrollable note grid — sticky-note style */}
      <div className="flex-1 overflow-y-auto px-3 py-3 grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3 auto-rows-min content-start">
        {loading && (
          <p className="text-gray-500 text-sm text-center py-4">Loading…</p>
        )}

        {!loading && notes.length === 0 && (
          <div className="text-center py-8">
            <p className="text-gray-500 text-sm">No notes yet.</p>
            <p className="text-gray-600 text-xs mt-1">
              Type above or drag & drop a file to capture your first note.
            </p>
          </div>
        )}

        {notes.map((note) => (
          <NoteCard
            key={note.id}
            note={note}
            onPin={onPin}
            onArchive={onArchive}
            onDelete={onDelete}
          />
        ))}
      </div>
    </div>
  );
}
