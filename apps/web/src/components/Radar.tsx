'use client';
/* ================= 雷达图：3–8 轴动态 SVG；>8 自动降级为 LV 排行条 =================
   这张图只回答「六项之间谁强谁弱」，不承担绝对数值的职责：
   半径一律按当前最高项归一（最高项落在 94% 处），量程写在图底部，
   精确等级看右侧属性卡。早期成长时 value 只有零点几，若按满级刻度画
   会整片塌在圆心，看不出任何分布。
*/
import React from 'react';
import type { Attr } from '@/lib/types';
import { ATTR_LV_MAX } from '@/lib/model';

/** 量程下限：属性还极小时按这个量程画，避免一个点就顶满整张图。 */
const MIN_SCALE = 0.05;
/** 顶点留边：最高属性落在 94% 半径处，不贴外圈。 */
const EDGE = 0.94;

/**
 * 面板上的连续等级（0–50）。后端 value 值域 0–100、面板折半，
 * lv 是它的整数部分、frac 是级内小数 —— 两者相加才是没被取整的真实进度。
 */
const continuousLevel = (a: Attr): number => Math.max(0, Math.min(ATTR_LV_MAX, a.lv + a.frac));

export function RadarViewTitle(n: number): string {
  return n > 8 ? '属性排行' : n === 6 ? '六维属性' : `${n} 维属性`;
}

export function Radar({ attrs }: { attrs: Attr[] }) {
  const N = attrs.length;

  // 量程跟随当前最高项：任何阶段都能看出形状差异，量程值标在底部。
  const levels = attrs.map(continuousLevel);
  const scale = Math.max(Math.max(...levels, 0), MIN_SCALE);
  const ratio = levels.map((v) => Math.min(1, v / scale));

  // 超过 8 轴 → 排行条视图
  if (N > 8) {
    const rows = attrs.map((a, i) => ({ a, r: ratio[i] })).sort((x, y) => y.r - x.r);
    return (
      <div>
        <div style={{ fontSize: 12, color: 'var(--text-3)', marginBottom: 10 }}>
          属性已超过 8 项 · 雷达图自动切换为排行视图（条长为相对占比，右侧为面板等级）
        </div>
        {rows.map(({ a, r }) => (
          <div
            key={a.id}
            style={{ display: 'flex', alignItems: 'center', gap: 10, margin: '8px 0' }}
          >
            <span style={{ width: 34, fontWeight: 700, fontSize: 13, color: a.color }}>
              {a.name}
            </span>
            <span
              style={{
                flex: 1,
                height: 12,
                border: '2px solid var(--ink)',
                borderRadius: 3,
                background: 'var(--track)',
                overflow: 'hidden',
              }}
            >
              <i
                style={{
                  display: 'block',
                  height: '100%',
                  width: `${Math.round(r * 100)}%`,
                  background: a.color,
                }}
              />
            </span>
            <b
              style={{
                fontFamily: 'var(--px)',
                fontSize: 11,
                color: a.color,
                width: 30,
                textAlign: 'right',
              }}
            >
              {a.lv}
            </b>
          </div>
        ))}
      </div>
    );
  }

  // 画布加宽（360×300）+ 圆心右移，保证左侧右对齐标签不被裁切
  const cx = 180,
    cy = 142,
    R = 92;
  const pt = (i: number, r: number): [number, number] => {
    const a = -Math.PI / 2 + (i * 2 * Math.PI) / N;
    return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
  };
  const poly = (r: number) =>
    Array.from({ length: N }, (_, i) =>
      pt(i, r)
        .map((v) => v.toFixed(1))
        .join(','),
    ).join(' ');

  const rOf = (i: number) => R * EDGE * ratio[i];
  const rings = [0.2, 0.4, 0.6, 0.8, 1].map((p) => R * p);
  const dp = attrs
    .map((_, i) =>
      pt(i, rOf(i))
        .map((v) => v.toFixed(1))
        .join(','),
    )
    .join(' ');

  return (
    <svg viewBox="0 0 360 300" width="100%" role="img" aria-label="属性分布雷达图">
      {rings.map((r) => (
        <polygon key={r} points={poly(r)} fill="none" stroke="#E5DCC2" strokeWidth="1.2" />
      ))}
      {attrs.map((_, i) => {
        const [x, y] = pt(i, R);
        return <line key={i} x1={cx} y1={cy} x2={x} y2={y} stroke="#E5DCC2" strokeWidth="1.2" />;
      })}
      <polygon
        className="rd"
        points={dp}
        fill="rgba(255,176,32,.30)"
        stroke="#E2483D"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      {attrs.map((a, i) => {
        const [x, y] = pt(i, rOf(i));
        return (
          <circle
            key={a.id}
            cx={x.toFixed(1)}
            cy={y.toFixed(1)}
            r="4"
            fill={a.color}
            stroke="#1F1F1F"
            strokeWidth="1.5"
          />
        );
      })}
      {attrs.map((a, i) => {
        const a2 = -Math.PI / 2 + (i * 2 * Math.PI) / N;
        const c = Math.cos(a2),
          s = Math.sin(a2);
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
            fill={a.color}
          >
            {a.name}
          </text>
        );
      })}
      <text x="12" y="288" fontSize="11" fontFamily="Noto Sans SC" fill="#7A7260">
        按当前最高项缩放 · 量程 0–{scale.toFixed(2)} 级
      </text>
    </svg>
  );
}
