import { useEffect, useState, useCallback } from "react";
import { useLikeStore } from "@/stores/useLikeStore";
import { useNapCatStore } from "@/stores/useNapCatStore";
import { startBatchLike, pauseEngine, resumeEngine } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import HeroBanner from "@/components/dashboard/HeroBanner";
import NapCatSetup from "@/components/dashboard/NapCatSetup";
import StatCard from "@/components/dashboard/StatCard";

function formatCountdown(nextRunTime: string | null): string {
  if (!nextRunTime) return "--:--:--";
  const diff = new Date(nextRunTime).getTime() - Date.now();
  if (diff <= 0) return "即将执行";
  const h = Math.floor(diff / 3600000);
  const m = Math.floor((diff % 3600000) / 60000);
  const s = Math.floor((diff % 60000) / 1000);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function napCatStatusLabel(
  status: ReturnType<typeof useNapCatStore.getState>["status"],
): { connected: boolean; label: string } {
  if (status === "running") return { connected: true, label: "已连接" };
  if (status === "starting") return { connected: false, label: "重连中" };
  return { connected: false, label: "未连接" };
}

function LikeButton({
  disabled,
  tooltip,
  onClick,
  label,
}: {
  disabled: boolean;
  tooltip: string;
  onClick: () => void;
  label: string;
}) {
  if (tooltip) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger>
            <Button disabled={disabled} onClick={onClick}>
              {label}
            </Button>
          </TooltipTrigger>
          <TooltipContent>{tooltip}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }
  return (
    <Button disabled={disabled} onClick={onClick}>
      {label}
    </Button>
  );
}

export default function Dashboard() {
  const {
    dailyStats,
    isRunning,
    isPaused,
    batchProgress,
    nextRunTime,
    fetchDailyStats,
    fetchEngineStatus,
  } = useLikeStore();
  const { status: napCatStatus } = useNapCatStore();

  // Task 4: tick 驱动倒计时，每秒 re-render 来重新计算
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 1000);
    return () => clearInterval(id);
  }, []);
  const countdown = formatCountdown(nextRunTime);

  // Task 5: 30 秒自动刷新
  useEffect(() => {
    fetchDailyStats();
    fetchEngineStatus();
    const id = setInterval(() => {
      fetchDailyStats();
      fetchEngineStatus();
    }, 30000);
    return () => clearInterval(id);
  }, [fetchDailyStats, fetchEngineStatus]);

  const { connected, label: napCatLabel } = napCatStatusLabel(napCatStatus);

  const handleStartBatch = useCallback(async () => {
    try {
      useLikeStore.setState({ isRunning: true });
      await startBatchLike();
    } catch (e) {
      useLikeStore.setState({ isRunning: false });
      console.error("start_batch_like failed", e);
    }
  }, []);

  const handleTogglePause = useCallback(
    async (checked: boolean) => {
      try {
        if (checked) {
          await resumeEngine();
        } else {
          await pauseEngine();
        }
      } catch (e) {
        console.error("toggle pause failed", e);
      }
    },
    [],
  );

  // 立即点赞按钮禁用条件
  const batchDisabled = isPaused || isRunning || !connected;
  const batchTooltip = isPaused
    ? "引擎已暂停"
    : !connected
      ? "运行环境未连接"
      : isRunning
        ? "正在执行中..."
        : "";

  const showSetup = napCatStatus !== "running";

  // NapCat 未就绪：仅显示 HeroBanner + 引导卡片
  if (showSetup) {
    return (
      <div className="page-enter flex flex-col gap-4">
        <HeroBanner />
        <NapCatSetup />
      </div>
    );
  }

  // 空状态
  if (!dailyStats && !isRunning) {
    return (
      <div className="page-enter flex h-full flex-col">
        <HeroBanner />
        <div className="flex flex-1 flex-col items-center justify-center gap-3 py-8">
          <span className="text-[48px]" role="img" aria-label="空状态">
            🐱
          </span>
          <p className="text-[length:var(--text-body)] text-text-secondary">
            还没有开始点赞哦~
          </p>
          <LikeButton
            disabled={batchDisabled}
            tooltip={batchTooltip}
            onClick={handleStartBatch}
            label="立即点赞"
          />
        </div>
      </div>
    );
  }

  return (
    <div className="page-enter flex flex-col gap-4">
      {/* Hero Banner */}
      <HeroBanner />

      {/* 统计卡片 2×2 Grid */}
      <div className="grid grid-cols-2 gap-3">
        <StatCard
          title="今日已赞"
          value={dailyStats?.totalLiked ?? 0}
          gradientFrom="#f2a7c3"
          gradientTo="#c3a7f2"
          icon="💖"
        />
        <StatCard
          title="回赞人数"
          value={dailyStats?.replyCount ?? 0}
          gradientFrom="#f2cfa7"
          gradientTo="#f2a7c3"
          icon="🔄"
        />
        <StatCard
          title="剩余名额"
          value={dailyStats?.availableScheduled ?? 0}
          gradientFrom="#a7c7f2"
          gradientTo="#a7f2d4"
          icon="📊"
        />
        <StatCard
          title="下次点赞"
          value={countdown}
          gradientFrom="#c3a7f2"
          gradientTo="#f2a7c3"
          icon="⏰"
        />
      </div>

      {/* 操作区 */}
      <div className="flex items-center gap-4 rounded-lg bg-bg-card px-4 py-3">
        <LikeButton
          disabled={batchDisabled}
          tooltip={batchTooltip}
          onClick={handleStartBatch}
          label={isRunning ? "执行中..." : "立即点赞"}
        />

        <div className="flex items-center gap-2">
          <Switch
            checked={!isPaused}
            onCheckedChange={handleTogglePause}
            aria-label={isPaused ? "恢复引擎" : "暂停引擎"}
          />
          <span className="text-[length:var(--text-caption)] text-text-secondary">
            {isPaused ? "已暂停" : "运行中"}
          </span>
        </div>

        {/* NapCat 状态 */}
        <div className="ml-auto flex items-center gap-1.5">
          <span
            className={`inline-block size-2 rounded-full ${connected ? "bg-mint" : "bg-coral"}`}
          />
          <span className="text-[length:var(--text-stat-label)] text-text-muted">
            {napCatLabel}
          </span>
        </div>
      </div>

      {/* 进度展示 */}
      {batchProgress && (
        <div className="rounded-lg bg-bg-card px-4 py-3">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-[length:var(--text-caption)] text-text-secondary">
              正在为 {batchProgress.nickname} 点赞...
            </span>
            <span className="text-[length:var(--text-caption)] text-text-primary">
              {batchProgress.current}/{batchProgress.total}
            </span>
          </div>
          <div className="h-1.5 overflow-hidden rounded-full bg-bg-elevated">
            <div
              className="h-full rounded-full bg-primary transition-all duration-300"
              style={{
                width: `${(batchProgress.current / Math.max(batchProgress.total, 1)) * 100}%`,
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
