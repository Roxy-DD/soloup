'use client';
/* ================= 雷达图：3–8 轴动态 SVG；>8 自动降级为 LV 排行条 ================= */
import React from 'react';
import type { Attr } from '@/lib/types';
import { ATTR_LV_MAX, attrLv } from '@/lib/model';

export function RadarViewTitle(n: number): string {
  return n > 8 ? '属性排行' : n === 6 ? '六维属性' : `${n} 维属性`;
}

export function Radar({ attrs }: { attrs: Attr[] }) {
  const N = attrs.length;
  const MAX = ATTR_LV_MAX;

  // 超过 8 轴 → 排行条视图
  if (N > 8) {
    const sorted = [...attrs].sort((a, b) => attrLv(b.ep) - attrLv(a.ep));
    return (
      <div>
        <div style={{ fontSize: 12, color: 'var(--text-3)', marginBottom: 10 }}>
          属性已超过 8 项 · 雷达图自动切换为排行视图
        </div>
        {sorted.map((a) => (
          <div key={a.id} style={{ display: 'flex', alignItems: 'center', gap: 10, margin: '8px 0' }}>
            <span style={{ width: 34, fontWeight: 700, fontSize: 13, color: a.color }}>{a.name}</span>
            <span style={{ flex: 1, height: 12, border: '2px solid var(--ink)', borderRadius: 3, background: 'var(--track)', overflow: 'hidden' }}>
              <i style={{ display: 'block', height: '100%', width: `${Math.min(100, Math.round((attrLv(a.ep) / MAX) * 100))}%`, background: a.color }} />
            </span>
            <b style={{ fontFamily: 'var(--px)', fontSize: 11, color: a.color, width: 30, textAlign: 'right' }}>{attrLv(a.ep)}</b>
          </div>
        ))}
      </div>
    );
  }

  // 画布加宽（360×290）+ 圆心右移，保证左侧右对齐标签不被裁切
  const cx = 180, cy = 142, R = 92;
  const pt = (i: number, r: number): [number, number] => {
    const a = -Math.PI / 2 + (i * 2 * Math.PI) / N;
    return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
  };
  const poly = (r: number) =>
    Array.from({ length: N }, (_, i) => pt(i, r).map((v) => v.toFixed(1)).join(',')).join(' ');

  const rings = [18.4, 36.8, 55.2, 73.6, 92];
  const dp = attrs
    .map((a, i) => pt(i, (R * Math.min(MAX, attrLv(a.ep))) / MAX).map((v) => v.toFixed(1)).join(','))
    .join(' ');

  return (
    <svg viewBox="0 0 360 290" width="100%" role="img" aria-label="属性雷达图">
      {rings.map((r) => (
        <polygon key={r} points={poly(r)} fill="none" stroke="#E5DCC2" strokeWidth="1.2" />
      ))}
      {attrs.map((_, i) => {
        const [x, y] = pt(i, R);
        return <line key={i} x1={cx} y1={cy} x2={x} y2={y} stroke="#E5DCC2" strokeWidth="1.2" />;
      })}
      <polygon className="rd" points={dp} fill="rgba(255,176,32,.30)" stroke="#E2483D" strokeWidth="2.5" strokeLinejoin="round" />
      {attrs.map((a, i) => {
        const r = (R * Math.min(MAX, attrLv(a.ep))) / MAX;
        const [x, y] = pt(i, r);
        return <circle key={a.id} cx={x.toFixed(1)} cy={y.toFixed(1)} r="4" fill={a.color} stroke="#1F1F1F" strokeWidth="1.5" />;
      })}
      {attrs.map((a, i) => {
        const a2 = -Math.PI / 2 + (i * 2 * Math.PI) / N;
        const c = Math.cos(a2), s = Math.sin(a2);
        const [lx, ly] = pt(i, R + 22);
        const anchor = c > 0.35 ? 'start' : c < -0.35 ? 'end' : 'middle';
        const dy = s > 0.35 ? 10 : s < -0.35 ? -2 : 4;
        return (
          <text
            key={a.id}
            x={lx.toFixed(1)}
            y={(ly + dy).toFixed(1)}
            textAnchor={anchor}
            fontSize="12"
            fontWeight="700"
            fontFamily="Noto Sans SC"
          >
            <tspan fill={a.color}>{a.name}</tspan>
            {' '}
            <tspan fill="#7A7260">LV{attrLv(a.ep)}</tspan>
          </text>
        );
      })}
    </svg>
  );
}
