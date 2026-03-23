import { useRef, useState, useCallback, useEffect } from "react";
import { ArrowDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { LogEntry } from "@/stores/useLogStore";

const ROW_HEIGHT = 28;
const BUFFER = 10;

const LEVEL_CLASS: Record<LogEntry["level"], string> = {
  info: "text-secondary",
  warn: "text-peach",
  error: "text-coral",
};

const LEVEL_LABEL: Record<LogEntry["level"], string> = {
  info: "INFO ",
  warn: "WARN ",
  error: "ERROR",
};

interface VirtualLogListProps {
  entries: LogEntry[];
}

export default function VirtualLogList({ entries }: VirtualLogListProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const isUserScrolledUp = useRef(false);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);
  const [showScrollToBottom, setShowScrollToBottom] = useState(false);

  // 计算可见范围
  const totalHeight = entries.length * ROW_HEIGHT;
  const visibleCount = Math.ceil(containerHeight / ROW_HEIGHT);
  const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER);
  const endIndex = Math.min(entries.length, startIndex + visibleCount + BUFFER * 2);
  const visibleEntries = entries.slice(startIndex, endIndex);
  const offsetY = startIndex * ROW_HEIGHT;

  // 监听容器尺寸
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver((obs) => {
      for (const entry of obs) {
        setContainerHeight(entry.contentRect.height);
      }
    });
    observer.observe(el);
    setContainerHeight(el.clientHeight);
    return () => observer.disconnect();
  }, []);

  // 自动滚动到底部
  useEffect(() => {
    const el = containerRef.current;
    if (!el || isUserScrolledUp.current) return;
    el.scrollTop = el.scrollHeight;
  }, [entries.length]);

  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    setScrollTop(el.scrollTop);

    const atBottom =
      el.scrollTop + el.clientHeight >= el.scrollHeight - ROW_HEIGHT * 2;
    isUserScrolledUp.current = !atBottom;
    setShowScrollToBottom(!atBottom);
  }, []);

  const scrollToBottom = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
    isUserScrolledUp.current = false;
    setShowScrollToBottom(false);
  }, []);

  return (
    <div className="relative flex-1 min-h-0">
      <div
        ref={containerRef}
        className="h-full overflow-y-auto scrollbar-thin"
        onScroll={handleScroll}
        role="log"
        aria-live="polite"
        aria-label="运行日志"
      >
        <div style={{ height: totalHeight, position: "relative" }}>
          <div
            style={{
              position: "absolute",
              top: offsetY,
              left: 0,
              right: 0,
            }}
          >
            {visibleEntries.map((entry) => (
              <div
                key={entry.id}
                className="flex items-center px-4 font-[family-name:var(--font-mono)] text-xs leading-[28px]"
                style={{ height: ROW_HEIGHT }}
              >
                <span className="text-text-muted">[{entry.timestamp}]</span>
                <span className="mx-1">[</span>
                <span
                  className={`inline-block w-[4.5ch] ${LEVEL_CLASS[entry.level]}`}
                >
                  {LEVEL_LABEL[entry.level]}
                </span>
                <span className="mx-1">]</span>
                <span className="text-text-primary truncate">
                  {entry.message}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* 回到底部按钮 */}
      {showScrollToBottom && (
        <Button
          variant="outline"
          size="icon-sm"
          className="absolute bottom-3 right-3 size-8 rounded-full bg-bg-elevated border-border"
          onClick={scrollToBottom}
          aria-label="回到底部"
        >
          <ArrowDown className="size-4" />
        </Button>
      )}
    </div>
  );
}
