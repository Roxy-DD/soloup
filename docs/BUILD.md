# 构建与分发指南

本文档描述如何从源码构建 Soloup 桌面应用并产出 Windows 分发安装包。

## 前置条件

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| Node.js | ≥ 22 | 前端构建（Next.js 15） |
| pnpm | 10.x | 包管理（workspace monorepo） |
| Rust（MSVC 工具链） | stable，`x86_64-pc-windows-msvc` | 后端 crate + Tauri 壳编译 |
| tauri-cli | 2.x（`cargo tauri --version` 验证） | 打包安装器 |

## 应用组成（三层）

```
┌─────────────────────────────────────────────┐
│ Tauri 壳（src-tauri/）                       │
│  ├─ WebView 加载 apps/web/out 静态文件        │
│  ├─ 启动 sidecar: soloup-server.exe (8787)   │ ← HTTP JSON-RPC 后端
│  ├─ 启动 sidecar: soloup-mcp.exe (8788)      │ ← MCP HTTP 服务
│  └─ 数据库指向 %APPDATA%/com.soloup.app/     │
└─────────────────────────────────────────────┘
```

- **前端**：`apps/web`（Next.js 15，`output: 'export'` 静态导出），运行时通过
  `POST http://localhost:8787/api/rpc` 调用后端（地址由 `NEXT_PUBLIC_SOLOUP_API` 控制，
  默认即此值，见 `apps/web/.env.local`）。
- **后端**：`backend/` Rust workspace，5 个 crate（core / store / solver / server / mcp）。
- **桌面壳**：`src-tauri/`，通过 `externalBin` 把两个后端 exe 作为 sidecar 打进安装包。

## 构建步骤（仓库根目录执行）

### 第 1 步：构建前端

```bash
pnpm install
pnpm --filter @soloup/web build
```

产物：`apps/web/out/`（静态文件）。Tauri 打包时以 `tauri.conf.json` 的
`frontendDist: "../apps/web/out"` 引用。

### 第 2 步：编译后端 release 二进制

```bash
cd backend
cargo build --release -p soloup-server -p soloup-mcp
```

产物：`backend/target/release/soloup-server.exe` 与 `soloup-mcp.exe`。

### 第 3 步：复制二进制到 Tauri resources（必须带目标三元组后缀）

```bash
cd ..
copy backend\target\release\soloup-server.exe src-tauri\resources\soloup-server-x86_64-pc-windows-msvc.exe
copy backend\target\release\soloup-mcp.exe    src-tauri\resources\soloup-mcp-x86_64-pc-windows-msvc.exe
```

> Tauri 的 `externalBin` 约定：找 `<名字>-<target-triple>.exe`。缺后缀会打包失败。
> 后缀以 `rustc -vV | grep host` 的输出为准（当前机器为 `x86_64-pc-windows-msvc`）。

### 第 4 步：打包

```bash
cd src-tauri
cargo tauri build
```

这一步会自动执行 `tauri.conf.json` 的 `beforeBuildCommand`（重新构建 web 前端），
然后编译 Tauri 壳并生成安装包。

产物位置：

| 产物 | 路径 |
|------|------|
| NSIS 安装器 | `src-tauri/target/release/bundle/nsis/Soloup_0.1.0_x64-setup.exe` |
| MSI 安装器（如 WiX 可用） | `src-tauri/target/release/bundle/msi/*.msi` |
| 绿色单 exe | `src-tauri/target/release/soloup-app.exe`（旁边需有两个 sidecar exe） |

只想要 NSIS：`cargo tauri build --bundles nsis`。

> 注意：第 4 步不会重新编译 backend crate，所以**改过后端代码必须先做第 2、3 步**，
> 否则打进安装包的是旧二进制。

## 一键脚本（可选）

把上述步骤串起来（PowerShell，仓库根目录执行）：

```powershell
pnpm install
pnpm --filter @soloup/web build
Push-Location backend
cargo build --release -p soloup-server -p soloup-mcp
Pop-Location
Copy-Item backend\target\release\soloup-server.exe src-tauri\resources\soloup-server-x86_64-pc-windows-msvc.exe -Force
Copy-Item backend\target\release\soloup-mcp.exe    src-tauri\resources\soloup-mcp-x86_64-pc-windows-msvc.exe -Force
Push-Location src-tauri
cargo tauri build --bundles nsis
Pop-Location
```

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

## 运行时行为（打包后）

- 首次启动：在 `%APPDATA%/com.soloup.app/soloup.db` 建库并写入演示种子数据。
- `soloup-server` 监听 `127.0.0.1:8787`（`/api/rpc` + `/health`）。
- `soloup-mcp` 以 HTTP 模式监听 `8788`（`/mcp` + `/health`），供 AI 客户端接入。
- 退出主程序不会自动结束 sidecar 进程（当前实现为 `mem::forget`，见
  `src-tauri/src/main.rs`；重复启动时旧进程会占住端口，任务管理器结束即可）。

## 常见问题

**Q：`cargo tauri build` 报找不到 `resources/soloup-server-x86_64-pc-windows-msvc.exe`？**
A：没做第 3 步，或目标三元组与当前机器不一致。

**Q：打包出的应用启动后白屏 / 网络错误？**
A：sidecar 没起来或端口被占。先看任务管理器有没有残留的 `soloup-server.exe`。

**Q：WiX 报错导致 msi 打包失败？**
A：装 WiX Toolset 或直接 `--bundles nsis` 跳过 msi。
