/* ================= 应用内更新（仅桌面壳可用） =================
 *
 * 后端是 Rust 侧的三个命令（src-tauri/src/update.rs）：
 *   update_version / update_check / update_install
 * 进度通过事件 `update-progress` 回推。
 *
 * 为什么不引官方的 @tauri-apps/plugin-updater 包装包：见 update.rs 顶部注释。
 * 浏览器里（手机通过局域网打开）没有这套能力，调用一律返回「不可用」，
 * 由界面呈现成提示，而不是抛错。
 */

import { isDesktop } from './autostart';

/** 远端宣布的新版本信息 */
export type UpdateInfo = {
  current: string;
  version: string;
  notes: string | null;
  date: string | null;
};

/** 下载进度（后端事件负载） */
export type UpdateProgress = {
  phase: 'downloading' | 'finished';
  downloaded: number;
  total: number | null;
};

/** 检查更新的结果：区分「已是最新」与「检查失败」，界面提示不同 */
export type UpdateCheckResult =
  { ok: true; info: UpdateInfo | null } | { ok: false; error: string };

const PROGRESS_EVENT = 'update-progress';

async function invokeCmd<T>(cmd: string): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd);
}

/** 运行环境是否支持应用内更新 */
export function canUpdate(): boolean {
  return isDesktop();
}

/** 当前应用版本；浏览器环境返回 null */
export async function getAppVersion(): Promise<string | null> {
  if (!isDesktop()) return null;
  try {
    return await invokeCmd<string>('update_version');
  } catch (e) {
    console.error('[update] 读取版本失败', e);
    return null;
  }
}

/** 检查更新。info 为 null 表示已是最新版本。 */
export async function checkUpdate(): Promise<UpdateCheckResult> {
  if (!isDesktop()) return { ok: false, error: '仅桌面版支持应用内更新' };
  try {
    const info = await invokeCmd<UpdateInfo | null>('update_check');
    return { ok: true, info };
  } catch (e) {
    return { ok: false, error: (e as Error).message || String(e) };
  }
}

/**
 * 下载并安装更新。
 *
 * Windows 上安装阶段会启动 NSIS（`/P /R /UPDATE`：原地覆盖 + 装完自动重启），
 * 随后插件直接退出进程，所以**成功时这个 Promise 不会 resolve**。
 * 调用方据此把「没有报错也没有返回」当作正在安装，而不是卡住。
 */
export async function installUpdate(): Promise<void> {
  if (!isDesktop()) throw new Error('仅桌面版支持应用内更新');
  await invokeCmd<void>('update_install');
}

/** 订阅下载进度，返回取消订阅函数 */
export async function onUpdateProgress(cb: (p: UpdateProgress) => void): Promise<() => void> {
  const { listen } = await import('@tauri-apps/api/event');
  return listen<UpdateProgress>(PROGRESS_EVENT, (e) => cb(e.payload));
}
