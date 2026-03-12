// Chat input — text field + Send button, Enter submits, Shift+Enter newline

import { useState, useRef, useCallback } from 'react';

interface ChatInputProps {
  onSend: (question: string) => void;
  disabled?: boolean;
}

export function ChatInput({ onSend, disabled }: ChatInputProps) {
  const [value, setValue] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = useCallback(() => {
    if (!value.trim() || disabled) return;
    onSend(value.trim());
    setValue('');
    textareaRef.current?.focus();
  }, [value, onSend, disabled]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex gap-2 px-3 py-2 border-t border-gray-800 flex-shrink-0">
      <textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        placeholder="Ask about your notes… (Enter to send, Shift+Enter for newline)"
        rows={2}
        className="flex-1 bg-gray-900 border border-gray-700 focus:border-purple-500 text-gray-100 text-sm placeholder-gray-600 px-3 py-2 rounded-lg resize-none outline-none transition-colors"
      />
      <button
        onClick={handleSend}
        disabled={!value.trim() || disabled}
        className="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:opacity-40 disabled:cursor-not-allowed text-white text-sm rounded-lg transition-colors self-end"
      >
        Ask
      </button>
    </div>
  );
}
