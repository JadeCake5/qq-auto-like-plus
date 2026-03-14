import { useNapCatStore } from "@/stores/useNapCatStore";
import { useLikeStore } from "@/stores/useLikeStore";

function getMascotEmoji(
  napCatStatus: ReturnType<typeof useNapCatStore.getState>["status"],
  isPaused: boolean,
): string {
  if (typeof napCatStatus === "object" && "error" in napCatStatus) return "😰";
  if (isPaused) return "😴";
  if (napCatStatus === "running") return "😊";
  if (napCatStatus === "waitingForLogin" || napCatStatus === "starting")
    return "👋";
  return "😴";
}

function getStatusText(
  napCatStatus: ReturnType<typeof useNapCatStore.getState>["status"],
  isPaused: boolean,
  isRunning: boolean,
  totalLiked: number,
): string {
  if (typeof napCatStatus === "object" && "error" in napCatStatus)
    return "运行环境出了点问题...";
  if (napCatStatus !== "running") return "等待运行环境连接~";
  if (isRunning) return "正在努力点赞中...";
  if (isPaused) return "点赞引擎已暂停";
  if (totalLiked > 0) return `今天已经为 ${totalLiked} 位好友点赞啦~`;
  return "准备就绪，随时开始点赞！";
}

export default function HeroBanner() {
  const { status: napCatStatus, loginInfo } = useNapCatStore();
  const { dailyStats, isPaused, isRunning } = useLikeStore();

  const mascot = getMascotEmoji(napCatStatus, isPaused);
  const statusText = getStatusText(
    napCatStatus,
    isPaused,
    isRunning,
    dailyStats?.totalLiked ?? 0,
  );

  return (
    <div
      className="relative overflow-hidden rounded-lg px-5 py-4"
      style={{
        background:
          "linear-gradient(135deg, rgba(242,167,195,0.15), rgba(195,167,242,0.15))",
      }}
    >
      <div className="flex items-center gap-3">
        <span className="text-[40px] leading-none" role="img" aria-label="状态表情">
          {mascot}
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-[length:var(--text-heading)] font-bold text-text-primary">
            {loginInfo
              ? `${loginInfo.nickname}，你好！`
              : "欢迎回来！"}
          </p>
          <p className="text-[length:var(--text-caption)] text-text-secondary">
            {statusText}
          </p>
          {loginInfo && (
            <p className="mt-0.5 text-[length:var(--text-stat-label)] text-text-muted">
              QQ {loginInfo.qqNumber}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
