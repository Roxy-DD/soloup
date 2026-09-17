#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Child};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
};
use tauri_plugin_autostart::MacosLauncher;

/// 由「开机自启动」拉起时附带的参数：只驻留托盘，不弹主窗口。
/// 与下面 `tauri_plugin_autostart::init` 里注册的参数必须一致。
const SILENT_FLAG: &str = "--silent";

struct Sidecars {
    server: Child,
    mcp: Child,
}

impl Drop for Sidecars {
    fn drop(&mut self) {
        let _ = self.server.kill();
        let _ = self.mcp.kill();
    }
}

fn spawn_sidecar(name: &str, args: &[&str]) -> Child {
    let exe = std::env::current_exe()
        .expect("无法获取当前可执行文件路径")
        .parent()
        .expect("无法获取父目录")
        .join(name);
    eprintln!("[soloup] 启动: {}", exe.display());
    let mut cmd = Command::new(&exe);
    cmd.args(args);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd.spawn().unwrap_or_else(|e| panic!("启动 {} 失败: {}", exe.display(), e))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 开机自启动：Windows 下写 HKCU\Software\Microsoft\Windows\CurrentVersion\Run。
        // 注册的命令行带上 SILENT_FLAG，于是自启动拉起时不会弹窗（见 setup 里的处理）。
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![SILENT_FLAG]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .setup(|app| {
            // 开机自启拉起的实例只在托盘待命。放在 setup 最前面，尽量赶在窗口显露前隐藏。
            if std::env::args().any(|a| a == SILENT_FLAG) {
                match app.get_webview_window("main") {
                    Some(w) => {
                        let _ = w.hide();
                        eprintln!("[soloup] 开机自启：主窗口保持隐藏，仅托盘待命");
                    }
                    // 万一配置里的窗口晚于 setup 才创建，这里会取不到；此时退化成
                    // 「照常显示窗口」，只是静默失效，不会让应用起不来。
                    None => eprintln!("[soloup] 未取到主窗口，--silent 未生效（不影响启动）"),
                }
            }

            let db_path = app
                .path()
                .app_data_dir()
                .unwrap()
                .join("soloup.db");
            std::env::set_var("SOLOUP_DB_PATH", db_path.to_string_lossy().as_ref());

            let server = spawn_sidecar("soloup-server.exe", &[]);
            eprintln!("[soloup] soloup-server PID: {}", server.id());
            let mcp = spawn_sidecar("soloup-mcp.exe", &["--transport", "http", "--port", "8788"]);
            eprintln!("[soloup] soloup-mcp PID: {}", mcp.id());
            app.manage(Sidecars { server, mcp });

            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("人生 RPG 面板")
                .menu(&menu)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
