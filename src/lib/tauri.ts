import { invoke } from "@tauri-apps/api/core";
import type { ConfigEntry } from "@/types/config";
import type { EngineStatus } from "@/types/engine";
import type { BatchLikeResult } from "@/types/like";
import type { FriendWithTags, SyncFriendsResult, TagInfo } from "@/types/friends";
import type { QuotaStatus } from "@/types/stats";
import type { LoginInfo, NapCatStatus } from "@/types/napcat";
import type { HourlyStats, DailyStatsPoint, LikeTypeRatio, FriendRanking, StatsPeriod } from "@/types/stats";

export async function getConfig(): Promise<ConfigEntry[]> {
  return invoke<ConfigEntry[]>("get_config");
}

export async function updateConfig(
  key: string,
  value: string,
): Promise<void> {
  return invoke("update_config", { key, value });
}

export async function downloadNapcat(): Promise<void> {
  return invoke("download_napcat");
}

export async function importNapcat(zipPath: string): Promise<void> {
  return invoke("import_napcat", { zipPath });
}

export async function getNapCatStatus(): Promise<NapCatStatus> {
  return invoke<NapCatStatus>("get_napcat_status");
}

export async function startNapcat(): Promise<void> {
  return invoke("start_napcat");
}

export async function stopNapcat(): Promise<void> {
  return invoke("stop_napcat");
}

export async function getLoginInfo(): Promise<LoginInfo> {
  return invoke<LoginInfo>("get_login_info_cmd");
}

export async function startBatchLike(): Promise<BatchLikeResult> {
  return invoke<BatchLikeResult>("start_batch_like");
}

export async function pauseEngine(): Promise<void> {
  return invoke("pause_engine");
}

export async function resumeEngine(): Promise<void> {
  return invoke("resume_engine");
}

export async function getEngineStatus(): Promise<EngineStatus> {
  return invoke<EngineStatus>("get_engine_status");
}

export async function getNextRunTime(): Promise<string | null> {
  return invoke<string | null>("get_next_run_time");
}

export async function getDailyStats(): Promise<QuotaStatus> {
  return invoke<QuotaStatus>("get_daily_stats");
}

export async function enableAutostart(): Promise<void> {
  return invoke("enable_autostart");
}

export async function disableAutostart(): Promise<void> {
  return invoke("disable_autostart");
}

export async function isAutostartEnabled(): Promise<boolean> {
  return invoke<boolean>("is_autostart_enabled");
}

export async function getFriends(): Promise<FriendWithTags[]> {
  return invoke<FriendWithTags[]>("get_friends");
}

export async function syncFriends(): Promise<SyncFriendsResult> {
  return invoke<SyncFriendsResult>("sync_friends");
}

export async function getTags(): Promise<TagInfo[]> {
  return invoke<TagInfo[]>("get_tags");
}

export async function createTag(name: string, color: string): Promise<TagInfo> {
  return invoke<TagInfo>("create_tag", { name, color });
}

export async function updateTag(id: number, name: string, color: string): Promise<TagInfo> {
  return invoke<TagInfo>("update_tag", { id, name, color });
}

export async function deleteTag(id: number): Promise<void> {
  return invoke("delete_tag", { id });
}

export async function setFriendTags(friendId: number, tagIds: number[]): Promise<TagInfo[]> {
  return invoke<TagInfo[]>("set_friend_tags", { friendId, tagIds });
}

export async function updateTagStrategy(
  id: number,
  likeTimes: number | null,
  priority: string,
  autoLike: boolean,
  autoReply: boolean,
): Promise<TagInfo> {
  return invoke<TagInfo>("update_tag_strategy", { id, likeTimes, priority, autoLike, autoReply });
}

export async function getStatsDaily(date?: string): Promise<HourlyStats[]> {
  return invoke<HourlyStats[]>("get_stats_daily", { date: date ?? null });
}

export async function getStatsWeekly(): Promise<DailyStatsPoint[]> {
  return invoke<DailyStatsPoint[]>("get_stats_weekly");
}

export async function getStatsMonthly(): Promise<DailyStatsPoint[]> {
  return invoke<DailyStatsPoint[]>("get_stats_monthly");
}

export async function getLikeTypeRatio(period: StatsPeriod): Promise<LikeTypeRatio> {
  return invoke<LikeTypeRatio>("get_like_type_ratio", { period });
}

export async function getFriendRanking(period: StatsPeriod): Promise<FriendRanking[]> {
  return invoke<FriendRanking[]>("get_friend_ranking", { period });
}
