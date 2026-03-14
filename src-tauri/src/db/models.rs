use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

// ===== ConfigEntry =====

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

pub fn get_all_config(conn: &Connection) -> Result<Vec<ConfigEntry>, AppError> {
    let mut stmt = conn.prepare("SELECT key, value, updated_at FROM config ORDER BY key")?;
    let entries = stmt
        .query_map([], |row| {
            Ok(ConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn get_config_by_key(conn: &Connection, key: &str) -> Result<ConfigEntry, AppError> {
    conn.query_row(
        "SELECT key, value, updated_at FROM config WHERE key = ?1",
        [key],
        |row| {
            Ok(ConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::ConfigNotFound(key.to_string()),
        other => AppError::Database(other),
    })
}

pub fn upsert_config(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO config (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP",
        params![key, value],
    )?;
    Ok(())
}

// ===== DailyState =====

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyState {
    pub date: String,
    pub liked_count: i32,
    pub target_count: i32,
    pub is_completed: bool,
    pub last_run_at: Option<String>,
    pub scheduled_count: i32,
    pub reply_count: i32,
    pub manual_count: i32,
}

pub fn get_today_state(conn: &Connection, date: &str) -> Result<Option<DailyState>, AppError> {
    let result = conn.query_row(
        "SELECT date, liked_count, target_count, is_completed, last_run_at, scheduled_count, reply_count, manual_count FROM daily_state WHERE date = ?1",
        [date],
        |row| {
            Ok(DailyState {
                date: row.get(0)?,
                liked_count: row.get(1)?,
                target_count: row.get(2)?,
                is_completed: row.get::<_, i32>(3)? != 0,
                last_run_at: row.get(4)?,
                scheduled_count: row.get(5)?,
                reply_count: row.get(6)?,
                manual_count: row.get(7)?,
            })
        },
    );

    match result {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn upsert_today_state(
    conn: &Connection,
    date: &str,
    liked_count: i32,
    target_count: i32,
    is_completed: bool,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO daily_state (date, liked_count, target_count, is_completed, last_run_at)
         VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
         ON CONFLICT(date) DO UPDATE SET
            liked_count = excluded.liked_count,
            target_count = excluded.target_count,
            is_completed = excluded.is_completed,
            last_run_at = CURRENT_TIMESTAMP",
        params![date, liked_count, target_count, is_completed as i32],
    )?;
    Ok(())
}

/// 确保指定日期的 daily_state 记录存在
pub fn ensure_daily_state(conn: &Connection, date: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO daily_state (date) VALUES (?1)",
        [date],
    )?;
    Ok(())
}

/// 原子递增每日计数：同时更新 liked_count 和对应类型计数
pub fn increment_daily_count(
    conn: &Connection,
    date: &str,
    like_type: &str,
) -> Result<(), rusqlite::Error> {
    let type_column = match like_type {
        "scheduled" => "scheduled_count",
        "reply" => "reply_count",
        "manual" => "manual_count",
        _ => return Err(rusqlite::Error::InvalidParameterName(
            format!("无效的 like_type: {}", like_type),
        )),
    };

    ensure_daily_state(conn, date)?;

    conn.execute(
        &format!(
            "UPDATE daily_state SET liked_count = liked_count + 1, {} = {} + 1, last_run_at = CURRENT_TIMESTAMP WHERE date = ?1",
            type_column, type_column
        ),
        [date],
    )?;

    Ok(())
}

// ===== LikeHistory =====

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeHistory {
    pub id: i64,
    pub user_id: i64,
    pub times: i32,
    pub like_type: String,
    pub success: bool,
    pub error_msg: Option<String>,
    pub created_at: String,
}

/// 插入点赞历史记录
pub fn insert_like_history(
    conn: &Connection,
    user_id: i64,
    times: i32,
    like_type: &str,
    success: bool,
    error_msg: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO like_history (user_id, times, like_type, success, error_msg) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![user_id, times, like_type, success as i32, error_msg],
    )?;
    Ok(())
}

// ===== Friends =====

pub struct FriendRow {
    pub user_id: i64,
    pub nickname: String,
    pub remark: String,
}

/// 批量 upsert 好友信息（从 OneBot get_friend_list 同步）
pub fn upsert_friends_batch(conn: &Connection, friends: &[FriendRow]) -> Result<(), rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO friends (user_id, nickname, remark, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(user_id) DO UPDATE SET
                nickname = excluded.nickname,
                remark = excluded.remark,
                updated_at = CURRENT_TIMESTAMP"
        )?;
        for f in friends {
            stmt.execute(params![f.user_id, f.nickname, f.remark])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// ===== LikeHistory (查询) =====

/// 查询用户今日是否已被赞过（使用范围查询命中复合索引）
pub fn has_liked_today(
    conn: &Connection,
    user_id: i64,
    date: &str,
) -> Result<bool, rusqlite::Error> {
    let date_start = format!("{} 00:00:00", date);
    let date_end = format!("{} 23:59:59", date);
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM like_history WHERE user_id = ?1 AND success = 1 AND created_at >= ?2 AND created_at <= ?3",
        params![user_id, date_start, date_end],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

// ===== Tags =====

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRow {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub is_system: bool,
    pub like_times: Option<i32>,
    pub priority: String,
    pub auto_like: bool,
    pub auto_reply: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendWithTags {
    pub user_id: i64,
    pub nickname: String,
    pub remark: String,
    pub tags: Vec<TagRow>,
    pub liked_today: bool,
}

pub fn get_all_tags(conn: &Connection) -> Result<Vec<TagRow>, AppError> {
    let mut stmt = conn.prepare("SELECT id, name, color, is_system, like_times, priority, auto_like, auto_reply FROM tags ORDER BY id")?;
    let tags = stmt
        .query_map([], |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
                like_times: row.get(4)?,
                priority: row.get(5)?,
                auto_like: row.get::<_, i32>(6)? != 0,
                auto_reply: row.get::<_, i32>(7)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

pub fn get_default_tag_id(conn: &Connection) -> Result<i64, AppError> {
    let id: i64 = conn.query_row(
        "SELECT id FROM tags WHERE name = '默认' AND is_system = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn assign_default_tag_to_new_friends(conn: &Connection, user_ids: &[i64]) -> Result<(), AppError> {
    let default_tag_id = get_default_tag_id(conn)?;
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO friend_tags (friend_id, tag_id) VALUES (?1, ?2)"
    )?;
    for uid in user_ids {
        stmt.execute(params![uid, default_tag_id])?;
    }
    Ok(())
}

pub fn create_tag(conn: &Connection, name: &str, color: &str) -> Result<TagRow, AppError> {
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )?;
    let id = conn.last_insert_rowid();
    Ok(TagRow {
        id,
        name: name.to_string(),
        color: color.to_string(),
        is_system: false,
        like_times: None,
        priority: "medium".to_string(),
        auto_like: true,
        auto_reply: true,
    })
}

pub fn update_tag(conn: &Connection, id: i64, name: &str, color: &str) -> Result<TagRow, AppError> {
    let tag: (bool, String) = conn.query_row(
        "SELECT is_system, name FROM tags WHERE id = ?1",
        [id],
        |row| Ok((row.get::<_, i32>(0)? != 0, row.get(1)?)),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NapCat(format!("标签不存在: {}", id)),
        other => AppError::Database(other),
    })?;

    let (is_system, original_name) = tag;
    let final_name = if is_system { &original_name } else { name };

    conn.execute(
        "UPDATE tags SET name = ?2, color = ?3 WHERE id = ?1",
        params![id, final_name, color],
    )?;

    conn.query_row(
        "SELECT id, name, color, is_system, like_times, priority, auto_like, auto_reply FROM tags WHERE id = ?1",
        [id],
        |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
                like_times: row.get(4)?,
                priority: row.get(5)?,
                auto_like: row.get::<_, i32>(6)? != 0,
                auto_reply: row.get::<_, i32>(7)? != 0,
            })
        },
    ).map_err(|e| AppError::Database(e))
}

pub fn delete_tag(conn: &Connection, id: i64) -> Result<(), AppError> {
    let is_system: bool = conn.query_row(
        "SELECT is_system FROM tags WHERE id = ?1",
        [id],
        |row| Ok(row.get::<_, i32>(0)? != 0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NapCat(format!("标签不存在: {}", id)),
        other => AppError::Database(other),
    })?;

    if is_system {
        return Err(AppError::NapCat("系统标签不可删除".to_string()));
    }

    conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    Ok(())
}

pub fn set_friend_tags(conn: &Connection, friend_id: i64, tag_ids: &[i64]) -> Result<Vec<TagRow>, AppError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM friend_tags WHERE friend_id = ?1", [friend_id])?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO friend_tags (friend_id, tag_id) VALUES (?1, ?2)"
        )?;
        for tag_id in tag_ids {
            stmt.execute(params![friend_id, tag_id])?;
        }
    }
    tx.commit()?;

    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.is_system, t.like_times, t.priority, t.auto_like, t.auto_reply
         FROM friend_tags ft JOIN tags t ON ft.tag_id = t.id
         WHERE ft.friend_id = ?1 ORDER BY t.id"
    )?;
    let tags = stmt
        .query_map([friend_id], |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
                like_times: row.get(4)?,
                priority: row.get(5)?,
                auto_like: row.get::<_, i32>(6)? != 0,
                auto_reply: row.get::<_, i32>(7)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

pub fn get_friend_count(conn: &Connection) -> Result<i64, AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM friends", [], |row| row.get(0))?;
    Ok(count)
}

pub fn get_all_friends_with_tags(conn: &Connection, date: &str) -> Result<Vec<FriendWithTags>, AppError> {
    // Step 1: 查询所有好友
    let mut friend_stmt = conn.prepare("SELECT user_id, nickname, remark FROM friends ORDER BY nickname")?;
    let friends: Vec<(i64, String, String)> = friend_stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Step 2: 批量查询所有 friend_tags + tags 关联
    let mut tag_stmt = conn.prepare(
        "SELECT ft.friend_id, t.id, t.name, t.color, t.is_system, t.like_times, t.priority, t.auto_like, t.auto_reply
         FROM friend_tags ft JOIN tags t ON ft.tag_id = t.id"
    )?;
    let all_tags: Vec<(i64, i64, String, String, bool, Option<i32>, String, bool, bool)> = tag_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get::<_, i32>(4)? != 0,
                row.get(5)?,
                row.get(6)?,
                row.get::<_, i32>(7)? != 0,
                row.get::<_, i32>(8)? != 0,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Step 3: 用 HashMap 按 friend_id 分组标签
    let mut tag_map: HashMap<i64, Vec<TagRow>> = HashMap::new();
    for (friend_id, id, name, color, is_system, like_times, priority, auto_like, auto_reply) in all_tags {
        tag_map.entry(friend_id).or_default().push(TagRow {
            id,
            name,
            color,
            is_system,
            like_times,
            priority,
            auto_like,
            auto_reply,
        });
    }

    // Step 4: 批量查询今日已赞好友
    let date_start = format!("{} 00:00:00", date);
    let date_end = format!("{} 23:59:59", date);
    let mut liked_stmt = conn.prepare(
        "SELECT DISTINCT user_id FROM like_history
         WHERE created_at >= ?1 AND created_at <= ?2 AND success = 1"
    )?;
    let liked_ids: HashSet<i64> = liked_stmt
        .query_map(params![date_start, date_end], |row| row.get(0))?
        .collect::<Result<HashSet<_>, _>>()?;

    // Step 5: 组装结果
    Ok(friends
        .into_iter()
        .map(|(uid, nick, rem)| FriendWithTags {
            user_id: uid,
            nickname: nick,
            remark: rem,
            tags: tag_map.remove(&uid).unwrap_or_default(),
            liked_today: liked_ids.contains(&uid),
        })
        .collect())
}

pub fn update_tag_strategy(
    conn: &Connection,
    id: i64,
    like_times: Option<i32>,
    priority: &str,
    auto_like: bool,
    auto_reply: bool,
) -> Result<TagRow, AppError> {
    if !["high", "medium", "low"].contains(&priority) {
        return Err(AppError::NapCat(format!("无效的优先级: {}", priority)));
    }

    if let Some(times) = like_times {
        if !(1..=20).contains(&times) {
            return Err(AppError::NapCat("点赞次数必须在 1-20 之间".to_string()));
        }
    }

    conn.execute(
        "UPDATE tags SET like_times = ?2, priority = ?3, auto_like = ?4, auto_reply = ?5 WHERE id = ?1",
        params![id, like_times, priority, auto_like as i32, auto_reply as i32],
    )?;

    conn.query_row(
        "SELECT id, name, color, is_system, like_times, priority, auto_like, auto_reply FROM tags WHERE id = ?1",
        [id],
        |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
                like_times: row.get(4)?,
                priority: row.get(5)?,
                auto_like: row.get::<_, i32>(6)? != 0,
                auto_reply: row.get::<_, i32>(7)? != 0,
            })
        },
    ).map_err(|e| AppError::Database(e))
}

pub fn get_friend_tags(conn: &Connection, user_id: i64) -> Result<Vec<TagRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.is_system, t.like_times, t.priority, t.auto_like, t.auto_reply
         FROM friend_tags ft JOIN tags t ON ft.tag_id = t.id
         WHERE ft.friend_id = ?1 ORDER BY t.id"
    )?;
    let tags = stmt
        .query_map([user_id], |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
                like_times: row.get(4)?,
                priority: row.get(5)?,
                auto_like: row.get::<_, i32>(6)? != 0,
                auto_reply: row.get::<_, i32>(7)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}
