'use client';
/* ================= 技能页：技能树（可折叠）+ 规则卡 ================= */
import React, { useState } from 'react';
import type { AppState, SkillGroup, SkillLeaf, SkillTreeNode } from '@/lib/types';
import { countLeaves, skillLv, tierColor, tierOf } from '@/lib/model';
import { SkillDrawer, type DrawerState } from './SkillDrawer';

type SkillAction =
  | { type: 'SKILL_ARCHIVE'; id: string }
  | { type: 'SKILL_DELETE'; id: string }
  | { type: 'SKILL_RENAME'; id: string; name: string }
  | { type: 'SKILL_MOVE'; id: string; targetPath: string }
  | { type: 'SKILL_ADD'; parentIdPath: string; name: string; attrIds: string[] }
  | { type: 'SKILL_RESTORE'; id: string }
  | { type: 'BRANCH_ADD'; level: 'group' | 'sub'; name: string; parentPath?: string }
  | { type: 'BRANCH_RENAME'; id: string; name: string }
  | { type: 'BRANCH_ARCHIVE'; id: string }
  | { type: 'BRANCH_DELETE'; id: string };

export function SkillsTab({ state, dispatch, toast }: {
  state: AppState;
  dispatch: React.Dispatch<SkillAction>;
  toast: (m: string) => void;
}) {
  const [closed, setClosed] = useState<Set<string>>(new Set());
  const [drawer, setDrawer] = useState<DrawerState | null>(null);
  const [seq, setSeq] = useState(0);
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; nodeId: string; nodeName: string } | null>(null);

  const toggleBranch = (key: string) => {
    setClosed((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  };

  const openLeaf = (id: string) => setDrawer({ id, mode: 'view', confirm: null });
  const openAdd = () => setDrawer({ id: null, mode: 'add', confirm: null });

  const handleBranchContext = (e: React.MouseEvent, nodeId: string, nodeName: string) => {
    e.preventDefault();
    e.stopPropagation();
    setCtxMenu({ x: e.clientX, y: e.clientY, nodeId, nodeName });
  };

  const closeCtxMenu = () => setCtxMenu(null);

  const renameBranch = () => {
    if (!ctxMenu) return;
    const newName = prompt('新名称', ctxMenu.nodeName);
    if (newName && newName.trim() && newName.trim() !== ctxMenu.nodeName) {
      dispatch({ type: 'BRANCH_RENAME', id: ctxMenu.nodeId, name: newName.trim() });
      toast(`已重命名为「${newName.trim()}」`);
    }
    closeCtxMenu();
  };

  const archiveBranch = () => {
    if (!ctxMenu) return;
    if (confirm(`归档「${ctxMenu.nodeName}」？归档后其下所有技能将不再显示，历史记录保留。`)) {
      dispatch({ type: 'BRANCH_ARCHIVE', id: ctxMenu.nodeId });
      toast(`已归档「${ctxMenu.nodeName}」`);
    }
    closeCtxMenu();
  };

  const deleteBranch = () => {
    if (!ctxMenu) return;
    if (confirm(`删除「${ctxMenu.nodeName}」？此操作不可撤销。如有子技能需先删除/归档。`)) {
      try {
        dispatch({ type: 'BRANCH_DELETE', id: ctxMenu.nodeId });
        toast(`已删除「${ctxMenu.nodeName}」`);
      } catch {
        toast('删除失败：请先删除或归档子技能');
      }
    }
    closeCtxMenu();
  };

  const renderNode = (node: SkillTreeNode, gi: number, path: string): React.ReactNode => {
    if (!('ch' in node)) {
      const lv = skillLv(node.uses);
      return (
        <li key={node.id}>
          <div className="tree-node leaf" onClick={() => openLeaf(node.id)}
            role="button" tabIndex={0}
            onKeyDown={(e) => { if (e.key === 'Enter') openLeaf(node.id); }}>
            <span className="dot" />
            <span className="nm">{node.name}</span>
            <span className="tier" style={{ color: tierColor(lv), borderColor: tierColor(lv) }}>
              {tierOf(lv)} {lv}
            </span>
            <span className="uses">×{node.uses}</span>
          </div>
        </li>
      );
    }
    const key = `${gi}:${path}${node.name}`;
    const isClosed = closed.has(key);
    return (
      <li key={key} className={isClosed ? 'closed' : ''}>
        <div className="tree-node branch" onClick={() => toggleBranch(key)}
          onContextMenu={(e) => handleBranchContext(e, node.id ?? '', node.name)}
          role="button" tabIndex={0} aria-expanded={!isClosed}
          onKeyDown={(e) => { if (e.key === 'Enter') toggleBranch(key); }}>
          <span className="dot" />
          <span className="nm">{node.name}</span>
          <span style={{ fontSize: 11, color: 'var(--text-3)', fontWeight: 700 }}>{countLeaves(node)} 技能</span>
          <span className="tw">▾</span>
        </div>
        <ul>{node.ch.map((c) => renderNode(c, gi, `${path}${node.name}/`))}</ul>
      </li>
    );
  };

  return (
    <section className="page" aria-label="技能树">
      <div className="section-title" style={{ marginTop: 4 }}>
        <h2>技能树</h2>
        <button className="link-btn" onClick={openAdd}>新建 +</button>
      </div>
      <p style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 12 }}>
        可新建大类 / 子类 / 叶子技能 · 只有叶子可被打卡 · 点击叶子查看详情 · 右键分支可重命名/归档/删除
      </p>
      <div className="card">
        <div className="tree">
          {state.skills.map((g: SkillGroup, gi) => (
            <div key={g.id}>
              <div className="group-label" onContextMenu={(e) => handleBranchContext(e, g.id, g.name)}>{g.name}</div>
              <ul style={{ listStyle: 'none' }}>
                {g.ch.map((c) => renderNode(c, gi, ''))}
              </ul>
            </div>
          ))}
        </div>
      </div>

      {ctxMenu && (
        <>
          <div style={{ position: 'fixed', inset: 0, zIndex: 999 }} onClick={closeCtxMenu} onContextMenu={(e) => { e.preventDefault(); closeCtxMenu(); }} />
          <div style={{
            position: 'fixed', left: ctxMenu.x, top: ctxMenu.y, zIndex: 1000,
            background: 'var(--card)', border: '2px solid var(--ink)', borderRadius: 6,
            boxShadow: '3px 3px 0 var(--ink)', padding: '4px 0', minWidth: 140,
          }}>
            <div role="button" tabIndex={0} onClick={renameBranch}
              style={{ padding: '8px 16px', cursor: 'pointer', fontWeight: 700, fontSize: 14 }}
              onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--panel)')}
              onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}>
              重命名
            </div>
            <div role="button" tabIndex={0} onClick={archiveBranch}
              style={{ padding: '8px 16px', cursor: 'pointer', fontWeight: 700, fontSize: 14 }}
              onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--panel)')}
              onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}>
              归档
            </div>
            <div role="button" tabIndex={0} onClick={deleteBranch}
              style={{ padding: '8px 16px', cursor: 'pointer', fontWeight: 700, fontSize: 14, color: 'var(--red)' }}
              onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--panel)')}
              onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}>
              删除
            </div>
          </div>
        </>
      )}

      <div className="skill-grid">
        <div className="mini-card">
          <h4>熟练度公式</h4>
          <div className="bd" style={{ fontFamily: 'monospace', fontSize: 13, marginBottom: 6 }}>LV = 4·ln(1 + 使用次数 / 2)⌋</div>
          <div className="bd">见习Ⅰ 1–4 · 熟练Ⅱ 5–9 · 精通 10–14 · 大师Ⅳ 15–19 · 宗师Ⅴ 20+</div>
        </div>
        <div className="mini-card">
          <h4>关联规则</h4>
          <div className="bd">使用技能 → 点亮关联属性（每日每属性封顶 +1）</div>
          <div className="bd" style={{ marginTop: 4 }}>权重不叠加，仅用于项目贡献度展示</div>
        </div>
      </div>

      <SkillDrawer
        state={state}
        drawer={drawer}
        setDrawer={setDrawer}
        dispatch={dispatch}
        toast={toast}
        seq={seq}
        setSeq={setSeq}
      />
    </section>
  );
}

export type { SkillLeaf };
