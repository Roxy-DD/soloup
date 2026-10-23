/* ================= 内联 SVG 图标（视觉与原型一致，零依赖） ================= */
import type { ReactNode } from 'react';

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

/** 成就卡徽记（描边式）。
 *
 *  与旧版填充式图标的区别不是风格偏好，而是**用法**：徽记现在压在插画窗的
 *  渐变场景上，填充色块会把底纹整块盖掉，描边才能让场景透过去。
 *  统一 24×24、无填充、线宽由 CSS 给（`.zone-emb svg{stroke-width}`），
 *  这样金卡能单独调线宽与描边色，不必换一套图。
 */
export const ACH_SIGILS: Record<string, ReactNode> = {
  spark: (
    <>
      <path d="M12 1.8v5.6M12 16.6v5.6M1.8 12h5.6M16.6 12h5.6" />
      <path d="M4.8 4.8l3.4 3.4M15.8 15.8l3.4 3.4M19.2 4.8l-3.4 3.4M8.2 15.8l-3.4 3.4" />
    </>
  ),
  rails: (
    <>
      <path d="M6.4 2.6v18.8M17.6 2.6v18.8" />
      <path d="M6.4 6.8h11.2M6.4 12h11.2M6.4 17.2h11.2" />
    </>
  ),
  cal: (
    <>
      <rect x="3" y="5" width="18" height="16" rx="2" />
      <path d="M8 2.8v4.2M16 2.8v4.2M3 10.4h18" />
      <path d="M8.4 15h2.2M13.4 15h2.2M8.4 18h2.2" />
    </>
  ),
  sunrise: (
    <>
      <path d="M2.6 18.6h18.8" />
      <path d="M12 2.4v4.4" />
      <path d="M4.8 7.2l2.8 2.8M19.2 7.2l-2.8 2.8" />
      <path d="M6.8 18.6a5.2 5.2 0 0 1 10.4 0" />
      <path d="M9.4 21.6h5.2" />
    </>
  ),
  bricks: (
    <>
      <path d="M3 5.4h18v5.4H3zM3 13.2h18v5.4H3z" />
      <path d="M9.4 5.4v5.4M14.6 13.2v5.4" />
    </>
  ),
  hex: (
    <>
      <path d="M12 1.8l8.8 5.1v10.2L12 22.2 3.2 17.1V6.9z" />
      <path d="M12 7.6l3.8 2.2v4.4L12 16.4l-3.8-2.2V9.8z" />
    </>
  ),
  scroll: (
    <>
      <path d="M6 2.8h8.6l4 4v14.4H6z" />
      <path d="M14.6 2.8v4.2h4" />
      <path d="M9 12.6l2 2 4.2-4.4" />
    </>
  ),
  peak: (
    <>
      <path d="M1.6 20.4l6.8-13 3.7 6.9 2.6-3.6 7.7 9.7z" />
      <path d="M6.6 10.2l1.8 1.8 1.8-2.4" />
    </>
  ),
  rings: (
    <>
      <circle cx="12" cy="12" r="9.2" />
      <circle cx="12" cy="12" r="5.6" />
      <circle cx="12" cy="12" r="2.1" />
    </>
  ),
  hourglass: (
    <>
      <path d="M7 3h10M7 21h10M8 3v2.6c0 2.1 4 4.6 4 6.4s-4 4.3-4 6.4V21M16 3v2.6c0 2.1-4 4.6-4 6.4s4 4.3 4 6.4V21" />
      <path d="M10.4 17.6h3.2" />
    </>
  ),
  stack3: (
    <>
      <path d="M12 2.4l8.6 4.3L12 11 3.4 6.7z" />
      <path d="M3.4 11.6L12 15.9l8.6-4.3" />
      <path d="M3.4 16.4L12 20.7l8.6-4.3" />
    </>
  ),
  stack2: (
    <>
      <path d="M12 4.2l8.6 4.4-8.6 4.4L3.4 8.6z" />
      <path d="M3.4 14.2L12 18.6l8.6-4.4" />
    </>
  ),
  anvil: (
    <>
      <path d="M13.6 3.4l6.9 6.9-2.6 2.6-6.9-6.9z" />
      <path d="M11.8 5.2L4.4 12.6" />
      <path d="M3 20.8l7.6-7.6" />
      <path d="M2.4 21.4l1.6-1.6" />
    </>
  ),
  pillar: (
    <>
      <path d="M4 21h16" />
      <path d="M6.4 21V8.4h11.2V21" />
      <path d="M3.2 8.4h17.6L12 2.2z" />
      <path d="M10.2 21v-5.4h3.6V21" />
    </>
  ),
  flame: (
    <>
      <path d="M12 22c4.1 0 7.1-2.9 7.1-6.7 0-4.7-5.1-7.2-4.1-12-4.1 1-5.6 4.1-5.6 6.7 0 1.5.5 2.6.5 3.6-1.5-.5-2.5-2-2.5-3.1C5.4 12.4 4.9 15 4.9 15.3 4.9 19.1 7.9 22 12 22z" />
      <path d="M12 22v-4.2" />
    </>
  ),
  quadrant: (
    <>
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <path d="M12 3v18M3 12h18" />
      <circle cx="7.5" cy="7.5" r="1.2" />
      <circle cx="16.5" cy="16.5" r="1.2" />
    </>
  ),
  moon: (
    <>
      <circle cx="12" cy="12" r="9" />
      <circle cx="9.4" cy="9.6" r="1.5" />
      <circle cx="14.6" cy="13.6" r="2.1" />
      <circle cx="9.8" cy="15.4" r="1.1" />
    </>
  ),
  ingot: (
    <>
      <path d="M6.8 14.4h10.4l3.4 6.2H3.4z" />
      <path d="M9.2 14.4V8.6h5.6v5.8" />
      <path d="M8 10.4h8" />
      <path d="M9.2 3.4h5.6" />
    </>
  ),
  vessel: (
    <>
      <path d="M8.2 2.8h7.6l-1 5.2c3.1 2 4.4 4.2 4.4 7.1 0 4.1-3.3 6.1-7.2 6.1s-7.2-2-7.2-6.1c0-2.9 1.3-5.1 4.4-7.1z" />
      <path d="M8.6 12.6c1.2.9 5.6.9 6.8 0" />
    </>
  ),
  cal1: (
    <>
      <rect x="3" y="5" width="18" height="16" rx="2" />
      <path d="M8 2.8v4.2M16 2.8v4.2M3 10.4h18" />
      <circle cx="8.6" cy="15.4" r="1.6" />
    </>
  ),
  blade: (
    <>
      <path d="M3.4 20.6L17.2 6.8l3.4-3.4 1.4 1.4-3.4 3.4L4.8 22z" />
      <path d="M2.4 18.4l3.2 3.2" />
    </>
  ),
  dots3: (
    <>
      <circle cx="5.6" cy="18" r="1.9" />
      <circle cx="12" cy="12" r="2.3" />
      <circle cx="18.4" cy="6" r="2.7" />
    </>
  ),
  /* 未登记的成就（用户自定义的）走这颗星，保证任何 id 都画得出来 */
  star: <path d="M12 2.4l2.9 6.1 6.7.9-4.9 4.6 1.2 6.6L12 17.5 6.1 20.6l1.2-6.6L2.4 9.4l6.7-.9z" />,
};

export function AchIcon({ name }: { name: string }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      {ACH_SIGILS[name] ?? ACH_SIGILS.star}
    </svg>
  );
}
