/* ================= 内联 SVG 图标（视觉与原型一致，零依赖） ================= */
export const NAV_ICONS: Record<string, string> = {
  dashboard: 'M12 3l8 4v5c0 5-3.5 8-8 9-4.5-1-8-4-8-9V7z',
  skills: 'M12 7.2v4.3M12 11.5L6.8 16M12 11.5l5.2 4.5',
  log: 'M12 20h9M16.5 3.5a2.1 2.1 0 013 3L7 19l-4 1 1-4z',
  projects: 'M5 21V4M5 4h12l-2.5 4L17 12H5',
  achievements: 'M3 5h18v14H3zM7 9h4M7 13h7',
};

export function NavIcon({ name }: { name: string }) {
  if (name === 'skills') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="5" r="2.2" />
        <circle cx="6" cy="18" r="2.2" />
        <circle cx="18" cy="18" r="2.2" />
        <path d={NAV_ICONS.skills} />
      </svg>
    );
  }
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d={NAV_ICONS[name]} />
    </svg>
  );
}

export function GearIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="12" r="3.2" />
      <path d="M12 2.8v3M12 18.2v3M2.8 12h3M18.2 12h3M5.5 5.5l2.1 2.1M16.4 16.4l2.1 2.1M18.5 5.5l-2.1 2.1M7.6 16.4l-2.1 2.1" />
    </svg>
  );
}

/** 成就卡图标（填充式，与原型 path 一致） */
export const ACH_ICONS: Record<string, string> = {
  star: 'M12 2l2.4 6.9H22l-5.8 4.3 2.2 6.8L12 15.8 5.6 20l2.2-6.8L2 8.9h7.6z',
  twin: 'M4 12h6M14 12h6M7 7l10 10',
  moon: 'M21 14.5A8.5 8.5 0 1110 3.5 7 7 0 0021 14.5z',
  sun: 'M12 8a4 4 0 100 8 4 4 0 000-8zM12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M19.1 4.9L17 7M7 17l-2.1 2.1',
  mountain: 'M3 20L10 7l4 7 3-4 4 10z',
  crown: 'M3 18l2-10 4.5 4L12 5l2.5 7L19 8l2 10z',
  scroll: 'M6 4h12v16l-6-4-6 4z',
  gem: 'M12 3l7 6-7 12L5 9z',
};

export function AchIcon({ name }: { name: string }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d={ACH_ICONS[name] ?? ACH_ICONS.star} />
    </svg>
  );
}
