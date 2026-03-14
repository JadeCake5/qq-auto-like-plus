import { create } from "zustand";
import type { HourlyStats, DailyStatsPoint, LikeTypeRatio, FriendRanking, StatsPeriod } from "@/types/stats";
import { getStatsDaily, getStatsWeekly, getStatsMonthly, getLikeTypeRatio, getFriendRanking } from "@/lib/tauri";

interface StatsStore {
  hourlyData: HourlyStats[];
  weeklyData: DailyStatsPoint[];
  monthlyData: DailyStatsPoint[];
  typeRatio: LikeTypeRatio | null;
  friendRanking: FriendRanking[];
  currentPeriod: StatsPeriod;
  isLoading: boolean;

  fetchDailyStats: (date?: string) => Promise<void>;
  fetchWeeklyStats: () => Promise<void>;
  fetchMonthlyStats: () => Promise<void>;
  fetchLikeTypeRatio: (period?: StatsPeriod) => Promise<void>;
  fetchFriendRanking: (period?: StatsPeriod) => Promise<void>;
  setPeriod: (period: StatsPeriod) => Promise<void>;
}

export const useStatsStore = create<StatsStore>((set, get) => ({
  hourlyData: [],
  weeklyData: [],
  monthlyData: [],
  typeRatio: null,
  friendRanking: [],
  currentPeriod: "week",
  isLoading: false,

  fetchDailyStats: async (date) => {
    try {
      set({ isLoading: true });
      const data = await getStatsDaily(date);
      set({ hourlyData: data });
    } catch {
      // 静默处理，UI 层 toast
    } finally {
      set({ isLoading: false });
    }
  },

  fetchWeeklyStats: async () => {
    try {
      set({ isLoading: true });
      const data = await getStatsWeekly();
      set({ weeklyData: data });
    } catch {
      // 静默处理
    } finally {
      set({ isLoading: false });
    }
  },

  fetchMonthlyStats: async () => {
    try {
      set({ isLoading: true });
      const data = await getStatsMonthly();
      set({ monthlyData: data });
    } catch {
      // 静默处理
    } finally {
      set({ isLoading: false });
    }
  },

  fetchLikeTypeRatio: async (period) => {
    try {
      const p = period ?? get().currentPeriod;
      const data = await getLikeTypeRatio(p);
      set({ typeRatio: data });
    } catch {
      // 静默处理
    }
  },

  fetchFriendRanking: async (period) => {
    try {
      const p = period ?? get().currentPeriod;
      const data = await getFriendRanking(p);
      set({ friendRanking: data });
    } catch {
      // 静默处理
    }
  },

  setPeriod: async (period) => {
    set({ currentPeriod: period });
    const store = get();
    if (period === "day") {
      await store.fetchDailyStats();
    } else if (period === "week") {
      await store.fetchWeeklyStats();
    } else {
      await store.fetchMonthlyStats();
    }
    await Promise.all([
      store.fetchLikeTypeRatio(period),
      store.fetchFriendRanking(period),
    ]);
  },
}));
