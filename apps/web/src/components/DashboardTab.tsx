'use client';
/* ================= 面板页：Hero + 雷达 + 属性卡 + 生命进度轴 ================= */
import React from 'react';
import type { AppState, Attr } from '@/lib/types';
import { attrFrac, attrLv, collectLeaves, parseKey, skillFrac, skillLv, tierColor, tierOf, todayKey, weekDays } from '@/lib/model';
import { Radar, RadarViewTitle } from './Radar';
import { GearIcon } from './icons';

export function DashboardTab({ state, onCheckin, onOpenAttrMgr, onOpenSettings, onOpenSkill }: {
  state: AppState;
  onCheckin: () => void;
  onOpenAttrMgr: () => void;
  onOpenSettings: () => void;
  onOpenSkill: (id: string) => void;
}) {
  const { attrs, records, settings } = state;
  const today = todayKey();
  const litToday = new Set(records.find((r) => r.date === today)?.attrsLit ?? []);
  const totalLv = Math.round(attrs.reduce((s, a) => s + attrLv(a.ep), 0) / Math.max(1, attrs.length));

  return (
    <section className="page" aria-label="角色面板">
      <div className="hero">
        <div className="avatar">{settings.avatar}</div>
        <div>
          <h1>{settings.nickname}</h1>
          <div className="sub">
            见习人生玩家 · 🔥 连续记录 <b><HeroStreak records={state.records} /></b> 天 · 本周 <b>{weekDays(records)}</b> 天
          </div>
        </div>
        <div className="lv-badge">
          <div className="n">{totalLv}</div>
          <div className="t">人生等级</div>
        </div>
        <button className="link-btn gear" onClick={onOpenSettings} aria-label="设置">
          <GearIcon />
        </button>
      </div>

      <div className="dash-grid">
        <div className="card radar-card">
          <div className="section-title" style={{ margin: '0 0 8px' }}>
            <h2>{RadarViewTitle(attrs.length)}</h2>
          </div>
          <Radar attrs={attrs} />
        </div>

        <div>
          <div className="section-title" style={{ margin: '0 0 12px' }}>
            <h2>属性面板</h2>
            <button className="link-btn" onClick={onOpenAttrMgr}>管理属性 +</button>
          </div>
          <div className="attr-list">
            {attrs.map((a) => (
              <AttrCard key={a.id} attr={a} lit={litToday.has(a.id)} />
            ))}
          </div>
          <div className="card cta-card" style={{ padding: 16 }}>
            <button className="btn-primary" onClick={onCheckin}>⚔ 记录今天 · 打卡</button>
          </div>
        </div>
      </div>

      <LifeBar settings={state.settings} />
      <TopSkills skills={state.skills} onOpenSkill={onOpenSkill} />
    </section>
  );
}

/** 连续记录天数（截止今天或昨天） */
function HeroStreak({ records }: { records: AppState['records'] }) {
  const days = new Set(records.filter((r) => r.skillIds.length > 0).map((r) => r.date));
  const key = (n: number) => {
    const d = new Date();
    d.setDate(d.getDate() - n);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  };
  let start = -1;
  if (days.has(key(0))) start = 0;
  else if (days.has(key(1))) start = 1;
  if (start < 0) return 0;
  let n = start, c = 0;
  while (days.has(key(n))) { c += 1; n += 1; }
  return c;
}

export function AttrCard({ attr, lit }: { attr: Attr; lit: boolean }) {
  const lv = attrLv(attr.ep);
  const frac = Math.round(attrFrac(attr.ep) * 100);
  return (
    <div className={`attr-card${lit ? ' lit' : ''}`} style={{ '--ac': attr.color, '--at': attr.tint } as React.CSSProperties}>
      <div className="attr-ico">{attr.name[0]}</div>
      <div className="attr-info">
        <div className="row1">
          <span className="name">{attr.name}</span>
          <span className="en">{attr.en}</span>
        </div>
        <div className="bar"><i style={{ width: `${frac}%` }} /></div>
      </div>
      <div className="attr-lv">{lv}</div>
      <span className="lit-mark">✦ 今日已点亮</span>
    </div>
  );
}

function LifeBar({ settings }: { settings: AppState['settings'] }) {
  const birth = parseKey(settings.birth);
  const end = new Date(birth.getFullYear() + settings.lifeExp, birth.getMonth(), birth.getDate());
  const now = new Date();
  const pct = Math.min(100, ((now.getTime() - birth.getTime()) / (end.getTime() - birth.getTime())) * 100);
  const days = Math.floor((now.getTime() - birth.getTime()) / 864e5);
  const fmt = (d: Date) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;

  return (
    <div className="card life-card">
      <div className="life-head">
        <h2 style={{ fontSize: 17, fontWeight: 900, display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ width: 10, height: 10, background: 'var(--red)', border: '2px solid var(--ink)', boxShadow: '2px 2px 0 var(--ink)', display: 'inline-block' }} />
          生命进度轴
        </h2>
        <div className="pct">{pct.toFixed(1)}%</div>
      </div>
      <div className="life-track">
        <div className="life-fill" style={{ width: `${pct}%` }} />
        <div className="life-ticks">{Array.from({ length: 10 }, (_, i) => <i key={i} />)}</div>
      </div>
      <div className="life-labels">
        <span>{fmt(birth)}</span>
        <span>{fmt(end)}</span>
      </div>
      <div className="life-days">
        已活过 <b>{days.toLocaleString('zh-CN')}</b> 天 · 预期寿命 {settings.lifeExp} 岁（可在设置中调整）
      </div>
    </div>
  );
}

function TopSkills({ skills, onOpenSkill }: { skills: AppState['skills']; onOpenSkill: (id: string) => void }) {
  const leaves = collectLeaves(skills);
  const top = leaves
    .map((l) => ({ leaf: l, lv: skillLv(l.uses) }))
    .sort((a, b) => b.lv - a.lv || b.leaf.uses - a.leaf.uses)
    .slice(0, 5);

  if (top.length === 0) return null;

  return (
    <div className="card top-skills-card">
      <div className="section-title" style={{ margin: '0 0 12px' }}>
        <h2 style={{ fontSize: 17, fontWeight: 900, display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ width: 10, height: 10, background: 'var(--yellow)', border: '2px solid var(--ink)', boxShadow: '2px 2px 0 var(--ink)', display: 'inline-block' }} />
          最强技能
        </h2>
      </div>
      <div className="top-skills-list">
        {top.map(({ leaf, lv }, i) => {
          const frac = Math.round(skillFrac(leaf.uses) * 100);
          return (
            <div key={leaf.id} className="top-skill-item" onClick={() => onOpenSkill(leaf.id)}
              role="button" tabIndex={0}
              onKeyDown={(e) => { if (e.key === 'Enter') onOpenSkill(leaf.id); }}
              style={{ animationDelay: `${i * 60}ms` }}>
              <span className="rank">#{i + 1}</span>
              <span className="skill-name">{leaf.name}</span>
              <span className="tier-badge" style={{ color: tierColor(lv), borderColor: tierColor(lv) }}>
                {tierOf(lv)} {lv}
              </span>
              <div className="skill-bar-wrap">
                <div className="skill-bar-fill" style={{ width: `${frac}%`, background: tierColor(lv) }} />
              </div>
              <span className="uses-count">×{leaf.uses}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
