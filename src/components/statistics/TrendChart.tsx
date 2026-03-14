import {
  BarChart,
  Bar,
  LineChart,
  Line,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
} from "recharts";
import type { HourlyStats, DailyStatsPoint, StatsPeriod } from "@/types/stats";

function fillDateGaps(data: DailyStatsPoint[], days: number): DailyStatsPoint[] {
  const map = new Map(data.map((d) => [d.date, d]));
  const result: DailyStatsPoint[] = [];
  const today = new Date();
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
    result.push(
      map.get(key) ?? { date: key, count: 0, scheduled: 0, reply: 0, manual: 0 },
    );
  }
  return result;
}

const tooltipStyle = {
  background: "#231f31",
  border: "1px solid #3a3450",
  borderRadius: 8,
};

interface TrendChartProps {
  period: StatsPeriod;
  hourlyData: HourlyStats[];
  weeklyData: DailyStatsPoint[];
  monthlyData: DailyStatsPoint[];
}

export default function TrendChart({
  period,
  hourlyData,
  weeklyData,
  monthlyData,
}: TrendChartProps) {
  if (period === "day") {
    return (
      <div role="img" aria-label="今日每小时点赞趋势柱状图">
        <BarChart data={hourlyData} width={560} height={260}>
          <CartesianGrid strokeDasharray="3 3" stroke="#2a2540" />
          <XAxis
            dataKey="hour"
            stroke="#8b85a0"
            tick={{ fontSize: 11 }}
            tickFormatter={(h: number) => `${h}时`}
          />
          <YAxis stroke="#8b85a0" tick={{ fontSize: 11 }} allowDecimals={false} />
          <Tooltip
            contentStyle={tooltipStyle}
            labelFormatter={(h) => `${h}:00`}
          />
          <Bar dataKey="count" fill="#f2a7c3" radius={[4, 4, 0, 0]} />
        </BarChart>
      </div>
    );
  }

  const days = period === "week" ? 7 : 30;
  const raw = period === "week" ? weeklyData : monthlyData;
  const filledData = fillDateGaps(raw, days);

  return (
    <div
      role="img"
      aria-label={
        period === "week" ? "近7天点赞趋势折线图" : "近30天点赞趋势折线图"
      }
    >
      <LineChart data={filledData} width={560} height={260}>
        <defs>
          <linearGradient id="areaGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#c3a7f2" stopOpacity={0.3} />
            <stop offset="100%" stopColor="#c3a7f2" stopOpacity={0} />
          </linearGradient>
        </defs>
        <CartesianGrid strokeDasharray="3 3" stroke="#2a2540" />
        <XAxis
          dataKey="date"
          stroke="#8b85a0"
          tick={{ fontSize: 11 }}
          tickFormatter={(d: string) => d.slice(5)}
        />
        <YAxis stroke="#8b85a0" tick={{ fontSize: 11 }} allowDecimals={false} />
        <Tooltip contentStyle={tooltipStyle} />
        <Area
          type="monotone"
          dataKey="count"
          fill="url(#areaGradient)"
          stroke="none"
        />
        <Line
          type="monotone"
          dataKey="count"
          stroke="#f2a7c3"
          strokeWidth={2}
          dot={{ fill: "#f2a7c3", r: 3 }}
        />
      </LineChart>
    </div>
  );
}
