import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { NapCatStatus, LoginInfo } from "@/types/napcat";

interface NapCatStore {
  status: NapCatStatus;
  loginInfo: LoginInfo | null;
  qrCodeSrc: string | null;
  setStatus: (status: NapCatStatus) => void;
  setLoginInfo: (info: LoginInfo) => void;
  setQrCodeSrc: (src: string | null) => void;
  fetchStatus: () => Promise<void>;
  fetchLoginInfo: () => Promise<void>;
}

export const useNapCatStore = create<NapCatStore>((set) => ({
  status: "notInstalled",
  loginInfo: null,
  qrCodeSrc: null,
  setStatus: (status) => {
    set({ status });
    // 登录成功或非等待登录状态时清除二维码
    if (status === "running" || status === "notInstalled" || status === "ready") {
      set({ qrCodeSrc: null });
    }
  },
  setLoginInfo: (info) => set({ loginInfo: info }),
  setQrCodeSrc: (src) => set({ qrCodeSrc: src }),
  fetchStatus: async () => {
    try {
      const status = await invoke<NapCatStatus>("get_napcat_status");
      set({ status });
    } catch {
      // 静默忽略
    }
  },
  fetchLoginInfo: async () => {
    try {
      const info = await invoke<LoginInfo>("get_login_info_cmd");
      set({ loginInfo: info });
    } catch {
      // 未登录时静默忽略
    }
  },
}));
