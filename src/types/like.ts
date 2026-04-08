/** 批量点赞进度事件 payload（对应 like:progress 事件） */
export interface BatchLikeProgress {
  current: number;
  total: number;
  userId: number;
  nickname: string;
  success: boolean;
  skipped: boolean;
  errorMsg?: string | null;
}

/** 批量点赞完成事件 payload（对应 like:batch-complete 事件） */
export interface BatchLikeResult {
  total: number;
  successCount: number;
  skippedCount: number;
  failedCount: number;
}

/** 回赞完成事件 payload（对应 like:reply-complete 事件） */
export interface ReplyLikeResult {
  operatorId: number;
  times: number;
  success: boolean;
  skipped: boolean;
  skipReason: string | null;
}
