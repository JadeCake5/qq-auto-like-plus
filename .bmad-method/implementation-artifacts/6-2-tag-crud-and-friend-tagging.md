# Story 6.2: 标签 CRUD 与好友标签管理

Status: Done

## Story

As a 用户,
I want 创建自定义标签并给好友打标签,
so that 可以对不同好友进行分组管理。

## Acceptance Criteria

1. **标签管理区域**：好友管理页面顶部显示标签管理区域，列出所有标签（彩色 badge）
2. **创建标签**：支持创建标签：输入名称、选择颜色（马卡龙色选择器）
3. **编辑标签**：支持编辑标签：修改名称和颜色
4. **删除标签**：支持删除标签：确认对话框后删除，关联的 friend_tags 记录级联删除
5. **预置标签**：预置标签（默认、重要、不赞）不可删除
6. **好友标签操作**：好友列表中每个好友可点击添加/移除标签（多选弹出面板）
7. **多标签支持**：一个好友可以同时属于多个标签
8. **数据持久化**：标签操作通过 Tauri commands 持久化到 tags 和 friend_tags 表
9. **操作反馈**：操作成功显示 toast 通知
10. **Tauri Commands**：提供 `create_tag`、`update_tag`、`delete_tag`、`set_friend_tags` 四个命令

## Tasks / Subtasks

