# Soloup - 人生 RPG 面板

把你投入在各项技能上的**每日行为**，通过一套可解释的数学模型，转换成**技能等级 → 属性面板 → 成就卡牌**的成长反馈；同时开放本地 MCP 服务，让 AI 能代为查询与管理这个面板。

![Soloup 面板预览](docs/images/preview.png)

## 功能

- **技能管理** — 创建技能、每日打卡记录投入时间，自动计算等级与熟练度
- **属性派生** — 属性完全由技能加权派生，无需手工维护
- **技能树** — 可视化技能关联与前置关系，支持拖拽排序
- **成就系统** — 名称、描述、稀有度、解锁条件、隐藏与前置关系全部由后端数据驱动；可见成就（明确目标）+ 隐藏成就（渐进揭示）
- **项目轴** — 按项目维度组织技能与记录（有起止日期，区间内的打卡按日期归集到项目）
- **每日提醒** — 可配置时间的原生通知打卡提醒
- **数据导入/导出** — JSON 全量备份与恢复
- **MCP 服务** — 本地 MCP server，AI 可查询和管理面板数据（41 个工具）
- **局域网服务** — 一键开放同一 Wi-Fi 下的访问，配二维码；手机上「添加到主屏幕」即成为独立 App（PWA）
- **系统托盘** — 关闭窗口时最小化到托盘，后台持续运行
- **开机自启动** — 可选登录系统后自动驻留托盘（桌面版；网页端无此能力）
- **自动备份** — 数据库 schema 升级前自动落一份一致性快照，升级失败可回滚

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Next.js 15 (static export) + React 19 + Tailwind CSS 4 + TanStack Query |
| 后端 | Rust — soloup-core / soloup-store / soloup-solver / soloup-server / soloup-mcp |
| 桌面 | Tauri v2 (WebView2) |
| 数据库 | SQLite (单文件，WAL 模式) |
| CI/CD | GitHub Actions (Windows NSIS 自动构建并发布 Release) |

## 架构

三个进程协作，全部跑在本机，不依赖任何云服务：

```
┌──────────────────────────────────────────────────────────┐
│ Tauri 壳 (src-tauri/)                                     │
│  ├─ WebView 加载 apps/web/out 静态文件                     │
│  ├─ sidecar: soloup-server.exe   0.0.0.0:8787            │
│  │    ├─ /api/rpc    JSON-RPC（面板的唯一数据通道）          │
│  │    ├─ /health                                         │
│  │    └─ /           托管前端静态页（供局域网设备访问）       │
│  └─ sidecar: soloup-mcp.exe      127.0.0.1:8788          │
│       └─ /mcp        MCP over HTTP，供 AI 客户端接入        │
│                                                          │
│  SQLite: %APPDATA%/com.soloup.app/soloup.db               │
└──────────────────────────────────────────────────────────┘
```

数据流是单向的：**前端不算数**。所有等级、属性、成就判定都由 Rust 后端结算后返回，前端只负责展示。这样面板、MCP、手机端三方看到的数字永远一致。

## 项目结构

```
soloup/
├── apps/web/                  # Next.js 前端（output: 'export' 静态导出）
│   ├── src/components/        # 面板 / 属性 / 技能树 / 成就 / 设置等视图
│   ├── src/lib/api.ts         # RPC 客户端（按页面 hostname 反推后端地址）
│   ├── src/lib/model.ts       # 展示层派生（不重算等级）
│   └── public/                # PWA manifest 与图标
├── backend/
│   └── crates/
│       ├── soloup-core/       # 领域模型、曲线与不变量（纯函数、无 IO）
│       ├── soloup-store/      # SQLite Repository、schema 迁移、备份
│       ├── soloup-solver/     # 数值结算引擎
│       ├── soloup-server/     # JSON-RPC 服务 + 静态页托管（sidecar）
│       └── soloup-mcp/        # MCP 服务（sidecar）
├── src-tauri/                 # Tauri 桌面壳
├── scripts/                   # 构建脚本（sidecar 复制、UPX 压缩）
├── docs/                      # 技术规格与构建文档
└── .github/workflows/         # GitHub Actions 发布流水线
```

## 开发

### 前置要求

