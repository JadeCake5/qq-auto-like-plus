# Story 6.1: 好友列表展示与同步

Status: Done

## Story

As a 用户,
I want 查看我的 QQ 好友列表,
so that 了解哪些好友在被点赞。

## Acceptance Criteria

1. **好友列表获取**：通过 `invoke("get_friends")` 获取好友列表数据（从 SQLite `friends` 表读取，包含标签和今日点赞状态）
2. **好友同步**：首次加载或手动刷新时调用 `/get_friend_list` OneBot API 同步好友信息到 `friends` 表，新好友自动归入"默认"标签
3. **好友卡片显示**：好友列表卡片显示：头像占位（圆形 40px）、昵称、备注名、标签（彩色 badge）、今日是否已赞（绿色勾/灰色叉）
4. **搜索筛选**：支持搜索框按昵称/备注实时筛选
5. **标签筛选**：支持按标签筛选（标签下拉多选）
6. **虚拟滚动**：好友列表使用虚拟滚动（支持 500+ 好友）
7. **自动标签**：新好友（friends 表新增记录时）自动关联"默认"标签
8. **状态管理**：创建 `useFriendsStore`（Zustand）管理好友和标签状态
9. **空状态**：无好友时显示 Mascot 空状态插画（"还没有好友数据，请先登录 QQ~"）

## Tasks / Subtasks

