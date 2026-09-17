# 构建与分发指南

本文档描述如何从源码构建 Soloup 桌面应用、产出 Windows 分发安装包，以及局域网部署与常见问题排查。

> 各版本改了什么，见仓库根目录的 [CHANGELOG.md](../CHANGELOG.md)。

## 前置条件

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| Node.js | ≥ 22 | 前端构建（Next.js 15） |
| pnpm | 10.x | 包管理（workspace monorepo） |
| Rust（MSVC 工具链） | stable，`x86_64-pc-windows-msvc` | 后端 crate + Tauri 壳编译 |
| tauri-cli | 2.x（`cargo tauri --version` 验证） | 打包安装器 |
| UPX | 可选，默认路径 `D:\soft\upx-5.0.2-win64\upx.exe` | 压缩主 exe，可用 `UPX_BIN` 覆盖；缺失则自动跳过 |

## 应用组成（三层）

```
┌──────────────────────────────────────────────────────────┐
│ Tauri 壳（src-tauri/）                                     │
│  ├─ WebView 加载 apps/web/out 静态文件                     │
│  ├─ 启动 sidecar: soloup-server.exe   0.0.0.0:8787        │
│  │     /api/rpc  JSON-RPC 后端                             │
│  │     /         托管前端静态页（局域网设备访问）             │
│  └─ 启动 sidecar: soloup-mcp.exe      127.0.0.1:8788      │
│        /mcp      MCP over HTTP                             │
│  数据库指向 %APPDATA%/com.soloup.app/                      │
└──────────────────────────────────────────────────────────┘
```

- **前端**：`apps/web`（Next.js 15，`output: 'export'` 静态导出）。运行时通过
  `POST <当前页面主机>:8787/api/rpc` 调用后端 —— 地址由 `src/lib/api.ts` 的
  `hostOf()` 在**运行时**按页面 hostname 反推（这样手机、桌面壳、next dev 三种
  场景都能自动对上）。仅当后端不在页面所在主机时，才需要用
  `NEXT_PUBLIC_SOLOUP_API` 强制覆盖，见 `apps/web/.env.example`。
- **后端**：`backend/` Rust workspace，5 个 crate（core / store / solver / server / mcp）。
- **桌面壳**：`src-tauri/`，通过 `externalBin` 把两个后端 exe 作为 sidecar 打进安装包，
  并用 `bundle.resources` 一并打包 `resources/web`。

### 后端环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SOLOUP_PORT` | `8787` | 后端监听端口 |
| `SOLOUP_DB_PATH` | `~/.soloup/soloup.db` | 数据库路径（Tauri 壳会改写为 app data 目录） |
| `SOLOUP_WEB_DIR` | 自动探测 `apps/web/out` | 前端静态产物目录 |
| `SOLOUP_SKIP_BACKUP` | 未设置 | 设为 `1` 时跳过迁移前自动备份 |
| `SOLOUP_TARGET_TRIPLE` | 由 `rustc -vV` 推断 | 复制 sidecar 时的目标三元组后缀 |

## 一键构建（推荐）

仓库根目录执行：

```bash
pnpm install
pnpm build:dist
```

`build:dist` 串起完整链路：

1. `pnpm build` — 前端静态导出到 `apps/web/out/`
2. `pnpm build:backend` — `cargo build --release -p soloup-server -p soloup-mcp`
3. `node scripts/copy-sidecars.mjs` — 复制两个 sidecar（带目标三元组后缀）到
   `src-tauri/resources/`，并把 `apps/web/out` 复制到 `src-tauri/resources/web`
4. `cargo tauri build` — 编译 Tauri 壳并生成 NSIS 安装包
5. `node scripts/upx-compress.mjs` — UPX 压缩主 exe，并用 makensis 重建安装包

产物：

| 产物 | 路径 |
|------|------|
| NSIS 安装器（最终分发物） | `src-tauri/target/release/bundle/nsis/Soloup_<版本>_x64-setup.exe` |
| 绿色单 exe | `src-tauri/target/release/soloup-app.exe`（旁边需有两个 sidecar exe） |

## 分步构建（排查问题用）

### 第 1 步：构建前端

```bash
pnpm install
pnpm --filter @soloup/web build
```

产物：`apps/web/out/`。Tauri 打包时由 `tauri.conf.json` 的
`frontendDist: "../apps/web/out"` 引用。

### 第 2 步：编译后端 release 二进制

```bash
cd backend
cargo build --release -p soloup-server -p soloup-mcp
```

产物：`backend/target/release/soloup-server.exe` 与 `soloup-mcp.exe`。

### 第 3 步：复制二进制到 Tauri resources

```bash
cd ..
node scripts/copy-sidecars.mjs
```

脚本会自动完成两件事：

- 把两个后端 exe 复制成 `src-tauri/resources/<名字>-<target-triple>.exe`
- 把 `apps/web/out` 复制成 `src-tauri/resources/web`

> Tauri 的 `externalBin` 约定：找 `<名字>-<target-triple>.exe`，缺后缀会打包失败。
> 后缀默认取 `rustc -vV` 的 host（当前为 `x86_64-pc-windows-msvc`）。
>
> **第 1 步必须先做完**：如果 `apps/web/out` 不存在，脚本只会打一条 warning 然后
> 跳过 web 目录，产出的安装包就少了 `resources/web`，桌面版对外提供局域网页面
> 会失败（手机扫码打不开）。CI 里为此加了一道硬校验。

