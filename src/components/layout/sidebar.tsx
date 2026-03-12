// Collapsible left sidebar — Obsidian-style navigation with memory stats

import type { MemoryStats } from '../../types';

type View = 'notes' | 'ask' | 'settings';

interface SidebarProps {
  activeView: View;
  onViewChange: (view: View) => void;
  stats: MemoryStats;
  llmOnline: boolean;
  collapsed: boolean;
  onToggle: () => void;
}

export function Sidebar({
  activeView, onViewChange, stats, llmOnline, collapsed, onToggle,
}: SidebarProps) {
  return (
    <aside
      className={`flex flex-col h-full bg-gray-900 border-r border-gray-800 transition-all duration-200 flex-shrink-0 ${
        collapsed ? 'w-12' : 'w-52'
      }`}
    >
      {/* Toggle + Brand */}
      <div className="flex items-center h-11 px-2 border-b border-gray-800 flex-shrink-0">
        {!collapsed && (
          <span className="text-purple-400 font-bold tracking-wide text-sm ml-1 flex-1">
            Notal
          </span>
        )}
        <button
          onClick={onToggle}
          className="p-1.5 rounded-md text-gray-500 hover:text-gray-200 hover:bg-gray-800 transition-colors cursor-pointer"
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          <SidebarToggleIcon collapsed={collapsed} />
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex flex-col gap-0.5 p-1.5 flex-1">
        <NavItem
          icon={<NotesIcon />}
          label="Notes"
          active={activeView === 'notes'}
          collapsed={collapsed}
          onClick={() => onViewChange('notes')}
        />
        <NavItem
          icon={<AskIcon />}
          label="Ask AI"
          active={activeView === 'ask'}
          collapsed={collapsed}
          onClick={() => onViewChange('ask')}
        />
        <NavItem
          icon={<SettingsIcon />}
          label="Settings"
          active={activeView === 'settings'}
          collapsed={collapsed}
          onClick={() => onViewChange('settings')}
        />
      </nav>

      {/* Memory stats footer */}
      <div className="border-t border-gray-800 p-2 flex-shrink-0">
        {collapsed ? (
          <div className="flex flex-col items-center gap-1.5">
            <span className="text-gray-500 text-[10px]">{stats.total}</span>
            <span className={`w-2 h-2 rounded-full ${llmOnline ? 'bg-green-500' : 'bg-red-500'}`} />
          </div>
        ) : (
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-gray-500">Memory</span>
              <span className="text-gray-400">{stats.total} total</span>
            </div>
            <div className="flex gap-2 text-[11px]">
              <LayerStat color="bg-amber-500" label="W" count={stats.working} />
              <LayerStat color="bg-blue-500" label="E" count={stats.episodic} />
              <LayerStat color="bg-purple-500" label="S" count={stats.semantic} />
            </div>
            {stats.unenriched > 0 && (
              <span className="text-yellow-500 text-[10px]">
                {stats.unenriched} pending
              </span>
            )}
            <div className="flex items-center gap-1.5 mt-0.5">
              <span className={`w-1.5 h-1.5 rounded-full ${llmOnline ? 'bg-green-500' : 'bg-red-500'}`} />
              <span className="text-gray-500 text-[10px]">
                {llmOnline ? 'LLM online' : 'LLM offline'}
              </span>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
}

// ─── Sub-components ──────────────────────────────────────────

function NavItem({
  icon, label, active, collapsed, onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  collapsed: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={collapsed ? label : undefined}
      className={`flex items-center gap-2 px-2.5 py-2 rounded-md text-sm transition-colors cursor-pointer ${
        active
          ? 'bg-purple-600/20 text-purple-300'
          : 'text-gray-400 hover:text-gray-200 hover:bg-gray-800'
      } ${collapsed ? 'justify-center' : ''}`}
    >
      {icon}
      {!collapsed && <span>{label}</span>}
    </button>
  );
}

function LayerStat({ color, label, count }: { color: string; label: string; count: number }) {
  return (
    <span className="flex items-center gap-1">
      <span className={`w-1.5 h-1.5 rounded-full ${color} inline-block`} />
      <span className="text-gray-500">{label}</span>
      <span className="text-gray-300">{count}</span>
    </span>
  );
}

// ─── Icons (Lucide-style SVG, 18x18) ──────────────────────────

function SidebarToggleIcon({ collapsed }: { collapsed: boolean }) {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
      strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      {collapsed ? (
        <>
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <line x1="9" y1="3" x2="9" y2="21" />
          <polyline points="14 9 17 12 14 15" />
        </>
      ) : (
        <>
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <line x1="9" y1="3" x2="9" y2="21" />
          <polyline points="15 9 12 12 15 15" />
        </>
      )}
    </svg>
  );
}

function NotesIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
      strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <polyline points="10 9 9 9 8 9" />
    </svg>
  );
}

function AskIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
      strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
      strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}
