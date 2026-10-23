; ================= NSIS 安装钩子 =================
;
; 目的：让「覆盖安装」真的可用，不再需要用户先手动卸载。
;
; 背景：Soloup 关闭窗口只是隐藏到托盘，主进程与两个 sidecar（soloup-server /
; soloup-mcp）会一直活着。而 Tauri 的 NSIS 模板在检测到已装旧版时，默认走
; 「先卸载再安装」，紧接着调用 CheckIfAppIsRunning —— 一旦发现有同名进程在
; 跑，卸载就会中断，安装随之失败，用户只能手动去「应用和功能」里卸载。
;
; 这两个钩子都插在 CheckIfAppIsRunning **之前** 执行（安装段与卸载段各一次），
; 因此只要先把残留进程结束掉，覆盖安装与卸载都能顺利进行。
;
; 关于强杀：本应用的写操作一律即时落库（不存在「内存里攒着没保存」的状态），
; 强杀不会丢数据；托盘里可能正在跑的后台服务也会随进程结束被系统回收。

!macro SoloupKillRunning
  DetailPrint "正在关闭运行中的 Soloup…"
  ; 主程序（托盘常驻）
  nsExec::Exec 'taskkill /F /IM soloup-app.exe'
  Pop $0
  ; sidecar：后端与 MCP
  nsExec::Exec 'taskkill /F /IM soloup-server.exe'
  Pop $0
  nsExec::Exec 'taskkill /F /IM soloup-mcp.exe'
  Pop $0
  ; 给文件句柄一点释放时间，避免紧接着的文件复制撞上「正在使用中」
  Sleep 400
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro SoloupKillRunning
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro SoloupKillRunning
!macroend
