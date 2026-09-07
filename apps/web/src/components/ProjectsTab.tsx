'use client';
/* ================= 项目页：项目卡 + 事件轴 + 新建项目抽屉 + 自动能力收益 =================
   项目 = 纯时间范围。周期内每天的打卡记录自动归入项目：技能使用、属性点亮全部免绑定推导。 */
import React, { useEffect, useState } from 'react';
import type { AppState, Project } from '@/lib/types';
import { attrMapOf, projectGains, todayKey } from '@/lib/model';
import { Drawer } from './Drawer';
import { useToast } from './Toast';

export function ProjectsTab({ state, dispatch }: {
  state: AppState;
  dispatch: (a: { type: 'PROJECT_ADD'; name: string; start: string; end?: string } | { type: 'PROJECT_UPDATE'; id: string; patch: Partial<{ name: string; start: string; end: string | null; status: string; color: string | null; description: string | null }> } | { type: 'PROJECT_EVENT'; projectId: string; event: { date: string; title: string; tags: string[]; gains: [string, number][] } } | { type: 'PROJECT_FINISH'; projectId: string; end: string } | { type: 'PROJECT_DELETE'; projectId: string }) => void;
}) {
  const toast = useToast();
  const [formOpen, setFormOpen] = useState(false);
  const [name, setName] = useState('');
  const [start, setStart] = useState(todayKey());
  const [end, setEnd] = useState(''); // 空字符串 = 进行中
  // 结束项目二次确认：记录待确认的项目 id，再点一次才真正结束
  const [confirmEnd, setConfirmEnd] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [editProject, setEditProject] = useState<Project | null>(null);
  const [editName, setEditName] = useState('');
  const [editStart, setEditStart] = useState('');
  const [editEnd, setEditEnd] = useState('');
  const am = attrMapOf(state.attrs);

  useEffect(() => {
    if (formOpen) { setName(''); setStart(todayKey()); setEnd(''); }
  }, [formOpen]);

  useEffect(() => {
    if (editProject) {
      setEditName(editProject.name);
      setEditStart(editProject.start);
      setEditEnd(editProject.end ?? '');
    }
  }, [editProject]);

  const createProject = () => {
    const v = name.trim();
    if (!v) { toast('请填写项目名称'); return; }
    if (!start) { toast('请选择开始日期'); return; }
    if (end && end < start) { toast('结束日期不能早于开始日期'); return; }
    dispatch({ type: 'PROJECT_ADD', name: v, start, end: end || undefined });
    toast(end ? `项目「${v}」已创建 · ${start} → ${end}` : `项目「${v}」已创建 · ${start} 起 · 进行中`);
    setFormOpen(false);
  };

  const finishProject = (p: Project) => {
    const t = todayKey();
    dispatch({ type: 'PROJECT_FINISH', projectId: p.id, end: t });
    dispatch({ type: 'PROJECT_EVENT', projectId: p.id, event: { date: t, title: '项目完成', tags: [], gains: [] } });
    toast(`「${p.name}」已结束 · 周期 ${p.start} → ${t}`);
    setConfirmEnd(null);
  };

  const deleteProject = (p: Project) => {
    dispatch({ type: 'PROJECT_DELETE', projectId: p.id });
    toast(`「${p.name}」已删除`);
    setConfirmDelete(null);
  };

  const submitEdit = () => {
    if (!editProject) return;
    const v = editName.trim();
    if (!v) { toast('请填写项目名称'); return; }
    if (editEnd && editEnd < editStart) { toast('结束日期不能早于开始日期'); return; }
    dispatch({
      type: 'PROJECT_UPDATE',
      id: editProject.id,
      patch: { name: v, start: editStart, end: editEnd || null },
    });
    toast(`「${v}」已更新`);
    setEditProject(null);
  };

  // 最新进行中的项目 = 列表里第一个 doing（新建的项目插在最前）
  const latestDoing = state.projects.find((p) => p.status === 'doing');

  return (
    <section className="page" aria-label="项目与事件轴">
      <div className="section-title" style={{ marginTop: 4 }}>
        <h2>项目 · 事件轴</h2>
        <button className="link-btn" onClick={() => setFormOpen(true)}>新建项目 +</button>
      </div>
      <p style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 12 }}>
        项目只需设定时间范围 · 周期内打卡的技能与属性自动归入，无需手动绑定
      </p>

      {state.projects.map((p) => {
        const validDays = state.records.filter((r) =>
          r.date >= p.start && (!p.end || r.date <= p.end) && r.skillIds.length > 0).length;
        const gains = projectGains(state.records, p);
        const maxG = Math.max(1, ...gains.map((g) => g.count));
        const isLatest = latestDoing?.id === p.id;
        return (
          <div className="card" key={p.id} style={{ marginBottom: 14 }}>
            <div className="proj-head">
              <span className={`st ${p.status}`} />
              <h3>{p.name}</h3>
              {isLatest && p.status === 'doing' && (
                <span className="tag" style={{ color: 'var(--blue)', background: 'var(--blue-tint, #E8EEFC)', fontSize: 11 }}>最新</span>
              )}
            </div>
            <div className="proj-date">
              {p.start} → {p.end ?? '进行中'} · {p.status === 'done' ? '已结束' : '进行中'} · 有效打卡 {validDays} 天
            </div>
            {p.events.length > 0 && (
              <div className="timeline">
                {p.events.map((ev, i) => (
                  <div className="tl-item" key={i}>
                    <div className="t1">
                      {ev.title}
                      <span className="date">{ev.date.slice(5)}</span>
                    </div>
                    {(ev.tags.length > 0 || ev.gains.length > 0) && (
                      <div className="tags">
                        {ev.tags.map((t) => <span key={t}>{t}</span>)}
                        {ev.gains.map(([aid, n]) => (
                          <span key={aid} className="attr" style={{ '--c': am[aid]?.color } as React.CSSProperties}>
                            {am[aid]?.name ?? aid} +{n}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
            <div className="section-title" style={{ margin: '8px 0 4px' }}>
              <h2 style={{ fontSize: 13, color: 'var(--text-2)', fontWeight: 900 }}>周期内能力收益（自动统计）</h2>
            </div>
            {gains.length > 0 ? (
              <div className="gain">
                {gains.map(({ attrId, count }) => {
                  const a = am[attrId];
                  if (!a) return null;
                  return (
                    <div className="g-row" key={attrId}>
                      <span className="g-name">{a.name}</span>
                      <span className="g-bar"><i style={{ '--c': a.color, width: `${(count / maxG) * 100}%` } as React.CSSProperties} /></span>
                      <span className="g-val" style={{ '--c': a.color } as React.CSSProperties}>+{count} 天</span>
                    </div>
                  );
                })}
              </div>
            ) : (
              <div className="gain">
                <div style={{ fontSize: 12, color: 'var(--text-3)' }}>
                  项目周期内完成每日打卡后，这里会自动统计期间点亮的属性
                </div>
              </div>
            )}
            <div className="form-ops" style={{ marginTop: 12 }}>
              <button className="btn-ghost" onClick={() => setEditProject(p)}>编辑</button>
              {p.status === 'doing' && (
                confirmEnd === p.id ? (
                  <>
                    <button className="btn-primary" onClick={() => finishProject(p)}>确认结束（{todayKey()}）</button>
                    <button className="btn-ghost" onClick={() => setConfirmEnd(null)}>再想想</button>
                  </>
                ) : (
                  <button className={isLatest ? 'btn-primary' : 'btn-ghost'} onClick={() => setConfirmEnd(p.id)}>
                    结束项目
                  </button>
                )
              )}
              {confirmDelete === p.id ? (
                <>
                  <button className="btn-primary" style={{ background: 'var(--red)', borderColor: 'var(--red)' }} onClick={() => deleteProject(p)}>确认删除</button>
                  <button className="btn-ghost" onClick={() => setConfirmDelete(null)}>取消</button>
                </>
              ) : (
                <button className="btn-ghost" style={{ color: 'var(--red)' }} onClick={() => setConfirmDelete(p.id)}>删除项目</button>
              )}
              <button className="btn-ghost" onClick={() => setFormOpen(true)}>+ 新项目</button>
            </div>
          </div>
        );
      })}
      {state.projects.length === 0 && (
        <div className="card" style={{ textAlign: 'center', color: 'var(--text-3)' }}>还没有项目 · 点击右上角「新建项目」开始</div>
      )}

      <Drawer open={formOpen} title="新建项目" onClose={() => setFormOpen(false)} label="新建项目">
        <div className="fld">
          <label htmlFor="p-name">项目名称</label>
          <input id="p-name" type="text" placeholder="例如：毕业论文攻坚" maxLength={20}
            value={name} onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') createProject(); }} />
        </div>
        <div className="fld">
          <label htmlFor="p-start">开始日期</label>
          <input id="p-start" type="date" value={start} onChange={(e) => setStart(e.target.value)} />
        </div>
        <div className="fld">
          <label htmlFor="p-end">结束日期（留空 = 进行中，之后可随时用「结束项目」收尾）</label>
          <input id="p-end" type="date" value={end} min={start} onChange={(e) => setEnd(e.target.value)} />
        </div>
        <p style={{ fontSize: 12, color: 'var(--text-3)', margin: '4px 0 10px' }}>
          创建后无需绑定技能——周期内每天的打卡会自动归入本项目，属性收益实时推导。
        </p>
        <div className="form-ops">
          <button className="btn-primary" onClick={createProject}>创建项目</button>
          <button className="btn-ghost" onClick={() => setFormOpen(false)}>取消</button>
        </div>
      </Drawer>

      <Drawer open={!!editProject} title="编辑项目" onClose={() => setEditProject(null)} label="编辑项目">
        <div className="fld">
          <label htmlFor="e-name">项目名称</label>
          <input id="e-name" type="text" placeholder="项目名称" maxLength={20}
            value={editName} onChange={(e) => setEditName(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') submitEdit(); }} />
        </div>
        <div className="fld">
          <label htmlFor="e-start">开始日期</label>
          <input id="e-start" type="date" value={editStart} onChange={(e) => setEditStart(e.target.value)} />
        </div>
        <div className="fld">
          <label htmlFor="e-end">结束日期（留空 = 进行中）</label>
          <input id="e-end" type="date" value={editEnd} min={editStart} onChange={(e) => setEditEnd(e.target.value)} />
        </div>
        <div className="form-ops">
          <button className="btn-primary" onClick={submitEdit}>保存</button>
          <button className="btn-ghost" onClick={() => setEditProject(null)}>取消</button>
        </div>
      </Drawer>
    </section>
  );
}
