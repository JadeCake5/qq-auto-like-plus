import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ConfigEntry, AppConfig } from "@/types/config";
import { parseConfigEntries } from "@/types/config";

interface SettingsStore {
  config: AppConfig | null;
  fetchConfig: () => Promise<void>;
  updateConfig: (key: string, value: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  config: null,
  fetchConfig: async () => {
    try {
      const entries = await invoke<ConfigEntry[]>("get_config");
      set({ config: parseConfigEntries(entries) });
    } catch {
      // 静默忽略
    }
  },
  updateConfig: async (key, value) => {
    try {
      await invoke("update_config", { key, value });
      const entries = await invoke<ConfigEntry[]>("get_config");
      set({ config: parseConfigEntries(entries) });
    } catch {
      // 静默忽略
    }
  },
}));
