'use client';
/* ================= 全局状态：Rust 引擎（HTTP /api/rpc）为唯一数据源 =================
 * 前端不再本地推导数值：属性 value / 技能 level / 记录 / 统计全部来自 soloup-server。
 * 本文件只做两件事：① 后端数据 ⇄ 视图模型映射；② 把写操作转发为 RPC（成功后重取）。
 */
import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import type { AppState, Attr, Project, Settings, SkillGroup, SkillTreeNode } from './types';
import { rpc } from './api';
import { seedState } from './seed';
import { todayKey } from './model';

/* ---------- 后端返回结构（soloup-server） ---------- */
export interface RustAttribute {
  id: string; name: string; description: string | null;
  category: string | null; color: string | null; value: number; x: number;
}
export interface RustTreeNode {
  id: string; name: string; leaf: boolean; archived: boolean;
  level: number; attrs: [string, number][]; children: RustTreeNode[];
}
export interface RustRecord {
  date: string; skillIds: string[]; attrsLit: string[];
  projectId: string | null; note: string | null;
}
export interface RustProject {
  id: string; name: string; description: string | null;
  startDate: string; endDate: string | null; status: string; color: string | null;
}
export interface RustAchievement { id: string; unlocked: boolean }
export interface Bootstrap {
  date: string;
  attributes: RustAttribute[];
  tree: RustTreeNode[];
  records: RustRecord[];
  projects: RustProject[];
  achievements: RustAchievement[];
  profile: Record<string, unknown>;
  life: { born: string; expectancy: number };
  stats: Record<string, number>;
  today: { date: string; checked: string[]; litAttrs: string[] } | null;
}

/* ---------- 映射：后端派生值 → 视图模型 ---------- */
/** 属性：视图按 attrLv(ep)=⌊5·ln(1+ep/3)⌋ 展示，令其等于后端 ⌊value/2⌋（value∈[0,100] → 等级 0–50） */
export const epFromValue = (value: number) => 3 * (Math.exp(Math.max(0, value) / 10) - 1);
/** 技能：视图按 skillLv(uses)=⌊4·ln(1+uses/2)⌋ 展示，令其等于后端 level */
export const usesFromLevel = (level: number) => 2 * (Math.exp(Math.max(0, level) / 4) - 1);

const FALLBACK_COLOR = '#1F2937';
const tintOf = (color: string | null) => (color ?? FALLBACK_COLOR) + '22';

function mapNode(n: RustTreeNode): SkillTreeNode {
  if (n.leaf) {
    const leaf: import('./types').SkillLeaf = {
      id: n.id,
      name: n.name,
      uses: usesFromLevel(n.level),
      attrs: (n.attrs ?? []) as [string, number][],
    };
    if (n.archived) leaf.archived = true;
    return leaf;
  }
  return { id: n.id, name: n.name, ch: n.children.map(mapNode) };
}

function mapBootstrap(b: Bootstrap): AppState {
  const attrs: Attr[] = b.attributes.map((a) => ({
    id: a.id,
    name: a.name,
    en: a.name,
    color: a.color ?? FALLBACK_COLOR,
    tint: tintOf(a.color),
    ep: epFromValue(a.value),
  }));

  const projects: Project[] = b.projects.map((p) => ({
    id: p.id,
    name: p.name,
    start: p.startDate,
    ...(p.endDate ? { end: p.endDate } : {}),
    status: p.status === 'completed' ? 'done' : 'doing',
    events: [],
  }));

  const unlocked: Record<string, string> = {};
  for (const a of b.achievements ?? []) if (a.unlocked) unlocked[a.id] = todayKey();

  const prof = b.profile ?? {};
  const settings: Settings = {
    nickname: (prof.nickname as string) ?? '人生玩家',
    avatar: (prof.avatar as string) ?? '人',
    birth: b.life?.born ?? '2002-05-20',
    lifeExp: b.life?.expectancy ?? 120,
    remind: (prof.remind as string) ?? '20:00',
    motion: prof.motion !== false,
    lockHistory: prof.lockHistory === true,
  };

  return {
    version: 1,
    attrs,
    skills: b.tree.map((n) => ({ id: n.id, name: n.name, ch: n.children.map(mapNode) })) as SkillGroup[],
    records: (b.records ?? []).map((r) => ({
      date: r.date, skillIds: r.skillIds ?? [], attrsLit: r.attrsLit ?? [],
    })),
    projects,
    unlocked,
    settings,
  };
}

