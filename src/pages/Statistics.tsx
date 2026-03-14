import { useEffect } from "react";
import { useStatsStore } from "@/stores/useStatsStore";
import TrendChart from "@/components/statistics/TrendChart";
import TypePieChart from "@/components/statistics/TypePieChart";
import FriendRanking from "@/components/statistics/FriendRanking";
import type { StatsPeriod } from "@/types/stats";

const PERIOD_TABS: { key: StatsPeriod; label: string }[] = [
  { key: "day", label: "日" },
  { key: "week", label: "周" },
  { key: "month", label: "月" },
];

export default function Statistics() {
  const {
    hourlyData,
    weeklyData,
    monthlyData,
    typeRatio,
    friendRanking,
    currentPeriod,
    isLoading,
    setPeriod,
  } = useStatsStore();

  useEffect(() => {
    setPeriod("week");
  }, [setPeriod]);

  const hasNoData =
    !isLoading &&
    hourlyData.length === 0 &&
    weeklyData.length === 0 &&
    monthlyData.length === 0 &&
    !typeRatio &&
    friendRanking.length === 0;

  if (hasNoData) {
    return (
      <div className="page-enter flex h-full flex-col items-center justify-center gap-3 py-8">
        <span className="text-[48px]" role="img" aria-label="空状态">
          📊
        </span>
        <p className="text-[length:var(--text-body)] text-text-secondary">
          还没有点赞数据，等明天再来看看吧~
        </p>
      </div>
    );
  }

  const currentData =
    currentPeriod === "day"
      ? hourlyData
      : currentPeriod === "week"
        ? weeklyData
        : monthlyData;

  if (isLoading && currentData.length === 0) {
    return (
      <div className="page-enter flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div className="h-8 w-24 animate-pulse rounded-[10px] bg-bg-elevated" />
          <div className="flex gap-1">
            {[1, 2, 3].map((n) => (
              <div
                key={n}
                className="h-8 w-10 animate-pulse rounded-[10px] bg-bg-elevated"
              />
            ))}
          </div>
        </div>
        <div className="h-[260px] animate-pulse rounded-[14px] bg-bg-card" />
        <div className="grid grid-cols-2 gap-3">
          <div className="h-[280px] animate-pulse rounded-[14px] bg-bg-card" />
          <div className="h-[280px] animate-pulse rounded-[14px] bg-bg-card" />
        </div>
      </div>
    );
  }

  return (
    <div className="page-enter flex flex-col gap-4">
      {/* 标题 + 时间范围 Tab */}
      <div className="flex items-center justify-between">
        <h1 className="text-[length:var(--text-display)] font-bold text-text-primary">
          数据统计
        </h1>
        <div className="flex gap-1 rounded-[10px] bg-bg-card p-1" role="tablist" aria-label="时间范围">
          {PERIOD_TABS.map((tab) => (
            <button
              key={tab.key}
              role="tab"
              aria-selected={currentPeriod === tab.key}
              onClick={() => setPeriod(tab.key)}
              className={`rounded-[8px] px-3 py-1.5 text-[length:var(--text-caption)] font-medium transition-colors ${
                currentPeriod === tab.key
                  ? "bg-primary text-white"
                  : "text-text-secondary hover:text-text-primary"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* 主趋势图表 */}
      <div className="overflow-hidden rounded-[14px] bg-bg-card p-4">
        <TrendChart
          period={currentPeriod}
          hourlyData={hourlyData}
          weeklyData={weeklyData}
          monthlyData={monthlyData}
        />
      </div>

      {/* 下半区：饼图 + 排行榜 */}
      <div className="grid grid-cols-2 gap-3">
        <div className="rounded-[14px] bg-bg-card p-4">
          <h2 className="mb-3 text-[length:var(--text-body)] font-semibold text-text-primary">
            点赞类型占比
          </h2>
          <div className="flex justify-center">
            <TypePieChart typeRatio={typeRatio} />
          </div>
        </div>
        <div className="rounded-[14px] bg-bg-card p-4">
          <h2 className="mb-3 text-[length:var(--text-body)] font-semibold text-text-primary">
            好友互动排行
          </h2>
          <FriendRanking ranking={friendRanking} />
        </div>
      </div>
    </div>
  );
}
