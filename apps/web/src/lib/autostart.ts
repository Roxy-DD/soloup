/* ================= 开机自启动（仅桌面壳可用） =================
 *
 * 为什么状态不存进 soloup.db：注册表里的那条 Run 记录本身就是唯一真相。
 * 再往库里存一份，就会出现「库说开着、系统其实关了」的漂移 —— 用户可能
 * 在任务管理器的「启动」页禁用、重装系统后条目消失、或直接用别的账号登录。
 * 所以这里读写都直接对着系统，库里不落字段，也就没有迁移。
 *
 * 浏览器里没有这个能力（手机通过局域网打开的页面同样如此），读取一律返回
 * null，交给界面呈现为「不可用」，而不是抛错。
 */

/** 是否跑在 Tauri 桌面壳里。手机浏览器打开同一页面时为 false。 */
export function isDesktop(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * 调 autostart 插件命令。
 *
 * 这里直接用 invoke 而不引官方 JS 包装包：包装层总共就三行 invoke，多一个
 * 依赖不划算（这个仓库装依赖很慢）。
 * 命令名取自插件源码 `PluginBuilder::new("autostart")` + `generate_handler!`：
 * enable / disable / is_enabled。
 */
async function invokePlugin<T>(cmd: string): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd);
}

/**
 * 读取系统里真实的自启动状态。
 * @returns 桌面壳内返回 true / false；非桌面环境返回 null（能力不可用）
 */
export async function getAutostart(): Promise<boolean | null> {
  if (!isDesktop()) return null;
  try {
    return await invokePlugin<boolean>('plugin:autostart|is_enabled');
  } catch (e) {
    console.error('[autostart] 读取失败', e);
    return null;
  }
}

/**
 * 开启 / 关闭开机自启动。
 *
 * 立即写系统并生效，与设置面板的「保存设置」无关（同局域网开关的做法）。
 * @returns 是否写入成功；非桌面环境恒为 false
 */
export async function setAutostart(enabled: boolean): Promise<boolean> {
  if (!isDesktop()) return false;
  try {
    await invokePlugin(`plugin:autostart|${enabled ? 'enable' : 'disable'}`);
    return true;
  } catch (e) {
    console.error('[autostart] 写入失败', e);
    return false;
  }
}
