// Hook for Ask Zone — chat messages state, askQuestion, suggested questions

import { useState, useCallback } from 'react';
import type { ChatMessage, AskResponse } from '../types';
import { askAi } from '../services/tauri-commands';

const SUGGESTED_QUESTIONS = [
  'What are the main themes in my notes?',
  'What should I focus on next?',
  'Summarize everything I have saved.',
  'Find connections between my notes.',
];

function makeId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

export function useAsk() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const askQuestion = useCallback(async (question: string) => {
    if (!question.trim()) return;

    const userMsg: ChatMessage = {
      id: makeId(),
      role: 'user',
      content: question.trim(),
      timestamp: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, userMsg]);
    setLoading(true);
    setError(null);

    try {
      const response: AskResponse = await askAi(question.trim());

      const assistantMsg: ChatMessage = {
        id: makeId(),
        role: 'assistant',
        content: response.answer,
        citations: response.citations,
        timestamp: new Date().toISOString(),
      };

      setMessages((prev) => [...prev, assistantMsg]);
    } catch (e) {
      const errMsg: ChatMessage = {
        id: makeId(),
        role: 'assistant',
        content: `Error: ${String(e)}`,
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errMsg]);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const clearMessages = useCallback(() => {
    setMessages([]);
    setError(null);
  }, []);

  return {
    messages,
    loading,
    error,
    suggestedQuestions: SUGGESTED_QUESTIONS,
    askQuestion,
    clearMessages,
  };
}
