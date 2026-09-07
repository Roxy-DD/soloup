'use client';
/* ================= 成就页：收集进度 + 卡牌墙（3D 翻面 + SSR v3 光效） ================= */
import React, { useMemo, useState } from 'react';
import type { AppState, Rarity, AchStats } from '@/lib/types';
import { deriveStats } from '@/lib/model';
import { ACHIEVEMENTS } from '@/lib/seed';
import { AchIcon } from './icons';

const RARITY_CODE: Record<Rarity, string> = { 普通: 'N', 稀有: 'R', 史诗: 'SR', 传说: 'SSR' };
const RARITY_COLOR: Record<Rarity, string> = { 普通: 'var(--rn)', 稀有: 'var(--rr)', 史诗: 'var(--rsr)', 传说: 'var(--rssr)' };

export function AchievementsTab({ state }: { state: AppState }) {
  const stats = useMemo(() => deriveStats(state), [state]);
  const unlockedCount = ACHIEVEMENTS.filter((a) => state.unlocked[a.id]).length;
  const total = ACHIEVEMENTS.length;

  const byRarity = (r: Rarity) => {
    const all = ACHIEVEMENTS.filter((a) => a.rarity === r);
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
              return <i key={r} style={{ width: `${(all / total) * 100}%`, background: RARITY_COLOR[r], position: 'relative' }}>
                {got > 0 && <i style={{ position: 'absolute', inset: 0, width: `${(got / all) * 100}%`, background: RARITY_COLOR[r], borderRight: '2px solid var(--card)' }} />}
              </i>;
            })}
          </div>
          <div className="coll-pct">{Math.round((unlockedCount / total) * 100)}%</div>
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
        {ACHIEVEMENTS.map((a) => {
          const unlocked = !!state.unlocked[a.id];
          const date = state.unlocked[a.id];
          return <Acard key={a.id} def={a} unlocked={unlocked} date={date} stats={stats} unlockedMap={state.unlocked} />;
        })}
      </div>
    </section>
  );
}

function Acard({ def, unlocked, date, stats, unlockedMap }: {
  def: (typeof ACHIEVEMENTS)[number];
  unlocked: boolean;
  date?: string;
  stats: AchStats;
  unlockedMap: Record<string, string>;
}) {
  const [flip, setFlip] = useState(false);
  const legendary = def.legendary && unlocked;
  const epic = def.rarity === '史诗' && unlocked;

  const depsMet = !def.requires?.length || def.requires.every((id) => !!unlockedMap[id]);
  const progText = def.prog?.(stats);
  const hasProg = !!progText;

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
    } else if (def.revealAt && hasProg) {
      const progVal = estimateProgress(stats, def);
      if (progVal >= def.revealAt) {
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
      style={{ '--rc': def.rc } as React.CSSProperties}
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
            <div className="aicon"><AchIcon name={def.icon} /></div>
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
              <span className="pxcode" style={{ fontSize: 10 }}>NO.{String(ACHIEVEMENTS.indexOf(def) + 1).padStart(3, '0')}</span>
            </div>
          </div>
        </div>
        <div className="face back">
          <div className="back-mark">人生RPG</div>
          <div className="qq">{unlocked ? '★' : deepLocked ? '🔒' : '？'}</div>
          <div style={{ fontSize: 11, color: 'var(--text-2)', textAlign: 'center', padding: '0 14px' }}>
            {unlocked ? '点击翻回正面'
              : deepLocked ? '需要先解锁前置成就'
              : hasProg ? `进度：${progText}`
              : '条件尚未达成'}
          </div>
        </div>
      </div>
      <div className="hint">点击翻面</div>
    </div>
  );
}

function estimateProgress(stats: AchStats, def: (typeof ACHIEVEMENTS)[number]): number {
  const id = def.id;
  if (id === 'first') return Math.min(stats.totalDays / 1, 1);
  if (id === 'twin') return Math.min(stats.maxLitOneDay / 2, 1);
  if (id === 'allsix') return Math.min(stats.maxLitOneDay / stats.totalAttrs, 1);
  return 0;
}
