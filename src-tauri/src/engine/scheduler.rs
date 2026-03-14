use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::sync::Mutex as TokioMutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::commands::like::BatchLikeRunning;
use crate::db::DbState;
use crate::onebot::OneBotClientState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub is_paused: bool,
    pub is_running_batch: bool,
    pub next_run_time: Option<String>,
    pub schedule_hour: u32,
    pub schedule_minute: u32,
}

struct SchedulerInner {
    scheduler: JobScheduler,
    current_job_id: Option<Uuid>,
    is_paused: bool,
    schedule_hour: u32,
    schedule_minute: u32,
}

#[derive(Clone)]
pub struct LikeScheduler {
    inner: Arc<TokioMutex<SchedulerInner>>,
}

pub type LikeSchedulerState = Arc<LikeScheduler>;

/// 本地时间转 UTC cron 表达式
fn build_cron_expr(local_hour: u32, local_minute: u32) -> String {
    use chrono::Local;
    let now = Local::now();
    let offset_secs = now.offset().local_minus_utc();
    let offset_hours = offset_secs / 3600;

    let utc_hour = ((local_hour as i32 - offset_hours) % 24 + 24) % 24;
    format!("0 {} {} * * *", local_minute, utc_hour)
}

impl LikeScheduler {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            inner: Arc::new(TokioMutex::new(SchedulerInner {
                scheduler,
                current_job_id: None,
                is_paused: false,
                schedule_hour: 0,
                schedule_minute: 5,
            })),
        })
    }

    /// 初始化并启动调度器
    pub async fn start(
        &self,
        db: DbState,
        onebot: OneBotClientState,
        app: tauri::AppHandle,
        running: BatchLikeRunning,
    ) -> Result<(), anyhow::Error> {
        // 1. 从 config 读取定时时间和暂停状态
        let (hour, minute, paused) = {
            let conn = db.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            let h: u32 = crate::db::models::get_config_by_key(&conn, "schedule_hour")
                .ok().and_then(|c| c.value.parse().ok()).unwrap_or(0);
            let m: u32 = crate::db::models::get_config_by_key(&conn, "schedule_minute")
                .ok().and_then(|c| c.value.parse().ok()).unwrap_or(5);
            let p: bool = crate::db::models::get_config_by_key(&conn, "engine_paused")
                .ok().map(|c| c.value == "true").unwrap_or(false);
            (h, m, p)
        };

        let mut inner = self.inner.lock().await;
        inner.schedule_hour = hour;
        inner.schedule_minute = minute;
        inner.is_paused = paused;

        // 2. 如果不是暂停状态，注册 cron job
        if !paused {
            let cron_expr = build_cron_expr(hour, minute);
            let job_id = Self::add_cron_job(
                &mut inner, &cron_expr, db.clone(), onebot.clone(),
                app.clone(), running.clone(),
            ).await?;
            inner.current_job_id = Some(job_id);
        }

        // 3. 启动调度器
        inner.scheduler.start().await?;
        tracing::info!(
            "调度器已启动: {:02}:{:02} (本地时间), paused={}",
            hour, minute, paused
        );

        // 4. 通知前端
        let status = Self::build_status_from(&mut inner).await;
        let _ = app.emit("engine:status-changed", &status);

        Ok(())
    }

    /// 重新调度（用户修改定时时间后调用）
    pub async fn reschedule(
        &self,
        hour: u32,
        minute: u32,
        db: DbState,
        onebot: OneBotClientState,
        app: tauri::AppHandle,
        running: BatchLikeRunning,
    ) -> Result<(), anyhow::Error> {
        let mut inner = self.inner.lock().await;
        inner.schedule_hour = hour;
        inner.schedule_minute = minute;

        // 移除旧 job
        if let Some(old_id) = inner.current_job_id.take() {
            inner.scheduler.remove(&old_id).await?;
            tracing::info!("旧 cron job 已移除: {}", old_id);
        }

        // 如果不在暂停状态，注册新 job
        if !inner.is_paused {
            let cron_expr = build_cron_expr(hour, minute);
            let job_id = Self::add_cron_job(
                &mut inner, &cron_expr, db, onebot,
                app.clone(), running,
            ).await?;
            inner.current_job_id = Some(job_id);
        }

        let status = Self::build_status_from(&mut inner).await;
        let _ = app.emit("engine:status-changed", &status);
        tracing::info!("调度器已重新配置: {:02}:{:02}", hour, minute);
        Ok(())
    }

    /// 暂停调度
    pub async fn pause(
        &self,
        db: &DbState,
        app: &tauri::AppHandle,
    ) -> Result<(), anyhow::Error> {
        let mut inner = self.inner.lock().await;
        if inner.is_paused {
            return Ok(());
        }

        // 移除 cron job
        if let Some(old_id) = inner.current_job_id.take() {
            inner.scheduler.remove(&old_id).await?;
        }
        inner.is_paused = true;

        // 持久化暂停状态
        {
            let conn = db.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            crate::db::models::upsert_config(&conn, "engine_paused", "true")
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }

        let status = Self::build_status_from(&mut inner).await;
        let _ = app.emit("engine:status-changed", &status);
        tracing::info!("调度器已暂停");
        Ok(())
    }

    /// 恢复调度
    pub async fn resume(
        &self,
        db: DbState,
        onebot: OneBotClientState,
        app: tauri::AppHandle,
        running: BatchLikeRunning,
    ) -> Result<(), anyhow::Error> {
        let mut inner = self.inner.lock().await;
        if !inner.is_paused {
            return Ok(());
        }
        inner.is_paused = false;

        // 持久化恢复状态
        {
            let conn = db.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            crate::db::models::upsert_config(&conn, "engine_paused", "false")
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }

        // 重新注册 cron job
        let cron_expr = build_cron_expr(inner.schedule_hour, inner.schedule_minute);
        let job_id = Self::add_cron_job(
            &mut inner, &cron_expr, db, onebot,
            app.clone(), running,
        ).await?;
        inner.current_job_id = Some(job_id);

        let status = Self::build_status_from(&mut inner).await;
        let _ = app.emit("engine:status-changed", &status);
        tracing::info!("调度器已恢复");
        Ok(())
    }

    /// 获取下次执行时间
    pub async fn get_next_run_time(&self) -> Option<String> {
        let mut inner = self.inner.lock().await;
        if let Some(job_id) = inner.current_job_id {
            match inner.scheduler.next_tick_for_job(job_id).await {
                Ok(Some(dt)) => Some(dt.to_rfc3339()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 获取引擎状态
    pub async fn get_status(&self) -> EngineStatus {
        let mut inner = self.inner.lock().await;
        Self::build_status_from(&mut inner).await
    }

    /// 创建并添加 cron job
    async fn add_cron_job(
        inner: &mut SchedulerInner,
        cron_expr: &str,
        db: DbState,
        onebot: OneBotClientState,
        app: tauri::AppHandle,
        running: BatchLikeRunning,
    ) -> Result<Uuid, anyhow::Error> {
        let job = Job::new_async(cron_expr, move |_uuid, _lock| {
            let db = db.clone();
            let onebot = onebot.clone();
            let app = app.clone();
            let running = running.clone();
            Box::pin(async move {
                // 防止重复执行
                if running.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    tracing::warn!("定时触发时批量点赞仍在执行，跳过本次");
                    return;
                }
                tracing::info!("定时任务触发: 开始批量点赞");
                let result = crate::engine::like_executor::run_batch_like(
                    &db, &onebot, &app, "scheduled"
                ).await;
                running.store(false, std::sync::atomic::Ordering::SeqCst);
                match result {
                    Ok(r) => tracing::info!("定时批量点赞完成: {:?}", r),
                    Err(e) => {
                        tracing::error!("定时批量点赞异常: {}", e);
                        let _ = app.emit("like:batch-error", e.to_string());
                    }
                }
            })
        })?;
        let job_id = inner.scheduler.add(job).await?;
        tracing::info!("Cron job 已注册: {} (expr: {})", job_id, cron_expr);
        Ok(job_id)
    }

    async fn build_status_from(inner: &mut SchedulerInner) -> EngineStatus {
        let next_run = if let Some(job_id) = inner.current_job_id {
            inner.scheduler.next_tick_for_job(job_id).await
                .ok().flatten().map(|dt| dt.to_rfc3339())
        } else {
            None
        };

        EngineStatus {
            is_paused: inner.is_paused,
            is_running_batch: false,
            next_run_time: next_run,
            schedule_hour: inner.schedule_hour,
            schedule_minute: inner.schedule_minute,
        }
    }
}
