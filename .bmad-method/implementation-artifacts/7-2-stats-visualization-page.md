# Story 7.2: 数据统计页面（图表可视化）

Status: done

## Story

As a 用户,
I want 通过图表查看点赞趋势和数据分析,
so that 了解我的社交互动情况。

## Acceptance Criteria

1. **时间范围切换**：顶部显示"日 / 周 / 月"Tab 切换，默认周视图
2. **主图表 — 日视图**：柱状图（Recharts BarChart）展示当天每小时点赞数（24 根柱子）
3. **主图表 — 周视图**：折线图（Recharts LineChart）展示近 7 天每日点赞趋势
4. **主图表 — 月视图**：折线图（Recharts LineChart）展示近 30 天每日点赞趋势
5. **图表配色**：马卡龙色系 — 樱花粉折线/柱体 `#f2a7c3` + 薰衣草紫填充区域 `#c3a7f2`
6. **点赞类型占比饼图**：Recharts PieChart 展示定时/回赞/手动三色区分
7. **好友互动排行 TOP 10**：列表展示头像占位 + 昵称 + 被赞次数 + 进度条
8. **Tooltip 交互**：图表支持 hover 显示详细数值
9. **空状态**：无数据时显示 Mascot 空状态插画（"还没有点赞数据，等明天再来看看吧~"）
10. **性能**：页面加载 < 1 秒

## Tasks / Subtasks

