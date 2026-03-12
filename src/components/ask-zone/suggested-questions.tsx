// Suggested question buttons shown when chat history is empty

interface SuggestedQuestionsProps {
  questions: string[];
  onSelect: (question: string) => void;
}

export function SuggestedQuestions({ questions, onSelect }: SuggestedQuestionsProps) {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 px-6 py-8">
      <p className="text-gray-500 text-sm">Ask anything about your notes…</p>
      <div className="flex flex-wrap gap-2 justify-center">
        {questions.map((q) => (
          <button
            key={q}
            onClick={() => onSelect(q)}
            className="px-3 py-1.5 rounded-lg border border-gray-700 bg-gray-900 hover:border-purple-500 hover:text-purple-300 text-gray-400 text-xs transition-colors"
          >
            {q}
          </button>
        ))}
      </div>
    </div>
  );
}
