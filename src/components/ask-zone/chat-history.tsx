// Scrollable chat history — auto-scrolls to bottom on new messages

import { useEffect, useRef } from 'react';
import type { ChatMessage } from '../../types';
import { ChatMessageBubble } from './chat-message';
import { SuggestedQuestions } from './suggested-questions';

interface ChatHistoryProps {
  messages: ChatMessage[];
  suggestedQuestions: string[];
  loading: boolean;
  onSuggestedQuestion: (q: string) => void;
  onCitationClick?: (noteId: number) => void;
}

export function ChatHistory({
  messages,
  suggestedQuestions,
  loading,
  onSuggestedQuestion,
  onCitationClick,
}: ChatHistoryProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  if (messages.length === 0 && !loading) {
    return (
      <SuggestedQuestions
        questions={suggestedQuestions}
        onSelect={onSuggestedQuestion}
      />
    );
  }

  return (
    <div className="flex-1 overflow-y-auto px-3 py-2">
      {messages.map((msg) => (
        <ChatMessageBubble
          key={msg.id}
          message={msg}
          onCitationClick={onCitationClick}
        />
      ))}

      {loading && (
        <div className="flex justify-start mb-3">
          <div className="bg-gray-800/50 rounded-lg rounded-bl-none px-4 py-2.5">
            <span className="flex gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-gray-500 animate-bounce [animation-delay:0ms]" />
              <span className="w-1.5 h-1.5 rounded-full bg-gray-500 animate-bounce [animation-delay:150ms]" />
              <span className="w-1.5 h-1.5 rounded-full bg-gray-500 animate-bounce [animation-delay:300ms]" />
            </span>
          </div>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}
