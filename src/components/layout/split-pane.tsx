// Vertical split pane with draggable divider — top and bottom panels

import { useState, useRef, useCallback, useEffect } from 'react';

interface SplitPaneProps {
  top: React.ReactNode;
  bottom: React.ReactNode;
}

const MIN_HEIGHT_PX = 150;

export function SplitPane({ top, bottom }: SplitPaneProps) {
  const [topRatio, setTopRatio] = useState(0.5);
  const containerRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragging.current = true;
  }, []);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!dragging.current || !containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const totalHeight = rect.height - 6; // subtract divider height
      const offsetY = e.clientY - rect.top;
      const clampedTop = Math.max(MIN_HEIGHT_PX, Math.min(offsetY, totalHeight - MIN_HEIGHT_PX));
      setTopRatio(clampedTop / totalHeight);
    };

    const onMouseUp = () => { dragging.current = false; };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }, []);

  const topPct = `${(topRatio * 100).toFixed(2)}%`;
  const bottomPct = `${((1 - topRatio) * 100).toFixed(2)}%`;

  return (
    <div ref={containerRef} className="flex flex-col flex-1 overflow-hidden">
      <div style={{ height: topPct }} className="overflow-hidden flex flex-col min-h-0">
        {top}
      </div>

      {/* Draggable divider */}
      <div
        onMouseDown={onMouseDown}
        className="h-1.5 bg-gray-800 hover:bg-purple-600 cursor-row-resize flex-shrink-0 transition-colors duration-150"
      />

      <div style={{ height: bottomPct }} className="overflow-hidden flex flex-col min-h-0">
        {bottom}
      </div>
    </div>
  );
}
