// Fixed bottom status bar — layer counts and LLM provider status

import type { MemoryStats } from '../../types';

interface StatusBarProps {
  stats: MemoryStats;
  llmOnline?: boolean;
}

export function StatusBar({ stats, llmOnline = false }: StatusBarProps) {
  return (
    <footer className="flex items-center gap-4 px-4 py-1.5 bg-gray-900 border-t border-gray-800 text-xs flex-shrink-0">
      <span className="text-gray-500">Memory:</span>

      <span className="flex items-center gap-1">
        <span className="w-2 h-2 rounded-full bg-amber-500 inline-block" />
        <span className="text-amber-400">Working</span>
        <span className="text-gray-300 ml-0.5">{stats.working}</span>
      </span>

      <span className="flex items-center gap-1">
        <span className="w-2 h-2 rounded-full bg-blue-500 inline-block" />
        <span className="text-blue-400">Episodic</span>
        <span className="text-gray-300 ml-0.5">{stats.episodic}</span>
      </span>

      <span className="flex items-center gap-1">
        <span className="w-2 h-2 rounded-full bg-purple-500 inline-block" />
        <span className="text-purple-400">Semantic</span>
        <span className="text-gray-300 ml-0.5">{stats.semantic}</span>
      </span>

      <span className="text-gray-500">|</span>
      <span className="text-gray-400">Total: <span className="text-gray-200">{stats.total}</span></span>

      {stats.unenriched > 0 && (
        <span className="text-yellow-500 text-xs">
          {stats.unenriched} pending enrichment
        </span>
      )}

      <span className="ml-auto flex items-center gap-1.5">
        <span
          className={`w-2 h-2 rounded-full ${llmOnline ? 'bg-green-500' : 'bg-red-500'}`}
        />
        <span className="text-gray-400">{llmOnline ? 'LLM online' : 'LLM offline'}</span>
      </span>
    </footer>
  );
}
