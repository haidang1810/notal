// Note Zone — full-page view combining NoteInput + NoteList with header

import type { LayerFilter } from '../../hooks/use-notes';
import type { Note } from '../../types';
import { NoteInput } from './note-input';
import { NoteList } from './note-list';

interface NoteZoneProps {
  notes: Note[];
  layerFilter: LayerFilter;
  onLayerFilter: (layer: LayerFilter) => void;
  onCreateNote: (text: string) => Promise<void>;
  onPin: (id: number, pinned: boolean) => void;
  onArchive: (id: number) => void;
  onDelete: (id: number) => void;
  loading?: boolean;
}

export function NoteZone({
  notes, layerFilter, onLayerFilter,
  onCreateNote, onPin, onArchive, onDelete, loading,
}: NoteZoneProps) {
  return (
    <section className="flex flex-col h-full overflow-hidden">
      {/* View header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-gray-800 flex-shrink-0">
        <h1 className="text-sm font-semibold text-gray-200">Notes</h1>
        <span className="text-xs text-gray-500">{notes.length} notes</span>
      </div>

      <NoteInput onSubmit={onCreateNote} disabled={loading} />
      <NoteList
        notes={notes}
        layerFilter={layerFilter}
        onLayerFilter={onLayerFilter}
        onPin={onPin}
        onArchive={onArchive}
        onDelete={onDelete}
        loading={loading}
      />
    </section>
  );
}
