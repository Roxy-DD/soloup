#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

/// 由「开机自启动」拉起时附带的参数：只驻留托盘，不弹主窗口。
/// 与下面 `tauri_plugin_autostart::init` 里注册的参数必须一致。
const SILENT_FLAG: &str = "--silent";

struct Sidecars {
    server: Child,
    mcp: Child,
    #[cfg(windows)]
    job: JobHandle,
}

#[cfg(windows)]
struct JobHandle(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
unsafe impl Send for JobHandle {}

#[cfg(windows)]
unsafe impl Sync for JobHandle {}

impl Drop for Sidecars {
    fn drop(&mut self) {
        stop_child(&mut self.server, "soloup-server");
        stop_child(&mut self.mcp, "soloup-mcp");
        #[cfg(windows)]
        unsafe {
            if !self.job.0.is_null() {
                windows_sys::Win32::Foundation::CloseHandle(self.job.0);
            }
        }
    }
}

fn stop_child(child: &mut Child, name: &str) {
    match child.try_wait() {
        Ok(Some(_)) => return,
        Ok(None) => {}
        Err(e) => eprintln!("[soloup] 检查 {name} 进程状态失败: {e}"),
    }
    if let Err(e) = child.kill() {
        eprintln!("[soloup] 结束 {name} 进程失败: {e}");
    }
    if let Err(e) = child.wait() {
        eprintln!("[soloup] 回收 {name} 进程失败: {e}");
    }
}

fn create_sidecar_job() -> Result<JobHandle, String> {
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if job.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let ok = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of_val(&info) as u32,
        )
    };
    if ok == 0 {
        let error = std::io::Error::last_os_error().to_string();
        unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
        return Err(error);
    }
    Ok(JobHandle(job))
}

#[cfg(windows)]
fn assign_to_sidecar_job(job: &JobHandle, child: &Child) -> Result<(), String> {
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
    use windows_sys::Win32::System::Threading::OpenProcess;
    use windows_sys::Win32::System::Threading::PROCESS_SET_QUOTA;
    use windows_sys::Win32::System::Threading::PROCESS_TERMINATE;

    let process = unsafe { OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, child.id()) };
    if process.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let ok = unsafe { AssignProcessToJobObject(job.0, process) };
    unsafe { windows_sys::Win32::Foundation::CloseHandle(process) };
    if ok == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(())
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
    cmd.spawn()
        .unwrap_or_else(|e| panic!("启动 {} 失败: {}", exe.display(), e))
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

            let db_path = app.path().app_data_dir().unwrap().join("soloup.db");
            std::env::set_var("SOLOUP_DB_PATH", db_path.to_string_lossy().as_ref());

            #[cfg(windows)]
            let job = create_sidecar_job().expect("创建 sidecar 生命周期任务失败");

            let server = spawn_sidecar("soloup-server.exe", &[]);
            #[cfg(windows)]
            assign_to_sidecar_job(&job, &server).expect("绑定 soloup-server 到生命周期任务失败");
            eprintln!("[soloup] soloup-server PID: {}", server.id());
            let mcp = spawn_sidecar("soloup-mcp.exe", &["--transport", "http", "--port", "8788"]);
            #[cfg(windows)]
            assign_to_sidecar_job(&job, &mcp).expect("绑定 soloup-mcp 到生命周期任务失败");
            eprintln!("[soloup] soloup-mcp PID: {}", mcp.id());
            app.manage(Sidecars {
                server,
                mcp,
                #[cfg(windows)]
                job,
            });

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
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.state::<Arc<AtomicBool>>().store(true, Ordering::SeqCst);
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let exiting = window
                    .app_handle()
                    .try_state::<Arc<AtomicBool>>()
                    .is_some_and(|flag| flag.load(Ordering::SeqCst));
                if !exiting {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .manage(Arc::new(AtomicBool::new(false)))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