/* ---------- Actions（与视图交互保持原契约） ---------- */
export type Action =
  | { type: 'HYDRATE'; state: AppState }
  | { type: 'RESET' }
  | { type: 'SET_SETTINGS'; patch: Partial<Settings> }
  | { type: 'ATTR_ADD'; name: string; en: string; palette: [string, string] }
  | { type: 'ATTR_UPDATE'; id: string; name: string; en: string; palette: [string, string] }
  | { type: 'ATTR_DELETE'; id: string }
  | { type: 'ATTR_MOVE'; index: number; dir: -1 | 1 }
  | { type: 'SKILL_ADD'; parentIdPath: string; name: string; attrIds: string[] }
  | { type: 'BRANCH_ADD'; level: 'group' | 'sub'; name: string; parentPath?: string }
  | { type: 'BRANCH_RENAME'; id: string; name: string }
  | { type: 'BRANCH_ARCHIVE'; id: string }
  | { type: 'BRANCH_DELETE'; id: string }
  | { type: 'SKILL_RENAME'; id: string; name: string }
  | { type: 'SKILL_MOVE'; id: string; targetPath: string }
  | { type: 'SKILL_ARCHIVE'; id: string }
  | { type: 'SKILL_RESTORE'; id: string }
  | { type: 'SKILL_DELETE'; id: string }
  | { type: 'SAVE_RECORD'; date: string; skillIds: string[] }
  | { type: 'CHECKIN_CLEAR'; date: string }
  | { type: 'PROJECT_ADD'; name: string; start: string; end?: string }
  | { type: 'PROJECT_UPDATE'; id: string; patch: Partial<{ name: string; start: string; end: string | null; status: string; color: string | null; description: string | null }> }
  | { type: 'PROJECT_EVENT'; projectId: string; event: import('./types').ProjectEvent }
  | { type: 'PROJECT_FINISH'; projectId: string; end: string }
  | { type: 'PROJECT_DELETE'; projectId: string }
  | { type: 'UNLOCK'; id: string; date: string };

/** "艺术 / 绘画" → 后端分支 id */
function branchIdByPath(skills: SkillGroup[], path: string): string | null {
  const parts = path.split(' / ').filter(Boolean);
  let level: SkillTreeNode[] = skills;
  let found: SkillTreeNode | undefined;
  for (const p of parts) {
    found = level.find((n) => 'ch' in n && n.name === p);
    if (!found) return null;
    level = (found as { ch: SkillTreeNode[] }).ch;
  }
  return (found as { id?: string } | undefined)?.id ?? null;
}

/** 写操作 → RPC */
async function applyAction(a: Action, state: AppState): Promise<void> {
  switch (a.type) {
    case 'HYDRATE':
    case 'UNLOCK': // 成就由后端判定
      return;
    case 'RESET':
      await rpc('seed.demo');
      return;
    case 'SAVE_RECORD':
      await rpc('checkin', { date: a.date, skillIds: a.skillIds });
      return;
    case 'SKILL_ADD': {
      const parentId = branchIdByPath(state.skills, a.parentIdPath);
      const w = a.attrIds.length ? +(1 / a.attrIds.length).toFixed(2) : 1;
      await rpc('skill.create', {
        name: a.name,
        parentId,
        category: 'cognitive',
        links: a.attrIds.map((attributeId) => ({ attributeId, weight: w })),
      });
      return;
    }
    case 'BRANCH_ADD': {
      const parentId = a.parentPath ? branchIdByPath(state.skills, a.parentPath) : null;
      await rpc('skill.create', { name: a.name, parentId, category: 'cognitive' });
      return;
    }
    case 'SKILL_RENAME':
      await rpc('skill.updateProfile', { id: a.id, name: a.name });
      return;
    case 'SKILL_MOVE':
      await rpc('skill.move', { id: a.id, parentId: branchIdByPath(state.skills, a.targetPath) });
      return;
    case 'SKILL_ARCHIVE':
      await rpc('skill.archive', { id: a.id });
      return;
    case 'SKILL_RESTORE':
      await rpc('skill.restore', { id: a.id });
      return;
    case 'SKILL_DELETE':
      await rpc('skill.delete', { id: a.id });
      return;
    case 'BRANCH_RENAME':
      await rpc('skill.updateProfile', { id: a.id, name: a.name });
      return;
    case 'BRANCH_ARCHIVE':
      await rpc('skill.archive', { id: a.id });
      return;
    case 'BRANCH_DELETE':
      await rpc('skill.delete', { id: a.id });
      return;
    case 'ATTR_ADD':
      await rpc('attribute.create', { name: a.name, color: a.palette[0] });
      return;
    case 'ATTR_UPDATE':
      await rpc('attribute.update', { id: a.id, name: a.name, color: a.palette[0] });
      return;
    case 'ATTR_DELETE':
      await rpc('attribute.delete', { id: a.id });
      return;
    case 'ATTR_MOVE': {
      const j = a.index + a.dir;
      if (j < 0 || j >= state.attrs.length) return;
      await rpc('attribute.update', { id: state.attrs[a.index].id, sort: j });
      await rpc('attribute.update', { id: state.attrs[j].id, sort: a.index });
      return;
    }
    case 'SET_SETTINGS': {
      const p = a.patch;
      if ('birth' in p || 'lifeExp' in p) {
        await rpc('life.save', {
          born: p.birth ?? state.settings.birth,
          expectancy: p.lifeExp ?? state.settings.lifeExp,
        });
      }
      await rpc('profile.save', {
        nickname: p.nickname ?? state.settings.nickname,
        avatar: p.avatar ?? state.settings.avatar,
        remind: p.remind ?? state.settings.remind,
        motion: p.motion ?? state.settings.motion,
        lockHistory: p.lockHistory ?? state.settings.lockHistory,
      });
      return;
    }
    case 'PROJECT_ADD':
      await rpc('project.create', { name: a.name, start: a.start, end: a.end ?? null });
      return;
    case 'PROJECT_UPDATE':
      await rpc('project.update', { id: a.id, ...a.patch });
      return;
    case 'PROJECT_FINISH':
      await rpc('project.finish', { id: a.projectId, end: a.end });
      return;
    case 'PROJECT_DELETE':
      await rpc('project.delete', { id: a.projectId });
      return;
    case 'CHECKIN_CLEAR':
      await rpc('checkin.clear', { date: a.date });
      return;
    case 'PROJECT_EVENT':
      // 后端暂无项目事件表：忽略（视图层归集仍按记录计算）
      return;
  }
}

