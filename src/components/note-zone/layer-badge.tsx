// Small colored badge for memory layer — Working/Episodic/Semantic

interface LayerBadgeProps {
  layer: string;
}

const LAYER_STYLES: Record<string, string> = {
  working: 'bg-amber-500/20 text-amber-400 border border-amber-500/40',
  episodic: 'bg-blue-500/20 text-blue-400 border border-blue-500/40',
  semantic: 'bg-purple-500/20 text-purple-400 border border-purple-500/40',
};

const LAYER_LABELS: Record<string, string> = {
  working: 'Working',
  episodic: 'Episodic',
  semantic: 'Semantic',
};

export function LayerBadge({ layer }: LayerBadgeProps) {
  const style = LAYER_STYLES[layer] ?? 'bg-gray-700 text-gray-400';
  const label = LAYER_LABELS[layer] ?? layer;

  return (
    <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium uppercase tracking-wide ${style}`}>
      {label}
    </span>
  );
}
