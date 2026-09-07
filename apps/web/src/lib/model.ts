/* ================= 增长模型 · 日期工具 · 树工具 · 统计推导 ================= */
import type { AchStats, AppState, Attr, DayRecord, SkillGroup, SkillLeaf, SkillTreeNode } from './types';
import { isLeaf } from './types';

/* ---------- 增长公式（设计文档 4 节） ---------- */
export const ATTR_LV_MAX = 50;

/** 属性：LV = ⌊5·ln(1+EP/3)⌋，EP 每属性每天封顶 +1 */
export function attrLv(ep: number): number {
  return Math.min(ATTR_LV_MAX, Math.floor(5 * Math.log(1 + Math.max(0, ep) / 3)));
}

/** 升到 n 级所需 EP：3·(e^(n/5) − 1) */
export function epForLevel(n: number): number {
  return 3 * (Math.exp(n / 5) - 1);
}

/** 属性当前级内进度 0–1（真实推导，无演示偏移） */
export function attrFrac(ep: number): number {
  if (attrLv(ep) >= ATTR_LV_MAX) return 1;
  return (5 * Math.log(1 + Math.max(0, ep) / 3)) % 1;
}

/** 技能：LV = ⌊4·ln(1+uses/2)⌋ */
export function skillLv(uses: number): number {
  return Math.floor(4 * Math.log(1 + Math.max(0, uses) / 2));
}

/** 技能当前级内进度 0–1 */
export function skillFrac(uses: number): number {
  return (4 * Math.log(1 + Math.max(0, uses) / 2)) % 1;
}

export const TIERS: [number, string][] = [
  [1, '见习Ⅰ'], [5, '熟练Ⅱ'], [10, '精通Ⅲ'], [15, '大师Ⅳ'], [20, '宗师Ⅴ'],
];
export function tierOf(lv: number): string {
  let t = TIERS[0][1];
  for (const [m, n] of TIERS) if (lv >= m) t = n;
  return t;
}
export function tierColor(lv: number): string {
  if (lv >= 20) return 'var(--yellow)';
  if (lv >= 15) return 'var(--red)';
  if (lv >= 10) return 'var(--rsr)';
  if (lv >= 5) return 'var(--blue)';
  return 'var(--rn)';
}

/* ---------- 日期工具（本地时区，YYYY-MM-DD） ---------- */
export function dateKey(d: Date): string {
  const p = (x: number) => String(x).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}
export function todayKey(): string {
  return dateKey(new Date());
}
export function addDays(d: Date, n: number): Date {
  const x = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  x.setDate(x.getDate() + n);
  return x;
}
/** 距今天 n 天前的日期 key（n=0 即今天） */
export function daysAgoKey(n: number): string {
  return dateKey(addDays(new Date(), -n));
}
/** 解析 YYYY-MM-DD 为本地 Date */
export function parseKey(k: string): Date {
  const [y, m, d] = k.split('-').map(Number);
  return new Date(y, (m || 1) - 1, d || 1);
}

/* ---------- 技能树工具 ---------- */
/** 收集全部未归档叶子（按树顺序） */
export function collectLeaves(skills: SkillGroup[]): SkillLeaf[] {
  const out: SkillLeaf[] = [];
  const walk = (list: SkillTreeNode[]) => {
    for (const n of list) {
      if (isLeaf(n)) { if (!n.archived) out.push(n); } else walk(n.ch);
    }
  };
  walk(skills);
  return out;
}

/** 收集全部叶子（含已归档） */
export function collectLeavesAll(skills: SkillGroup[]): SkillLeaf[] {
  const out: SkillLeaf[] = [];
  const walk = (list: SkillTreeNode[]) => {
    for (const n of list) {
      if (isLeaf(n)) out.push(n); else walk(n.ch);
    }
  };
  walk(skills);
  return out;
}

export function countLeaves(n: SkillTreeNode): number {
  return isLeaf(n) ? 1 : n.ch.reduce((s, c) => s + countLeaves(c), 0);
}

/** 全部分支及路径（"艺术 / 绘画"），用于新建/移动技能选父级 */
export function collectBranches(skills: SkillGroup[]): { node: SkillGroup | Exclude<SkillTreeNode, SkillLeaf>; path: string }[] {
  const out: { node: Exclude<SkillTreeNode, SkillLeaf>; path: string }[] = [];
  const walk = (list: SkillTreeNode[], path: string) => {
    for (const n of list) {
      if (!isLeaf(n)) {
        const p = path ? `${path} / ${n.name}` : n.name;
        out.push({ node: n, path: p });
        walk(n.ch, p);
      }
    }
  };
  walk(skills, '');
  return out;
}

/** 摘除叶子（返回是否成功；不修改入参，调用方先深拷贝） */
export function removeFromParent(list: SkillTreeNode[], leaf: SkillLeaf): boolean {
  const i = list.indexOf(leaf);
  if (i > -1) { list.splice(i, 1); return true; }
  return list.some((n) => !isLeaf(n) && removeFromParent(n.ch, leaf));
}

/** 叶子的父级路径 */
export function parentPathOf(skills: SkillGroup[], leaf: SkillLeaf): string {
  const found = collectBranches(skills).find((b) => b.node.ch.includes(leaf));
  return found ? found.path : '';
}

/** 深拷贝技能树（叶子对象引用随拷贝变化，需按 id 重查） */
export function cloneSkills(skills: SkillGroup[]): SkillGroup[] {
  return JSON.parse(JSON.stringify(skills)) as SkillGroup[];
}

