'use client';
/* ================= 技能抽屉：view / edit / move / add 四态 + 归档删除二次确认 ================= */
import React, { useEffect, useRef, useState } from 'react';
import type { AppState, SkillLeaf } from '@/lib/types';
import { attrMapOf, collectBranches, findLeaf, parentPathOf, skillLv, skillFrac, tierColor, tierOf } from '@/lib/model';
import { Drawer } from './Drawer';

export interface DrawerState {
  id: string | null;
  mode: 'view' | 'edit' | 'move' | 'add';
  confirm: 'archive' | 'delete' | null;
}

export function SkillDrawer({ state, drawer, setDrawer, dispatch, toast, seq, setSeq }: {
  state: AppState;
  drawer: DrawerState | null;
  setDrawer: (d: DrawerState | null) => void;
  dispatch: (a:
    | { type: 'SKILL_ARCHIVE'; id: string }
    | { type: 'SKILL_DELETE'; id: string }
    | { type: 'SKILL_RESTORE'; id: string }
    | { type: 'SKILL_RENAME'; id: string; name: string }
    | { type: 'SKILL_MOVE'; id: string; targetPath: string }
    | { type: 'SKILL_ADD'; parentIdPath: string; name: string; attrIds: string[] }
    | { type: 'BRANCH_ADD'; level: 'group' | 'sub'; name: string; parentPath?: string }
    | { type: 'BRANCH_RENAME'; id: string; name: string }
    | { type: 'BRANCH_ARCHIVE'; id: string }
    | { type: 'BRANCH_DELETE'; id: string }
  ) => void;
  toast: (m: string) => void;
  seq: number;
  setSeq: (n: number) => void;
}) {
  const [name, setName] = useState('');
  const [attrIds, setAttrIds] = useState<string[]>([]);
  const [kind, setKind] = useState<'leaf' | 'sub' | 'group'>('leaf');
  const [selectedParent, setSelectedParent] = useState('');
  const bodyRef = useRef<HTMLDivElement>(null);
  const am = attrMapOf(state.attrs);

  const leaf: SkillLeaf | null = drawer?.id ? findLeaf(state.skills, drawer.id) ?? null : null;
  const open = !!drawer;
  const title = ({ view: '技能详情', edit: '编辑技能', move: '移动技能', add: '新建技能' } as const)[drawer?.mode ?? 'view'];

  // 打开时初始化表单 + 聚焦输入框
  useEffect(() => {
    if (!drawer) return;
    if (drawer.mode === 'add') {
      setName(''); setAttrIds([]); setSelectedParent('');
      // 树为空时默认建大类，否则默认建叶子
      setKind(state.skills.length ? 'leaf' : 'group');
    } else if (drawer.mode === 'edit' && leaf) setName(leaf.name);
    const t = setTimeout(() => {
      const el = bodyRef.current?.querySelector('input');
      if (el) (el as HTMLInputElement).focus();
    }, 80);
    return () => clearTimeout(t);
  }, [drawer?.mode, drawer?.id, seq]);

  const close = () => setDrawer(null);

  const submitAdd = () => {
    const v = name.trim();
    if (!v) { toast('请填写名称'); return; }

    if (kind === 'group') {
      if (state.skills.some((g) => g.name === v)) { toast('已存在同名大类'); return; }
      dispatch({ type: 'BRANCH_ADD', level: 'group', name: v });
      toast(`已创建大类「${v}」· 现在可以在它下面建子类了`);
      setSeq(seq + 1);
      close();
      return;
    }

    if (kind === 'sub') {
      const pv = selectedParent || state.skills[0]?.name;
      if (!pv) { toast('请选择所属大类'); return; }
      dispatch({ type: 'BRANCH_ADD', level: 'sub', name: v, parentPath: pv });
      toast(`已创建子类「${v}」· 挂在 ${pv} 下`);
      setSeq(seq + 1);
      close();
      return;
    }

    // 叶子技能
    if (attrIds.length < 1 || attrIds.length > 3) { toast('请选择 1–3 项关联属性'); return; }
    const pv = selectedParent || branches[0]?.path;
    if (!pv) { toast('请选择父级'); return; }
    const parent = branches.find((b) => b.path === pv);
    if (parent && parent.node.ch.some((c) => c.name === v)) { toast('该分类下已有同名技能'); return; }
    dispatch({ type: 'SKILL_ADD', parentIdPath: pv, name: v, attrIds });
    toast(`已创建「${v}」· 熟练度从见习Ⅰ开始`);
    setSeq(seq + 1);
    close();
  };

  const submitEdit = () => {
    if (!leaf) return;
    const v = name.trim();
    if (!v) { toast('名称不能为空'); return; }
    dispatch({ type: 'SKILL_RENAME', id: leaf.id, name: v });
    toast('已保存 · 名称更新');
    setDrawer({ ...drawer!, mode: 'view' });
  };

  const submitMove = (targetPath: string) => {
    if (!leaf) return;
    dispatch({ type: 'SKILL_MOVE', id: leaf.id, targetPath });
    toast(`已移动到 ${targetPath} · 历史记录保留`);
    setDrawer({ ...drawer!, mode: 'view' });
  };

  const branches = collectBranches(state.skills);
  const curPath = leaf ? parentPathOf(state.skills, leaf) : '';

  return (
    <Drawer open={open} title={title} onClose={close} label="技能详情">
      <div ref={bodyRef}>
        {drawer?.mode === 'add' && (
          <>
            <div className="fld">
              <label>新建类型</label>
              <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                {([['group', '大类'], ['sub', '子类'], ['leaf', '叶子技能']] as const).map(([k, lbl]) => (
                  <button key={k} type="button" className={`chip${kind === k ? ' on' : ''}`}
                    aria-pressed={kind === k} onClick={() => setKind(k)}>
                    {lbl}
                  </button>
                ))}
              </div>
            </div>
            <div className="fld">
              <label htmlFor="f-name">
                {kind === 'group' ? '大类名称（技能树顶层分类）'
                  : kind === 'sub' ? '子类名称（挂在某个大类下）'
                  : '技能名称（叶子技能，创建后即可打卡）'}
              </label>
              <input type="text" id="f-name" maxLength={12}
                placeholder={kind === 'group' ? '例如：身体' : kind === 'sub' ? '例如：运动' : '例如：水彩'}
                value={name} onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') submitAdd(); }} />
            </div>
            {kind === 'sub' && (
              <div className="fld" key="pa-sub">
                <label>所属大类</label>
                {state.skills.map((g) => {
                  const on = selectedParent === g.name || (!selectedParent && state.skills[0]?.id === g.id);
                  return (
                    <div key={g.id} className="choice" role="button" tabIndex={0}
                      onClick={() => setSelectedParent(g.name)}
                      onKeyDown={(e) => { if (e.key === 'Enter') setSelectedParent(g.name); }}
                      style={{
                        display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px',
                        border: `2px solid ${on ? 'var(--accent)' : 'var(--ink)'}`,
                        borderRadius: 6, background: on ? 'var(--panel)' : 'var(--card)',
                        boxShadow: on ? '2px 2px 0 var(--accent)' : '2px 2px 0 var(--ink)',
                        cursor: 'pointer', fontWeight: 700, fontSize: 14, transition: 'all .15s',
                      }}>
                      <span style={{
                        width: 10, height: 10, borderRadius: '50%',
                        background: on ? 'var(--accent)' : 'transparent',
                        border: `2px solid ${on ? 'var(--accent)' : 'var(--ink)'}`, flexShrink: 0,
                      }} />
                      {g.name}
                    </div>
                  );
                })}
              </div>
            )}
            {kind === 'leaf' && !branches.length && (
              <p style={{ color: 'var(--text-3)', fontSize: 13 }}>
                还没有任何分类——先切到「大类」创建一个，再建子类 / 技能。
              </p>
            )}
            {kind === 'leaf' && branches.length > 0 && (
              <>
                <div className="fld" key="pa-leaf">
                  <label>选择父级</label>
                  {branches.map((b) => {
                    const on = selectedParent === b.path || (!selectedParent && branches[0]?.path === b.path);
                    return (
                      <div key={b.path} className="choice" role="button" tabIndex={0}
                        onClick={() => setSelectedParent(b.path)}
                        onKeyDown={(e) => { if (e.key === 'Enter') setSelectedParent(b.path); }}
                        style={{
                          display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px',
                          border: `2px solid ${on ? 'var(--accent)' : 'var(--ink)'}`,
                          borderRadius: 6, background: on ? 'var(--panel)' : 'var(--card)',
                          boxShadow: on ? '2px 2px 0 var(--accent)' : '2px 2px 0 var(--ink)',
                          cursor: 'pointer', fontWeight: 700, fontSize: 14, transition: 'all .15s',
                        }}>
                        <span style={{
                          width: 10, height: 10, borderRadius: '50%',
                          background: on ? 'var(--accent)' : 'transparent',
                          border: `2px solid ${on ? 'var(--accent)' : 'var(--ink)'}`, flexShrink: 0,
                        }} />
                        {b.path}
                      </div>
                    );
                  })}
                </div>
                <div className="fld">
                  <label>关联属性（选 1–3 项，权重自动均分）</label>
                  {state.attrs.map((a) => {
                    const on = attrIds.includes(a.id);
                    return (
                      <div key={a.id} className="choice" role="button" tabIndex={0}
                        onClick={() => setAttrIds((prev) => on ? prev.filter((x) => x !== a.id) : [...prev, a.id].slice(0, 3))}
                        onKeyDown={(e) => { if (e.key === 'Enter') setAttrIds((prev) => on ? prev.filter((x) => x !== a.id) : [...prev, a.id].slice(0, 3)); }}
                        style={{
                          display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px',
                          border: `2px solid ${on ? a.color : 'var(--ink)'}`,
                          borderRadius: 6, background: on ? (a.tint ?? 'var(--panel)') : 'var(--card)',
                          boxShadow: on ? `2px 2px 0 ${a.color}` : '2px 2px 0 var(--ink)',
                          cursor: 'pointer', fontWeight: 700, fontSize: 14, transition: 'all .15s',
                        }}>
                        <span style={{
                          width: 10, height: 10, borderRadius: 2, background: on ? a.color : 'transparent',
                          border: `2px solid ${a.color}`, flexShrink: 0,
                        }} />
                        {a.name}
                      </div>
                    );
                  })}
                </div>
              </>
            )}
            <div className="form-ops">
              <button className="btn-primary" onClick={submitAdd}>创建</button>
              <button className="btn-ghost" onClick={close}>取消</button>
            </div>
          </>
        )}

        {drawer && drawer.mode !== 'add' && !leaf && <p style={{ color: 'var(--text-3)' }}>技能不存在或已删除。</p>}

        {drawer && leaf && drawer.mode === 'view' && (
          <>
            {drawer.confirm === 'archive' && (
              <div className="warn-box">
                归档后「{leaf.name}」不再显示在技能树和打卡列表，历史记录保留。
                <div className="form-ops">
                  <button className="btn-primary" onClick={() => {
                    dispatch({ type: 'SKILL_ARCHIVE', id: leaf.id });
                    toast(`已归档「${leaf.name}」· 不再显示但数据保留`);
                    close();
                  }}>确认归档</button>
                  <button className="btn-ghost" onClick={() => setDrawer({ ...drawer, confirm: null })}>再想想</button>
                </div>
              </div>
            )}
            {drawer.confirm === 'delete' && (
              <div className="warn-box">
                删除后「{leaf.name}」将从技能树移除。历史使用记录按 id 保留在日志里，不受影响。
                <div className="form-ops">
                  <button className="btn-primary" onClick={() => {
                    dispatch({ type: 'SKILL_DELETE', id: leaf.id });
                    toast(`已删除「${leaf.name}」· 历史记录按 id 保留`);
                    close();
                  }}>确认删除</button>
                  <button className="btn-ghost" onClick={() => setDrawer({ ...drawer, confirm: null })}>再想想</button>
                </div>
              </div>
            )}
            {!drawer.confirm && (
              <div className="d-ops">
                <button onClick={() => setDrawer({ ...drawer, mode: 'edit' })}>编辑</button>
                <button onClick={() => setDrawer({ ...drawer, mode: 'move' })}>移动</button>
                {leaf.archived && (
                  <button onClick={() => {
                    dispatch({ type: 'SKILL_RESTORE', id: leaf.id });
                    toast(`已恢复「${leaf.name}」· 重新显示在技能树中`);
                    close();
                  }}>恢复</button>
                )}
                {!leaf.archived && <button onClick={() => setDrawer({ ...drawer, confirm: 'archive' })}>归档</button>}
                <button className="danger" onClick={() => setDrawer({ ...drawer, confirm: 'delete' })}>删除</button>
              </div>
            )}
            <div style={{ marginTop: 12, marginBottom: 8 }}>
              <span className="tier" style={{ color: tierColor(skillLv(leaf.uses)), borderColor: tierColor(skillLv(leaf.uses)), fontSize: 12, padding: '3px 10px' }}>
                {tierOf(skillLv(leaf.uses))} · LV {skillLv(leaf.uses)}
              </span>
              <div className="bar" style={{ maxWidth: 220 }}>
                <i style={{ width: `${Math.round(skillFrac(leaf.uses) * 100)}%`, background: tierColor(skillLv(leaf.uses)) }} />
              </div>
            </div>
            <div style={{ fontSize: 13, color: 'var(--text-2)' }}>
              累计使用 <b style={{ color: 'var(--ink)' }}>{leaf.uses}</b> 次 · 所属：{curPath || '根目录'}
            </div>
            <div style={{ fontSize: 13, color: 'var(--text-2)', marginTop: 8 }}>关联属性</div>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginTop: 6 }}>
              {leaf.attrs.map(([aid, w]) => {
                const a = am[aid];
                if (!a) return null;
                return (
                  <span key={aid} className="tag" style={{ color: a.color, background: a.tint }}>
                    {a.name} · 权重 {w}
                  </span>
                );
              })}
            </div>
          </>
        )}

        {drawer && leaf && drawer.mode === 'edit' && (
          <>
            <div className="fld">
              <label htmlFor="f-name">技能名称</label>
              <input type="text" id="f-name" maxLength={12} value={name}
                onChange={(e) => setName(e.target.value)} />
            </div>
            <div className="form-ops">
              <button className="btn-primary" onClick={submitEdit}>保存</button>
              <button className="btn-ghost" onClick={() => setDrawer({ ...drawer, mode: 'view' })}>取消</button>
            </div>
          </>
        )}

        {drawer && leaf && drawer.mode === 'move' && (
          <div className="fld">
            <label>把「{leaf.name}」移动到（当前：{curPath || '根目录'}）· 换父不丢历史</label>
            {branches.map((b) => {
              const on = b.path === curPath;
              return (
                <div key={b.path} className="choice" role="button" tabIndex={0}
                  onClick={() => submitMove(b.path)}
                  onKeyDown={(e) => { if (e.key === 'Enter') submitMove(b.path); }}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px',
                    border: `2px solid ${on ? 'var(--accent)' : 'var(--ink)'}`,
                    borderRadius: 6, background: on ? 'var(--panel)' : 'var(--card)',
                    boxShadow: on ? '2px 2px 0 var(--accent)' : '2px 2px 0 var(--ink)',
                    cursor: 'pointer', fontWeight: 700, fontSize: 14, transition: 'all .15s',
                  }}>
                  <span style={{
                    width: 10, height: 10, borderRadius: '50%',
                    background: on ? 'var(--accent)' : 'transparent',
                    border: `2px solid ${on ? 'var(--accent)' : 'var(--ink)'}`, flexShrink: 0,
                  }} />
                  {b.path}
                </div>
              );
            })}
            <div className="form-ops">
              <button className="btn-ghost" onClick={() => setDrawer({ ...drawer, mode: 'view' })}>取消</button>
            </div>
          </div>
        )}
      </div>
    </Drawer>
  );
}
