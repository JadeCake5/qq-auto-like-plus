import { PieChart, Pie, Cell, Legend, Tooltip } from "recharts";
import type { LikeTypeRatio } from "@/types/stats";

const TYPE_CONFIG: Record<string, { label: string; color: string }> = {
  scheduled: { label: "定时点赞", color: "#f2a7c3" },
  reply: { label: "回赞", color: "#a7c7f2" },
  manual: { label: "手动点赞", color: "#c3a7f2" },
};

const tooltipStyle = {
  background: "#231f31",
  border: "1px solid #3a3450",
  borderRadius: 8,
};

interface TypePieChartProps {
  typeRatio: LikeTypeRatio | null;
}

export default function TypePieChart({ typeRatio }: TypePieChartProps) {
  const pieData =
    typeRatio && typeRatio.total > 0
      ? (
          [
            { key: "scheduled", name: TYPE_CONFIG.scheduled.label, value: typeRatio.scheduled, color: TYPE_CONFIG.scheduled.color },
            { key: "reply", name: TYPE_CONFIG.reply.label, value: typeRatio.reply, color: TYPE_CONFIG.reply.color },
            { key: "manual", name: TYPE_CONFIG.manual.label, value: typeRatio.manual, color: TYPE_CONFIG.manual.color },
          ] as const
        ).filter((d) => d.value > 0)
      : [];

  if (pieData.length === 0) {
    return (
      <div className="flex h-[240px] w-[240px] items-center justify-center">
        <div className="flex size-[160px] items-center justify-center rounded-full border-4 border-dashed border-text-muted/20">
          <span className="text-[length:var(--text-caption)] text-text-muted">
            暂无数据
          </span>
        </div>
      </div>
    );
  }

  return (
    <div role="img" aria-label="点赞类型占比饼图">
      <PieChart width={240} height={240}>
        <Pie
          data={pieData}
          dataKey="value"
          nameKey="name"
          cx="50%"
          cy="50%"
          outerRadius={80}
          innerRadius={40}
          stroke="none"
          label={({ percent }: { percent?: number }) =>
            `${((percent ?? 0) * 100).toFixed(0)}%`
          }
        >
          {pieData.map((entry) => (
            <Cell key={entry.key} fill={entry.color} />
          ))}
        </Pie>
        <Legend />
        <Tooltip contentStyle={tooltipStyle} />
      </PieChart>
    </div>
  );
}