- [x] Task 1: 扩展 db/models.rs — 标签 CRUD 函数 (AC: #2, #3, #4, #5, #8)
  - [x] 1.1 新增函数 `create_tag(conn, name, color) -> Result<TagRow, AppError>`
    - INSERT INTO tags (name, color) VALUES (?1, ?2) RETURNING id, name, color, is_system
    - name 唯一约束失败时返回友好错误
  - [x] 1.2 新增函数 `update_tag(conn, id, name, color) -> Result<TagRow, AppError>`
    - 先检查 is_system=1 则不允许修改 name（系统标签名称不可改）
    - UPDATE tags SET name=?2, color=?3 WHERE id=?1 AND is_system=0 OR (id=?1 AND is_system=1)
    - 系统标签允许改颜色但不改名称
  - [x] 1.3 新增函数 `delete_tag(conn, id) -> Result<(), AppError>`
    - 先检查 is_system=1 则拒绝删除
    - DELETE FROM tags WHERE id=?1 AND is_system=0
    - friend_tags 级联删除由 FK ON DELETE CASCADE 自动处理
  - [x] 1.4 新增函数 `set_friend_tags(conn, friend_id, tag_ids) -> Result<(), AppError>`
    - 事务内：先删除该好友所有 friend_tags，再批量插入新关联
    - DELETE FROM friend_tags WHERE friend_id=?1
    - INSERT INTO friend_tags (friend_id, tag_id) VALUES (?1, ?2) 批量

- [x] Task 2: 扩展 commands/friends.rs — 新增 4 个 Tauri 命令 (AC: #8, #10)
  - [x] 2.1 `create_tag(db, name, color) -> Result<TagRow, String>`
  - [x] 2.2 `update_tag(db, id, name, color) -> Result<TagRow, String>`
  - [x] 2.3 `delete_tag(db, id) -> Result<(), String>`
  - [x] 2.4 `set_friend_tags(db, friend_id, tag_ids) -> Result<Vec<TagRow>, String>`
    - 设置后重新查询该好友的标签并返回
  - [x] 2.5 在 lib.rs invoke_handler 注册 4 个新命令

- [x] Task 3: 扩展前端类型与 invoke wrapper (AC: #8)
  - [x] 3.1 在 `src/types/friends.ts` 新增：
    ```typescript
    export interface CreateTagParams {
      name: string;
      color: string;
    }
    export interface UpdateTagParams {
      id: number;
      name: string;
      color: string;
    }
    ```
  - [x] 3.2 在 `src/lib/tauri.ts` 新增 4 个 invoke wrapper：
    ```typescript
    export async function createTag(name: string, color: string): Promise<TagInfo>
    export async function updateTag(id: number, name: string, color: string): Promise<TagInfo>
    export async function deleteTag(id: number): Promise<void>
    export async function setFriendTags(friendId: number, tagIds: number[]): Promise<TagInfo[]>
    ```

- [x] Task 4: 扩展 useFriendsStore — 标签 CRUD actions (AC: #8)
  - [x] 4.1 新增 actions：
    ```typescript
    createTag: (name: string, color: string) => Promise<TagInfo | null>;
    updateTag: (id: number, name: string, color: string) => Promise<TagInfo | null>;
    deleteTag: (id: number) => Promise<boolean>;
    setFriendTags: (friendId: number, tagIds: number[]) => Promise<void>;
    ```
  - [x] 4.2 createTag：调用 invoke → 成功后追加到 tags 数组
  - [x] 4.3 updateTag：调用 invoke → 成功后更新 tags 数组中对应项 + 更新 friends 中引用的 tag 信息
  - [x] 4.4 deleteTag：调用 invoke → 成功后从 tags 中移除 + 从 friends 的 tags 数组中移除 + 清理 selectedTagIds
  - [x] 4.5 setFriendTags：调用 invoke → 成功后更新对应 friend 的 tags 字段

- [x] Task 5: 创建 TagManager 组件 (AC: #1, #2, #3, #4, #5, #9)
  - [x] 5.1 在 `src/components/friends/TagManager.tsx` 创建标签管理区域
  - [x] 5.2 显示所有标签列表（TagBadge 样式 + 编辑/删除按钮）
  - [x] 5.3 "新建标签" 按钮 → 弹出 TagEditDialog
  - [x] 5.4 编辑按钮 → 弹出 TagEditDialog（预填当前值）
  - [x] 5.5 删除按钮 → 确认对话框 → 调用 deleteTag
  - [x] 5.6 系统标签（is_system=true）隐藏删除按钮，编辑仅允许改颜色

- [x] Task 6: 创建 TagEditDialog 组件 (AC: #2, #3, #9)
  - [x] 6.1 在 `src/components/friends/TagEditDialog.tsx` 创建标签编辑弹窗
  - [x] 6.2 表单：标签名称 Input + 颜色选择器（8 色马卡龙预设）
  - [x] 6.3 马卡龙色预设：`#f2a7c3`(樱花粉)、`#a7c7f2`(天空蓝)、`#c3a7f2`(薰衣草紫)、`#a7f2d4`(薄荷绿)、`#f2cfa7`(蜜桃橙)、`#f28b8b`(珊瑚红)、`#9b95a8`(灰紫)、`#a7f2f0`(水蓝)
  - [x] 6.4 模式区分：新建模式（空表单）/ 编辑模式（预填 + 系统标签名称只读）
  - [x] 6.5 表单校验：名称非空、不重复（前端预校验）
  - [x] 6.6 保存成功后 toast 通知 + 关闭弹窗

- [x] Task 7: 创建 FriendTagPopover 组件 (AC: #6, #7, #9)
  - [x] 7.1 在 `src/components/friends/FriendTagPopover.tsx` 创建好友标签选择弹出面板
  - [x] 7.2 点击好友卡片的标签区域 → 弹出标签选择面板
  - [x] 7.3 面板列出所有标签 + checkbox 多选
  - [x] 7.4 当前好友已有的标签预勾选
  - [x] 7.5 勾选/取消后立即调用 setFriendTags 保存
  - [x] 7.6 成功后 toast 通知

- [x] Task 8: 修改 FriendCard — 集成标签操作入口 (AC: #6)
  - [x] 8.1 FriendCard 标签区域添加点击触发 FriendTagPopover
  - [x] 8.2 无标签时显示 "+标签" 文字按钮

- [x] Task 9: 修改 Friends.tsx — 集成 TagManager (AC: #1)
  - [x] 9.1 在工具栏和好友列表之间插入 TagManager 组件
  - [x] 9.2 TagManager 可折叠（默认展开，点击标题切换）

## Dev Notes

### 已有基础设施（直接复用！）

**数据库表已存在 — 不需要新建 migration：**
- `tags` 表（migration 007_tags.sql）：id, name(UNIQUE), color, is_system, created_at
- `friend_tags` 表（migration 007_tags.sql）：friend_id, tag_id（复合主键），FK CASCADE
- 预置种子数据：默认(#9b95a8, system)、重要(#f2a7c3, system)、不赞(#f28b8b, system)

**Rust 函数已存在 — 必须复用：**

| 模块 | 函数 | 用途 |
|------|------|------|
| `db/models.rs` | `get_all_tags(conn)` | 查询所有标签（已有） |
| `db/models.rs` | `get_default_tag_id(conn)` | 获取"默认"标签 ID（已有） |
| `db/models.rs` | `assign_default_tag_to_new_friends(conn, user_ids)` | 为新好友分配默认标签（已有） |
| `db/models.rs` | `get_all_friends_with_tags(conn, date)` | 查询好友+标签+今日状态（已有） |
| `db/models.rs` | `TagRow` | 标签行结构体（已有） |
| `db/models.rs` | `FriendWithTags` | 好友+标签结构体（已有） |
| `commands/friends.rs` | `get_friends`, `get_tags` | 已有的查询 commands |
| `engine/quota.rs` | `today()` | 获取今日日期（已有） |

**前端已存在 — 必须复用：**

| 文件 | 状态 |
|------|------|
| `src/pages/Friends.tsx` | 已完成 — 需在其中插入 TagManager |
| `src/components/friends/FriendCard.tsx` | 已完成 — 需添加标签操作入口 |
| `src/stores/useFriendsStore.ts` | 已完成 — 需扩展 CRUD actions |
| `src/types/friends.ts` | 已完成 — 需新增参数类型 |
| `src/lib/tauri.ts` | 已完成 — 需新增 4 个 invoke wrapper |
| `src/components/ui/dialog.tsx` | shadcn/ui Dialog 组件（已有） |
| `src/components/ui/input.tsx` | shadcn/ui Input 组件（已有） |
| `src/components/ui/button.tsx` | shadcn/ui Button 组件（已有） |

### 架构合规要点

**Rust 代码：**
- 所有对外结构体必须 `#[serde(rename_all = "camelCase")]`
- Tauri commands 返回 `Result<T, String>`，错误转换 `.map_err(|e| e.to_string())`
- 新增 CRUD 函数放在 `db/models.rs`（唯一数据库访问点）
- 命令函数放在 `commands/friends.rs`（已有文件，追加）
- 使用 `tracing::info!` / `tracing::error!`，禁止 `println!`
- 使用 `?` 操作符，禁止 `unwrap()` / `expect()` 在生产代码
- db lock 不跨 await — 所有新增 commands 是同步的（无需 async，因为只操作 SQLite）
- set_friend_tags 在事务中执行（先 DELETE 后 INSERT）

**前端代码：**
- 组件文件 PascalCase：`TagManager.tsx`、`TagEditDialog.tsx`、`FriendTagPopover.tsx`
- invoke wrapper 放 `src/lib/tauri.ts`
- 使用 shadcn/ui 基础组件（Dialog、Input、Button、Badge）
- 颜色使用 CSS 变量：`text-text-primary`、`bg-bg-card`、`bg-bg-elevated`
- toast 通知使用 `sonner` 的 `toast.success()` / `toast.error()`（已有）
- Store 中 try-catch 静默处理错误，UI 层做 toast 通知
- 弹窗使用 shadcn/ui Dialog（Radix UI 内置焦点陷阱 + ESC 关闭）

### 关键实现细节

**db/models.rs — create_tag 实现模式：**

```rust
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
    })
}
```

**db/models.rs — update_tag 实现（系统标签只改颜色）：**

```rust
pub fn update_tag(conn: &Connection, id: i64, name: &str, color: &str) -> Result<TagRow, AppError> {
    // 先查询标签是否存在及是否为系统标签
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

    Ok(TagRow {
        id,
        name: final_name.to_string(),
        color: color.to_string(),
        is_system,
    })
}
```

**db/models.rs — delete_tag 实现（系统标签拒绝）：**

```rust
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
    // friend_tags 级联删除由 FK ON DELETE CASCADE 自动处理
    Ok(())
}
```

**db/models.rs — set_friend_tags 实现（事务）：**

```rust
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

    // 返回该好友当前的标签列表
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.is_system
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}
```

**commands/friends.rs — 新增 commands 模式：**

```rust
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
```

**lib.rs invoke_handler 追加（追加在现有 friends 命令后）：**

```rust
commands::friends::create_tag,
commands::friends::update_tag,
commands::friends::delete_tag,
commands::friends::set_friend_tags,
```

### 前端组件实现指南

**TagManager 组件结构：**
```
┌──────────────────────────────────────────┐
│ 标签管理 ▼          [+ 新建标签] 按钮    │
├──────────────────────────────────────────┤
│ [默认] [重要] [不赞] [自定义1✎✗] [...]  │
└──────────────────────────────────────────┘
```
- 系统标签：只有 ✎（编辑颜色），无 ✗（删除）
- 自定义标签：✎（编辑）+ ✗（删除）
- 使用 badge 样式，背景为标签颜色，白色文字

**TagEditDialog 组件结构：**
```
┌──────────────────────────────────┐
│ 新建标签 / 编辑标签              │
├──────────────────────────────────┤
│ 标签名称: [________]             │
│                                  │
│ 选择颜色:                        │
│ ● ● ● ●                        │
│ ● ● ● ●                        │
├──────────────────────────────────┤
│              [取消]  [保存]      │
└──────────────────────────────────┘
```
- 颜色选择器：8 个圆形色块，选中时有 ring 高亮
- 系统标签编辑模式：名称 Input disabled

**FriendTagPopover 组件结构：**
```
┌──────────────────────┐
│ ☑ 默认               │
│ ☐ 重要               │
│ ☑ 自定义1            │
│ ☐ 不赞               │
└──────────────────────┘
```
- 弹出面板使用 Popover 或绝对定位 div
- checkbox 列表，勾选/取消即时保存
- 点击面板外部关闭

**马卡龙预设色板常量（放 TagEditDialog 或 constants）：**
```typescript
const PRESET_COLORS = [
  "#f2a7c3", // 樱花粉
  "#a7c7f2", // 天空蓝
  "#c3a7f2", // 薰衣草紫
  "#a7f2d4", // 薄荷绿
  "#f2cfa7", // 蜜桃橙
  "#f28b8b", // 珊瑚红
  "#9b95a8", // 灰紫
  "#a7f2f0", // 水蓝
];
```

### useFriendsStore 扩展模式

```typescript
// 新增 actions（追加到现有 store）
createTag: async (name, color) => {
  try {
    const tag = await invoke<TagInfo>("create_tag", { name, color });
    set((s) => ({ tags: [...s.tags, tag] }));
    return tag;
  } catch {
    return null;
  }
},
updateTag: async (id, name, color) => {
  try {
    const tag = await invoke<TagInfo>("update_tag", { id, name, color });
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
deleteTag: async (id) => {
  try {
    await invoke("delete_tag", { id });
    set((s) => ({
      tags: s.tags.filter((t) => t.id !== id),
      friends: s.friends.map((f) => ({
        ...f,
        tags: f.tags.filter((t) => t.id !== id),
      })),
      selectedTagIds: s.selectedTagIds.filter((i) => i !== id),
    }));
    return true;
  } catch {
    return false;
  }
},
setFriendTags: async (friendId, tagIds) => {
  try {
    const tags = await invoke<TagInfo[]>("set_friend_tags", { friendId, tagIds });
    set((s) => ({
      friends: s.friends.map((f) =>
        f.userId === friendId ? { ...f, tags } : f
      ),
    }));
  } catch {
    // 静默
  }
},
```

### Story 6.1 QA 发现的问题（注意！）

- **P2-F1**：`commands/friends.rs:36-43` 直接写 SQL（获取 existing_ids）违反架构边界 — 本 Story 不改（不在 scope 内），但新增的 commands 必须通过 `db/models.rs` 函数
- **P3-F1**：Store 直接 invoke vs tauri.ts wrapper 不一致 — 现有 store 用 `invoke` 直接调用，新增 actions 保持一致（直接 invoke），wrapper 供其他组件直接使用
- **P3-F2**：`Friends.tsx:14` 的 `const ref` 命名 — 不在本 Story scope 内

### 不要做的事情

- **不要新建 migration 文件** — tags 和 friend_tags 表已在 007_tags.sql 创建
- **不要修改 `db/migrations.rs`** — 不需要新 migration
- **不要修改 `onebot/client.rs`** — 标签管理不涉及 OneBot API
- **不要修改 `engine/` 下任何文件** — 标签策略是 Story 6.3 的内容
- **不要在这个 Story 实现标签策略（按标签配置点赞次数/优先级）** — 那是 Story 6.3
- **不要修改 `tray/mod.rs`** — 托盘不受标签管理影响
- **不要修改 `webhook/mod.rs`** — Webhook 与标签管理无关
- **不要创建 `friends/tags.rs` Rust 模块** — 标签 CRUD 查询放在 `db/models.rs`，commands 放在 `commands/friends.rs`，Story 6.3 的策略逻辑才需要 `friends/` 模块
- **不要添加新的 npm 依赖** — shadcn/ui Dialog + 基础 HTML 就能实现颜色选择器和弹出面板
- **不要修改 sync_friends 命令** — 同步逻辑不受标签 CRUD 影响
- **不要修改 `get_all_friends_with_tags` 函数** — 已有的查询已经包含标签信息

### Project Structure Notes

新增文件：
```
src/
├── components/
│   └── friends/
│       ├── TagManager.tsx              # NEW — 标签管理区域（列表 + 新建/编辑/删除）
│       ├── TagEditDialog.tsx           # NEW — 标签编辑弹窗（名称 + 颜色选择）
│       └── FriendTagPopover.tsx        # NEW — 好友标签选择弹出面板
```

修改文件：
```
src-tauri/src/db/models.rs              # MODIFY — 新增 create_tag, update_tag, delete_tag, set_friend_tags
src-tauri/src/commands/friends.rs       # MODIFY — 新增 4 个 Tauri commands
src-tauri/src/lib.rs                    # MODIFY — invoke_handler 注册 4 个新命令
src/types/friends.ts                    # MODIFY — 新增 CreateTagParams, UpdateTagParams 类型
src/lib/tauri.ts                        # MODIFY — 新增 4 个 invoke wrapper
src/stores/useFriendsStore.ts           # MODIFY — 新增 createTag, updateTag, deleteTag, setFriendTags actions
src/components/friends/FriendCard.tsx   # MODIFY — 添加标签操作入口（点击触发 FriendTagPopover）
src/pages/Friends.tsx                   # MODIFY — 插入 TagManager 组件
```

**路径与架构对齐验证：**
- `commands/friends.rs` 追加 CRUD commands — 与 architecture.md `commands/friends.rs # 好友管理：get_list, update_tags` 一致 ✅
- `db/models.rs` 新增 CRUD 函数 — 遵循唯一数据库访问点规则 ✅
- `TagManager.tsx` 放 `components/friends/` — 与 UX 设计 `components/friends/` 组织一致 ✅
- 无新 migration、无新 npm 依赖、无新 Tauri events ✅
- 标签 CRUD 全部同步操作（不需要 async command）— 除非需要 async，否则不用 ✅

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 6.2: 标签 CRUD 与好友标签管理]
- [Source: .bmad-method/planning-artifacts/epics.md#Epic 6: 好友管理与标签系统 — FR43, FR44, FR45, FR46]
- [Source: .bmad-method/planning-artifacts/architecture.md#项目结构 — commands/friends.rs, components/friends/TagManager.tsx, TagBadge]
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则 — commands/ 唯一前端入口, db/models.rs 唯一数据库访问]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — Rust snake_case, serde camelCase]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri IPC 命令模式 — invoke + Result<T, String>]
- [Source: .bmad-method/planning-artifacts/architecture.md#强制规则 7 条]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单 — 禁止 println/unwrap/直接 SQL]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#TagBadge 组件 — 马卡龙色标签]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#FriendCard 组件 — 点击展开详情]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#标签编辑 — 内联编辑 + 马卡龙色板 8 色]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#表单模式 — 实时验证 + 即时保存]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#按钮层级 — Primary/Secondary/Ghost/Danger]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#反馈模式 — Toast 通知]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#Dialog 出现 — scale 0.95→1, 200ms]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#色彩系统 — 马卡龙色板]
- [Source: .bmad-method/implementation-artifacts/6-1-friend-list-and-sync.md — 前置 Story 完整实现参考]
- [Source: .bmad-method/implementation-artifacts/6-1-friend-list-and-sync.md#QA Results — P2-F1 直接 SQL 问题]
- [Source: src-tauri/src/db/models.rs — TagRow, FriendWithTags, get_all_tags, get_all_friends_with_tags]
- [Source: src-tauri/src/commands/friends.rs — get_friends, sync_friends, get_tags]
- [Source: src-tauri/src/lib.rs — invoke_handler 注册模式, 行 274-276 现有 friends 命令位置]
- [Source: src-tauri/src/errors.rs — AppError 枚举定义]
- [Source: src-tauri/migrations/007_tags.sql — tags/friend_tags 表结构 + 种子数据]
- [Source: src/stores/useFriendsStore.ts — 现有 store 模式（invoke 直接调用, try-catch 静默）]
- [Source: src/types/friends.ts — TagInfo, FriendWithTags, SyncFriendsResult 类型定义]
- [Source: src/lib/tauri.ts — 现有 invoke wrapper 模式]
- [Source: src/pages/Friends.tsx — 现有页面结构（标题栏+工具栏+列表）]
- [Source: src/components/friends/FriendCard.tsx — 现有卡片组件结构]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

无错误，一次通过。

### Completion Notes List

- 所有 9 个 Task 及全部 Subtask 均已完成
- Rust: `cargo check` 通过，无新 warning
- TypeScript: `tsc --noEmit` 通过
- ESLint: `pnpm lint` 通过（修复了 TagEditDialog 中 useEffect+setState 问题，改为 key 重挂载模式；FriendTagPopover 改为 useMemo 驱动）
- 4 个 Rust CRUD 函数遵循 db/models.rs 唯一数据库访问点
- 4 个 Tauri commands 遵循 Result<T, String> + map_err 模式
- 前端使用 @base-ui/react Dialog（项目已有依赖，无新依赖）
- 所有 serde rename_all camelCase 合规
- 无 println/unwrap/expect 在新增代码中

### File List

**新增文件：**
- src/components/friends/TagManager.tsx
- src/components/friends/TagEditDialog.tsx
- src/components/friends/FriendTagPopover.tsx

**修改文件：**
- src-tauri/src/db/models.rs — 新增 create_tag, update_tag, delete_tag, set_friend_tags 函数
- src-tauri/src/commands/friends.rs — 新增 4 个 Tauri commands
- src-tauri/src/lib.rs — invoke_handler 注册 4 个新命令
- src/types/friends.ts — 新增 CreateTagParams, UpdateTagParams 类型
- src/lib/tauri.ts — 新增 createTag, updateTag, deleteTag, setFriendTags wrapper
- src/stores/useFriendsStore.ts — 新增 4 个 CRUD actions
- src/components/friends/FriendCard.tsx — 集成 FriendTagPopover + "+标签" 入口
- src/pages/Friends.tsx — 集成 TagManager 组件

## QA Results

**Reviewed:** 2026-03-14
**Reviewer:** Quinn (Test Architect)
**Gate Decision:** CONCERNS (non-blocking)

**AC Coverage:** 10/10 PASS (100%)
**Architecture Compliance:** 7/7 enforcement rules PASS, 0 anti-patterns

### Findings

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| P2-F1 | medium | `db/models.rs:308,333-334,339` | `AppError::NapCat` 被滥用于标签错误（"标签不存在"、"系统标签不可删除"），Display 输出 "NapCat 错误: ..." 前缀误导用户。建议新增 `TagError(String)` 变体。 |
| P2-F2 | medium | `TagManager.tsx:63-87` | `<button>` 内嵌 `<Button>`（嵌套交互元素）违反 HTML 规范，影响无障碍访问。建议外层改为 `<div>` 或重构布局。 |
| P3-F1 | low | `FriendTagPopover.tsx:36-46` | 快速连续切换标签时存在竞态条件：第二次 toggle 基于旧 `friendTags` 可能覆盖第一次变更。概率低，建议使用本地乐观状态。 |
| P3-F2 | low | `types/friends.ts:21-30` | `CreateTagParams` 和 `UpdateTagParams` 已定义但全项目无引用，属死代码。 |
| P4-I1 | info | `useFriendsStore.ts` | Store 直接 invoke 而非 tauri.ts wrapper — 与 6.1 模式一致，设计选择。 |
| P4-I2 | info | `TagManager.tsx:17,25` | 使用 `(typeof tags)[0]` 类型推断而非导入 TagInfo — 功能正确。 |

### Summary

实现质量整体良好。10/10 AC 全通过，后端 CRUD 函数正确遵循 db/models.rs 唯一数据库访问点规则（吸取 6.1 P2-F1 教训），serde camelCase 完整，无 println/unwrap。前端组件遵循项目模式。2 个 P2 关注点（错误类型语义 + 嵌套 button）不阻塞发布但建议修复。

### Change Log

- 2026-03-14: Story 6.2 实现完成，全部 9 Task 通过验证
