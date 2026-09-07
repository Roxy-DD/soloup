'use client';
/* ================= 记录页：热力图（真实记录）+ 今日建议（弱项属性/推荐技能）+ 勾选技能 + 点亮预览 + 提交 ================= */
import React, { useMemo, useState } from 'react';
import type { AppState, Attr, DayRecord, SkillLeaf } from '@/lib/types';
import { collectLeaves, attrMapOf, todayKey, todaySuggestions, weekDays, deriveStats } from '@/lib/model';
import { useToast } from './Toast';

export function CheckinTab({ state, dispatch }: {
  state: AppState;
  dispatch: (a: { type: 'SAVE_RECORD'; date: string; skillIds: string[] }) => void;
}) {
  const toast = useToast();
  const today = todayKey();
  const leaves = useMemo(() => collectLeaves(state.skills), [state.skills]);
  const am = attrMapOf(state.attrs);
  const todayRecord = state.records.find((r) => r.date === today);

  const [selected, setSelected] = useState<Set<string>>(new Set(todayRecord?.skillIds ?? []));
  const [editing, setEditing] = useState(!todayRecord);
  const [saving, setSaving] = useState(false);

  const toggle = (id: string) => {
    if (!editing) { toast('点击下方按钮进入编辑状态'); return; }
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const litAttrs = useMemo(() => {
    const set = new Set<string>();
    for (const id of selected) {
      const leaf = leaves.find((l) => l.id === id);
      if (leaf) for (const [aid] of leaf.attrs) if (am[aid]) set.add(aid);
    }
    return [...set];
  }, [selected, leaves, am]);

  // 今日建议：弱项属性 + 推荐技能（model.todaySuggestions 是纯函数，便于复用/单测）。
  const suggestions = useMemo(() => {
    const litSet = todayRecord
      ? new Set(todayRecord.attrsLit)
      : new Set(litAttrs);
    return todaySuggestions(state, selected, litSet);
  }, [state, selected, litAttrs, todayRecord]);

  // 热力图用：叶子 id→名字、统计（本周/连击/近 12 周累计）
  const leafNames = useMemo(() => Object.fromEntries(leaves.map((l) => [l.id, l.name])), [leaves]);
  const dStats = useMemo(() => {
    const s = deriveStats(state);
    return { week: weekDays(state.records), streak: s.streak, total: s.totalDays };
  }, [state]);

  const submit = () => {
    if (!selected.size) { toast('至少选择 1 个技能'); return; }
    setSaving(true);
    setTimeout(() => {
      dispatch({ type: 'SAVE_RECORD', date: today, skillIds: [...selected] });
      setSaving(false);
      setEditing(false);
      toast(`已点亮 ${litAttrs.length} 项属性 · ${selected.size} 项技能熟练度 +1`);
    }, 700);
  };

  const todayStr = new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric', weekday: 'long' });

  return (
    <section className="page" aria-label="每日记录">
      <div className="section-title" style={{ marginTop: 4 }}><h2>每日记录</h2></div>
      <div className="date-bar">
        <div className="d">{todayStr}</div>
        <div className="streak">🔥 今日 {todayRecord ? '已记录' : '未记录'}</div>
      </div>

      <div className="card heat-card" style={{ marginBottom: 14 }}>
        <div className="heat-card-row">
          <div className="heat-side">
            <div style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 4, fontWeight: 700 }}>
              近 12 周记录热力 · 悬停/点格子看当天明细
            </div>
            <HeatMap records={state.records} am={am} leafNames={leafNames} stats={dStats} />
          </div>
          <div className="sug-side" aria-label="今日建议">
            <div className="sug-title">今日建议</div>
            <Suggestions weak={suggestions.weak} skills={suggestions.skills} am={am} onPick={(id) => toggle(id)} editing={editing} />
          </div>
        </div>
      </div>

      <div className="card">
        <div style={{ fontWeight: 900, marginBottom: 4 }}>
          {editing ? '勾选今天使用过的技能' : '今日已提交 · 可再次编辑更新'}
        </div>
        {state.skills.map((g) => {
          const groupLeaves = leaves.filter((l) => containsLeaf(g, l));
          if (!groupLeaves.length) return null;
          return (
            <div key={g.id}>
              <div className="group-label">{g.name}</div>
              <div className="chips">
                {groupLeaves.map((l) => (
                  <button key={l.id} className={`chip${selected.has(l.id) ? ' on' : ''}`}
                    aria-pressed={selected.has(l.id)} onClick={() => toggle(l.id)}>
                    {l.name}
                  </button>
                ))}
              </div>
            </div>
          );
        })}
        <div className="preview">
          本次将点亮 →{' '}
          {selected.size === 0
            ? <span style={{ color: 'var(--text-3)' }}>（尚未选择技能）</span>
            : litAttrs.map((aid) => (
              <span key={aid} className="lit-chip" style={{ background: am[aid]?.tint, color: am[aid]?.color }}>
                {am[aid]?.name ?? aid}
              </span>
            ))}
          {litAttrs.length > 0 && <span style={{ fontSize: 11, color: 'var(--text-3)' }}>（重复属性每日只 +1）</span>}
        </div>
        <button className="btn-primary" onClick={editing ? submit : () => { setEditing(true); toast('进入编辑状态 · 修改后再次提交将更新记录'); }} disabled={saving}>
          {saving
            ? <>记录中<span className="px-load"><i /><i /><i /></span></>
            : editing ? '提交今日记录' : '✓ 编辑今日记录'}
        </button>
      </div>
    </section>
  );
}

