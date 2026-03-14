import { create } from "zustand";

export interface LogEntry {
  id: string;
  timestamp: string;
  level: "info" | "warn" | "error";
  message: string;
  source?: string;
}

export type LogFilter = "all" | "info" | "warn" | "error";

export const MAX_LOG_ENTRIES = 2000;

interface LogStore {
  entries: LogEntry[];
  filter: LogFilter;
  searchKeyword: string;
  addEntry: (entry: LogEntry) => void;
  clear: () => void;
  setFilter: (filter: LogFilter) => void;
  setSearchKeyword: (keyword: string) => void;
  getFilteredEntries: () => LogEntry[];
}

export const useLogStore = create<LogStore>((set, get) => ({
  entries: [],
  filter: "all",
  searchKeyword: "",
  addEntry: (entry) =>
    set((state) => {
      const next = [...state.entries, entry];
      if (next.length > MAX_LOG_ENTRIES) {
        return { entries: next.slice(next.length - MAX_LOG_ENTRIES) };
      }
      return { entries: next };
    }),
  clear: () => set({ entries: [] }),
  setFilter: (filter) => set({ filter }),
  setSearchKeyword: (keyword) => set({ searchKeyword: keyword }),
  getFilteredEntries: () => {
    const { entries, filter, searchKeyword } = get();
    let filtered = entries;
    if (filter !== "all") {
      filtered = filtered.filter((e) => e.level === filter);
    }
    if (searchKeyword) {
      const kw = searchKeyword.toLowerCase();
      filtered = filtered.filter((e) => e.message.toLowerCase().includes(kw));
    }
    return filtered;
  },
}));