/** 按 id 在树中找叶子（含已归档） */
export function findLeaf(skills: SkillGroup[], id: string): SkillLeaf | undefined {
  const walk = (list: SkillTreeNode[]): SkillLeaf | undefined => {
    for (const n of list) {
      if (isLeaf(n)) { if (n.id === id) return n; } else {
        const f = walk(n.ch);
        if (f) return f;
      }
    }
    return undefined;
  };
  return walk(skills);
}

/* ---------- 统计推导 ---------- */
export function deriveStats(state: AppState): AchStats {
  const { records, attrs, skills, projects } = state;
  const days = new Set(records.filter((r) => r.skillIds.length > 0).map((r) => r.date));
  const totalDays = days.size;

  // 连续记录（截止今天或昨天）
  let streak = 0;
  const start = days.has(todayKey()) ? 0 : days.has(daysAgoKey(1)) ? 1 : -1;
  if (start >= 0) {
    let n = start;
    while (days.has(daysAgoKey(n))) { streak += 1; n += 1; }
  }

  const maxLitOneDay = records.reduce((m, r) => Math.max(m, r.attrsLit.length), 0);
  const maxAttrLv = attrs.reduce((m, a) => Math.max(m, attrLv(a.ep)), 0);
  const maxSkillLv = collectLeaves(skills).reduce((m, l) => Math.max(m, skillLv(l.uses)), 0);
  const skillCheckinCounts = new Map<string, number>();
  for (const r of records) {
    for (const sid of r.skillIds) {
      skillCheckinCounts.set(sid, (skillCheckinCounts.get(sid) ?? 0) + 1);
    }
  }
  const maxSkillCheckins = skillCheckinCounts.size > 0 ? Math.max(...skillCheckinCounts.values()) : 0;
  const projectsDone = projects.filter((p) => p.status === 'done').length;
  return { totalDays, streak, maxLitOneDay, maxAttrLv, maxSkillLv, maxSkillCheckins, projectsDone, totalAttrs: attrs.length };
}

/** 本周（周一起）记录天数 */
export function weekDays(records: DayRecord[]): number {
  const now = new Date();
  const dow = (now.getDay() + 6) % 7; // 周一=0
  let c = 0;
  for (let i = 0; i <= dow; i++) {
    if (records.some((r) => r.date === daysAgoKey(i) && r.skillIds.length > 0)) c += 1;
  }
  return c;
}

/** 项目周期内打卡自动归集的属性收益（每日每属性点亮 +1，无需手动绑定技能） */
export function projectGains(
  records: DayRecord[],
  project: { start: string; end?: string },
): { attrId: string; count: number }[] {
  const counter = new Map<string, number>();
  for (const r of records) {
    if (r.date < project.start) continue;
    if (project.end && r.date > project.end) continue;
    for (const aid of r.attrsLit) counter.set(aid, (counter.get(aid) || 0) + 1);
  }
  return [...counter.entries()]
    .map(([attrId, count]) => ({ attrId, count }))
    .sort((a, b) => b.count - a.count);
}

/** 属性 id → Attr 索引表 */
export function attrMapOf(attrs: Attr[]): Record<string, Attr> {
  return Object.fromEntries(attrs.map((a) => [a.id, a]));
}

/* ---------- 今日建议：弱项属性 + 推荐技能（纯函数，便于复用与单测） ---------- */

/**
 * 取今日仍未点亮的弱项属性：按 EP 升序，最多 3 项。
 * - 全部为 0 时返回前 3 项（让新手也能看到建议）。
 * - 属性不足 3 项时按实际数量返回。
 */
export function weakAttrsOf(state: AppState, litAttrIds: ReadonlySet<string>, max = 3): Attr[] {
  const all = [...state.attrs];
  const rest = all.filter((a) => !litAttrIds.has(a.id));
  rest.sort((a, b) => a.ep - b.ep);
  return rest.slice(0, max);
}

/**
 * 取今日推荐技能：覆盖「弱项属性」最多且未选中的未归档叶子，最多 3 项。
 * 打分 = 该叶子所链接的弱项属性数量（并列时按"叶子 EP 总权重 × 已使用次数"启发排）。
 */
export function suggestedSkillsOf(
  state: AppState,
  selectedSkillIds: ReadonlySet<string>,
  weak: ReadonlySet<string>,
  max = 3,
): SkillLeaf[] {
  if (weak.size === 0) return [];
  const leaves = collectLeaves(state.skills);
  const scored = leaves
    .filter((l) => !selectedSkillIds.has(l.id))
    .map((l) => {
      const covers = l.attrs.reduce((n, [aid]) => (weak.has(aid) ? n + 1 : n), 0);
      const gain = l.attrs.reduce((s, [aid, w]) => (weak.has(aid) ? s + w : s), 0);
      return { leaf: l, covers, gain };
    })
    .filter((s) => s.covers > 0);
  scored.sort((a, b) => b.covers - a.covers || b.gain - a.gain || b.leaf.uses - a.leaf.uses);
  return scored.slice(0, max).map((s) => s.leaf);
}

/** 一站式组合：弱项属性 + 推荐技能（弱项 EP 升序、推荐技能按覆盖度降序）。 */
export function todaySuggestions(
  state: AppState,
  selectedSkillIds: ReadonlySet<string>,
  litAttrIds: ReadonlySet<string>,
): { weak: Attr[]; skills: SkillLeaf[] } {
  const weak = weakAttrsOf(state, litAttrIds);
  const weakSet = new Set(weak.map((a) => a.id));
  const skills = suggestedSkillsOf(state, selectedSkillIds, weakSet);
  return { weak, skills };
}