### 第 4 步：打包

```bash
cd src-tauri
cargo tauri build
```

这一步会自动执行 `tauri.conf.json` 的 `beforeBuildCommand`（重新构建 web 前端），
然后编译 Tauri 壳并生成安装包。

只想要 NSIS：`cargo tauri build --bundles nsis`。

> **注意**：第 4 步不会重新编译 backend crate。改过后端代码必须重做第 2、3 步，
> 否则打进安装包的是旧二进制。

### 第 5 步（可选）：UPX 压缩

```bash
node scripts/upx-compress.mjs
```

压缩 `soloup-app.exe` 并用 makensis 重建安装包。安装包名按
`tauri.conf.json` 的版本号自动解析，不写死版本。

## 不打包桌面版、直接跑的两种方式

**纯开发模式**（热更新前端）：

```bash
# 终端 1：后端
cd backend && cargo run -p soloup-server
# 终端 2：前端
cd apps/web && pnpm dev
```

**桌面开发模式**（Tauri 壳 + 热更新）：

```bash
cd src-tauri && cargo tauri dev
```

注意：桌面模式下 sidecar 从 `resources/` 取**上次手动复制的** exe；
`cargo tauri dev` 不会自动重编 backend。

## 局域网访问（手机 / 平板）

后端**始终**绑定 `0.0.0.0`，但非本机来源必须显式开启开关才放行：

1. 构建前端静态产物：`pnpm --filter @soloup/web build`（产出 `apps/web/out/`）
2. 启动后端：`cd backend && cargo run -p soloup-server`
3. 打开面板 → `设置 → 局域网服务` → 打开开关
4. 界面会显示 `http://<局域网IP>:8787/` 与一个**二维码**，手机相机扫一下即可打开
5. Safari 分享菜单 →「添加到主屏幕」，即成为独立应用（无地址栏）

要点：

- 网页与 API **同源同端口（8787）**，由 `soloup-server` 直接托管 `apps/web/out`，
  不需要额外的静态服务器，也不涉及跨域。
- 二维码由后端 `qrcode` crate 生成 SVG（`lan.qr` op），前端只做内联展示，
  不引入任何前端二维码库；关闭开关或无局域网地址时该接口返回 `null`。
- 关闭开关时，局域网设备仍能连上端口，但一律返回 **403**，数据不出本机。
- 开关改动**即时生效**，无需重启后端（每次请求实时读取 `settings.lan_enabled`）。
- 局域网 IP 由后端按默认路由探测，会自动跳过 Hyper-V / WSL / VMware 那类虚拟网卡。
- 若 Windows 防火墙弹窗，需允许 `soloup-server` 的**专用网络**访问。
- 打包版：`copy-sidecars.mjs` 会把 `apps/web/out` 复制到 `src-tauri/resources/web`，
  由 `tauri.conf.json` 的 `bundle.resources` 一起打进安装包，因此桌面安装版同样能
  对外提供局域网页面。

> **不要**在 `apps/web/.env.local` 里设置 `NEXT_PUBLIC_SOLOUP_API`。该变量是构建期
> 内联的，一旦写死，运行时的 hostname 反推逻辑全部失效 —— 手机上打开页面会去连
> 手机自己的 `localhost:8787`，永远连不上。默认留空即可。

## 运行时行为（打包后）

- 首次启动：在 `%APPDATA%/com.soloup.app/soloup.db` 建库并写入演示种子数据。
- `soloup-server` 监听 `0.0.0.0:8787`（`/api/rpc` + `/health` + 静态页）。
- `soloup-mcp` 监听 **`127.0.0.1:8788`**（`/mcp` + `/health`），只对回环开放，
  供本机 AI 客户端接入；手机上看到「未启动」属预期。
- 退出主程序时 `Sidecars` 的 `Drop` 会 kill 掉两个 sidecar 进程
  （见 `src-tauri/src/main.rs`）。若曾异常退出留下孤儿进程占住端口，
  在任务管理器结束 `soloup-server.exe` 即可。

## 常见问题

**Q：`cargo tauri build` 报找不到 `resources/soloup-server-x86_64-pc-windows-msvc.exe`？**
A：没做第 3 步，或目标三元组与当前机器不一致。

**Q：`LNK1104: 无法打开文件 soloup-server.exe`？**
A：上一次跑的后端进程还占着 exe。先 `Get-Process soloup-server | Stop-Process` 再编译。

**Q：打包出的应用启动后白屏 / 网络错误？**
A：sidecar 没起来或端口被占。先看任务管理器有没有残留的 `soloup-server.exe`。

**Q：手机扫码打开了页面，但数据不显示 / 一直转圈？**
A：检查 `apps/web/.env.local` 是否设了 `NEXT_PUBLIC_SOLOUP_API`（见上文警告），
然后重新 `pnpm --filter @soloup/web build`。另外确认设置里的局域网开关是开的。

**Q：`pnpm add` 新依赖卡住不动？**
A：本仓库 pnpm 在 link 阶段可能被 Windows Defender 拖死。多数新依赖其实可以
绕开（例如二维码直接用了 Rust crate，前端零新增依赖）。必要时改用
`cargo add` 处理后端依赖。

**Q：WiX 报错导致 msi 打包失败？**
A：本仓库 `tauri.conf.json` 只配了 `nsis` target，不会走 WiX。
