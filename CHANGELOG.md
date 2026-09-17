# 变更记录

本文件记录 Soloup 的重要变更。格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.1.0] - 2026-09-17

这一版的主线是**手机可用**（局域网 + PWA）和**数据驱动收口**（前端不再自己算数），
外加数据库升级的安全网（迁移前自动备份）。

### 新增

- **局域网服务**：设置页新增开关，可在同一 Wi-Fi 下用手机/平板访问面板；
  界面展示访问地址并提供二维码，扫码即开。手机 Safari「添加到主屏幕」后
  成为一个独立应用（无地址栏）
- **二维码**：由后端 `qrcode` crate 生成 SVG（新 op `lan.qr`），前端零新增依赖
- **PWA 元信息**：`manifest.webmanifest`、apple-touch-icon、theme-color、
  `apple-mobile-web-app-capable` 等，使 iOS 添加到主屏幕后为独立窗口
- **迁移前自动备份**：检测到数据库 schema 版本落后时，先用 `VACUUM INTO` 落一份
  一致性快照到 `backups/`，再执行迁移；保留最近 10 份。备份失败会拒绝启动，
  可用环境变量 `SOLOUP_SKIP_BACKUP=1` 跳过
- **V5 迁移**：补齐 9 条内置成就的 `hidden` / `requires_json` / `reveal_at`
  三个字段（此前一直停留在加列时的默认值）

### 变更

- **成就页完全数据驱动**：名称、描述、稀有度、隐藏、前置、揭示阈值、解锁判定
  全部来自后端；前端的硬编码成就定义已删除，只保留「稀有度→配色」「成就 id→图标」
  两处展示映射。未解锁卡背的进度文案改由条件 DSL 通用生成
- **属性/技能展示值直接取后端结算结果**，前端不再从等级反推投入量
- **后端接口地址改为运行时解析**：按当前页面 hostname 反推（`src/lib/api.ts` 的
  `hostOf()`），使手机、桌面壳、`next dev` 三种场景共用同一份构建产物
- 后端**始终**绑定 `0.0.0.0`，非回环来源由开关（`lan_gate`）逐请求校验，
  因此开关改动即时生效，无需重启进程
- **版本号改为单一来源**：`meta` op 与 MCP `serverInfo` 返回的版本号改由
  `env!("CARGO_PKG_VERSION")` 编译期注入，不再在源码里手写（此前写死为 `0.1.0`）
- 打包流程把 `apps/web/out` 一并复制进 `src-tauri/resources/web`，
  安装版同样能对外提供局域网页面

### 修复

- **前端等级反推导致数字虚高**：原实现从等级倒推"投入量/经验"，得到的是
  与真实数据无关的百万级数字（且被重复累加）。改为直接展示后端结算值
- **排行榜"使用次数"不实**：同样的问题，改为后端返回的真实打卡天数
- **局域网客户端实际不通**：`apps/web/.env.local` 里的 `NEXT_PUBLIC_SOLOUP_API`
  会在构建期内联进 bundle，使 `hostOf()` 的运行时反推失效 —— 手机打开页面时
  请求全打到手机自己的 `localhost:8787`。已停用该文件并补充 `.env.example` 说明
- **CI 产出的安装包缺少 `resources/web`**：发布工作流在 `copy-sidecars` 之前
  没有构建前端，脚本走 warning 分支静默跳过。已调整步骤顺序并加了一道硬校验
- `scripts/upx-compress.mjs` 写死 `Soloup_0.1.0_x64-setup.exe`，改为按
  `tauri.conf.json` 的版本号解析，并复用目录中已有的安装包名
- `src-tauri/Cargo.toml` 的 `edition = "2026"` 是非法值（Cargo 稳定版只认到
  2024），这是 v1.0.1 那次 CI 构建失败的直接原因。已改回 `2021`
- `eslint` / `prettier` 的忽略表都没排除 Next.js 静态导出目录 `apps/web/out`，
  导致 `pnpm lint` 会去检查上千个生成的压缩 JS。两处均已补上 `out/`

### 移除

- 前端硬编码的 `ACHIEVEMENTS` 成就定义常量
- 前端从等级/数值反推的辅助函数（`attrLv` / `epForLevel` / `skillLv` 等）
- `apps/web/.env.local`（内容已由 `.env.example` 说明替代）

### 升级说明

- 后端启动时会自动把数据库从 schema v4 迁到 **v5**，并在迁移前自动备份。
  业务数据（打卡记录、技能、属性、项目）不受影响
- ⚠️ V5 迁移会**覆写**这 9 条内置成就的 `hidden` / `requires` / `reveal_at`
  设置。如果你手动调整过这些字段，升级后需要重新设置；**自定义成就完全不受影响**

## [1.0.1] - 2026-09-10

- 修复子技能创建流程
- 修正 README 内容

> 该版本只打了 tag：CI 因 `src-tauri/Cargo.toml` 的非法 `edition` 值构建失败，
> 未产出 Release 产物。该问题已在 1.1.0 修复。

## [0.1.0] - 2026-09-07

首个版本。

- 技能管理、每日打卡、双轨等级曲线（指数饱和 / Logistic Sigmoid）
- 属性跨技能加权派生，技能树与关联权重
- 项目轴、每日提醒、系统托盘、数据 JSON 导入导出
- 本地 MCP 服务（41 个工具），AI 可代为查询与管理面板
- Tauri v2 桌面壳 + Next.js 15 静态前端 + Rust 后端三进程架构
- GitHub Actions 自动构建 Windows NSIS 安装包

[1.1.0]: https://github.com/Roxy-DD/soloup/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/Roxy-DD/soloup/compare/v0.1.0...v1.0.1
[0.1.0]: https://github.com/Roxy-DD/soloup/releases/tag/v0.1.0
