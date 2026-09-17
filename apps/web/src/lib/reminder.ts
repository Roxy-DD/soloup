'use client';

import { todayKey } from './model';

async function sendNativeNotification(title: string, body: string) {
  try {
    const { isPermissionGranted, requestPermission, sendNotification } =
      await import('@tauri-apps/plugin-notification');
    let granted = await isPermissionGranted();
    if (!granted) {
      granted = (await requestPermission()) === 'granted';
    }
    if (granted) {
      sendNotification({ title, body });
      return;
    }
  } catch {
    // Tauri plugin not available, fall through to browser
  }

  if ('Notification' in window) {
    if (Notification.permission === 'default') {
      await Notification.requestPermission();
    }
    if (Notification.permission === 'granted') {
      new Notification(title, { body, icon: '/favicon.ico' });
    }
  }
}

/** 'HH:MM' → 当日第几分钟；格式非法或越界返回 null。 */
function minutesOfDay(hhmm: string): number | null {
  const m = /^(\d{1,2}):(\d{2})$/.exec(hhmm.trim());
  if (!m) return null;
  const h = Number(m[1]);
  const min = Number(m[2]);
  if (h > 23 || min > 59) return null;
  return h * 60 + min;
}

export type ReminderDeps = {
  /** 目标提醒时刻（本地时区，'HH:MM'）。返回空串或非法值表示不提醒。 */
  getRemindTime: () => string;
  /** 今天是否已经记录过；返回 true 则跳过本次提醒（避免「今天还没有记录」的误报）。 */
  hasCheckedInToday: () => boolean;
  toast: (m: string) => void;
};

/**
 * 每日打卡提醒。
 *
 * 定时器在窗口隐藏时会被 WebView 降频甚至挂起，所以判定逻辑刻意**不要求精确命中目标分钟**：
 * 只要「当前时间已过目标时刻、今天还没提醒过、今天也还没记录」，就补发一次。
 * 窗口重新可见 / 重新获得焦点时再主动判定一次，覆盖定时器被整体挂起的情况。
 */
export function startReminder({
  getRemindTime,
  hasCheckedInToday,
  toast,
}: ReminderDeps): () => void {
  if (typeof window === 'undefined') return () => {};

  const STORAGE_KEY = 'soloup_last_reminder_date';

  // 调用方把这些回调写成读最新状态的闭包，这里不再缓存快照
  const deps = { getRemindTime, hasCheckedInToday, toast };

  let timer: ReturnType<typeof setTimeout> | null = null;
  let stopped = false;

  const check = () => {
    if (stopped) return;

    const target = minutesOfDay(deps.getRemindTime());
    if (target === null) return;

    const now = new Date();
    const nowMin = now.getHours() * 60 + now.getMinutes();

    // 用「已到点」而非「正好等于」：降频会整分钟跳过，精确相等等于丢一天的提醒
    if (nowMin < target) return;

    const today = todayKey();
    if (localStorage.getItem(STORAGE_KEY) === today) return;
    if (deps.hasCheckedInToday()) return;

    const title = '人生 RPG · 每日打卡提醒';
    const msg = '今天还没有记录，快来点亮你的技能吧！';

    sendNativeNotification(title, msg).catch(() => {
      deps.toast(`打卡提醒：${msg}`);
    });

    localStorage.setItem(STORAGE_KEY, today);
  };

  // 每轮重算到下一个整分的距离，避免固定间隔累积漂移（旧的 60s interval 会慢慢滑过目标分钟）
  const schedule = () => {
    if (stopped) return;
    const now = new Date();
    const msToNextMinute = (60 - now.getSeconds()) * 1000 - now.getMilliseconds();
    timer = setTimeout(() => {
      check();
      schedule();
    }, msToNextMinute);
  };

  // 休眠 / 最小化到托盘 / 切换标签页之后回来时，立刻补一次判定
  const onWake = () => {
    if (document.visibilityState === 'visible') check();
  };
  document.addEventListener('visibilitychange', onWake);
  window.addEventListener('focus', onWake);

  schedule();

  return () => {
    stopped = true;
    if (timer) clearTimeout(timer);
    document.removeEventListener('visibilitychange', onWake);
    window.removeEventListener('focus', onWake);
  };
}
