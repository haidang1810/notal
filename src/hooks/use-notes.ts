// Hook for notes state — fetch, create, delete, pin, archive, layer filter

import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { Note } from '../types';
import * as cmd from '../services/tauri-commands';

export type LayerFilter = 'all' | 'working' | 'episodic' | 'semantic';

export function useNotes() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [layerFilter, setLayerFilter] = useState<LayerFilter>('all');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const layer = layerFilter === 'all' ? undefined : layerFilter;
      const data = await cmd.getNotes(layer);
      setNotes(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [layerFilter]);

  // Initial fetch + re-fetch when layer filter changes
  useEffect(() => {
    refresh();
  }, [refresh]);

  // Listen for enrichment events to refresh the list
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen('note_enriched', () => {
      refresh();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => { unlisten?.(); };
  }, [refresh]);

  const createNote = useCallback(async (text: string, source?: string) => {
    setError(null);
    try {
      await cmd.createNote(text, source);
      await refresh();
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, [refresh]);

  const deleteNote = useCallback(async (id: number) => {
    setError(null);
    try {
      await cmd.deleteNote(id);
      setNotes((prev) => prev.filter((n) => n.id !== id));
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, []);

  const pinNote = useCallback(async (id: number, pinned: boolean) => {
    setError(null);
    try {
      await cmd.updateNote(id, undefined, pinned, undefined);
      setNotes((prev) => prev.map((n) => n.id === id ? { ...n, pinned } : n));
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, []);

  const archiveNote = useCallback(async (id: number) => {
    setError(null);
    try {
      await cmd.updateNote(id, undefined, undefined, true);
      setNotes((prev) => prev.filter((n) => n.id !== id));
    } catch (e) {
      setError(String(e));
      throw e;
    }
  }, []);

  return {
    notes,
    layerFilter,
    setLayerFilter,
    loading,
    error,
    refresh,
    createNote,
    deleteNote,
    pinNote,
    archiveNote,
  };
}