/** 判断叶子是否属于组（任意层级） */
function containsLeaf(group: AppState['skills'][number], leaf: { id: string }): boolean {
  const walk = (list: AppState['skills'][number]['ch']): boolean =>
    list.some((n) => 'ch' in n ? walk(n.ch) : n.id === leaf.id);
  return walk(group.ch);
}

/**
 * 近 12 周记录热力图（真实绑定 records，非占位）：
 * - 按周分列、顶部标注换月；今天描边高亮；未来日期淡化。
 * - 悬停（移动端点按）格子 → 下方详情条显示当天点亮属性与使用技能。
 */
function HeatMap({ records, am, leafNames, stats }: {
  records: AppState['records'];
  am: Record<string, Attr | undefined>;
  leafNames: Record<string, string>;
  stats: { week: number; streak: number; total: number };
}) {
  const today = todayKey();
  const { cols, byDate } = useMemo(() => {
    const map = new Map(records.map((r) => [r.date, r]));
    const now = new Date();
    const dow = (now.getDay() + 6) % 7; // 周一=0
    const p = (x: number) => String(x).padStart(2, '0');
    const out: { cells: { key: string; rec: DayRecord | undefined; future: boolean }[]; head: string }[] = [];
    for (let w = 0; w < 12; w++) {
      const day0 = new Date(now.getFullYear(), now.getMonth(), now.getDate() - dow - (11 - w) * 7);
      const cells: { key: string; rec: DayRecord | undefined; future: boolean }[] = [];
      for (let d = 0; d < 7; d++) {
        const day = new Date(day0.getFullYear(), day0.getMonth(), day0.getDate() + d);
        const key = `${day.getFullYear()}-${p(day.getMonth() + 1)}-${p(day.getDate())}`;
        cells.push({ key, rec: map.get(key), future: day.getTime() > now.getTime() });
      }
      out.push({ head: day0.getDate() <= 7 ? `${day0.getMonth() + 1}月` : '', cells });
    }
    return { cols: out, byDate: map };
  }, [records]);

  const [hoverKey, setHoverKey] = useState<string | null>(null);
  const [pinKey, setPinKey] = useState<string | null>(null);
  const show = pinKey ?? hoverKey;
  const shown = show ? byDate.get(show) : undefined;

  const cls = (c: number) => (c >= 5 ? 'h4' : c === 4 ? 'h3' : c >= 2 ? 'h2' : c === 1 ? 'h1' : '');
  const dLabel = (key: string) => {
    const [y, m, d] = key.split('-').map(Number);
    const wd = ['日', '一', '二', '三', '四', '五', '六'][new Date(y, (m || 1) - 1, d || 1).getDay()];
    return `${m}月${d}日 周${wd}`;
  };
  const sNames = shown ? shown.skillIds.map((i) => leafNames[i]).filter(Boolean) : [];
  const aNames = shown ? shown.attrsLit.map((i) => am[i]).filter((a): a is Attr => !!a) : [];

  return (
    <>
      <div className="heat-stats" aria-label="记录统计">
        <span className="heat-stat">本周打卡 <b>{stats.week}/7</b></span>
        <span className="heat-stat">当前连续 <b>{stats.streak}</b> 天</span>
        <span className="heat-stat">近12周累计 <b>{stats.total}</b> 天</span>
      </div>

      <div className="heat-detail" aria-live="polite">
        {show ? (
          shown ? (
            <>
              <span className="dt">{dLabel(show)}</span>
              <span>点亮 <b className="dt">{shown.attrsLit.length}</b> 项</span>
              {aNames.map((a) => (
                <span key={a.id} className="chip-mini" style={{ background: a.tint, color: a.color }}>{a.name}</span>
              ))}
              {sNames.length > 0 && <span className="dim">技能：{sNames.join('、')}</span>}
            </>
          ) : (
            <><span className="dt dim">{dLabel(show)}</span><span className="dim">未打卡</span></>
          )
        ) : (
          <span className="dim">悬停或点按格子查看当天明细</span>
        )}
      </div>

      <div className="heat-cols" aria-label="近 12 周记录热力图" onMouseLeave={() => setHoverKey(null)}>
        {cols.map((col, wi) => (
          <div className="heat-col" key={wi}>
            <div className="heat-col-head">{col.head}</div>
            {col.cells.map((c) => (
              <button
                key={c.key}
                type="button"
                className={['heat-cell', c.future ? 'future' : '', c.rec ? cls(c.rec.attrsLit.length) : '', c.key === today ? 'today' : ''].join(' ').trim()}
                aria-label={c.key}
                aria-pressed={pinKey === c.key}
                onClick={() => setPinKey((k) => (k === c.key ? null : c.key))}
                onMouseEnter={() => setHoverKey(c.key)}
              />
            ))}
          </div>
        ))}
      </div>

      <div className="heat-legend">
        <span><i style={{ background: '#EFE7D2', boxShadow: 'inset 0 0 0 1px rgba(31,31,31,.06)' }} />无</span>
        <span><i style={{ background: '#FFE3A8' }} />1</span>
        <span><i style={{ background: '#FFB020' }} />2–3</span>
        <span><i style={{ background: '#F97B4F' }} />4</span>
        <span><i style={{ background: '#E2483D' }} />5–6</span>
        <span className="leg-today"><i className="heat-cell today" />今天</span>
      </div>
    </>
  );
}

