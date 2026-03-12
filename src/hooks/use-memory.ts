// Hook for memory stats — polls every 30s, refreshes on note_enriched event.
// Also tracks LLM online/offline state via "llm_status" backend events.

import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { MemoryStats } from '../types';
import { getMemoryStats } from '../services/tauri-commands';

const POLL_INTERVAL_MS = 30_000;

export function useMemory() {
  const [stats, setStats] = useState<MemoryStats>({
    total: 0,
    working: 0,
    episodic: 0,
    semantic: 0,
    unenriched: 0,
  });
  const [loading, setLoading] = useState(false);
  // Starts as true (optimistic) — backend will emit "offline" quickly if unavailable
  const [isOnline, setIsOnline] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getMemoryStats();
      setStats(data);
    } catch {
      // Non-fatal — keep showing last known stats
    } finally {
      setLoading(false);
    }
  }, []);

  // Initial fetch + polling
  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [refresh]);

  // Refresh on enrichment events
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen('note_enriched', () => {
      refresh();
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [refresh]);

  // Track LLM availability from enrichment worker broadcasts
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>('llm_status', (event) => {
      setIsOnline(event.payload === 'online');
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  return { stats, loading, refresh, isOnline };
}
