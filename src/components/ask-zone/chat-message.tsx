// Chat message bubble — user right-aligned, AI left-aligned with [#ID] citation links

import type { ChatMessage } from '../../types';

interface ChatMessageProps {
  message: ChatMessage;
  onCitationClick?: (noteId: number) => void;
}

/** Parse text with [#ID] patterns into React nodes with clickable citation spans. */
function renderWithCitations(
  text: string,
  onCitationClick?: (noteId: number) => void,
): React.ReactNode[] {
  const parts: React.ReactNode[] = [];
  const pattern = /\[#(\d+)\]/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    const id = Number(match[1]);
    parts.push(
      <button
        key={match.index}
        onClick={() => onCitationClick?.(id)}
        className="text-purple-400 hover:text-purple-300 hover:underline font-mono text-xs"
      >
        {match[0]}
      </button>,
    );
    lastIndex = pattern.lastIndex;
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts;
}

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleTimeString(undefined, {
      hour: '2-digit', minute: '2-digit',
    });
  } catch {
    return '';
  }
}

export function ChatMessageBubble({ message, onCitationClick }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'} mb-3`}>
      <div
        className={`max-w-[80%] rounded-lg px-3 py-2 text-sm leading-relaxed ${
          isUser
            ? 'bg-blue-900/30 text-blue-100 rounded-br-none'
            : 'bg-gray-800/50 text-gray-200 rounded-bl-none'
        }`}
      >
        <p className="whitespace-pre-wrap break-words">
          {isUser
            ? message.content
            : renderWithCitations(message.content, onCitationClick)}
        </p>
        <span className="block text-[10px] text-gray-500 mt-1 text-right">
          {formatTime(message.timestamp)}
        </span>
      </div>
    </div>
  );
}
