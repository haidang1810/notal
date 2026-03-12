// Root app component — sidebar layout with view-based navigation.
// Routes to special windows based on URL path (set by Tauri window config).

import { useState } from 'react';
import { Sidebar } from './components/layout/sidebar';
import { NoteZone } from './components/note-zone/note-zone';
import { AskZone } from './components/ask-zone/ask-zone';
import { SettingsPage } from './components/settings/settings-page';
import { QuickCaptureWindow } from './components/quick-capture/quick-capture-window';
import { ClipboardToastWindow } from './components/clipboard-toast-window';
import { ScreenshotCaptureWindow } from './components/screenshot-capture-window';
import { useNotes } from './hooks/use-notes';
import { useAsk } from './hooks/use-ask';
import { useMemory } from './hooks/use-memory';
import { useSettings } from './hooks/use-settings';

type View = 'notes' | 'ask' | 'settings';

export default function App() {
  const path = window.location.pathname;

  if (path === '/quick-capture') return <QuickCaptureWindow />;
  if (path === '/clipboard-toast') return <ClipboardToastWindow />;
  if (path === '/screenshot-capture') return <ScreenshotCaptureWindow />;

  return <MainApp />;
}

function MainApp() {
  const [activeView, setActiveView] = useState<View>('notes');
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  const {
    notes, layerFilter, setLayerFilter, loading: notesLoading,
    createNote, deleteNote, pinNote, archiveNote,
  } = useNotes();

  const {
    messages, loading: askLoading, suggestedQuestions, askQuestion,
  } = useAsk();

  const { stats, isOnline } = useMemory();
  const { settings, saveSettings } = useSettings();

  return (
    <div className="h-screen flex bg-gray-950 text-gray-100 overflow-hidden">
      {/* Collapsible sidebar */}
      <Sidebar
        activeView={activeView}
        onViewChange={setActiveView}
        stats={stats}
        llmOnline={isOnline}
        collapsed={sidebarCollapsed}
        onToggle={() => setSidebarCollapsed((c) => !c)}
      />

      {/* Main content area */}
      <main className="flex-1 flex flex-col overflow-hidden min-w-0">
        {activeView === 'notes' && (
          <NoteZone
            notes={notes}
            layerFilter={layerFilter}
            onLayerFilter={setLayerFilter}
            onCreateNote={createNote}
            onPin={pinNote}
            onArchive={archiveNote}
            onDelete={deleteNote}
            loading={notesLoading}
          />
        )}

        {activeView === 'ask' && (
          <AskZone
            messages={messages}
            suggestedQuestions={suggestedQuestions}
            loading={askLoading}
            onAsk={askQuestion}
          />
        )}

        {activeView === 'settings' && (
          <SettingsPage
            settings={settings}
            onSave={saveSettings}
            onClose={() => setActiveView('notes')}
          />
        )}
      </main>
    </div>
  );
}