/* ---------- Context ---------- */
interface StoreCtx {
  state: AppState;
  dispatch: React.Dispatch<Action>;
  hydrated: boolean;
  reload: () => Promise<void>;
}
const Ctx = createContext<StoreCtx | null>(null);

export function StoreProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AppState>(() => seedState());
  const [hydrated, setHydrated] = useState(false);

  const reload = useCallback(async () => {
    const b = await rpc<Bootstrap>('bootstrap');
    setState(mapBootstrap(b));
  }, []);

  useEffect(() => {
    let alive = true;
    const MAX_RETRIES = 10;
    const RETRY_DELAY = 500;

    const tryBootstrap = async (attempt: number) => {
      try {
        const b = await rpc<Bootstrap>('bootstrap');
        if (alive) setState(mapBootstrap(b));
      } catch (e) {
        if (attempt < MAX_RETRIES && alive) {
          setTimeout(() => tryBootstrap(attempt + 1), RETRY_DELAY);
          return;
        }
        console.error('[soloup] 后端不可用，显示占位数据：', e);
      } finally {
        if (alive && attempt >= MAX_RETRIES) setHydrated(true);
      }
      if (alive) setHydrated(true);
    };

    tryBootstrap(0);
    return () => { alive = false; };
  }, []);

  useEffect(() => {
    if (!hydrated) return;
    let alive = true;
    const POLL_INTERVAL = 10_000;

    const poll = async () => {
      if (!alive || document.visibilityState === 'hidden') return;
      try {
        const b = await rpc<Bootstrap>('bootstrap');
        if (alive) setState(mapBootstrap(b));
      } catch { /* silently ignore — next poll will retry */ }
    };

    const id = setInterval(poll, POLL_INTERVAL);
    const onVis = () => { if (document.visibilityState === 'visible') void poll(); };
    document.addEventListener('visibilitychange', onVis);
    return () => { alive = false; clearInterval(id); document.removeEventListener('visibilitychange', onVis); };
  }, [hydrated]);

  const dispatch = useCallback<React.Dispatch<Action>>((action) => {
    if (action.type === 'HYDRATE') {
      setState(action.state);
      return;
    }
    void (async () => {
      try {
        await applyAction(action, state);
        await reload();
      } catch (e) {
        console.error('[soloup] 操作失败：', action.type, e);
      }
    })();
  }, [state, reload]);

  const value = useMemo(() => ({ state, dispatch, hydrated, reload }), [state, dispatch, hydrated, reload]);

  // 后端数据到达前不渲染内容：避免 SSR 首帧（占位）与客户端首帧（后端真实值）水合不一致。
  if (!hydrated) {
    return (
      <div className="app" aria-busy="true">
        <main style={{ padding: 24, color: 'var(--text-2, #666)' }}>正在从引擎载入…</main>
      </div>
    );
  }
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useStore(): StoreCtx {
  const c = useContext(Ctx);
  if (!c) throw new Error('useStore 必须在 StoreProvider 内使用');
  return c;
}
