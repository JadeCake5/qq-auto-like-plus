export type NapCatStatus =
  | "notInstalled"
  | "downloading"
  | "extracting"
  | "ready"
  | "starting"
  | "waitingForLogin"
  | "running"
  | { error: string };

export interface DownloadProgress {
  percentage: number;
  downloadedBytes: number;
  totalBytes: number;
  speedBps: number;
  etaSeconds: number;
}

export interface ExtractProgress {
  currentFile: number;
  totalFiles: number;
  percentage: number;
}

export interface LoginInfo {
  qqNumber: string;
  nickname: string;
}
