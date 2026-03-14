use std::collections::HashSet;

use serde::Serialize;
use tauri::{Emitter, State};

use crate::db::{self, DbState};
use crate::db::models::{FriendRow, FriendWithTags, TagRow};
use crate::engine::quota;
use crate::onebot::OneBotClientState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFriendsResult {
    pub total: i64,
    pub new_count: i64,
}

#[tauri::command]
pub fn get_friends(db: State<'_, DbState>) -> Result<Vec<FriendWithTags>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    db::models::get_all_friends_with_tags(&conn, &quota::today()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_friends(
    db: State<'_, DbState>,
    onebot: State<'_, OneBotClientState>,
) -> Result<SyncFriendsResult, String> {
    // 1. 调用 OneBot API 获取远端好友列表
    let remote_friends = onebot.get_friend_list().await.map_err(|e| e.to_string())?;

    let (total, new_count) = {
        let conn = db.lock().map_err(|e| e.to_string())?;

        // 2. 获取当前数据库中的好友 user_id 集合
        let mut stmt = conn
            .prepare("SELECT user_id FROM friends")
            .map_err(|e| e.to_string())?;
        let existing_ids: HashSet<i64> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<HashSet<_>, _>>()
            .map_err(|e| e.to_string())?;

        // 3. 转换为 FriendRow 并批量 upsert
        let friend_rows: Vec<FriendRow> = remote_friends
            .iter()
            .map(|f| FriendRow {
                user_id: f.user_id,
                nickname: f.nickname.clone(),
                remark: f.remark.clone(),
            })
            .collect();
        db::models::upsert_friends_batch(&conn, &friend_rows).map_err(|e| e.to_string())?;

        // 4. 找出新好友
        let new_ids: Vec<i64> = remote_friends
            .iter()
            .filter(|f| !existing_ids.contains(&f.user_id))
            .map(|f| f.user_id)
            .collect();

        // 5. 为新好友分配"默认"标签
        if !new_ids.is_empty() {
            tracing::info!("发现 {} 个新好友，分配默认标签", new_ids.len());
            db::models::assign_default_tag_to_new_friends(&conn, &new_ids)
                .map_err(|e| e.to_string())?;
        }

        (remote_friends.len() as i64, new_ids.len() as i64)
    };

    tracing::info!("好友同步完成: 总数={}, 新增={}", total, new_count);
    Ok(SyncFriendsResult { total, new_count })
}

#[tauri::command]
pub fn get_tags(db: State<'_, DbState>) -> Result<Vec<TagRow>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    db::models::get_all_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(db: State<'_, DbState>, name: String, color: String) -> Result<TagRow, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    db::models::create_tag(&conn, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag(db: State<'_, DbState>, id: i64, name: String, color: String) -> Result<TagRow, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    db::models::update_tag(&conn, id, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(db: State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    db::models::delete_tag(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_friend_tags(db: State<'_, DbState>, friend_id: i64, tag_ids: Vec<i64>) -> Result<Vec<TagRow>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    db::models::set_friend_tags(&conn, friend_id, &tag_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag_strategy(
    db: State<'_, DbState>,
    app: tauri::AppHandle,
    id: i64,
    like_times: Option<i32>,
    priority: String,
    auto_like: bool,
    auto_reply: bool,
) -> Result<TagRow, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let tag = db::models::update_tag_strategy(&conn, id, like_times, &priority, auto_like, auto_reply)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("config:updated", ());
    Ok(tag)
}
