/** 每日名额状态（对应 Rust QuotaStatus） */
export interface QuotaStatus {
  date: string;
  dailyLimit: number;
  reservedForReply: number;
  totalLiked: number;
  scheduledCount: number;
  replyCount: number;
  manualCount: number;
  availableScheduled: number;
  availableReply: number;
}

/** 每小时统计数据点（日视图） */
export interface HourlyStats {
  hour: number;
  count: number;
}

/** 每日统计数据点（周/月视图） */
export interface DailyStatsPoint {
  date: string;
  count: number;
  scheduled: number;
  reply: number;
  manual: number;
}

/** 点赞类型占比 */
export interface LikeTypeRatio {
  scheduled: number;
  reply: number;
  manual: number;
  total: number;
}

/** 好友互动排行 */
export interface FriendRanking {
  userId: number;
  nickname: string;
  totalLikes: number;
}

/** 统计时间范围 */
export type StatsPeriod = "day" | "week" | "month";