- Node.js >= 22
- pnpm >= 10
- Rust (stable)
- [Tauri v2 系统依赖](https://v2.tauri.app/start/prerequisites/)

### 安装与运行

```bash
pnpm install

# 终端 1：后端（Rust）
cd backend && cargo run -p soloup-server

# 终端 2：前端（热更新）
cd apps/web && pnpm dev
```

打开 http://localhost:3000 即可。前端会自动把后端地址解析到同主机的 8787 端口。

桌面壳开发模式：

```bash
cd src-tauri && cargo tauri dev
```

> 桌面模式下 sidecar 从 `src-tauri/resources/` 取**上次手动复制的** exe，
> `cargo tauri dev` 不会自动重编 backend —— 改过后端代码要先跑
> `pnpm build:backend && node scripts/copy-sidecars.mjs`。

### 常用命令

```bash
pnpm build              # 构建前端（静态导出到 apps/web/out）
pnpm build:backend      # 构建后端 sidecar（release）
pnpm typecheck          # TypeScript 类型检查
pnpm test               # 运行测试
pnpm lint               # ESLint 检查
pnpm format             # Prettier 格式化
```

## 构建安装包

```bash
pnpm build:dist
```

这一步串起完整链路：前端静态导出 → 后端 release 编译 → 复制 sidecar 与 web 资源 → `cargo tauri build` → UPX 压缩主程序并重建 NSIS 安装包。

产物位于 `src-tauri/target/release/bundle/nsis/Soloup_<版本>_x64-setup.exe`。

细节（含手动分步流程、常见报错）见 [docs/BUILD.md](docs/BUILD.md)。

## 局域网访问（手机 / 平板）

1. 确保手机与电脑在**同一 Wi-Fi**
2. 打开面板 → `设置 → 局域网服务` → 打开开关
3. 界面显示 `http://<局域网IP>:8787/` 与二维码，手机扫码即可打开
4. Safari 分享菜单 →「添加到主屏幕」，即成为独立应用（无地址栏）

开关关闭时局域网设备一律收到 **403**，数据不出本机。首次使用若 Windows 防火墙弹窗，需允许**专用网络**访问。

## 数据与备份

数据库位置取决于运行方式：

| 运行方式 | 数据库路径 |
|---|---|
| 桌面安装版 / `cargo tauri dev` | `%APPDATA%\com.soloup.app\soloup.db` |
| 直接跑 `cargo run -p soloup-server` | `~/.soloup/soloup.db` |

其它要点：

- WAL 模式、外键开启；schema 版本由后端启动时自动迁移
- **检测到 schema 版本落后会先做一份一致性快照**，落在同目录的 `backups/` 下，
  命名 `soloup-<时间戳>-v<原版本>.db`，保留最近 10 份
- 快照用 `VACUUM INTO` 生成，**不要用文件复制代替** —— WAL 模式下直接拷 `.db`
  会丢掉还在 `-wal` 里未合并的写入
- 备份失败会拒绝启动（逃生舱：环境变量 `SOLOUP_SKIP_BACKUP=1`）
- 面板的导入/导出（设置页）是另一条通道，产出的是 JSON 全量数据

## CI/CD 与发布

推送 `v*` tag 会自动触发 GitHub Actions：在 windows-latest 上构建前端、编译后端
sidecar、打包 NSIS 安装器，并把产物发布到 [Releases](https://github.com/Roxy-DD/soloup/releases)。

```bash
# 1) 改版本号（四处：package.json / src-tauri/tauri.conf.json /
#    src-tauri/Cargo.toml / backend/Cargo.toml 的 workspace.package.version）
# 2) 更新 CHANGELOG.md
# 3) 提交并推 tag
git tag v1.2.0
git push origin main --tags
```

也可以在 Actions 页面手动 `workflow_dispatch` 触发。

## 文档

| 文档 | 内容 |
|---|---|
| [CHANGELOG.md](CHANGELOG.md) | 各版本变更记录 |
| [docs/BUILD.md](docs/BUILD.md) | 构建、打包、局域网部署、常见问题 |
| [docs/soloupmath.md](docs/soloupmath.md) | 技术规格与决策记录（数学模型、参数表、MCP 工具清单、黄金测试用例） |

> `soloupmath.md` 是**带日期的决策日志**，记录了项目从早期 TypeScript 方案演进到
> 当前 Rust 架构的全过程。文档中涉及 TypeScript / better-sqlite3 的章节属于历史记录，
> 与当前实现不一致处以**代码**为准；文首有「实现现状」声明。

## 许可

[AGPL-3.0](LICENSE) —— 因为 Soloup 会把面板作为网络服务对外提供（局域网访问），
AGPL 能确保这部分能力对使用者保持开源。