/**
 * 今日建议卡：左侧"弱项属性"（低 EP、未点亮） + 右侧"推荐技能"（覆盖弱项最多）。
 * 推荐技能可点击直接加入勾选；编辑态下方说明。无数据时给出友好提示。
 */
function Suggestions({
  weak,
  skills,
  am,
  onPick,
  editing,
}: {
  weak: Attr[];
  skills: SkillLeaf[];
  am: Record<string, Attr | undefined>;
  onPick: (id: string) => void;
  editing: boolean;
}) {
  const hasAttrs = weak.length > 0;
  const hasSkills = skills.length > 0;

  return (
    <div className="sug-body">
      <div className="sug-block">
        <div className="sug-h">弱项属性</div>
        {hasAttrs ? (
          <ul className="sug-attrs">
            {weak.map((a) => (
              <li key={a.id}>
                <span className="sug-dot" style={{ background: a.color }} />
                <span className="sug-name">{a.name}</span>
                <span className="sug-lv" style={{ color: a.color }}>Lv {Math.floor(5 * Math.log(1 + Math.max(0, a.ep) / 3))}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="sug-empty">还没有属性，先到「属性」抽屉里创建吧。</p>
        )}
      </div>

      <div className="sug-block">
        <div className="sug-h">推荐技能</div>
        {hasSkills ? (
          <ul className="sug-skills">
            {skills.map((l) => (
              <li key={l.id}>
                <button
                  type="button"
                  className="sug-chip"
                  onClick={() => onPick(l.id)}
                  disabled={!editing}
                  title={editing ? '点击加入今日勾选' : '进入编辑后可加入'}
                >
                  <span className="sug-chip-name">{l.name}</span>
                  <span className="sug-chip-tags">
                    {l.attrs
                      .filter(([aid]) => am[aid])
                      .map(([aid, w]) => (
                        <span key={aid} className="sug-tag" style={{ color: am[aid]?.color, background: am[aid]?.tint }}>
                          {am[aid]?.name}·{Math.round(w * 100)}%
                        </span>
                      ))}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="sug-empty">{hasAttrs ? '暂无针对弱项的未勾选技能。' : '添加技能后将自动生成建议。'}</p>
        )}
      </div>
    </div>
  );
}
