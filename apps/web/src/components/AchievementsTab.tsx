'use client';
/* ================= 成就页：收集进度 + 卡牌墙（3D 翻面 + SSR v3 光效） =================
   成就定义（名称 / 描述 / 稀有度 / 隐藏与前置 / 达成条件）全部来自后端 achievements 表，
   前端只保留两类展示映射：稀有度 → 配色与代号、成就 id → 图标 key。
   进度文案由条件 DSL 通用生成（见 model.achProgress），不再按 id 写死。 */
import React, { useMemo, useState } from 'react';
import type { AchStats, AchievementDef, AppState, Rarity } from '@/lib/types';
import { achProgress, deriveStats } from '@/lib/model';
import { AchIcon } from './icons';

const RARITY_CODE: Record<Rarity, string> = { 普通: 'N', 稀有: 'R', 史诗: 'SR', 传说: 'SSR' };
const RARITY_COLOR: Record<Rarity, string> = { 普通: 'var(--rn)', 稀有: 'var(--rr)', 史诗: 'var(--rsr)', 传说: 'var(--rssr)' };
/** 卡面主色（CSS 变量 --rc），与稀有度一一对应 */
const RARITY_RC: Record<Rarity, string> = { 普通: '#6B7280', 稀有: '#2F6FED', 史诗: '#7C3AED', 传说: '#FFB020' };
/** 成就 id → 图标 key（icons.tsx）；纯展示资源，未登记的（自定义）成就走默认星形 */
const ACH_ICON: Record<string, string> = {
  first: 'star', twin: 'twin', week7: 'moon', dawn: 'sun', d100: 'mountain',
  allsix: 'gem', done1: 'scroll', grand: 'crown', tenk: 'scroll',
};
const iconOf = (id: string) => ACH_ICON[id] ?? 'star';

export function AchievementsTab({ state }: { state: AppState }) {
  const stats = useMemo(() => deriveStats(state), [state]);
  // 后端按 type/rarity/name 排序下发；面板改按难度分（points）升序摆放，梯度从易到难
  const list = useMemo(
    () => [...state.achievements].sort((a, b) => a.points - b.points || a.id.localeCompare(b.id)),
    [state.achievements],
  );
  const total = list.length;
  const unlockedCount = list.filter((a) => state.unlocked[a.id]).length;

  const byRarity = (r: Rarity) => {
    const all = list.filter((a) => a.rarity === r);
    const got = all.filter((a) => state.unlocked[a.id]).length;
    return { all: all.length, got };
  };

  return (
    <section className="page" aria-label="成就卡册">
      <div className="section-title" style={{ marginTop: 4 }}><h2>成就卡册</h2></div>
      <div className="card" style={{ marginBottom: 18 }}>
        <div className="coll-head">
          <div className="coll-num">{unlockedCount}<small> / {total}</small></div>
          <div className="coll-bar">
            {(['普通', '稀有', '史诗', '传说'] as Rarity[]).map((r) => {
              const { all, got } = byRarity(r);
              const w = total > 0 ? (all / total) * 100 : 0;
              return <i key={r} style={{ width: `${w}%`, background: RARITY_COLOR[r], position: 'relative' }}>
                {got > 0 && all > 0 && <i style={{ position: 'absolute', inset: 0, width: `${(got / all) * 100}%`, background: RARITY_COLOR[r], borderRight: '2px solid var(--card)' }} />}
              </i>;
            })}
          </div>
          <div className="coll-pct">{total > 0 ? Math.round((unlockedCount / total) * 100) : 0}%</div>
        </div>
        <div className="coll-legend">
          {(['普通', '稀有', '史诗', '传说'] as Rarity[]).map((r) => {
            const { all, got } = byRarity(r);
            return (
              <span key={r}>
                <i style={{ background: RARITY_COLOR[r] }} />
                {RARITY_CODE[r]} {r} ×{got}/{all}
              </span>
            );
          })}
        </div>
      </div>

      <div className="card-wall">
        {list.map((a, index) => (
          <Acard
            key={a.id}
            def={a}
            no={index + 1}
            unlocked={!!state.unlocked[a.id]}
            date={state.unlocked[a.id]}
            stats={stats}
            unlockedMap={state.unlocked}
          />
        ))}
      </div>
    </section>
  );
}

