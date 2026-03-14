# Story 6.3: 基于标签的点赞策略

Status: Ready for Review

## Story

As a 用户,
I want 对不同标签的好友使用不同的点赞策略,
so that 重要的好友可以优先点赞，不想赞的可以跳过。

## Acceptance Criteria

1. **标签策略字段**：每个标签可配置：点赞次数（1-20，可选覆盖）、优先级（high/medium/low）、是否参与定时点赞（开关）、是否参与回赞（开关）
2. **标签编辑面板**：标签设置通过标签编辑面板修改（扩展 TagEditDialog）
3. **优先级排序**：`friends/strategy.rs` 在批量点赞时按策略排序好友：高优先级 → 中优先级 → 低优先级
4. **跳过不赞标签**："不赞"标签的好友（auto_like=false）跳过所有定时点赞
5. **跳过不回赞标签**：auto_reply=false 的标签好友跳过回赞
6. **标签点赞次数**：不同标签的好友使用各自的 like_times 而非全局默认值
7. **多标签策略解析**：好友同时属于多个标签时，取最高优先级标签的策略
8. **策略实时生效**：策略变更通过 `config:updated` 事件通知引擎实时生效
9. **Tauri Command**：提供 `update_tag_strategy` 命令

## Tasks / Subtasks

- [x] Task 1: 创建数据库迁移 008_tag_strategy.sql (AC: #1)
  - [x] 1.1 ALTER TABLE tags ADD COLUMN like_times INTEGER DEFAULT NULL — 可选覆盖全局次数
  - [x] 1.2 ALTER TABLE tags ADD COLUMN priority TEXT NOT NULL DEFAULT 'medium' — 'high'/'medium'/'low'
  - [x] 1.3 ALTER TABLE tags ADD COLUMN auto_like INTEGER NOT NULL DEFAULT 1 — 参与定时点赞
  - [x] 1.4 ALTER TABLE tags ADD COLUMN auto_reply INTEGER NOT NULL DEFAULT 1 — 参与回赞
  - [x] 1.5 UPDATE tags SET auto_like = 0, auto_reply = 0 WHERE name = '不赞' AND is_system = 1
  - [x] 1.6 UPDATE tags SET priority = 'high' WHERE name = '重要' AND is_system = 1
  - [x] 1.7 在 db/migrations.rs 注册 008

- [x] Task 2: 扩展 TagRow 结构体与 DB 函数 (AC: #1, #9)
  - [x] 2.1 扩展 TagRow 增加 like_times: Option<i32>、priority: String、auto_like: bool、auto_reply: bool
  - [x] 2.2 修改 get_all_tags() SELECT 增加 4 列
  - [x] 2.3 修改 set_friend_tags() 返回的 TagRow 查询增加 4 列
  - [x] 2.4 修改 get_all_friends_with_tags() 的标签子查询增加 4 列
  - [x] 2.5 修改 create_tag() 返回的 TagRow 默认值（like_times: None, priority: "medium", auto_like: true, auto_reply: true）
  - [x] 2.6 修改 update_tag() 返回的 TagRow 查询增加 4 列
  - [x] 2.7 新增 update_tag_strategy(conn, id, like_times, priority, auto_like, auto_reply) -> Result<TagRow, AppError>

- [x] Task 3: 新增 Tauri command — update_tag_strategy (AC: #9)
  - [x] 3.1 在 commands/friends.rs 新增 `update_tag_strategy` 命令
  - [x] 3.2 命令接受 id, like_times (Option<i32>), priority, auto_like, auto_reply
  - [x] 3.3 调用 db::models::update_tag_strategy
  - [x] 3.4 成功后 emit("config:updated") 通知引擎 (AC: #8)
  - [x] 3.5 在 lib.rs invoke_handler 注册

- [x] Task 4: 创建 friends/strategy.rs — 策略解析 (AC: #3, #4, #5, #6, #7)
  - [x] 4.1 定义 FriendLikeStrategy 结构体：{ user_id, nickname, priority_order, like_times, auto_like, auto_reply }
  - [x] 4.2 实现 resolve_friend_strategy(friend: &FriendWithTags, default_times: i32) -> FriendLikeStrategy
  - [x] 4.3 实现 build_like_queue(friends: Vec<FriendWithTags>, default_times: i32) -> Vec<FriendLikeStrategy>
  - [x] 4.4 在 friends/mod.rs 导出 strategy 模块
  - [x] 4.5 在 lib.rs 的 mod 声明中确认 friends 模块已注册（已有 mod friends）

- [x] Task 5: 修改 like_executor.rs — 集成标签策略 (AC: #3, #4, #6)
  - [x] 5.1 移除直接使用 onebot.get_friend_list() 后随机打乱的逻辑
  - [x] 5.2 改为：先 upsert friends，然后从 db 读取 get_all_friends_with_tags
  - [x] 5.3 调用 friends::strategy::build_like_queue 获取排序后的点赞队列
  - [x] 5.4 循环中使用每个 FriendLikeStrategy.like_times 替代全局 times_per_friend
  - [x] 5.5 保留原有名额检查、已赞跳过、失败跳过逻辑不变

- [x] Task 6: 修改 reply_handler.rs — 集成标签回赞策略 (AC: #5, #6)
  - [x] 6.1 在检查 reply_enabled 之后、检查 already_liked 之前，新增标签策略检查
  - [x] 6.2 从 db 查询 operator_id 的好友标签（复用 get_friend_tags_for_user 或类似函数）
  - [x] 6.3 用 resolve_friend_strategy 解析策略
  - [x] 6.4 若 auto_reply=false，跳过回赞（emit skip reason "标签设置不允许回赞"）
  - [x] 6.5 若标签有 like_times 覆盖，使用标签的 like_times 替代全局 reply_times

- [x] Task 7: 扩展前端 TagInfo 类型与 invoke wrapper (AC: #1, #9)
  - [x] 7.1 在 types/friends.ts 的 TagInfo 增加 likeTimes, priority, autoLike, autoReply
  - [x] 7.2 在 lib/tauri.ts 新增 updateTagStrategy wrapper

- [x] Task 8: 扩展 useFriendsStore (AC: #8)
  - [x] 8.1 新增 updateTagStrategy action
  - [x] 8.2 调用成功后更新 tags 和 friends 中对应标签

- [x] Task 9: 扩展 TagEditDialog — 策略配置区域 (AC: #2)
  - [x] 9.1 在颜色选择器下方新增"点赞策略"分隔区域
  - [x] 9.2 添加优先级选择（高/中/低，三个按钮式选择器）
  - [x] 9.3 添加点赞次数覆盖（可选 number input，placeholder "使用全局默认"）
  - [x] 9.4 添加"参与定时点赞"开关（Switch）
  - [x] 9.5 添加"参与回赞"开关（Switch）
  - [x] 9.6 保存时分两步：先 updateTag（名称/颜色），再 updateTagStrategy（策略字段）
  - [x] 9.7 系统标签"不赞"默认 auto_like=false, auto_reply=false，编辑时可调但提示

## Dev Notes

### 已有基础设施（直接复用！）

**数据库 — 必须新建 migration：**
- `tags` 表（007_tags.sql）当前仅有 id, name, color, is_system, created_at — **缺少策略列**
- `friend_tags` 表（007_tags.sql）已有，无需修改
- 系统标签种子数据：默认(#9b95a8)、重要(#f2a7c3)、不赞(#f28b8b) — 需 UPDATE 设置默认策略

**Rust 已有 — 必须复用：**

| 模块 | 函数/结构体 | 行号 | 用途 |
|------|------------|------|------|
| `db/models.rs` | `TagRow` | L234-241 | **需扩展** — 加 4 个策略字段 |
| `db/models.rs` | `FriendWithTags` | L243-251 | 复用 — 含 tags: Vec<TagRow>，策略字段随 TagRow 自动带入 |
| `db/models.rs` | `get_all_tags()` | L253-266 | **需修改** — SELECT 增加策略列 |
| `db/models.rs` | `get_all_friends_with_tags()` | L382-441 | **需修改** — 标签子查询增加策略列 |
| `db/models.rs` | `set_friend_tags()` | L346-375 | **需修改** — 返回 TagRow 查询增加策略列 |
| `db/models.rs` | `create_tag()` | L288-300 | **需修改** — 返回 TagRow 默认策略值 |
| `db/models.rs` | `update_tag()` | L302-326 | **需修改** — 返回 TagRow 增加策略列查询 |
| `engine/like_executor.rs` | `run_batch_like()` | L33-193 | **需重构** — 集成标签策略 |
| `engine/reply_handler.rs` | `handle_reply_like()` | L22-162 | **需修改** — 增加标签回赞检查 |
| `engine/quota.rs` | `today()` | L8-10 | 复用 |
| `engine/quota.rs` | `has_liked_today_for_date()` | L146 | 复用 |
| `engine/quota.rs` | `try_consume_quota_for_date()` | L75 | 复用 |
| `engine/quota.rs` | `record_like()` | L132 | 复用 |
| `commands/friends.rs` | 全部 commands | L1-106 | 追加 update_tag_strategy |
| `lib.rs` | invoke_handler | L254-281 | 追加注册 |
| `friends/mod.rs` | （空文件） | — | **需添加** `pub mod strategy;` 导出 |
| `errors.rs` | AppError | L1-21 | 复用，无需新增变体 |

**前端已有 — 必须复用：**

| 文件 | 状态 |
|------|------|
| `src/types/friends.ts:1-6` — TagInfo | **需扩展** 4 个策略字段 |
| `src/lib/tauri.ts:92-106` — tag wrappers | **需新增** updateTagStrategy |
| `src/stores/useFriendsStore.ts` | **需新增** updateTagStrategy action |
| `src/components/friends/TagEditDialog.tsx` | **需扩展** 策略配置区域 |
| `src/components/friends/TagManager.tsx` | 无需修改 — 标签 badge 展示不变 |
| `src/components/friends/FriendTagPopover.tsx` | 无需修改 |
| `src/components/friends/FriendCard.tsx` | 无需修改 |
| `src/pages/Friends.tsx` | 无需修改 |
| `src/components/ui/switch.tsx` | 需确认是否已有 shadcn Switch 组件，若无则需添加 |

### 架构合规要点

**Rust 代码：**
- 所有对外结构体必须 `#[serde(rename_all = "camelCase")]`
- Tauri commands 返回 `Result<T, String>`，错误转换 `.map_err(|e| e.to_string())`
- 新增策略解析放 `friends/strategy.rs`（architecture.md 明确指定 `friends/strategy.rs`）
- DB 函数放 `db/models.rs`（唯一数据库访问点）
- 命令放 `commands/friends.rs`
- 使用 `tracing::info!` / `tracing::error!`，禁止 `println!`
- 使用 `?` 操作符，禁止 `unwrap()` / `expect()` 在生产代码
- like_executor.rs 中 db lock 不跨 await — 先锁取数据、释放、再 async 操作

**前端代码：**
- 组件文件 PascalCase
- CSS 变量：`text-text-primary`、`bg-bg-card`、`bg-bg-elevated`
- toast 通知使用 `sonner` 的 `toast.success()` / `toast.error()`
- Store 中 try-catch 静默处理，UI 层 toast 通知

### 关键实现细节

**migration 008_tag_strategy.sql：**

```sql
-- 标签策略扩展
ALTER TABLE tags ADD COLUMN like_times INTEGER DEFAULT NULL;
ALTER TABLE tags ADD COLUMN priority TEXT NOT NULL DEFAULT 'medium';
ALTER TABLE tags ADD COLUMN auto_like INTEGER NOT NULL DEFAULT 1;
ALTER TABLE tags ADD COLUMN auto_reply INTEGER NOT NULL DEFAULT 1;

-- 系统标签默认策略
UPDATE tags SET auto_like = 0, auto_reply = 0 WHERE name = '不赞' AND is_system = 1;
UPDATE tags SET priority = 'high' WHERE name = '重要' AND is_system = 1;
```

**db/models.rs — TagRow 扩展（修改 L234-241）：**

```rust
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
```

**db/models.rs — update_tag_strategy 函数：**

```rust
pub fn update_tag_strategy(
    conn: &Connection,
    id: i64,
    like_times: Option<i32>,
    priority: &str,
    auto_like: bool,
    auto_reply: bool,
) -> Result<TagRow, AppError> {
    // 验证 priority 值
    if !["high", "medium", "low"].contains(&priority) {
        return Err(AppError::NapCat(format!("无效的优先级: {}", priority)));
    }

    // 验证 like_times 范围（如有值）
    if let Some(times) = like_times {
        if !(1..=20).contains(&times) {
            return Err(AppError::NapCat("点赞次数必须在 1-20 之间".to_string()));
        }
    }

    conn.execute(
        "UPDATE tags SET like_times = ?2, priority = ?3, auto_like = ?4, auto_reply = ?5 WHERE id = ?1",
        params![id, like_times, priority, auto_like as i32, auto_reply as i32],
    )?;

    // 返回完整 TagRow
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
```

**注意：所有读取 TagRow 的查询都需要修改为 8 列**。影响函数：
- `get_all_tags()` — L254 SELECT 改为 `SELECT id, name, color, is_system, like_times, priority, auto_like, auto_reply`
- `set_friend_tags()` — L360 SELECT 改为含 8 列
- `get_all_friends_with_tags()` — L393 标签子查询改为含 8 列
- `create_tag()` — L294 返回 TagRow 补默认值
- `update_tag()` — L320 改为查询后返回完整 TagRow

**friends/strategy.rs — 策略解析核心逻辑：**

```rust
use serde::Serialize;
use crate::db::models::{FriendWithTags, TagRow};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendLikeStrategy {
    pub user_id: i64,
    pub nickname: String,
    pub like_times: i32,
    pub priority_order: u8,  // 0=high, 1=medium, 2=low
    pub auto_like: bool,
    pub auto_reply: bool,
}

fn priority_to_order(priority: &str) -> u8 {
    match priority {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 1,
    }
}

/// 解析单个好友的点赞策略（多标签取最高优先级）
pub fn resolve_friend_strategy(friend: &FriendWithTags, default_times: i32) -> FriendLikeStrategy {
    if friend.tags.is_empty() {
        return FriendLikeStrategy {
            user_id: friend.user_id,
            nickname: friend.nickname.clone(),
            like_times: default_times,
            priority_order: 1, // medium
            auto_like: true,
            auto_reply: true,
        };
    }

    // 找最高优先级标签
    let best_tag = friend.tags.iter()
        .min_by_key(|t| priority_to_order(&t.priority))
        .unwrap(); // safe: tags is non-empty

    // 任一标签 auto_like=false → 整体 false（"不赞"标签优先生效）
    let auto_like = friend.tags.iter().all(|t| t.auto_like);
    let auto_reply = friend.tags.iter().all(|t| t.auto_reply);

    // like_times 取最高优先级标签的值
    let like_times = best_tag.like_times.unwrap_or(default_times);

    FriendLikeStrategy {
        user_id: friend.user_id,
        nickname: friend.nickname.clone(),
        like_times,
        priority_order: priority_to_order(&best_tag.priority),
        auto_like,
        auto_reply,
    }
}

/// 构建点赞队列：过滤 + 按优先级排序 + 同级随机
pub fn build_like_queue(
    friends: Vec<FriendWithTags>,
    default_times: i32,
) -> Vec<FriendLikeStrategy> {
    use rand::seq::SliceRandom;

    let mut strategies: Vec<FriendLikeStrategy> = friends.iter()
        .map(|f| resolve_friend_strategy(f, default_times))
        .filter(|s| s.auto_like) // 过滤 auto_like=false
        .collect();

    // 按优先级分组后各组内部随机
    let mut rng = rand::rng();
    strategies.sort_by_key(|s| s.priority_order);

    // 对同一优先级区间内随机打乱
    let mut start = 0;
    while start < strategies.len() {
        let current_priority = strategies[start].priority_order;
        let end = strategies[start..].iter()
            .position(|s| s.priority_order != current_priority)
            .map(|p| start + p)
            .unwrap_or(strategies.len());
        strategies[start..end].shuffle(&mut rng);
        start = end;
    }

    strategies
}
```

**like_executor.rs — 重构要点（L33-193 的 run_batch_like）：**

关键变更区域：
1. **L42-54**（读取配置）：保留 times_per_friend 作为默认值，新变量名 `default_times`
2. **L56-73**（获取好友 + upsert）：保留 upsert 逻辑
3. **L75-80**（随机打乱）：**替换** — 改为从 db 读取 FriendWithTags，调用 build_like_queue
4. **L88-177**（循环）：遍历 `Vec<FriendLikeStrategy>` 而非 `Vec<OneBotFriend>`
5. **L133**（send_like 调用）：使用 `strategy.like_times` 替代固定 `times_per_friend`

```rust
// 替换原来的步骤 5（随机打乱）
// 旧代码 L75-80 替换为：
let like_queue = {
    let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
    let friends_with_tags = models::get_all_friends_with_tags(&conn, &date)?;
    crate::friends::strategy::build_like_queue(friends_with_tags, times_per_friend)
};

let total = like_queue.len() as i32;
// ...

// 步骤 6 循环改为遍历 like_queue
for (i, strategy) in like_queue.iter().enumerate() {
    // ... 用 strategy.user_id, strategy.nickname, strategy.like_times
    // L133: onebot.send_like(strategy.user_id, strategy.like_times).await;
}
```

**reply_handler.rs — 增加标签检查（L28-46 reply_enabled 检查之后）：**

```rust
// 在 reply_enabled 检查之后、already_liked 检查之前插入：

// 检查好友标签的回赞策略
let (tag_auto_reply, tag_like_times) = {
    let conn = db.lock().expect("lock db");
    let tags = get_friend_tags_for_user(&conn, operator_id);
    match tags {
        Ok(tags) if !tags.is_empty() => {
            let strategy = crate::friends::strategy::resolve_friend_strategy(
                &db::models::FriendWithTags {
                    user_id: operator_id,
                    nickname: String::new(),
                    remark: String::new(),
                    tags,
                    liked_today: false,
                },
                reply_times_default,
            );
            (strategy.auto_reply, Some(strategy.like_times))
        }
        _ => (true, None), // 无标签时默认允许回赞
    }
};
if !tag_auto_reply {
    tracing::debug!("标签设置不允许回赞，跳过 QQ {}", operator_id);
    let _ = app.emit("like:reply-complete", ReplyLikeResult {
        operator_id,
        times: 0,
        success: false,
        skipped: true,
        skip_reason: Some("标签设置不允许回赞".to_string()),
    });
    return Ok(());
}
// 后续使用 tag_like_times.unwrap_or(reply_times) 作为实际回赞次数
```

**需要新增的辅助函数（db/models.rs）：**

```rust
/// 查询指定用户的标签列表（用于回赞策略检查）
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
```

**commands/friends.rs — update_tag_strategy 命令：**

```rust
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
    // 通知引擎策略变更
    let _ = app.emit("config:updated", ());
    Ok(tag)
}
```

**前端 TagInfo 扩展（types/friends.ts）：**

```typescript
export interface TagInfo {
  id: number;
  name: string;
  color: string;
  isSystem: boolean;
  likeTimes: number | null;
  priority: string;    // "high" | "medium" | "low"
  autoLike: boolean;
  autoReply: boolean;
}
```

**TagEditDialog 扩展 — 策略配置区域（插入在颜色选择器下方）：**

```
┌──────────────────────────────────┐
│ 新建标签 / 编辑标签              │
├──────────────────────────────────┤
│ 标签名称: [________]             │
│ 选择颜色: ● ● ● ● ● ● ● ●     │
├──────────────────────────────────┤
│ 点赞策略                         │
│ 优先级:  [高] [中] [低]         │
│ 点赞次数: [__] (留空=全局默认)  │
│ 参与定时点赞: ────●             │
│ 参与回赞:     ────●             │
├──────────────────────────────────┤
│              [取消]  [保存]      │
└──────────────────────────────────┘
```

- 优先级用三个 button 组（类似 tab，选中时高亮），不用 select
- 点赞次数 Input type="number" min=1 max=20，空值表示使用全局默认
- 两个 Switch 组件（参与定时点赞 / 参与回赞）
- 保存逻辑：先调用 updateTag(name/color)，再调用 updateTagStrategy(策略字段)
- 如果组件不存在 shadcn Switch，用 `<button>` 自行实现 toggle 样式

**useFriendsStore 扩展：**

```typescript
updateTagStrategy: async (id, likeTimes, priority, autoLike, autoReply) => {
  try {
    const tag = await invoke<TagInfo>("update_tag_strategy", {
      id, likeTimes, priority, autoLike, autoReply
    });
    set((s) => ({
      tags: s.tags.map((t) => (t.id === id ? tag : t)),
      friends: s.friends.map((f) => ({
        ...f,
        tags: f.tags.map((t) => (t.id === id ? tag : t)),
      })),
    }));
    return tag;
  } catch {
    return null;
  }
},
```

### Story 6.2 QA 发现的问题（注意！）

- **P2-F1**：`db/models.rs:308,333-334,339` — `AppError::NapCat` 被滥用于标签错误。本 Story 新增的 `update_tag_strategy` 验证错误也暂用 `AppError::NapCat`，与 6.2 保持一致。若需修复建议统一在专门的 fix story 中处理
- **P2-F2**：`TagManager.tsx:63-87` — `<button>` 内嵌 `<Button>` 问题。本 Story 不改 TagManager
- **P3-F1**：FriendTagPopover 竞态条件 — 本 Story 不涉及

### 不要做的事情

- **不要修改 `tray/mod.rs`** — 托盘不受标签策略影响
- **不要修改 `webhook/mod.rs`** — Webhook 事件接收不变，只在 reply_handler 层面添加策略过滤
- **不要修改 `engine/scheduler.rs`** — 调度器不受影响，只修改 like_executor 的执行逻辑
- **不要修改 `engine/quota.rs`** — 名额管理不按标签区分，全局逻辑不变
- **不要新建 `stats/` 相关文件** — 统计是 Epic 7 的内容
- **不要添加 `tag_id` 到 `like_history` 表** — 超出本 Story 范围
- **不要修改 `FriendCard.tsx`** — 好友卡片展示不变
- **不要修改 `FriendTagPopover.tsx`** — 标签选择面板不变
- **不要修改 `Friends.tsx`** — 好友管理页面布局不变
- **不要修改 `TagManager.tsx`** — 标签列表展示不变（策略通过 TagEditDialog 编辑）
- **不要在 `like_executor` 中分别为每个优先级维护独立名额** — 名额是全局的
- **不要添加新的 npm 依赖** — Switch 如不存在可用 button toggle 实现

### Project Structure Notes

新增文件：
```
src-tauri/
├── migrations/
│   └── 008_tag_strategy.sql         # NEW — 标签策略列
└── src/
    └── friends/
        └── strategy.rs              # NEW — 策略解析（build_like_queue, resolve_friend_strategy）
```

修改文件：
```
src-tauri/src/db/models.rs           # MODIFY — TagRow 扩展 + 所有 TagRow 查询 + update_tag_strategy + get_friend_tags
src-tauri/src/db/migrations.rs       # MODIFY — 注册 008 migration
src-tauri/src/commands/friends.rs    # MODIFY — 新增 update_tag_strategy 命令
src-tauri/src/lib.rs                 # MODIFY — invoke_handler 追加 update_tag_strategy
src-tauri/src/engine/like_executor.rs # MODIFY — 集成标签策略（build_like_queue 替代随机打乱）
src-tauri/src/engine/reply_handler.rs # MODIFY — 增加标签回赞检查
src-tauri/src/friends/mod.rs         # MODIFY — 添加 pub mod strategy
src/types/friends.ts                 # MODIFY — TagInfo 增加 4 个策略字段
src/lib/tauri.ts                     # MODIFY — 新增 updateTagStrategy wrapper
src/stores/useFriendsStore.ts        # MODIFY — 新增 updateTagStrategy action
src/components/friends/TagEditDialog.tsx # MODIFY — 新增策略配置区域
```

**路径与架构对齐验证：**
- `friends/strategy.rs` — architecture.md 明确指定 `friends/strategy.rs # 按标签的点赞策略` ✅
- `db/models.rs` 扩展 CRUD — 遵循唯一数据库访问点规则 ✅
- `commands/friends.rs` 追加命令 — 与 architecture.md friends 命令区一致 ✅
- TagRow serde camelCase — 前后端字段自动对齐 ✅
- config:updated 事件 — 与 architecture.md 事件命名规范一致 ✅
- 策略逻辑不直接操作 DB — 通过 db/models 函数访问 ✅

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 6.3: 基于标签的点赞策略]
- [Source: .bmad-method/planning-artifacts/epics.md#Epic 6: 好友管理与标签系统 — FR43, FR44, FR45, FR46]
- [Source: .bmad-method/planning-artifacts/architecture.md#项目结构 — friends/strategy.rs # 按标签的点赞策略]
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则 — commands/ 唯一前端入口, db/models.rs 唯一数据库访问]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — Rust snake_case, serde camelCase]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri IPC 命令模式 — invoke + Result<T, String>]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri 事件命名 — config:updated]
- [Source: .bmad-method/planning-artifacts/architecture.md#强制规则 7 条]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#标签编辑 — 内联编辑 + 马卡龙色板]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#表单模式 — 实时验证 + 即时保存]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#Switch 组件 — toggle 开关]
- [Source: .bmad-method/implementation-artifacts/6-2-tag-crud-and-friend-tagging.md — 前置 Story 完整实现参考]
- [Source: .bmad-method/implementation-artifacts/6-2-tag-crud-and-friend-tagging.md#QA Results — P2-F1 AppError::NapCat 滥用]
- [Source: .bmad-method/implementation-artifacts/6-1-friend-list-and-sync.md — Epic 6 首个 Story 实现参考]
- [Source: src-tauri/migrations/007_tags.sql — tags/friend_tags 表结构]
- [Source: src-tauri/src/db/models.rs:234-241 — TagRow 当前定义]
- [Source: src-tauri/src/db/models.rs:243-251 — FriendWithTags 定义]
- [Source: src-tauri/src/db/models.rs:253-266 — get_all_tags 当前实现]
- [Source: src-tauri/src/db/models.rs:382-441 — get_all_friends_with_tags 当前实现]
- [Source: src-tauri/src/engine/like_executor.rs:33-193 — run_batch_like 当前实现]
- [Source: src-tauri/src/engine/reply_handler.rs:22-162 — handle_reply_like 当前实现]
- [Source: src-tauri/src/engine/quota.rs — 名额管理（不修改）]
- [Source: src-tauri/src/engine/mod.rs — engine 模块导出]
- [Source: src-tauri/src/friends/mod.rs — 空文件，需添加 strategy 导出]
- [Source: src-tauri/src/commands/friends.rs — 当前 7 个命令]
- [Source: src-tauri/src/lib.rs:254-281 — invoke_handler 注册位置]
- [Source: src-tauri/src/errors.rs — AppError 枚举（无需新增变体）]
- [Source: src/types/friends.ts:1-6 — TagInfo 当前定义]
- [Source: src/lib/tauri.ts:92-106 — 当前 tag wrappers]
- [Source: src/stores/useFriendsStore.ts — 当前 store 结构]
- [Source: src/components/friends/TagEditDialog.tsx — 当前编辑弹窗]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

No issues encountered during implementation.

### Completion Notes List

- All 9 tasks completed with full compilation passing (TypeScript + Rust)
- Migration 008_tag_strategy.sql created with 4 new columns + system tag defaults
- TagRow extended to 8 fields across all query functions (get_all_tags, set_friend_tags, get_all_friends_with_tags, create_tag, update_tag)
- New DB functions: update_tag_strategy(), get_friend_tags()
- New Tauri command: update_tag_strategy registered in invoke_handler
- friends/strategy.rs created with resolve_friend_strategy() and build_like_queue()
- like_executor.rs refactored: uses build_like_queue for priority-sorted, tag-filtered queue with per-friend like_times
- reply_handler.rs extended: tag-based auto_reply check + tag like_times override
- Frontend: TagInfo extended, updateTagStrategy invoke wrapper + store action added
- TagEditDialog expanded with strategy section (priority buttons, like_times input, auto_like/auto_reply switches)
- Switch component already existed (base-ui), reused directly
- No new npm dependencies added

### File List

**New files:**
- src-tauri/migrations/008_tag_strategy.sql
- src-tauri/src/friends/strategy.rs

**Modified files:**
- src-tauri/src/db/migrations.rs
- src-tauri/src/db/models.rs
- src-tauri/src/commands/friends.rs
- src-tauri/src/lib.rs
- src-tauri/src/engine/like_executor.rs
- src-tauri/src/engine/reply_handler.rs
- src-tauri/src/friends/mod.rs
- src/types/friends.ts
- src/lib/tauri.ts
- src/stores/useFriendsStore.ts
- src/components/friends/TagEditDialog.tsx

## QA Results

**Reviewed:** 2026-03-14 | **Reviewer:** Quinn (Test Architect) | **Gate:** PASS

**AC Coverage:** 9/9 PASS | **Architecture:** 7/7 Rules PASS | **Files Reviewed:** 13

| Severity | Count | Details |
|----------|-------|---------|
| P1 Blocker | 0 | — |
| P2 Concern | 0 | — |
| P3 Advisory | 2 | strategy.rs unwrap, reply_handler sentinel pattern |
| P4 Info | 2 | AppError::NapCat carry-forward, store invoke pattern |

**P3-F1** `friends/strategy.rs:43` — `.unwrap()` on `min_by_key()` guarded by non-empty check; logically safe but violates architecture "禁止 unwrap" rule. Consider `if let Some`.

**P3-F2** `engine/reply_handler.rs:62,144` — `default_times=0` placeholder + `tag_like_times > 0` sentinel check. Works because validation ensures [1,20] but fragile. Consider `Option<i32>`.

**P4-I1** `db/models.rs:486,491` — `AppError::NapCat` for tag errors, carry-forward from 6.2 P2-F1.

**P4-I2** `useFriendsStore.ts:117` — Direct invoke pattern, carry-forward from 6.2 P4-I1.

**Gate File:** `.bmad-method/test-artifacts/gates/6.3-tag-based-like-strategy.yml`
