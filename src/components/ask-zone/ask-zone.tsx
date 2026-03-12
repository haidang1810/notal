// Ask Zone — full-page view combining ChatHistory + ChatInput with header

import type { ChatMessage } from '../../types';
import { ChatHistory } from './chat-history';
import { ChatInput } from './chat-input';

interface AskZoneProps {
  messages: ChatMessage[];
  suggestedQuestions: string[];
  loading: boolean;
  onAsk: (question: string) => void;
  onCitationClick?: (noteId: number) => void;
}

export function AskZone({
  messages,
  suggestedQuestions,
  loading,
  onAsk,
  onCitationClick,
}: AskZoneProps) {
  return (
    <section className="flex flex-col h-full overflow-hidden">
      {/* View header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-gray-800 flex-shrink-0">
        <h1 className="text-sm font-semibold text-gray-200">Ask AI</h1>
        {messages.length > 0 && (
          <span className="text-xs text-gray-500">{messages.length} messages</span>
        )}
      </div>

      <ChatHistory
        messages={messages}
        suggestedQuestions={suggestedQuestions}
        loading={loading}
        onSuggestedQuestion={onAsk}
        onCitationClick={onCitationClick}
      />

      <ChatInput onSend={onAsk} disabled={loading} />
    </section>
  );
}
