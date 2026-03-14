/** 引擎状态（对应 engine:status-changed 事件 + get_engine_status 命令） */
export interface EngineStatus {
  isPaused: boolean;
  isRunningBatch: boolean;
  nextRunTime: string | null;
  scheduleHour: number;
  scheduleMinute: number;
}
