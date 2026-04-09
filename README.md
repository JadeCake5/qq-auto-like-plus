# QQ Auto Like Plus

```python
Made for Xiaoyi
```
一款 Windows 桌面应用，自动为 QQ 好友主页点赞。双击即用，后台静默运行。

## 功能

- **定时全量点赞** — 每日自动获取好友列表，随机顺序逐个点赞
- **自动回赞** — 收到好友点赞后自动回赞
- **NapCat 一键管理** — 自动下载、启动、监控 NapCat Shell 进程
- **扫码登录** — 首次使用扫码，后续自动快速登录
- **系统托盘** — 最小化到托盘静默运行，右键菜单操作
- **好友分组** — 自定义标签，按分组配置不同点赞策略
- **数据统计** — 点赞记录、趋势图表、好友互动排行
- **开机自启** — 可选开机自动启动

## 技术栈

| 组件 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 后端 | Rust |
| 前端 | React + TypeScript |
| QQ 协议 | NapCat Shell (OneBot 11) |
| 数据库 | SQLite |

## 开发

### 前置要求

- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/)

### 启动开发环境

```bash
pnpm install
pnpm tauri dev
```

### 构建

```bash
pnpm tauri build
```

构建产物在 `src-tauri/target/release/bundle/` 下。

## 使用方式

1. 双击运行 `QQ Auto Like Plus.exe`
2. 首次运行会自动下载 NapCat Shell（也可手动导入）
3. 扫码登录 QQ
4. 应用自动最小化到系统托盘，开始后台运行

## 相关项目

- [NapCat](https://github.com/NapNeko/NapCatQQ) — QQ Bot 框架
- [OneBot 11](https://github.com/botuniverse/onebot-11) — 机器人协议规范

## License

MIT
