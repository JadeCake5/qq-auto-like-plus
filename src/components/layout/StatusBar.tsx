import { cn } from "@/lib/utils";
import { useNapCatStore } from "@/stores/useNapCatStore";
import type { NapCatStatus } from "@/types/napcat";

function getStatusColor(status: NapCatStatus): string {
  if (typeof status === "object") return "bg-coral";
  switch (status) {
    case "running":
    case "ready":
      return "bg-mint";
    case "starting":
    case "waitingForLogin":
    case "downloading":
    case "extracting":
      return "bg-peach";
    default:
      return "bg-coral";
  }
}

function getStatusText(status: NapCatStatus): string {
  if (typeof status === "object") return "错误";
  switch (status) {
    case "running":
      return "已连接";
    case "ready":
      return "就绪";
    case "starting":
      return "启动中";
    case "waitingForLogin":
      return "等待登录";
    case "downloading":
      return "下载中";
    case "extracting":
      return "解压中";
    case "notInstalled":
      return "未安装";
    default:
      return "未连接";
  }
}

function isTransientStatus(status: NapCatStatus): boolean {
  if (typeof status === "object") return false;
  return ["starting", "waitingForLogin", "downloading", "extracting"].includes(
    status,
  );
}

export function StatusBar() {
  const status = useNapCatStore((s) => s.status);
  const loginInfo = useNapCatStore((s) => s.loginInfo);

  return (
    <div className="flex h-9 w-full items-center justify-between bg-bg-card px-4 border-b border-border">
      <div className="flex items-center gap-2">
        <span
          className={cn(
            "inline-block h-2 w-2 rounded-full",
            getStatusColor(status),
            isTransientStatus(status) && "animate-pulse",
          )}
        />
        <span className="text-[length:var(--text-caption)] text-text-secondary">
          NapCat: {getStatusText(status)}
        </span>
      </div>
      <div className="text-[length:var(--text-caption)] text-text-muted">
        {loginInfo ? (
          <span>
            {loginInfo.nickname}{" "}
            <span className="text-text-muted">({loginInfo.qqNumber})</span>
          </span>
        ) : (
          <span>未登录</span>
        )}
      </div>
    </div>
  );
}
