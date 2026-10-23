//! 应用内更新：对着 GitHub Release 的 `latest.json` 检查新版本，下载并校验
//! 签名后交给 NSIS 安装器。
//!
//! 为什么不用官方的 `@tauri-apps/plugin-updater` JS 包装包：这里只需要
//! 「检查 / 安装 / 报进度」三件事，用 Rust 命令 + 事件就能覆盖，省掉一个新
//! npm 依赖（本仓库装依赖很慢），也不会动 lockfile。
//!
//! 安装阶段的行为（Windows）：插件用 `/P /R /UPDATE` 启动 NSIS 安装器 ——
//! `/UPDATE` 让安装器跳过「先卸载」那一段、直接原地覆盖，`/R` 让它在装完后
//! 重新拉起应用，随后插件自己 `std::process::exit(0)` 退出。也就是说这个
//! 命令在成功路径上**不会返回**，前端不要等它的返回值。

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// 下载进度事件名（前端 `listen('update-progress')`）。
const PROGRESS_EVENT: &str = "update-progress";

#[derive(Serialize, Clone)]
pub struct UpdateInfo {
    /// 当前运行版本
    pub current: String,
    /// 远端宣布的新版本
    pub version: String,
    /// 更新说明（来自 latest.json 的 notes）
    pub notes: Option<String>,
    /// 发布尔时间，RFC 3339 字符串
    pub date: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct UpdateProgress {
    /// downloading | finished
    pub phase: String,
    pub downloaded: u64,
    /// 服务端没给 Content-Length 时为 null
    pub total: Option<u64>,
}

fn version_of(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

/// 当前应用版本号（不联网）。设置页用它显示「当前版本 vX.Y.Z」。
#[tauri::command]
pub fn update_version(app: AppHandle) -> String {
    version_of(&app)
}

/// 检查更新。返回 `Ok(None)` 表示已是最新版本。
#[tauri::command]
pub async fn update_check(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;
    let found = updater.check().await.map_err(|e| e.to_string())?;
    Ok(found.map(|u| UpdateInfo {
        current: version_of(&app),
        version: u.version.clone(),
        notes: u.body.clone(),
        date: u.date.map(|d| d.to_string()),
    }))
}

/// 下载并安装更新。成功时进程会被安装器接管（本命令不返回）。
#[tauri::command]
pub async fn update_install(app: AppHandle) -> Result<(), String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Err("已经是最新版本".to_string());
    };

    // 两个回调都要读进度：下载中用 FnMut，收尾用 FnOnce，所以放进共享单元。
    let shared = std::sync::Arc::new(std::sync::Mutex::new((0u64, None::<u64>)));
    let progress_handle = app.clone();
    let progress_shared = shared.clone();
    let done_handle = app.clone();
    let done_shared = shared.clone();

    update
        .download_and_install(
            move |chunk_len, content_len| {
                let Ok(mut state) = progress_shared.lock() else {
                    return;
                };
                if state.1.is_none() {
                    state.1 = content_len;
                }
                state.0 += chunk_len as u64;
                let (downloaded, total) = (state.0, state.1);
                let _ = progress_handle.emit(
                    PROGRESS_EVENT,
                    UpdateProgress {
                        phase: "downloading".to_string(),
                        downloaded,
                        total,
                    },
                );
            },
            move || {
                let (done, size) = done_shared.lock().map(|s| (s.0, s.1)).unwrap_or((0, None));
                let _ = done_handle.emit(
                    PROGRESS_EVENT,
                    UpdateProgress {
                        phase: "finished".to_string(),
                        downloaded: done,
                        total: size,
                    },
                );
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
