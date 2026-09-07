'use client';

async function sendNativeNotification(title: string, body: string) {
  try {
    const { isPermissionGranted, requestPermission, sendNotification } = await import('@tauri-apps/plugin-notification');
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

export type ReminderDeps = {
  getRemindTime: () => string;
  toast: (m: string) => void;
};

export function startReminder({ getRemindTime, toast }: ReminderDeps): () => void {
  if (typeof window === 'undefined') return () => {};

  const STORAGE_KEY = 'soloup_last_reminder_date';
  const today = () => new Date().toISOString().slice(0, 10);

  const getRemindTimeRef = { current: getRemindTime };
  const toastRef = { current: toast };
  getRemindTimeRef.current = getRemindTime;
  toastRef.current = toast;

  let timer: ReturnType<typeof setInterval> | null = null;

  const check = () => {
    const now = new Date();
    const hhmm = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const target = getRemindTimeRef.current();
    if (hhmm !== target) return;

    const lastDate = localStorage.getItem(STORAGE_KEY);
    if (lastDate === today()) return;

    const title = '人生 RPG · 每日打卡提醒';
    const msg = '今天还没有记录，快来点亮你的技能吧！';

    sendNativeNotification(title, msg).catch(() => {
      toastRef.current(`打卡提醒：${msg}`);
    });

    localStorage.setItem(STORAGE_KEY, today());
  };

  // Align to next whole-minute boundary, then fire every 60s
  const now = new Date();
  const msToNextMinute = (60 - now.getSeconds()) * 1000 - now.getMilliseconds();
  const initialTimer = setTimeout(() => {
    check();
    timer = setInterval(check, 60_000);
  }, msToNextMinute);

  return () => {
    clearTimeout(initialTimer);
    if (timer) clearInterval(timer);
  };
}
