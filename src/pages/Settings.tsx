import { useEffect } from "react";
import { toast } from "sonner";
import { Heart, RefreshCw, Settings2, Server } from "lucide-react";
import { useSettingsStore } from "@/stores/useSettingsStore";
import {
  updateConfig as updateConfigApi,
  enableAutostart,
  disableAutostart,
} from "@/lib/tauri";
import { CONFIG_DEFAULTS } from "@/types/config";
import { SettingCard } from "@/components/settings/SettingCard";
import { SliderField } from "@/components/settings/SliderField";
import { TimePickerField } from "@/components/settings/TimePickerField";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

async function saveConfig(key: string, value: string) {
  try {
    await updateConfigApi(key, value);
    toast.success("设置已保存~");
  } catch {
    toast.error("保存失败，请重试");
  }
}

export default function Settings() {
  const { config, fetchConfig } = useSettingsStore();

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  if (!config) {
    return (
      <div className="page-enter flex h-full items-center justify-center">
        <span className="text-text-muted">加载中...</span>
      </div>
    );
  }

  const handleAutostart = async (enabled: boolean) => {
    try {
      if (enabled) {
        await enableAutostart();
      } else {
        await disableAutostart();
      }
      toast.success("设置已保存~");
    } catch {
      toast.error("保存失败，请重试");
    }
  };

  const handleDelayMinChange = async (val: number) => {
    await saveConfig("reply_delay_min", String(val));
    if (val > config.replyDelayMax) {
      await saveConfig("reply_delay_max", String(val));
    }
  };

  const handleDelayMaxChange = async (val: number) => {
    await saveConfig("reply_delay_max", String(val));
    if (val < config.replyDelayMin) {
      await saveConfig("reply_delay_min", String(val));
    }
  };

  const handlePortBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    const num = Number(e.target.value);
    if (isNaN(num) || num < 1024) {
      saveConfig("webhook_port", "1024");
    } else if (num > 65535) {
      saveConfig("webhook_port", "65535");
    } else {
      saveConfig("webhook_port", String(Math.floor(num)));
    }
  };

  const resetLikeDefaults = async () => {
    await saveConfig("daily_limit", String(CONFIG_DEFAULTS.dailyLimit));
    await saveConfig(
      "times_per_friend",
      String(CONFIG_DEFAULTS.timesPerFriend),
    );
    await saveConfig("schedule_hour", String(CONFIG_DEFAULTS.scheduleHour));
    await saveConfig(
      "schedule_minute",
      String(CONFIG_DEFAULTS.scheduleMinute),
    );
    await saveConfig("batch_interval", String(CONFIG_DEFAULTS.batchInterval));
  };

  const resetReplyDefaults = async () => {
    await saveConfig(
      "reply_like_enabled",
      String(CONFIG_DEFAULTS.replyLikeEnabled),
    );
    await saveConfig("reply_times", String(CONFIG_DEFAULTS.replyTimes));
    await saveConfig(
      "reserved_for_reply",
      String(CONFIG_DEFAULTS.reservedForReply),
    );
    await saveConfig("reply_delay_min", String(CONFIG_DEFAULTS.replyDelayMin));
    await saveConfig("reply_delay_max", String(CONFIG_DEFAULTS.replyDelayMax));
  };

  const resetSystemDefaults = async () => {
    try {
      await disableAutostart();
    } catch {
      /* ignore */
    }
    await saveConfig(
      "minimize_to_tray",
      String(CONFIG_DEFAULTS.minimizeToTray),
    );
  };

  const resetEnvDefaults = async () => {
    await saveConfig("onebot_api_url", CONFIG_DEFAULTS.onebotApiUrl);
    await saveConfig("webhook_port", String(CONFIG_DEFAULTS.webhookPort));
  };

  return (
    <div className="page-enter flex flex-col gap-4 overflow-y-auto pr-1">
      <h1 className="text-[length:var(--text-display)] font-bold text-text-primary">
        设置
      </h1>

      {/* 点赞设置 */}
      <SettingCard
        title="点赞设置"
        icon={<Heart className="size-4 text-primary" />}
        onResetDefaults={resetLikeDefaults}
      >
        <SliderField
          label="每日名额"
          value={config.dailyLimit}
          min={20}
          max={200}
          unit="人"
          onChange={(v) => saveConfig("daily_limit", String(v))}
        />
        <SliderField
          label="每人次数"
          value={config.timesPerFriend}
          min={1}
          max={20}
          unit="次"
          onChange={(v) => saveConfig("times_per_friend", String(v))}
        />
        <TimePickerField
          label="定时时间"
          hour={config.scheduleHour}
          minute={config.scheduleMinute}
          onHourChange={(h) => saveConfig("schedule_hour", String(h))}
          onMinuteChange={(m) => saveConfig("schedule_minute", String(m))}
        />
        <SliderField
          label="批次间隔"
          value={config.batchInterval}
          min={1}
          max={60}
          unit="秒"
          onChange={(v) => saveConfig("batch_interval", String(v))}
        />
      </SettingCard>

      {/* 回赞设置 */}
      <SettingCard
        title="回赞设置"
        icon={<RefreshCw className="size-4 text-secondary" />}
        onResetDefaults={resetReplyDefaults}
      >
        <div className="flex items-center justify-between">
          <span className="text-sm text-text-secondary">回赞开关</span>
          <Switch
            checked={config.replyLikeEnabled}
            onCheckedChange={(checked) =>
              saveConfig("reply_like_enabled", String(checked))
            }
            aria-label="回赞开关"
          />
        </div>
        <SliderField
          label="回赞次数"
          value={config.replyTimes}
          min={1}
          max={20}
          unit="次"
          onChange={(v) => saveConfig("reply_times", String(v))}
        />
        <SliderField
          label="预留名额"
          value={config.reservedForReply}
          min={0}
          max={100}
          unit="人"
          onChange={(v) => saveConfig("reserved_for_reply", String(v))}
        />
        <SliderField
          label="回赞延迟最小"
          value={config.replyDelayMin}
          min={0}
          max={60}
          unit="秒"
          onChange={handleDelayMinChange}
        />
        <SliderField
          label="回赞延迟最大"
          value={config.replyDelayMax}
          min={0}
          max={60}
          unit="秒"
          onChange={handleDelayMaxChange}
        />
      </SettingCard>

      {/* 系统设置 */}
      <SettingCard
        title="系统设置"
        icon={<Settings2 className="size-4 text-accent" />}
        onResetDefaults={resetSystemDefaults}
      >
        <div className="flex items-center justify-between">
          <span className="text-sm text-text-secondary">开机自启</span>
          <Switch
            checked={config.autoStart}
            onCheckedChange={handleAutostart}
            aria-label="开机自启"
          />
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm text-text-secondary">最小化到托盘</span>
          <Switch
            checked={config.minimizeToTray}
            onCheckedChange={(checked) =>
              saveConfig("minimize_to_tray", String(checked))
            }
            aria-label="最小化到托盘"
          />
        </div>
      </SettingCard>

      {/* 运行环境设置 */}
      <SettingCard
        title="运行环境设置"
        icon={<Server className="size-4 text-mint" />}
        onResetDefaults={resetEnvDefaults}
      >
        <div className="space-y-1">
          <span className="text-sm text-text-secondary">运行环境路径</span>
          <div className="flex items-center gap-2">
            <span className="flex-1 truncate rounded-lg border border-input bg-transparent px-2.5 py-1.5 text-sm text-text-muted">
              {config.napcatPath || "自动检测"}
            </span>
            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    variant="outline"
                    size="sm"
                    disabled
                    aria-label="修改运行环境路径"
                  />
                }
              >
                修改
              </TooltipTrigger>
              <TooltipContent>暂不支持修改</TooltipContent>
            </Tooltip>
          </div>
        </div>

        <div className="space-y-1">
          <span className="text-sm text-text-secondary">API 地址</span>
          <Input
            key={config.onebotApiUrl}
            defaultValue={config.onebotApiUrl}
            onBlur={(e) => saveConfig("onebot_api_url", e.target.value)}
            placeholder="http://127.0.0.1:3000"
            aria-label="API 地址"
          />
        </div>

        <div className="space-y-1">
          <span className="text-sm text-text-secondary">Webhook 端口</span>
          <Input
            key={config.webhookPort}
            type="number"
            defaultValue={String(config.webhookPort)}
            onBlur={handlePortBlur}
            min={1024}
            max={65535}
            placeholder="8080"
            aria-label="Webhook 端口"
          />
        </div>
      </SettingCard>
    </div>
  );
}
