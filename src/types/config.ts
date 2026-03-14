export interface ConfigEntry {
  key: string;
  value: string;
  updatedAt: string;
}

export interface AppConfig {
  dailyLimit: number;
  timesPerFriend: number;
  scheduleHour: number;
  scheduleMinute: number;
  autoStart: boolean;
  replyLikeEnabled: boolean;
  napcatPath: string;
  qqNumber: string;
  qqNickname: string;
  reservedForReply: number;
  batchInterval: number;
  replyTimes: number;
  replyDelayMin: number;
  replyDelayMax: number;
  minimizeToTray: boolean;
  onebotApiUrl: string;
  webhookPort: number;
}

export const CONFIG_DEFAULTS: AppConfig = {
  dailyLimit: 50,
  timesPerFriend: 10,
  scheduleHour: 0,
  scheduleMinute: 5,
  autoStart: false,
  replyLikeEnabled: false,
  napcatPath: "",
  qqNumber: "",
  qqNickname: "",
  reservedForReply: 10,
  batchInterval: 3,
  replyTimes: 10,
  replyDelayMin: 0,
  replyDelayMax: 0,
  minimizeToTray: true,
  onebotApiUrl: "http://127.0.0.1:3000",
  webhookPort: 8080,
};

export function parseConfigEntries(entries: ConfigEntry[]): AppConfig {
  const map = new Map(entries.map((e) => [e.key, e.value]));
  return {
    dailyLimit: Number(map.get("daily_limit") ?? CONFIG_DEFAULTS.dailyLimit),
    timesPerFriend: Number(
      map.get("times_per_friend") ?? CONFIG_DEFAULTS.timesPerFriend,
    ),
    scheduleHour: Number(
      map.get("schedule_hour") ?? CONFIG_DEFAULTS.scheduleHour,
    ),
    scheduleMinute: Number(
      map.get("schedule_minute") ?? CONFIG_DEFAULTS.scheduleMinute,
    ),
    autoStart: map.get("auto_start") === "true",
    replyLikeEnabled: map.get("reply_like_enabled") === "true",
    napcatPath: map.get("napcat_path") ?? CONFIG_DEFAULTS.napcatPath,
    qqNumber: map.get("qq_number") ?? CONFIG_DEFAULTS.qqNumber,
    qqNickname: map.get("qq_nickname") ?? CONFIG_DEFAULTS.qqNickname,
    reservedForReply: Number(
      map.get("reserved_for_reply") ?? CONFIG_DEFAULTS.reservedForReply,
    ),
    batchInterval: Number(
      map.get("batch_interval") ?? CONFIG_DEFAULTS.batchInterval,
    ),
    replyTimes: Number(
      map.get("reply_times") ?? CONFIG_DEFAULTS.replyTimes,
    ),
    replyDelayMin: Number(
      map.get("reply_delay_min") ?? CONFIG_DEFAULTS.replyDelayMin,
    ),
    replyDelayMax: Number(
      map.get("reply_delay_max") ?? CONFIG_DEFAULTS.replyDelayMax,
    ),
    minimizeToTray:
      (map.get("minimize_to_tray") ?? "true") === "true",
    onebotApiUrl:
      map.get("onebot_api_url") ?? CONFIG_DEFAULTS.onebotApiUrl,
    webhookPort: Number(
      map.get("webhook_port") ?? CONFIG_DEFAULTS.webhookPort,
    ),
  };
}