function Acard({ def, no, unlocked, date, stats, unlockedMap }: {
  def: AchievementDef;
  no: number;
  unlocked: boolean;
  date?: string;
  stats: AchStats;
  unlockedMap: Record<string, string>;
}) {
  const [flip, setFlip] = useState(false);
  const legendary = def.rarity === '传说' && unlocked;
  const epic = def.rarity === '史诗' && unlocked;

  const depsMet = def.requires.length === 0 || def.requires.every((id) => !!unlockedMap[id]);
  const prog = achProgress(stats, def.condition);

  let showName: boolean;
  let showDesc: boolean;
  let deepLocked = false;

  if (unlocked) {
    showName = true;
    showDesc = true;
  } else if (def.hidden) {
    if (!depsMet) {
      showName = false;
      showDesc = false;
      deepLocked = true;
    } else if (def.revealAt != null && prog) {
      if (prog.ratio >= def.revealAt) {
        showName = true;
        showDesc = true;
      } else {
        showName = true;
        showDesc = false;
      }
    } else {
      showName = false;
      showDesc = false;
    }
  } else {
    showName = true;
    showDesc = true;
  }

  const footLabel = unlocked ? date : deepLocked ? 'LOCKED' : (def.hidden ? '???' : '未解锁');

  return (
    <div
      className={`acard${flip ? ' flip' : ''}${unlocked ? '' : ' locked'}${legendary ? ' legendary' : ''}${epic ? ' epic' : ''}${deepLocked ? ' deep-locked' : ''}`}
      style={{ '--rc': RARITY_RC[def.rarity] } as React.CSSProperties}
      onClick={() => setFlip((f) => !f)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === 'Enter') setFlip((f) => !f); }}
      aria-label={`成就卡牌：${def.name}`}
    >
      <div className="acard-inner">
        <div className="face front">
          {legendary && (
            <>
              <div className="ssr-glow" /><div className="ssr-shine" />
              <div className="ssr-spark s1" /><div className="ssr-spark s2" /><div className="ssr-spark s3" />
            </>
          )}
          <div className={`rtag${def.rarity === '传说' ? ' rtag-ink' : ''}`}>{RARITY_CODE[def.rarity]}</div>
          <div className="face-mat">
            <div className="gem" />
            <div className="aicon"><AchIcon name={iconOf(def.id)} /></div>
            <div className="aname">{showName ? def.name : '？？？'}</div>
            <div className="rarity-tag">{def.rarity} · <span className="pxcode">{RARITY_CODE[def.rarity]}</span></div>
            <div className="adesc">
              {unlocked ? def.desc
                : showDesc ? def.desc
                : showName ? '似乎快要浮出水面…'
                : '与某张卡牌存在关联…'}
            </div>
            <div className="afoot">
              <span>{footLabel}</span>
              <span className="pxcode" style={{ fontSize: 10 }}>NO.{String(no).padStart(3, '0')}</span>
            </div>
          </div>
        </div>
        <div className="face back">
          <div className="back-mark">人生RPG</div>
          <div className="qq">{unlocked ? '★' : deepLocked ? '🔒' : '？'}</div>
          <div style={{ fontSize: 11, color: 'var(--text-2)', textAlign: 'center', padding: '0 14px' }}>
            {unlocked ? '点击翻回正面'
              : deepLocked ? '需要先解锁前置成就'
              : prog ? `进度：${prog.text}`
              : '条件尚未达成'}
          </div>
        </div>
      </div>
      <div className="hint">点击翻面</div>
    </div>
  );
}
