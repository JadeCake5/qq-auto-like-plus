import { Search, Trash2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { useLogStore, type LogFilter } from "@/stores/useLogStore";

interface LogToolbarProps {
  filteredCount: number;
  totalCount: number;
}

export default function LogToolbar({
  filteredCount,
  totalCount,
}: LogToolbarProps) {
  const filter = useLogStore((s) => s.filter);
  const setFilter = useLogStore((s) => s.setFilter);
  const setSearchKeyword = useLogStore((s) => s.setSearchKeyword);
  const clear = useLogStore((s) => s.clear);

  return (
    <div className="flex items-center gap-2 border-b border-border px-4 py-2">
      {/* 搜索框 */}
      <div className="relative flex-1">
        <Search className="absolute left-2 top-1/2 size-3.5 -translate-y-1/2 text-text-muted" />
        <Input
          placeholder="搜索日志..."
          className="h-7 pl-7 text-xs"
          onChange={(e) => setSearchKeyword(e.target.value)}
          aria-label="搜索日志"
        />
      </div>

      {/* 级别筛选 */}
      <Select
        value={filter}
        onValueChange={(val) => setFilter(val as LogFilter)}
      >
        <SelectTrigger size="sm" aria-label="日志级别筛选">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">全部</SelectItem>
          <SelectItem value="info">INFO</SelectItem>
          <SelectItem value="warn">WARN</SelectItem>
          <SelectItem value="error">ERROR</SelectItem>
        </SelectContent>
      </Select>

      {/* 清空按钮 */}
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={clear}
        aria-label="清空日志"
        className="text-text-muted hover:text-coral"
      >
        <Trash2 />
      </Button>

      {/* 日志计数 */}
      <span className="whitespace-nowrap text-[length:var(--text-caption)] text-text-muted">
        {filteredCount}/{totalCount}
      </span>
    </div>
  );
}
