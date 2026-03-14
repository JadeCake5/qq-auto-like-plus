import { useLogStore } from "@/stores/useLogStore";
import LogToolbar from "@/components/logs/LogToolbar";
import VirtualLogList from "@/components/logs/VirtualLogList";

export default function Logs() {
  const entries = useLogStore((s) => s.entries);
  const filteredEntries = useLogStore((s) => s.getFilteredEntries)();

  return (
    <div className="page-enter flex h-full flex-col gap-3">
      {/* 页面标题 */}
      <h1 className="text-[length:var(--text-display)] font-bold text-text-primary">
        运行日志
      </h1>

      {/* 日志卡片容器 */}
      <div className="flex flex-1 min-h-0 flex-col rounded-[14px] bg-bg-card">
        {/* 工具栏 */}
        <LogToolbar
          filteredCount={filteredEntries.length}
          totalCount={entries.length}
        />

        {/* 日志列表 / 空状态 */}
        {entries.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 py-8">
            <span className="text-[48px]" role="img" aria-label="空状态">
              📋
            </span>
            <p className="text-[length:var(--text-body)] text-text-secondary">
              还没有日志记录~运行后会出现在这里
            </p>
          </div>
        ) : (
          <VirtualLogList entries={filteredEntries} />
        )}
      </div>
    </div>
  );
}
