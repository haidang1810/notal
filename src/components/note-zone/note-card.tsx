// Sticky-note style card — square/rectangular like a paper note

import type { Note } from '../../types';
import { LayerBadge } from './layer-badge';

interface NoteCardProps {
  note: Note;
  onPin: (id: number, pinned: boolean) => void;
  onArchive: (id: number) => void;
  onDelete: (id: number) => void;
}

const LAYER_TOP_BORDER: Record<string, string> = {
  working: 'border-t-amber-500/70',
  episodic: 'border-t-blue-500/70',
  semantic: 'border-t-purple-500/70',
};

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString(undefined, {
      month: 'short', day: 'numeric',
    });
  } catch {
    return iso;
  }
}

export function NoteCard({ note, onPin, onArchive, onDelete }: NoteCardProps) {
  const topBorder = LAYER_TOP_BORDER[note.layer] ?? 'border-t-gray-700';
  const displayText = note.summary?.trim()
    ? note.summary
    : note.raw_text.slice(0, 200);

  return (
    <article
      className={`relative bg-gray-900/60 border border-gray-800/50 border-t-2 ${topBorder} rounded-lg p-3 flex flex-col group hover:bg-gray-900 hover:border-gray-700/60 transition-colors cursor-pointer aspect-square min-h-[180px]`}
    >
      {/* Header: badge + pin */}
      <div className="flex items-center gap-1.5 mb-2 flex-shrink-0">
        <LayerBadge layer={note.layer} />
        {note.pinned && (
          <span className="text-amber-400/80 text-[10px] font-medium">Pinned</span>
        )}
        {!note.enriched && (
          <span className="text-gray-500 text-[10px] ml-auto italic">Pending</span>
        )}
      </div>

      {/* Note content — fills available space */}
      <p className="text-gray-300 text-[12px] leading-relaxed flex-1 overflow-hidden line-clamp-[8]">
        {displayText}
      </p>

      {/* Footer: date + source */}
      <div className="flex items-center justify-between mt-2 pt-1.5 border-t border-gray-800/40 flex-shrink-0">
        <span className="text-[10px] text-gray-600">{formatDate(note.created_at)}</span>
        {note.source && (
          <span className="text-gray-600 text-[10px] truncate max-w-[80px]">
            {note.source}
          </span>
        )}
      </div>

      {/* Hover actions overlay — bottom of card */}
      <div className="absolute inset-x-0 bottom-0 flex justify-center gap-1 p-2 bg-gradient-to-t from-gray-900 via-gray-900/90 to-transparent rounded-b-lg opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={(e) => { e.stopPropagation(); onPin(note.id, !note.pinned); }}
          title={note.pinned ? 'Unpin' : 'Pin'}
          className="px-2 py-1 rounded text-[11px] text-gray-400 hover:text-amber-400 hover:bg-gray-800 transition-colors cursor-pointer"
        >
          {note.pinned ? 'Unpin' : 'Pin'}
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onArchive(note.id); }}
          title="Archive"
          className="px-2 py-1 rounded text-[11px] text-gray-400 hover:text-blue-400 hover:bg-gray-800 transition-colors cursor-pointer"
        >
          Archive
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onDelete(note.id); }}
          title="Delete"
          className="px-2 py-1 rounded text-[11px] text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors cursor-pointer"
        >
          Delete
        </button>
      </div>
    </article>
  );
}
