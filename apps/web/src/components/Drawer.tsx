'use client';
/* ================= 抽屉外壳：移动端底部滑出 / 桌面端右侧滑入 ================= */
import React from 'react';

export function Drawer({ open, title, onClose, children, label }: {
  open: boolean;
  title: string;
  onClose: () => void;
  children: React.ReactNode;
  label: string;
}) {
  return (
    <>
      <div className={`drawer-mask${open ? ' show' : ''}`} onClick={onClose} aria-hidden="true" />
      <aside className={`drawer-panel${open ? ' show' : ''}`} aria-label={label} aria-hidden={!open}>
        <div className="drawer-head">
          <h3>{title}</h3>
          <button className="x" onClick={onClose} aria-label="关闭">✕</button>
        </div>
        {children}
      </aside>
    </>
  );
}
