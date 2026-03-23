import { useState, useCallback, useRef, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useNapCatStore } from "@/stores/useNapCatStore";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { downloadNapcat, importNapcat, startNapcat, openNapcatDir, clearNapcatCache, updateNapcat } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import type { DownloadProgress, ExtractProgress } from "@/types/napcat";

interface NapCatLogEntry {
  id: number;
  level: string;
  event?: string;
  message: string;
  timestamp: string;
}

let napcatLogId = 0;

export default function NapCatSetup() {
  const { status, qrCodeSrc, setQrCodeSrc } = useNapCatStore();

  const [downloadProgress, setDownloadProgress] =
    useState<DownloadProgress | null>(null);
  const [extractProgress, setExtractProgress] =
    useState<ExtractProgress | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [napcatLogs, setNapcatLogs] = useState<NapCatLogEntry[]>([]);
  const logEndRef = useRef<HTMLDivElement>(null);

  // 监听 NapCat 结构化日志事件
  useTauriEvent<{ level: string; event?: string; message: string }>(
    "napcat:log",
    useCallback((payload) => {
      setNapcatLogs((prev) => {
        const entry: NapCatLogEntry = {
          id: ++napcatLogId,
          level: payload.level,
          event: payload.event,
          message: payload.message,
          timestamp: new Date().toLocaleTimeString("zh-CN", { hour12: false }),
        };
        const next = [...prev, entry];
        // 最多保留 100 条
        return next.length > 100 ? next.slice(-100) : next;
      });
    }, []),
  );

  // 自动滚动到底部
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [napcatLogs]);

  // 监听下载进度
  useTauriEvent<DownloadProgress>(
    "napcat:download-progress",
    useCallback((p: DownloadProgress) => setDownloadProgress(p), []),
  );

  // 监听解压进度
  useTauriEvent<ExtractProgress>(
    "napcat:extract-progress",
    useCallback((p: ExtractProgress) => setExtractProgress(p), []),
  );

  const handleDownload = useCallback(async () => {
    setActionError(null);
    try {
      await downloadNapcat();
    } catch (e) {
      setActionError(String(e));
    }
  }, []);

  const handleImport = useCallback(async () => {
    setActionError(null);
    try {
      const selected = await open({
        title: "选择 NapCat 压缩包",
        filters: [{ name: "ZIP", extensions: ["zip"] }],
        multiple: false,
        directory: false,
      });
      if (!selected) return;
      await importNapcat(selected);
    } catch (e) {
      setActionError(String(e));
    }
  }, []);

  const handleStart = useCallback(async () => {
    setActionError(null);
    setQrCodeSrc(null);
    try {
      await startNapcat();
    } catch (e) {
      setActionError(String(e));
    }
  }, [setQrCodeSrc]);

  const handleClearCache = useCallback(async () => {
    setActionError(null);
    try {
      const msg = await clearNapcatCache();
      setActionError(msg); // 显示清理结果
    } catch (e) {
      setActionError(String(e));
    }
  }, []);

  const handleUpdate = useCallback(async () => {
    setActionError(null);
    try {
      await updateNapcat();
    } catch (e) {
      setActionError(String(e));
    }
  }, []);

  // running 状态不显示引导卡片
  if (status === "running") return null;

  const isError = typeof status === "object" && "error" in status;

  return (
    <div className="overflow-hidden rounded-xl bg-bg-card ring-1 ring-foreground/10">
      <div className="px-5 py-4">
        {/* notInstalled */}
        {status === "notInstalled" && (
          <div className="flex flex-col items-center gap-4 py-2">
            <span className="text-[40px]" role="img" aria-label="欢迎">
              🚀
            </span>
            <div className="text-center">
              <p className="text-[length:var(--text-heading)] font-bold text-text-primary">
                欢迎使用 QQ Auto Like Plus
              </p>
              <p className="mt-1 text-[length:var(--text-caption)] text-text-secondary">
                首先需要安装运行环境 NapCat，请选择安装方式
              </p>
            </div>
            <div className="flex gap-3">
              <Button onClick={handleDownload}>在线下载</Button>
              <Button variant="outline" onClick={handleImport}>
                本地导入
              </Button>
            </div>
            {actionError && (
              <p className="max-w-full break-all text-center text-[length:var(--text-caption)] text-destructive">
                {actionError}
              </p>
            )}
          </div>
        )}

        {/* downloading */}
        {status === "downloading" && (
          <div className="flex flex-col gap-3 py-2">
            <div className="flex items-center gap-2">
              <span className="text-[24px] animate-bounce">⬇️</span>
              <p className="text-[length:var(--text-body)] font-medium text-text-primary">
                正在下载 NapCat...
              </p>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-bg-elevated">
              <div
                className="h-full rounded-full bg-primary transition-all duration-300"
                style={{
                  width: `${downloadProgress?.percentage ?? 0}%`,
                }}
              />
            </div>
            <div className="flex justify-between text-[length:var(--text-stat-label)] text-text-muted">
              <span>{formatPercent(downloadProgress?.percentage ?? 0)}</span>
              <span>
                {formatSpeed(downloadProgress?.speedBps ?? 0)}
                {downloadProgress && downloadProgress.etaSeconds > 0 && (
                  <> · 剩余 {formatEta(downloadProgress.etaSeconds)}</>
                )}
              </span>
            </div>
          </div>
        )}

        {/* extracting */}
        {status === "extracting" && (
          <div className="flex flex-col gap-3 py-2">
            <div className="flex items-center gap-2">
              <span className="text-[24px] animate-spin">📦</span>
              <p className="text-[length:var(--text-body)] font-medium text-text-primary">
                正在解压文件...
              </p>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-bg-elevated">
              <div
                className="h-full rounded-full bg-primary transition-all duration-300"
                style={{
                  width: `${extractProgress?.percentage ?? 0}%`,
                }}
              />
            </div>
            <p className="text-[length:var(--text-stat-label)] text-text-muted">
              {extractProgress
                ? `${extractProgress.currentFile} / ${extractProgress.totalFiles} 个文件`
                : "准备中..."}
            </p>
          </div>
        )}

        {/* ready */}
        {status === "ready" && (
          <div className="flex flex-col items-center gap-4 py-2">
            <span className="text-[40px]" role="img" aria-label="完成">
              ✅
            </span>
            <div className="text-center">
              <p className="text-[length:var(--text-heading)] font-bold text-text-primary">
                安装完成
              </p>
              <p className="mt-1 text-[length:var(--text-caption)] text-text-secondary">
                NapCat 已准备就绪，点击下方按钮启动
              </p>
            </div>
            <div className="flex gap-3">
              <Button onClick={handleStart}>启动 NapCat</Button>
              <Button variant="outline" onClick={handleUpdate}>
                检查更新
              </Button>
            </div>
            {actionError && (
              <p className="max-w-full break-all text-center text-[length:var(--text-caption)] text-destructive">
                {actionError}
              </p>
            )}
          </div>
        )}

        {/* starting */}
        {status === "starting" && (
          <div className="flex flex-col items-center gap-4 py-4">
            <span className="text-[40px] animate-pulse">⏳</span>
            <p className="text-[length:var(--text-body)] font-medium text-text-primary">
              NapCat 启动中，请稍候...
            </p>
            <NapCatLogPanel logs={napcatLogs} logEndRef={logEndRef} />
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={handleStart}>
                重新启动
              </Button>
            </div>
          </div>
        )}

        {/* waitingForLogin */}
        {status === "waitingForLogin" && (
          <div className="flex flex-col items-center gap-4 py-2">
            <p className="text-[length:var(--text-heading)] font-bold text-text-primary">
              扫码登录
            </p>
            <p className="text-[length:var(--text-caption)] text-text-secondary">
              请使用 QQ 扫描下方二维码完成登录
            </p>
            <div className="flex size-48 items-center justify-center rounded-lg bg-white">
              {qrCodeSrc ? (
                <img
                  src={qrCodeSrc}
                  alt="登录二维码"
                  className="size-44 object-contain"
                />
              ) : (
                <span className="text-[length:var(--text-caption)] text-text-muted">
                  等待二维码生成...
                </span>
              )}
            </div>
            <div className="flex flex-wrap justify-center gap-2">
              <Button variant="outline" onClick={handleStart}>
                重新启动
              </Button>
              <Button variant="outline" onClick={handleClearCache}>
                清理缓存
              </Button>
              <Button variant="outline" onClick={handleUpdate}>
                更新 NapCat
              </Button>
            </div>
            {actionError && (
              <p className="max-w-full break-all text-center text-[length:var(--text-caption)] text-destructive">
                {actionError}
              </p>
            )}
            <NapCatLogPanel logs={napcatLogs} logEndRef={logEndRef} />
          </div>
        )}

        {/* error */}
        {isError && (
          <div className="flex flex-col items-center gap-4 py-2">
            <span className="text-[40px]" role="img" aria-label="错误">
              😰
            </span>
            <div className="text-center">
              <p className="text-[length:var(--text-heading)] font-bold text-text-primary">
                出了点问题
              </p>
              <p className="mt-1 max-w-full break-all text-[length:var(--text-caption)] text-destructive">
                {(status as { error: string }).error}
              </p>
            </div>
            <div className="flex flex-wrap justify-center gap-2">
              <Button onClick={handleStart}>重试启动</Button>
              <Button variant="outline" onClick={handleClearCache}>
                清理缓存
              </Button>
              <Button variant="outline" onClick={handleUpdate}>
                更新 NapCat
              </Button>
              <Button variant="outline" onClick={handleDownload}>
                重新下载
              </Button>
            </div>
            <NapCatLogPanel logs={napcatLogs} logEndRef={logEndRef} />
          </div>
        )}

        {/* 打开 NapCat 目录（notInstalled 以外的状态显示） */}
        {status !== "notInstalled" && (
          <div className="flex justify-center border-t border-foreground/5 pt-3 mt-2">
            <button
              type="button"
              onClick={() => openNapcatDir()}
              className="text-[length:var(--text-caption)] text-text-muted hover:text-text-secondary transition-colors"
            >
              打开 NapCat 目录
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

function formatSpeed(bps: number): string {
  if (bps < 1024) return `${bps} B/s`;
  if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${(bps / 1024 / 1024).toFixed(1)} MB/s`;
}

function formatEta(seconds: number): string {
  if (seconds < 60) return `${seconds}秒`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}分${seconds % 60}秒`;
  return `${Math.floor(seconds / 3600)}时${Math.floor((seconds % 3600) / 60)}分`;
}

function NapCatLogPanel({
  logs,
  logEndRef,
}: {
  logs: NapCatLogEntry[];
  logEndRef: React.RefObject<HTMLDivElement | null>;
}) {
  if (logs.length === 0) return null;

  return (
    <div className="mt-2 w-full rounded-lg bg-bg-elevated/60 ring-1 ring-foreground/5">
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-foreground/5">
        <span className="text-[length:var(--text-stat-label)] font-medium text-text-secondary">
          实时日志
        </span>
        <span className="text-[length:var(--text-stat-label)] text-text-muted">
          {logs.length} 条
        </span>
      </div>
      <div className="max-h-40 overflow-y-auto px-3 py-2 font-mono text-[11px] leading-relaxed">
        {logs.map((entry) => (
          <div key={entry.id} className="flex gap-2">
            <span className="shrink-0 text-text-muted">{entry.timestamp}</span>
            <span
              className={
                entry.level === "error"
                  ? "text-destructive"
                  : entry.level === "warn"
                    ? "text-amber-500"
                    : entry.level === "success"
                      ? "text-emerald-500"
                      : "text-text-secondary"
              }
            >
              {entry.message}
            </span>
          </div>
        ))}
        <div ref={logEndRef} />
      </div>
    </div>
  );
}