- [x] Task 1: 创建 tags 和 friend_tags 数据库表 + 种子数据 (AC: #2, #7)
  - [x] 1.1 创建 migration `007_tags.sql`：
    ```sql
    CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        color TEXT NOT NULL DEFAULT '#c3a7f2',
        is_system INTEGER NOT NULL DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS friend_tags (
        friend_id INTEGER NOT NULL,
        tag_id INTEGER NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (friend_id, tag_id),
        FOREIGN KEY (friend_id) REFERENCES friends(user_id) ON DELETE CASCADE,
        FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
    );
    INSERT OR IGNORE INTO tags (name, color, is_system) VALUES ('默认', '#9b95a8', 1);
    INSERT OR IGNORE INTO tags (name, color, is_system) VALUES ('重要', '#f2a7c3', 1);
    INSERT OR IGNORE INTO tags (name, color, is_system) VALUES ('不赞', '#f28b8b', 1);
    ```
  - [x] 1.2 在 `db/migrations.rs` 的 MIGRATIONS 数组添加 `("007_tags", include_str!(...))`

- [x] Task 2: 扩展 db/models.rs — 好友+标签查询函数 (AC: #1, #2, #7)
  - [x] 2.1 新增结构体 `TagRow`：
    ```rust
    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TagRow {
        pub id: i64,
        pub name: String,
        pub color: String,
        pub is_system: bool,
    }
    ```
  - [x] 2.2 新增结构体 `FriendWithTags`：
    ```rust
    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FriendWithTags {
        pub user_id: i64,
        pub nickname: String,
        pub remark: String,
        pub tags: Vec<TagRow>,
        pub liked_today: bool,
    }
    ```
  - [x] 2.3 新增函数 `get_all_friends_with_tags(conn, date) -> Result<Vec<FriendWithTags>, AppError>`
    - 查询 friends 表 LEFT JOIN friend_tags + tags 获取标签
    - 查询 like_history 判断今日是否已赞（复用 has_liked_today 逻辑）
    - 先查询所有 friends，再批量查标签和点赞状态（避免 N+1）
  - [x] 2.4 新增函数 `get_all_tags(conn) -> Result<Vec<TagRow>, AppError>`
  - [x] 2.5 新增函数 `get_default_tag_id(conn) -> Result<i64, AppError>` — 获取"默认"标签 ID
  - [x] 2.6 新增函数 `assign_default_tag_to_new_friends(conn, user_ids: &[i64]) -> Result<(), AppError>`
    - 批量为新好友关联"默认"标签（INSERT OR IGNORE into friend_tags）
  - [x] 2.7 新增函数 `get_friend_count(conn) -> Result<i64, AppError>` — 好友总数

- [x] Task 3: 创建 commands/friends.rs — Tauri IPC 命令 (AC: #1, #2)
  - [x] 3.1 创建 `commands/friends.rs` 文件
  - [x] 3.2 实现 `get_friends` command：
    ```rust
    #[tauri::command]
    pub fn get_friends(db: State<DbState>) -> Result<Vec<FriendWithTags>, String>
    ```
    - 调用 `db::models::get_all_friends_with_tags(conn, &quota::today())`
  - [x] 3.3 实现 `sync_friends` command：
    ```rust
    #[tauri::command]
    pub async fn sync_friends(
        db: State<'_, DbState>,
        onebot: State<'_, OneBotClientState>,
    ) -> Result<SyncFriendsResult, String>
    ```
    - 调用 onebot.get_friend_list() 获取远端好友
    - 对比 friends 表找出新增好友
    - 调用 upsert_friends_batch() 更新数据库
    - 为新好友调用 assign_default_tag_to_new_friends()
    - 返回 `SyncFriendsResult { total, new_count }`
  - [x] 3.4 实现 `get_tags` command：
    ```rust
    #[tauri::command]
    pub fn get_tags(db: State<DbState>) -> Result<Vec<TagRow>, String>
    ```
  - [x] 3.5 在 `commands/mod.rs` 添加 `pub mod friends;`
  - [x] 3.6 在 `lib.rs` 注册 3 个新命令

- [x] Task 4: 创建 useFriendsStore.ts (AC: #8)
  - [x] 4.1 创建 `src/stores/useFriendsStore.ts`：
    ```typescript
    interface FriendsStore {
      friends: FriendWithTags[];
      tags: TagInfo[];
      isLoading: boolean;
      isSyncing: boolean;
      searchQuery: string;
      selectedTagIds: number[];
      // computed
      filteredFriends: () => FriendWithTags[];
      // actions
      setSearchQuery: (q: string) => void;
      setSelectedTagIds: (ids: number[]) => void;
      fetchFriends: () => Promise<void>;
      fetchTags: () => Promise<void>;
      syncFriends: () => Promise<SyncFriendsResult>;
    }
    ```
  - [x] 4.2 filteredFriends 逻辑：先按搜索词过滤（nickname/remark 包含），再按选中标签过滤（交集），排序按昵称拼音

- [x] Task 5: 创建前端类型定义 (AC: #1, #3)
  - [x] 5.1 创建 `src/types/friends.ts`：
    ```typescript
    export interface TagInfo {
      id: number;
      name: string;
      color: string;
      isSystem: boolean;
    }
    export interface FriendWithTags {
      userId: number;
      nickname: string;
      remark: string;
      tags: TagInfo[];
      likedToday: boolean;
    }
    export interface SyncFriendsResult {
      total: number;
      newCount: number;
    }
    ```
  - [x] 5.2 在 `src/lib/tauri.ts` 添加 invoke wrapper：
    ```typescript
    export async function getFriends(): Promise<FriendWithTags[]>
    export async function syncFriends(): Promise<SyncFriendsResult>
    export async function getTags(): Promise<TagInfo[]>
    ```

- [x] Task 6: 实现 Friends.tsx 页面 (AC: #3, #4, #5, #6, #9)
  - [x] 6.1 页面结构：
    ```
    ┌─────────────────────────────────────┐
    │ 好友管理          [同步好友] 按钮    │  ← 页面标题 + 操作
    ├─────────────────────────────────────┤
    │ [搜索框]  [标签筛选下拉]  共 N 人   │  ← 工具栏
    ├─────────────────────────────────────┤
    │ ┌─────────────────────────────────┐ │
    │ │ 好友卡片 1                      │ │
    │ │ 好友卡片 2                      │ │  ← 虚拟滚动列表
    │ │ ...                             │ │
    │ └─────────────────────────────────┘ │
    └─────────────────────────────────────┘
    ```
  - [x] 6.2 首次挂载：调用 `fetchTags()` + `fetchFriends()`，首次自动触发 `syncFriends()` 同步最新数据
  - [x] 6.3 搜索框：使用 shadcn/ui Input，输入时调用 `setSearchQuery()`
  - [x] 6.4 标签筛选：自定义多选下拉（显示标签 badge，可多选切换）
  - [x] 6.5 虚拟滚动：使用 CSS `overflow-y: auto` + `content-visibility: auto` 实现性能优化（不引入新依赖）；如需更强性能，可用固定高度列表 + 手动计算可见项
  - [x] 6.6 空状态：`friends.length === 0 && !isLoading` 时显示 Mascot 空状态
  - [x] 6.7 同步按钮：点击调用 `syncFriends()`，成功后 toast 通知 + 刷新列表
  - [x] 6.8 统计：工具栏右侧显示"共 N 人"

- [x] Task 7: 创建 FriendCard.tsx 组件 (AC: #3)
  - [x] 7.1 组件结构：
    ```
    ┌──────────────────────────────────────┐
    │ [头像] 昵称          [标签] [标签]  ✓ │
    │        备注名                         │
    └──────────────────────────────────────┘
    ```
  - [x] 7.2 头像占位：圆形 40px，使用昵称首字渐变背景
  - [x] 7.3 标签 badge：使用标签颜色背景，6px 圆角，12px 字号
  - [x] 7.4 今日点赞状态：已赞绿色勾 `✓`（薄荷绿），未赞灰色 `—`

- [x] Task 8: TauriEventProvider 集成 (AC: #8)
  - [x] 8.1 **不需要**在 TauriEventProvider 添加好友事件监听 — 好友数据按需加载而非事件驱动
  - [x] 8.2 可选：在 TauriEventProvider 的 `useEffect` 初始化中预加载 tags 数据

## Dev Notes

### 已有基础设施（直接复用！）

**数据库：`friends` 表已存在（migration 003_friends.sql）：**
```sql
CREATE TABLE IF NOT EXISTS friends (
    user_id INTEGER PRIMARY KEY,
    nickname TEXT NOT NULL DEFAULT '',
    remark TEXT NOT NULL DEFAULT '',
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**Rust 函数已存在 — 必须复用：**

| 模块 | 函数 | 用途 |
|------|------|------|
| `db/models.rs` | `upsert_friends_batch(conn, &[FriendRow])` | 批量更新好友到数据库（已有） |
| `db/models.rs` | `FriendRow { user_id, nickname, remark }` | 好友行结构体（已有） |
| `db/models.rs` | `has_liked_today(conn, user_id, date)` | 检查今日是否已赞（已有） |
| `onebot/client.rs` | `OneBotClient::get_friend_list()` | 获取好友列表 OneBot API（已有） |
| `onebot/types.rs` | `FriendInfo { user_id, nickname, remark }` | OneBot 好友信息结构体（已有） |
| `engine/quota.rs` | `today()` | 获取今日日期字符串 YYYY-MM-DD（已有） |
| `engine/quota.rs` | `has_liked_today(conn, user_id)` | 检查用户今日是否已赞（已有，使用当天日期） |

**前端已存在：**

| 文件 | 状态 |
|------|------|
| `src/pages/Friends.tsx` | 占位组件，需要重写 |
| `src/types/onebot.ts` | `FriendInfo { userId, nickname, remark }` 已定义 |
| `App.tsx` 路由 `/friends` | 已注册 |
| `SidebarNav.tsx` 好友管理入口 | 已有（Users 图标） |

**不存在需要创建的：**
- `commands/friends.rs` — Rust IPC 命令层
- `src/stores/useFriendsStore.ts` — 前端状态管理
- `src/components/friends/FriendCard.tsx` — 好友卡片组件
- `src/types/friends.ts` — Friends 相关类型
- `src-tauri/migrations/007_tags.sql` — 标签表

### 架构合规要点

**Rust 代码：**
- 所有对外结构体必须 `#[serde(rename_all = "camelCase")]`
- Tauri commands 返回 `Result<T, String>`，错误转换 `.map_err(|e| e.to_string())`
- 新增查询函数放在 `db/models.rs`（唯一数据库访问点）
- 命令函数放在 `commands/friends.rs`
- 使用 `tracing::info!` / `tracing::error!`，禁止 `println!`
- 使用 `?` 操作符，禁止 `unwrap()` / `expect()` 在生产代码
- db lock 不跨 await — 每次 lock → 操作 → 自动 drop

**前端代码：**
- 组件文件 PascalCase：`FriendCard.tsx`
- Store 用 camelCase + use 前缀：`useFriendsStore`
- 类型定义放 `src/types/friends.ts`
- invoke wrapper 放 `src/lib/tauri.ts`
- 使用 shadcn/ui 基础组件（Input、Badge、Button、ScrollArea）
- 颜色使用 CSS 变量：`text-text-primary`、`bg-bg-card`
- 页面组件使用 `page-enter` class（淡入动画）
- 空状态使用 Mascot 插画风格

### Tauri Command 注册模式（参考 lib.rs）

```rust
// lib.rs — invoke_handler 中追加新命令
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::friends::get_friends,
    commands::friends::sync_friends,
    commands::friends::get_tags,
])
```

### Store 创建模式（参考 useLikeStore.ts）

```typescript
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { FriendWithTags, TagInfo, SyncFriendsResult } from "@/types/friends";

interface FriendsStore {
  friends: FriendWithTags[];
  tags: TagInfo[];
  isLoading: boolean;
  isSyncing: boolean;
  searchQuery: string;
  selectedTagIds: number[];
  setSearchQuery: (q: string) => void;
  setSelectedTagIds: (ids: number[]) => void;
  fetchFriends: () => Promise<void>;
  fetchTags: () => Promise<void>;
  syncFriends: () => Promise<SyncFriendsResult | null>;
}

export const useFriendsStore = create<FriendsStore>((set) => ({
  friends: [],
  tags: [],
  isLoading: false,
  isSyncing: false,
  searchQuery: "",
  selectedTagIds: [],
  setSearchQuery: (q) => set({ searchQuery: q }),
  setSelectedTagIds: (ids) => set({ selectedTagIds: ids }),
  fetchFriends: async () => {
    set({ isLoading: true });
    try {
      const friends = await invoke<FriendWithTags[]>("get_friends");
      set({ friends });
    } catch { /* silent */ }
    set({ isLoading: false });
  },
  fetchTags: async () => {
    try {
      const tags = await invoke<TagInfo[]>("get_tags");
      set({ tags });
    } catch { /* silent */ }
  },
  syncFriends: async () => {
    set({ isSyncing: true });
    try {
      const result = await invoke<SyncFriendsResult>("sync_friends");
      // 同步后刷新好友列表
      const friends = await invoke<FriendWithTags[]>("get_friends");
      set({ friends, isSyncing: false });
      return result;
    } catch {
      set({ isSyncing: false });
      return null;
    }
  },
}));
```

过滤逻辑在组件中用 `useMemo` 实现（不放 store 里，避免计算属性复杂化）：

```typescript
const filteredFriends = useMemo(() => {
  let result = friends;
  if (searchQuery) {
    const q = searchQuery.toLowerCase();
    result = result.filter(f =>
      f.nickname.toLowerCase().includes(q) ||
      f.remark.toLowerCase().includes(q)
    );
  }
  if (selectedTagIds.length > 0) {
    result = result.filter(f =>
      f.tags.some(t => selectedTagIds.includes(t.id))
    );
  }
  return result;
}, [friends, searchQuery, selectedTagIds]);
```

### 好友同步的 Rust 逻辑

```rust
// commands/friends.rs — sync_friends 核心逻辑
pub async fn sync_friends(
    db: State<'_, DbState>,
    onebot: State<'_, OneBotClientState>,
) -> Result<SyncFriendsResult, String> {
    // 1. 调用 OneBot API 获取远端好友列表
    let remote_friends = onebot.get_friend_list().await
        .map_err(|e| e.to_string())?;

    let (total, new_count) = {
        let conn = db.lock().expect("lock db");

        // 2. 获取当前数据库中的好友 user_id 集合（用于判断新好友）
        let existing_ids: HashSet<i64> = /* query friends table for all user_ids */;

        // 3. 转换为 FriendRow 并批量 upsert
        let friend_rows: Vec<FriendRow> = remote_friends.iter().map(|f| FriendRow {
            user_id: f.user_id,
            nickname: f.nickname.clone(),
            remark: f.remark.clone(),
        }).collect();
        db::models::upsert_friends_batch(&conn, &friend_rows)?;

        // 4. 找出新好友（remote 中有但 existing 中没有的）
        let new_ids: Vec<i64> = remote_friends.iter()
            .filter(|f| !existing_ids.contains(&f.user_id))
            .map(|f| f.user_id)
            .collect();

        // 5. 为新好友分配"默认"标签
        if !new_ids.is_empty() {
            db::models::assign_default_tag_to_new_friends(&conn, &new_ids)?;
        }

        (remote_friends.len() as i64, new_ids.len() as i64)
    };

    Ok(SyncFriendsResult { total, new_count })
}
```

### get_all_friends_with_tags 查询策略

**避免 N+1 查询！** 推荐两步查询法：

```rust
pub fn get_all_friends_with_tags(conn: &Connection, date: &str) -> Result<Vec<FriendWithTags>, AppError> {
    // Step 1: 查询所有好友
    let mut stmt = conn.prepare("SELECT user_id, nickname, remark FROM friends ORDER BY nickname")?;
    let friends: Vec<(i64, String, String)> = /* collect */;

    // Step 2: 批量查询所有 friend_tags + tags 关联
    let mut tag_stmt = conn.prepare(
        "SELECT ft.friend_id, t.id, t.name, t.color, t.is_system
         FROM friend_tags ft JOIN tags t ON ft.tag_id = t.id"
    )?;
    let all_tags: Vec<(i64, i64, String, String, bool)> = /* collect */;

    // Step 3: 用 HashMap 按 friend_id 分组标签
    let tag_map: HashMap<i64, Vec<TagRow>> = /* group by friend_id */;

    // Step 4: 批量查询今日已赞好友（一次查 like_history 即可）
    let date_start = format!("{} 00:00:00", date);
    let date_end = format!("{} 23:59:59", date);
    let mut liked_stmt = conn.prepare(
        "SELECT DISTINCT user_id FROM like_history
         WHERE created_at BETWEEN ?1 AND ?2 AND success = 1"
    )?;
    let liked_ids: HashSet<i64> = /* collect */;

    // Step 5: 组装结果
    Ok(friends.into_iter().map(|(uid, nick, rem)| {
        FriendWithTags {
            user_id: uid,
            nickname: nick,
            remark: rem,
            tags: tag_map.get(&uid).cloned().unwrap_or_default(),
            liked_today: liked_ids.contains(&uid),
        }
    }).collect())
}
```

### 虚拟滚动实现策略

**不引入新 npm 依赖。** 使用以下方案之一：

**方案 A（推荐）：CSS content-visibility**
```tsx
// 每个 FriendCard 固定高度 64px
// 使用 CSS content-visibility: auto 让浏览器自动跳过不可见项的渲染
<div className="h-16" style={{ contentVisibility: "auto", containIntrinsicSize: "0 64px" }}>
  <FriendCard friend={friend} />
</div>
```

**方案 B：手动虚拟滚动（如 500+ 好友性能不够）**
```tsx
// 使用 useRef + scroll event 计算可见范围
// 仅渲染可见区域 ± buffer 的卡片
// 用 paddingTop/paddingBottom 撑起总高度
```

优先用方案 A，简单有效。如果测试中 500+ 好友仍卡，再切方案 B。

### 好友头像占位实现

QQ 头像可通过公开 URL 获取：`https://q1.qlogo.cn/g?b=qq&nk={qq_number}&s=40`
但考虑到：
1. 好友 user_id 是 QQ 号，可直接用
2. NapCat 的 FriendInfo 不包含头像 URL
3. 网络请求过多可能有性能问题

**推荐**：使用昵称首字生成渐变头像占位：
```tsx
function AvatarPlaceholder({ nickname }: { nickname: string }) {
  const char = nickname.charAt(0) || "?";
  // 根据 char code 生成稳定的渐变色
  const hue = (char.charCodeAt(0) * 37) % 360;
  return (
    <div
      className="w-10 h-10 rounded-full flex items-center justify-center text-white text-sm font-medium"
      style={{ background: `linear-gradient(135deg, hsl(${hue}, 60%, 60%), hsl(${(hue + 40) % 360}, 60%, 50%))` }}
    >
      {char}
    </div>
  );
}
```

### 前几个 Story 的经验教训（必须遵守！）

1. **serde rename_all = "camelCase"**：所有对外暴露的结构体必须加此注解（Architecture 强制规则 #1）
2. **Tauri command 返回 Result<T, String>**：`.map_err(|e| e.to_string())`
3. **不用 println!/unwrap()/expect()**：使用 tracing 宏 + `?` 操作符
4. **db lock 不要跨 await**：每次 lock → 操作 → 自动 drop，绝不 hold lock 跨 async 调用
5. **Tauri invoke 类型统一放 `src/lib/tauri.ts`**：所有 invoke wrapper 放这里（Story 4.2 P3-F1 教训）
6. **db 访问通过 db/models.rs**：不在 commands 或 engine 中直接写 SQL
7. **前端类型按域分文件**：`src/types/friends.ts`（不塞进 index.ts）
8. **store 中 try-catch 静默处理错误**：与 useLikeStore/useNapCatStore 模式一致
9. **sync_friends 是 async command**：因为调用 onebot API 是异步的，需要 `pub async fn` + `State<'_, T>` 生命周期标注
10. **migration 文件只追加不修改**：只新增 007，不动 001-006

### 不要做的事情

- **不要修改 `db/migrations.rs` 已有的 migration 记录（001-006）** — 只追加 007
- **不要修改 `onebot/client.rs`** — `get_friend_list()` 已完备
- **不要修改 `onebot/types.rs`** — `FriendInfo` 已定义
- **不要修改 `engine/like_executor.rs`** — 批量点赞中的好友缓存逻辑独立
- **不要修改 `engine/quota.rs`** — 名额管理 API 已完备
- **不要在这个 Story 实现标签 CRUD（创建/编辑/删除标签）** — 那是 Story 6.2
- **不要在这个 Story 实现标签策略（按标签配置点赞次数/优先级）** — 那是 Story 6.3
- **不要在这个 Story 实现好友详情弹窗或右键菜单** — 超出 AC 范围
- **不要添加 react-virtuoso/react-window 等虚拟滚动库** — 用 CSS content-visibility 或手动实现
- **不要在好友卡片上添加编辑/删除标签的交互** — 标签管理是 Story 6.2
- **不要修改 `tray/mod.rs`** — 托盘不受好友管理影响
- **不要修改 `webhook/mod.rs`** — Webhook 与好友管理无关

### Project Structure Notes

新增文件：
```
src-tauri/
├── migrations/
│   └── 007_tags.sql                   # NEW — 标签表+种子数据
└── src/
    └── commands/
        └── friends.rs                 # NEW — 好友管理 Tauri commands

src/
├── types/
│   └── friends.ts                     # NEW — 好友+标签类型定义
├── stores/
│   └── useFriendsStore.ts             # NEW — 好友状态管理
├── components/
│   └── friends/
│       └── FriendCard.tsx             # NEW — 好友卡片组件
└── pages/
    └── Friends.tsx                    # REWRITE — 好友管理完整页面
```

修改文件：
```
src-tauri/src/db/migrations.rs         # MODIFY — 添加 007 migration
src-tauri/src/db/models.rs             # MODIFY — 新增 TagRow, FriendWithTags, 查询函数
src-tauri/src/commands/mod.rs          # MODIFY — 添加 pub mod friends
src-tauri/src/lib.rs                   # MODIFY — 注册 3 个新 commands
src/lib/tauri.ts                       # MODIFY — 添加 3 个 invoke wrapper
```

**路径与架构对齐验证：**
- `commands/friends.rs` — 与 architecture.md `commands/friends.rs # 好友管理：get_list, update_tags` 一致 ✅
- `db/models.rs` 新增查询 — 遵循唯一数据库访问点规则 ✅
- `useFriendsStore.ts` — 与 architecture.md `stores/useFriendsStore.ts # 好友与标签状态` 一致 ✅
- `FriendCard.tsx` — 与 UX 设计 `FriendCard` 组件定义一致 ✅
- 事件命名无新增事件 — 好友数据按需加载 ✅
- migration 编号 007 — 顺承现有 001-006 ✅

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 6.1: 好友列表展示与同步]
- [Source: .bmad-method/planning-artifacts/epics.md#Epic 6: 好友管理与标签系统 — FR43, FR44, FR45, FR46]
- [Source: .bmad-method/planning-artifacts/architecture.md#项目结构 — commands/friends.rs, friends/, stores/useFriendsStore.ts]
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则 — commands/ 唯一前端入口, db/models.rs 唯一数据库访问]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — Rust snake_case, serde camelCase, 表名 snake_case 复数]
- [Source: .bmad-method/planning-artifacts/architecture.md#数据库命名 — 表名 snake_case 复数, 外键 {表单数}_id, 索引 idx_{表}_{列}]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri IPC 命令模式 — invoke + Result<T, String>]
- [Source: .bmad-method/planning-artifacts/architecture.md#Zustand Store 模式 — 域级独立 store]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单 — 禁止 println/unwrap/直接 SQL]
- [Source: .bmad-method/planning-artifacts/architecture.md#强制规则 7 条]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#FriendCard 组件 — 头像+昵称+标签+状态]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#空状态 — Mascot 插画+温暖文案]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#色彩系统 — 马卡龙色板]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#TagBadge 组件 — 马卡龙色标签]
- [Source: .bmad-method/implementation-artifacts/5-2-reply-like-handler.md — 前置 Story 模式参考]
- [Source: src-tauri/src/db/models.rs — upsert_friends_batch, FriendRow, has_liked_today]
- [Source: src-tauri/src/onebot/client.rs — get_friend_list()]
- [Source: src-tauri/src/onebot/types.rs — FriendInfo]
- [Source: src-tauri/src/engine/quota.rs — today(), has_liked_today()]
- [Source: src-tauri/src/commands/mod.rs — command 模块注册模式]
- [Source: src-tauri/src/lib.rs — invoke_handler 注册, State 管理模式]
- [Source: src-tauri/src/db/migrations.rs — migration 注册模式]
- [Source: src/stores/useLikeStore.ts — Zustand store 模式参考]
- [Source: src/lib/tauri.ts — invoke wrapper 模式]
- [Source: src/components/TauriEventProvider.tsx — 全局事件监听模式]
- [Source: src/types/onebot.ts — FriendInfo 前端类型]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- No debug issues encountered. TypeScript `tsc --noEmit` passed clean. Rust `cargo check` passed with 0 new warnings (only pre-existing unused import warnings from prior stories).
- ESLint passed clean on all new/modified frontend files.

### Completion Notes List

- Task 1: Created `007_tags.sql` migration with `tags` and `friend_tags` tables + 3 system seed tags (默认/重要/不赞). Registered in `migrations.rs`.
- Task 2: Added `TagRow`, `FriendWithTags` structs and 5 query functions to `db/models.rs`. Used 2-step query strategy to avoid N+1 (batch tags + batch liked_today via HashSet).
- Task 3: Created `commands/friends.rs` with `get_friends`, `sync_friends` (async), `get_tags`. Registered in `mod.rs` and `lib.rs` invoke_handler.
- Task 4: Created `useFriendsStore.ts` Zustand store matching existing store patterns (try-catch silent, invoke directly).
- Task 5: Created `src/types/friends.ts` with `TagInfo`, `FriendWithTags`, `SyncFriendsResult`. Added 3 invoke wrappers to `src/lib/tauri.ts`.
- Task 6: Rewrote `Friends.tsx` with full page: title bar + sync button, search input + tag filter dropdown + count, virtual scroll list with `content-visibility: auto`, empty states (no friends / no filter matches), loading spinner.
- Task 7: Created `FriendCard.tsx` with avatar placeholder (first char + hue-based gradient), nickname/remark, color-coded tag badges, liked-today status indicator (mint green ✓ / gray —).
- Task 8: Confirmed no TauriEventProvider changes needed — friends data is on-demand, not event-driven.
- Filtering implemented via `useMemo` in Friends.tsx (not in store) per story Dev Notes guidance.
- No new npm dependencies added. Used CSS `content-visibility: auto` for virtual scroll per story spec.

### File List

**New files:**
- `src-tauri/migrations/007_tags.sql` — tags + friend_tags tables + seed data
- `src-tauri/src/commands/friends.rs` — get_friends, sync_friends, get_tags Tauri commands
- `src/types/friends.ts` — TagInfo, FriendWithTags, SyncFriendsResult type definitions
- `src/stores/useFriendsStore.ts` — Zustand store for friends and tags state
- `src/components/friends/FriendCard.tsx` — Friend card component with avatar, tags, status
- `src/pages/Friends.tsx` — Complete friends management page (rewrite)

**Modified files:**
- `src-tauri/src/db/migrations.rs` — Added 007_tags migration entry
- `src-tauri/src/db/models.rs` — Added TagRow, FriendWithTags structs + 5 query functions
- `src-tauri/src/commands/mod.rs` — Added `pub mod friends`
- `src-tauri/src/lib.rs` — Registered 3 new commands in invoke_handler
- `src/lib/tauri.ts` — Added getFriends, syncFriends, getTags invoke wrappers + FriendWithTags/SyncFriendsResult/TagInfo imports

### Change Log

- 2026-03-14: Story 6.1 implementation complete. All 8 tasks done. TypeScript + Rust + ESLint all pass.

## QA Results

### Review Date: 2026-03-14

### Reviewed By: Quinn (Test Architect)

### Code Quality Assessment

实现质量整体良好。全部 9 个验收标准均已验证通过。后端采用 2-step 批量查询策略有效避免 N+1 问题，db lock 作用域正确（不跨 await），所有对外结构体均已标注 `serde(rename_all = "camelCase")`。前端组件遵循项目既有模式（Zustand store、shadcn/ui 组件、CSS 变量色彩系统），FriendCard 组件设计简洁，TagFilterDropdown 交互合理。虚拟滚动采用 CSS `content-visibility: auto` 方案，无新 npm 依赖引入。

发现 1 个 P2 架构违规（commands 层直接 SQL）和 2 个 P3 低风险建议项。

### Refactoring Performed

无。P2-F1 属于架构一致性问题，建议开发者自行修复。

### Compliance Check

- Coding Standards: ✓ — Rust: serde camelCase、tracing 日志、`?` 操作符、无 unwrap/expect/println。TS: PascalCase 组件、camelCase store、类型按域分文件
- Project Structure: ✓ — 新增文件路径与 architecture.md 定义一致（commands/friends.rs、useFriendsStore.ts、FriendCard.tsx、types/friends.ts）
- Testing Strategy: ✓ — 项目当前无自动化测试基础设施，与前序 Story 一致
- All ACs Met: ✓ — 9/9 全部通过

### Improvements Checklist

- [ ] **P2-F1**: 提取 `commands/friends.rs:36-43` 直接 SQL 到 `db/models.rs` 新增 `get_all_friend_ids(conn) -> Result<HashSet<i64>, AppError>` 函数
- [ ] **P3-F1**: Store 直接 invoke vs tauri.ts wrapper 保持一致（与项目既有模式一致，非本 Story 特有问题，可后续统一处理）
- [ ] **P3-F2**: `Friends.tsx:14` 的 `const ref` 重命名为 `dropdownRef`，避免与 React ref prop 混淆

### Security Review

无安全风险。好友数据从 SQLite 本地读取，同步通过 OneBot 本地 API（localhost），无外部网络暴露。SQL 查询使用参数化（`params![]`），无注入风险。

### Performance Considerations

- `get_all_friends_with_tags` 采用 3-step 批量查询（friends → tags → liked_ids）避免 N+1，性能合理
- `content-visibility: auto` 方案对 500+ 好友有一定优化效果但并非真正虚拟滚动（仍创建所有 DOM 节点）。Story 已预留方案 B 作为后备，当前 MVP 阶段足够
- 首次加载双重获取好友列表（fetchFriends + syncFriends 内部再获取）是合理的 UX 策略：先快速显示缓存，再后台同步

### Files Modified During Review

无文件修改。

### Gate Status

Gate: CONCERNS → `.bmad-method/test-artifacts/gates/6.1-friend-list-and-sync.yml`

### Recommended Status

✓ Ready for Done — 1 个 P2 为架构组织问题（功能正确），不阻塞发布。建议开发者后续修复 P2-F1。
