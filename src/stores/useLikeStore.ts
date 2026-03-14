import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { EngineStatus } from "@/types/engine";
import type { BatchLikeProgress, BatchLikeResult } from "@/types/like";
import type { QuotaStatus } from "@/types/stats";

interface LikeStore {
  dailyStats: QuotaStatus | null;
  isRunning: boolean;
  isPaused: boolean;
  batchProgress: BatchLikeProgress | null;
  lastBatchResult: BatchLikeResult | null;
  nextRunTime: string | null;
  setEngineStatus: (status: EngineStatus) => void;
  setBatchProgress: (progress: BatchLikeProgress) => void;
  onBatchComplete: (result: BatchLikeResult) => void;
  fetchDailyStats: () => Promise<void>;
  fetchEngineStatus: () => Promise<void>;
}

export const useLikeStore = create<LikeStore>((set) => ({
  dailyStats: null,
  isRunning: false,
  isPaused: false,
  batchProgress: null,
  lastBatchResult: null,
  nextRunTime: null,
  setEngineStatus: (status) =>
    set({
      isRunning: status.isRunningBatch,
      isPaused: status.isPaused,
      nextRunTime: status.nextRunTime,
    }),
  setBatchProgress: (progress) => set({ batchProgress: progress }),
  onBatchComplete: (result) =>
    set({ lastBatchResult: result, batchProgress: null }),
  fetchDailyStats: async () => {
    try {
      const stats = await invoke<QuotaStatus>("get_daily_stats");
      set({ dailyStats: stats });
    } catch {
      // 静默忽略
    }
  },
  fetchEngineStatus: async () => {
    try {
      const status = await invoke<EngineStatus>("get_engine_status");
      set({
        isRunning: status.isRunningBatch,
        isPaused: status.isPaused,
        nextRunTime: status.nextRunTime,
      });
    } catch {
      // 静默忽略
    }
  },
}));