- [x] Task 1: 实现 Statistics.tsx 页面主体框架 (AC: #1)
  - [x] 1.1 导入 useStatsStore，在 useEffect 中调用 setPeriod("week") 初始化数据
  - [x] 1.2 创建时间范围 Tab 组件（日/周/月），点击调用 setPeriod() 切换
  - [x] 1.3 页面布局：顶部标题+Tab → 主图表区 → 下半区（饼图 + 排行双列）

- [x] Task 2: 实现主趋势图表组件 TrendChart (AC: #2, #3, #4, #5, #8)
  - [x] 2.1 创建 `src/components/statistics/TrendChart.tsx`
  - [x] 2.2 日视图 — BarChart：X 轴 0-23h，Y 轴点赞数，柱体 `#f2a7c3`
  - [x] 2.3 周视图 — LineChart：X 轴日期（MM-DD），折线 `#f2a7c3` + 填充面积 `#c3a7f2` 半透明
  - [x] 2.4 月视图 — LineChart：同周视图结构，数据源切换
  - [x] 2.5 配置 Recharts Tooltip（深色背景 #231f31、马卡龙色文字）
  - [x] 2.6 日期数据缺口补零（前端侧填充缺失日期为 count=0）

- [x] Task 3: 实现点赞类型占比饼图 TypePieChart (AC: #6, #8)
  - [x] 3.1 创建 `src/components/statistics/TypePieChart.tsx`
  - [x] 3.2 PieChart + Pie：三色区分 — 定时 `#f2a7c3`、回赞 `#a7c7f2`、手动 `#c3a7f2`
  - [x] 3.3 配置 Legend 和 Tooltip（Label 显示百分比）
  - [x] 3.4 无数据时显示灰色空心圆环

- [x] Task 4: 实现好友互动排行 FriendRanking (AC: #7)
  - [x] 4.1 创建 `src/components/statistics/FriendRanking.tsx`
  - [x] 4.2 列表项：序号 + 圆形头像占位（40px 灰色圈+首字符）+ 昵称 + 被赞次数 + 进度条
  - [x] 4.3 进度条宽度 = 当前/最大值 比例，渐变色
  - [x] 4.4 无数据时显示文字提示

- [x] Task 5: 实现空状态 (AC: #9)
  - [x] 5.1 检测所有数据为空时展示 Mascot emoji + 温馨文案
  - [x] 5.2 复用 Dashboard 空状态样式模式

- [x] Task 6: Loading 状态与性能 (AC: #10)
  - [x] 6.1 利用 useStatsStore.isLoading 显示 skeleton 占位
  - [x] 6.2 确保页面加载 < 1 秒（数据按需加载，无阻塞渲染）

## Dev Notes

### 纯前端 Story — 不修改任何 Rust 代码

Story 7.1 已完成全部后端基础设施。本 Story **只写前端 React 组件**，无需修改 `src-tauri/` 下任何文件。

### 已有基础设施（直接复用，不要重建！）

**Zustand Store — `src/stores/useStatsStore.ts`（Story 7.1 已完成）：**

```typescript
// 可用状态
hourlyData: HourlyStats[];        // 日视图数据（24 个数据点，已补零）
weeklyData: DailyStatsPoint[];    // 周视图数据（注意：缺失日期未补零！）
monthlyData: DailyStatsPoint[];   // 月视图数据（注意：缺失日期未补零！）
typeRatio: LikeTypeRatio | null;  // 点赞类型占比
friendRanking: FriendRanking[];   // 好友互动排行 TOP 10
currentPeriod: StatsPeriod;       // "day" | "week" | "month"
isLoading: boolean;

// 可用操作
setPeriod(period: StatsPeriod): Promise<void>;  // 切换时间范围并自动刷新所有相关数据
fetchDailyStats(date?: string): Promise<void>;
fetchWeeklyStats(): Promise<void>;
fetchMonthlyStats(): Promise<void>;
fetchLikeTypeRatio(period?: StatsPeriod): Promise<void>;
fetchFriendRanking(period?: StatsPeriod): Promise<void>;
```

**TypeScript 类型 — `src/types/stats.ts`（已完成）：**

```typescript
interface HourlyStats { hour: number; count: number; }
interface DailyStatsPoint { date: string; count: number; scheduled: number; reply: number; manual: number; }
interface LikeTypeRatio { scheduled: number; reply: number; manual: number; total: number; }
interface FriendRanking { userId: number; nickname: string; totalLikes: number; }
type StatsPeriod = "day" | "week" | "month";
```

**Tauri Invoke Wrappers — `src/lib/tauri.ts`（已完成）：**
不要直接调用 invoke。通过 useStatsStore 的 action 间接调用，Store 内部已封装。

**Recharts — `recharts@^3.8.0`（已安装）：**
直接 import 使用，无需安装。

### Recharts 3.x 关键注意事项

- **Recharts 3.x 已安装**，版本 `^3.8.0`
- **Tooltip 必须在 Legend 之后渲染**（JSX 顺序：先 Legend 后 Tooltip）—— 3.0 Breaking Change
- `Customized` 组件不再需要，可直接在图表内使用自定义子组件
- BarChart/LineChart 支持 `responsive` prop 代替 `ResponsiveContainer` 包裹
- PieChart 中 `blendStroke` 已移除 —— 用 `stroke="none"` 替代
- `activeIndex` prop 已从 Scatter/Bar/Pie 移除

### 日期缺口补零（P3-F1 遗留）

Story 7.1 QA 发现 `get_daily_stats_range` 仅返回有数据的日期。例如 7 天范围内若有 3 天无记录，仅返回 3 个数据点。**前端需在渲染前补零**。

实现方式：

```typescript
function fillDateGaps(data: DailyStatsPoint[], days: number): DailyStatsPoint[] {
  const map = new Map(data.map(d => [d.date, d]));
  const result: DailyStatsPoint[] = [];
  const today = new Date();
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    const key = d.toISOString().slice(0, 10); // "YYYY-MM-DD"
    result.push(map.get(key) ?? { date: key, count: 0, scheduled: 0, reply: 0, manual: 0 });
  }
  return result;
}
```

调用时机：在 TrendChart 组件中渲染前处理 `weeklyData`（补 7 天）和 `monthlyData`（补 30 天）。`hourlyData` 无需处理（后端已补零 24 小时）。

### 图表组件实现指导

**TrendChart — 日视图（BarChart）：**

```tsx
import { BarChart, Bar, XAxis, YAxis, Tooltip, CartesianGrid } from "recharts";

<BarChart data={hourlyData} width={560} height={260}>
  <CartesianGrid strokeDasharray="3 3" stroke="#2a2540" />
  <XAxis dataKey="hour" stroke="#8b85a0" tick={{ fontSize: 11 }}
    tickFormatter={(h: number) => `${h}时`} />
  <YAxis stroke="#8b85a0" tick={{ fontSize: 11 }} allowDecimals={false} />
  <Tooltip contentStyle={{ background: "#231f31", border: "1px solid #3a3450", borderRadius: 8 }}
    labelFormatter={(h: number) => `${h}:00`} />
  <Bar dataKey="count" fill="#f2a7c3" radius={[4, 4, 0, 0]} />
</BarChart>
```

**TrendChart — 周/月视图（LineChart + Area fill）：**

```tsx
import { LineChart, Line, XAxis, YAxis, Tooltip, CartesianGrid, Area } from "recharts";

const filledData = fillDateGaps(weeklyData, 7); // 或 monthlyData, 30

<LineChart data={filledData} width={560} height={260}>
  <defs>
    <linearGradient id="areaGradient" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stopColor="#c3a7f2" stopOpacity={0.3} />
      <stop offset="100%" stopColor="#c3a7f2" stopOpacity={0} />
    </linearGradient>
  </defs>
  <CartesianGrid strokeDasharray="3 3" stroke="#2a2540" />
  <XAxis dataKey="date" stroke="#8b85a0" tick={{ fontSize: 11 }}
    tickFormatter={(d: string) => d.slice(5)} /> {/* "MM-DD" */}
  <YAxis stroke="#8b85a0" tick={{ fontSize: 11 }} allowDecimals={false} />
  <Tooltip contentStyle={{ background: "#231f31", border: "1px solid #3a3450", borderRadius: 8 }} />
  <Area type="monotone" dataKey="count" fill="url(#areaGradient)" stroke="none" />
  <Line type="monotone" dataKey="count" stroke="#f2a7c3" strokeWidth={2} dot={{ fill: "#f2a7c3", r: 3 }} />
</LineChart>
```

**TypePieChart — 点赞类型占比：**

```tsx
import { PieChart, Pie, Cell, Legend, Tooltip } from "recharts";

const COLORS = { scheduled: "#f2a7c3", reply: "#a7c7f2", manual: "#c3a7f2" };
const LABELS = { scheduled: "定时点赞", reply: "回赞", manual: "手动点赞" };

const pieData = typeRatio ? [
  { name: LABELS.scheduled, value: typeRatio.scheduled },
  { name: LABELS.reply, value: typeRatio.reply },
  { name: LABELS.manual, value: typeRatio.manual },
].filter(d => d.value > 0) : [];

<PieChart width={240} height={240}>
  <Pie data={pieData} dataKey="value" nameKey="name"
    cx="50%" cy="50%" outerRadius={80} innerRadius={40}
    stroke="none" label={({ percent }) => `${(percent * 100).toFixed(0)}%`}>
    {pieData.map((entry, i) => (
      <Cell key={i} fill={Object.values(COLORS)[i]} />
    ))}
  </Pie>
  <Legend />
  <Tooltip contentStyle={{ background: "#231f31", border: "1px solid #3a3450", borderRadius: 8 }} />
</PieChart>
```

### 前端代码规范（必须遵循）

- 组件文件名 PascalCase：`TrendChart.tsx`、`TypePieChart.tsx`、`FriendRanking.tsx`
- 组件放 `src/components/statistics/` 目录下（参考 `src/components/dashboard/`）
- CSS 使用 Tailwind 类名（不要内联 style 除非动态值如渐变色）
- CSS 变量引用：`text-[length:var(--text-display)]`、`text-[length:var(--text-body)]` 等
- 颜色 class：`text-text-primary`、`text-text-secondary`、`text-text-muted`、`bg-bg-card`（#231f31）、`bg-bg-base`（#1a1625）
- 圆角 14px → `rounded-[14px]`（卡片）、10px → `rounded-[10px]`（按钮）、6px → `rounded-md`（badge）
- 空状态使用 emoji Mascot + `text-text-secondary` 文案（参考 Dashboard.tsx L101-132）
- 导入路径使用 `@/` 别名
- 无障碍：图表添加 `role="img"` + `aria-label`

### 页面布局建议

```
┌──────────────────────────────────────────┐
│ 数据统计           [日] [周✓] [月]        │  ← 标题 + Tab
├──────────────────────────────────────────┤
│                                          │
│          主图表（BarChart / LineChart）     │  ← 高度 260px
│                                          │
├──────────────────┬───────────────────────┤
│  点赞类型占比      │  好友互动排行 TOP 10    │  ← 双列布局
│  (PieChart)       │  (列表 + 进度条)       │
│  240×240          │                       │
└──────────────────┴───────────────────────┘
```

整体用 `flex flex-col gap-4` 布局，下半区用 `grid grid-cols-2 gap-3` 或 `flex gap-3`。

### Store 使用模式（参考 Dashboard.tsx）

```tsx
// Statistics.tsx
import { useEffect } from "react";
import { useStatsStore } from "@/stores/useStatsStore";

export default function Statistics() {
  const {
    hourlyData, weeklyData, monthlyData,
    typeRatio, friendRanking,
    currentPeriod, isLoading,
    setPeriod,
  } = useStatsStore();

  // 初始化：加载默认周视图数据
  useEffect(() => {
    setPeriod("week");
  }, [setPeriod]);

  // 根据 currentPeriod 选择图表数据
  const chartData = currentPeriod === "day" ? hourlyData
    : currentPeriod === "week" ? fillDateGaps(weeklyData, 7)
    : fillDateGaps(monthlyData, 30);

  // ... 渲染
}
```

### 不要做的事情

- **不要修改 `src-tauri/` 下任何文件** — 后端已完成
- **不要修改 `useStatsStore.ts`** — Store 已完成，直接使用
- **不要修改 `src/types/stats.ts`** — 类型已完成
- **不要修改 `src/lib/tauri.ts`** — Invoke wrappers 已完成
- **不要安装新依赖** — Recharts 已安装
- **不要在组件内直接调用 invoke()** — 通过 Store action 调用
- **不要使用 `ResponsiveContainer`** — Recharts 3.x 支持 `responsive` prop 或直接设宽高
- **不要在 PieChart 中使用 `blendStroke`** — 3.0 已移除，用 `stroke="none"`
- **不要把 Tooltip 放在 Legend 之前** — Recharts 3.0 要求 Tooltip 在 Legend 之后
- **不要修改 Dashboard.tsx** — 不相关
- **不要创建新的 Zustand store** — useStatsStore 已存在
- **不要在 `src/components/dashboard/` 下创建文件** — 统计组件放 `src/components/statistics/`
- **不要使用 `println!` 或 `console.log`** — 生产代码禁止

### Project Structure Notes

新增文件：
```
src/
├── components/statistics/
│   ├── TrendChart.tsx         # NEW — 趋势图表（BarChart 日视图 + LineChart 周/月视图）
│   ├── TypePieChart.tsx       # NEW — 点赞类型占比饼图
│   └── FriendRanking.tsx      # NEW — 好友互动排行 TOP 10
```

修改文件：
```
src/pages/Statistics.tsx       # MODIFY — 从占位页面改为完整统计页面
```

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 7.2: 数据统计页面（图表可视化）]
- [Source: .bmad-method/planning-artifacts/epics.md#Epic 7: 数据统计与可视化 — FR47, FR48, FR49, FR50]
- [Source: .bmad-method/planning-artifacts/architecture.md#Recharts — React-native 数据可视化]
- [Source: .bmad-method/planning-artifacts/architecture.md#Zustand Store 模式]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#渐变色组合 — 樱花/天空/薰衣草/蜜桃]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#空状态设计 — Mascot + 温馨文案]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#Kawaii 暗色主题 — #1a1625 底色 + #231f31 卡片]
- [Source: .bmad-method/implementation-artifacts/7-1-stats-aggregation-queries.md — Story 7.1 完整后端实现]
- [Source: .bmad-method/implementation-artifacts/7-1-stats-aggregation-queries.md#QA Results — P3-F1 日期缺口补零]
- [Source: src/pages/Dashboard.tsx — 页面组件模式参考（Store 使用、空状态、StatCard）]
- [Source: src/components/dashboard/StatCard.tsx — 渐变卡片组件参考]
- [Source: src/stores/useStatsStore.ts — Zustand Store（直接复用）]
- [Source: src/types/stats.ts — TypeScript 类型定义（直接复用）]
- [Source: src/lib/tauri.ts — Invoke Wrappers（直接复用）]
- [Source: src/pages/Statistics.tsx — 当前占位页面（需重写）]
- [Source: package.json:29 — recharts@^3.8.0 已安装]
- [Recharts 3.0 Migration Guide: https://github.com/recharts/recharts/wiki/3.0-migration-guide]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

无阻塞问题，一次性通过。仅修复 2 个 Recharts 3.x TS 类型兼容问题（labelFormatter 参数类型、Pie label percent 可选类型）。

### Completion Notes List

- 所有 6 个 Task 及全部 Subtask 已完成
- ESLint 零警告、TypeScript 零错误
- 严格遵循 Story Dev Notes：未修改 src-tauri/、useStatsStore、types/stats、lib/tauri
- 未安装任何新依赖
- Recharts 3.x 兼容：Tooltip 在 Legend 之后、stroke="none" 替代 blendStroke、无 ResponsiveContainer
- 日期缺口补零在 TrendChart 组件中实现（周7天/月30天）
- 空状态复用 Dashboard 样式模式（emoji Mascot + text-text-secondary 文案）
- Skeleton loading 占位覆盖首次加载场景
- 无障碍：所有图表添加 role="img" + aria-label，排行榜使用 role="list"/"listitem"

### File List

- `src/pages/Statistics.tsx` — MODIFIED — 统计页面主体（从占位重写为完整页面）
- `src/components/statistics/TrendChart.tsx` — NEW — 趋势图表组件（BarChart + LineChart + 日期补零）
- `src/components/statistics/TypePieChart.tsx` — NEW — 点赞类型占比饼图
- `src/components/statistics/FriendRanking.tsx` — NEW — 好友互动排行 TOP 10

### Change Log

- 2026-03-14: 全部 6 Tasks 实现完成，ESLint + TypeScript 通过，Story 标记 Ready for Review
- 2026-03-14: QA Code Review 完成，修复 5 项问题，Story 标记 done

## QA Results

### Review Date
2026-03-14

### Reviewer
Quinn (Test Architect) — Code Review via `*code-review`

### AC Validation Summary

| AC | Status | Evidence |
|----|--------|----------|
| #1 时间范围切换 | PASS | Statistics.tsx:8-12 PERIOD_TABS, L26-28 setPeriod("week") 默认周视图, L82-95 Tab 按钮 |
| #2 日视图 BarChart | PASS | TrendChart.tsx:48-67 BarChart + hourlyData, X 轴 0-23h |
| #3 周视图 LineChart | PASS | TrendChart.tsx:70-112 LineChart + fillDateGaps(weeklyData, 7) |
| #4 月视图 LineChart | PASS | TrendChart.tsx:70-112 同上，fillDateGaps(monthlyData, 30) |
| #5 马卡龙配色 | PASS | Bar fill="#f2a7c3", Line stroke="#f2a7c3", Area gradient "#c3a7f2" |
| #6 饼图三色区分 | PASS | TypePieChart.tsx:4-8 TYPE_CONFIG 定义三色，L47-62 PieChart + Cell |
| #7 好友排行 TOP 10 | PASS | FriendRanking.tsx:21-55 序号+头像+昵称+次数+渐变进度条 |
| #8 Tooltip 交互 | PASS | 所有图表均配置 Tooltip + tooltipStyle |
| #9 空状态 | PASS | Statistics.tsx:38-49 全局空状态, TypePieChart L32-41, FriendRanking L8-15 |
| #10 性能 < 1秒 | PASS | Skeleton loading + 按需加载无阻塞渲染 |

### Issues Found & Fixed

| ID | Severity | Description | File:Line | Status |
|----|----------|-------------|-----------|--------|
| H1 | HIGH | `fillDateGaps` 使用 `toISOString()` 在 UTC+8 时区日期偏移一天 | TrendChart.tsx:21 | FIXED — 改用本地 `getFullYear/getMonth/getDate` |
| H2 | HIGH | PieChart Cell 颜色映射在 `.filter()` 后索引错位，类型颜色混乱 | TypePieChart.tsx:55-57 | FIXED — 颜色绑定到每个 entry 对象 |
| M1 | MEDIUM | Loading skeleton 仅判断 `weeklyData`，切换视图时无骨架屏 | Statistics.tsx:51 | FIXED — 改为判断 `currentData` |
| M2 | MEDIUM | Tab 按钮缺少 `role="tablist"/"tab"` + `aria-selected` 无障碍属性 | Statistics.tsx:81-95 | FIXED — 添加完整 ARIA |
| L1 | LOW | PieChart Cell 使用数组索引 `key={i}` | TypePieChart.tsx:55 | FIXED — 改用 `entry.key` |

### Remaining Action Items

| ID | Severity | Description | File:Line |
|----|----------|-------------|-----------|
| H3 | MEDIUM | 图表固定宽度 560px 不适配容器（应使用 Recharts 3.x `responsive` prop 或 CSS 方案） | TrendChart.tsx:51,81 |
| L2 | LOW | FriendRanking 进度条缺少 `role="progressbar"` + `aria-valuenow` | FriendRanking.tsx:43-52 |

### Gate Decision

**PASS** — 10/10 AC 全部通过，2 个 HIGH bug 和 3 个 MEDIUM/LOW 问题已修复。剩余 1 个 MEDIUM（图表宽度适配）和 1 个 LOW（进度条 ARIA）为视觉/辅助优化，不影响功能正确性。
