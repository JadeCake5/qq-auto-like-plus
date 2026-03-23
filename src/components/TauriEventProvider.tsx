import { useEffect } from "react";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { useNapCatStore } from "@/stores/useNapCatStore";
import { useLikeStore } from "@/stores/useLikeStore";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useLogStore } from "@/stores/useLogStore";
import type { NapCatStatus } from "@/types/napcat";
import type { EngineStatus } from "@/types/engine";
import type { BatchLikeProgress, BatchLikeResult, ReplyLikeResult } from "@/types/like";

let logId = 0;

export function TauriEventProvider() {
  const setNapCatStatus = useNapCatStore((s) => s.setStatus);
  const setQrCodeSrc = useNapCatStore((s) => s.setQrCodeSrc);
  const setEngineStatus = useLikeStore((s) => s.setEngineStatus);
  const setBatchProgress = useLikeStore((s) => s.setBatchProgress);
  const onBatchComplete = useLikeStore((s) => s.onBatchComplete);
  const fetchDailyStats = useLikeStore((s) => s.fetchDailyStats);
  const fetchConfig = useSettingsStore((s) => s.fetchConfig);

  useTauriEvent<NapCatStatus>("napcat:status-changed", setNapCatStatus);
  useTauriEvent<string>("napcat:qr-code", (payload) => {
    if (payload.startsWith("data:image/")) {
      setQrCodeSrc(payload);
    }
  });
  useTauriEvent<EngineStatus>("engine:status-changed", setEngineStatus);
  useTauriEvent<BatchLikeProgress>("like:progress", setBatchProgress);
  useTauriEvent<BatchLikeResult>("like:batch-complete", onBatchComplete);
  useTauriEvent<ReplyLikeResult>("like:reply-complete", (result) => {
    if (result.success) {
      fetchDailyStats();
    }
  });
  useTauriEvent<void>("config:updated", fetchConfig);

  useEffect(() => {
    useNapCatStore.getState().fetchStatus();
    useNapCatStore.getState().fetchLoginInfo();
    useLikeStore.getState().fetchEngineStatus();
    useLikeStore.getState().fetchDailyStats();
    useSettingsStore.getState().fetchConfig();

    // 加载历史日志（attachLogger 之前产生的）
    import("@/lib/tauri").then(({ getStartupLogs }) => {
      getStartupLogs()
        .then((entries) => {
          for (const entry of entries) {
            useLogStore.getState().addEntry({
              id: String(++logId),
              timestamp: entry.timestamp,
              level: entry.level,
              message: entry.message,
            });
          }
        })
        .catch((e) => console.error("加载历史日志失败:", e));
    });
  }, []);

  // attachLogger: 接收 Rust 后端 tracing 日志
  useEffect(() => {
    let detach: (() => void) | null = null;

    (async () => {
      try {
        const { attachLogger } = await import("@tauri-apps/plugin-log");
        detach = await attachLogger(({ level, message }) => {
          // 1=TRACE, 2=DEBUG → 忽略; 3=INFO, 4=WARN, 5=ERROR
          if (level <= 2) return;

          let mapped: "info" | "warn" | "error";
          if (level === 4) mapped = "warn";
          else if (level >= 5) mapped = "error";
          else mapped = "info";

          useLogStore.getState().addEntry({
            id: String(++logId),
            timestamp: new Date().toLocaleTimeString("zh-CN", {
              hour12: false,
            }),
            level: mapped,
            message,
          });
        });
      } catch (e) {
        console.error("attachLogger failed:", e);
      }
    })();

    return () => {
      detach?.();
    };
  }, []);

  return null;
}
