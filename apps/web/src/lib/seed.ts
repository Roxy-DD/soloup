/* ================= 种子数据（首次加载/重置时使用） ================= */
import type { AchievementDef, AppState, Attr, DayRecord, Project, Settings, SkillGroup } from './types';
import { epForLevel, daysAgoKey } from './model';

/** 由目标等级反推 EP，再追加少量余量让进度条有真实观感（extra 已核验不越级） */
function epFor(lv: number, extra: number): number {
  return Math.round(epForLevel(lv)) + extra;
}

export const SEED_ATTRS: Attr[] = [
  { id: 'vit', name: '体质', en: 'VIT', color: '#C6352B', tint: '#FDE8E6', ep: epFor(24, 37) },
  { id: 'int', name: '智力', en: 'INT', color: '#2456C4', tint: '#E8EFFF', ep: epFor(35, 91) },
  { id: 'crea', name: '创造', en: 'CREA', color: '#6D28D9', tint: '#F1E9FF', ep: epFor(29, 55) },
  { id: 'will', name: '意志', en: 'WILL', color: '#B45309', tint: '#FFF1D6', ep: epFor(18, 23) },
  { id: 'cha', name: '魅力', en: 'CHA', color: '#BE185D', tint: '#FDE9F3', ep: epFor(12, 3) },
  { id: 'sen', name: '感知', en: 'SEN', color: '#047857', tint: '#DFF7EE', ep: epFor(21, 11) },
];

/** 属性色板：[深字色, 淡底色] 配对（对比度已核验） */
export const ATTR_PALETTE: [string, string][] = [
  ['#C6352B', '#FDE8E6'], ['#2456C4', '#E8EFFF'], ['#6D28D9', '#F1E9FF'],
  ['#B45309', '#FFF1D6'], ['#BE185D', '#FDE9F3'], ['#047857', '#DFF7EE'],
  ['#0E7490', '#CFFAFE'], ['#4D7C0F', '#ECFCCB'],
];

export const SEED_SKILLS: SkillGroup[] = [
  { id: 'art', name: '艺术', ch: [
    { name: '绘画', ch: [
      { id: 'sketch', name: '素描', uses: 120, attrs: [['crea', .5], ['sen', .5]] },
      { id: 'oil', name: '油画', uses: 12, attrs: [['crea', .6], ['sen', .4]] },
      { id: 'digi', name: '数字绘画', uses: 45, attrs: [['crea', .7], ['int', .3]] },
    ] },
    { name: '音乐', ch: [
      { id: 'guitar', name: '吉他', uses: 60, attrs: [['crea', .5], ['cha', .5]] },
    ] },
  ] },
  { id: 'tech', name: '技术', ch: [
    { name: '编程', ch: [
      { id: 'fe', name: '前端开发', uses: 200, attrs: [['int', .6], ['crea', .4]] },
      { id: 'rust', name: 'Rust', uses: 30, attrs: [['int', .8], ['will', .2]] },
    ] },
  ] },
  { id: 'body', name: '身体', ch: [
    { name: '运动', ch: [
      { id: 'run', name: '跑步', uses: 90, attrs: [['vit', .7], ['will', .3]] },
      { id: 'gym', name: '力量训练', uses: 20, attrs: [['vit', .8], ['will', .2]] },
    ] },
  ] },
];

export const SEED_SETTINGS: Settings = {
  nickname: '堉钊',
  avatar: '钊',
  birth: '2002-05-20',
  lifeExp: 80,
  remind: '20:00',
  motion: true,
  lockHistory: false,
};

export const SEED_PROJECTS: Project[] = [
  {
    id: 'p-thesis', name: '毕业设计 · 萌趣记账小程序', start: '2026-03-01', status: 'doing',
    events: [
      { date: '2026-03-15', title: '开题与架构定型', tags: ['前端开发', '原型设计'], gains: [['int', 1], ['crea', 1]] },
      { date: '2026-05-20', title: 'AI 记账模块联调（GLM 接入）', tags: ['前端开发', '提示词设计'], gains: [['int', 1], ['sen', 1]] },
      { date: '2026-08-28', title: '论文初稿 + 去AI化打磨', tags: ['学术写作'], gains: [['int', 1], ['will', 1]] },
    ],
  },
  {
    id: 'p-ptz', name: 'PTZ 目标跟踪系统调研', start: '2025-10-01', end: '2025-12-20', status: 'done',
    events: [
      { date: '2025-10-18', title: '跟踪算法选型与综述', tags: ['文献调研'], gains: [['int', 1], ['sen', 1]] },
      { date: '2025-12-20', title: '低成本方案验证收尾', tags: ['系统设计'], gains: [['int', 1], ['will', 1]] },
    ],
  },
];

/** 成就定义：check 基于统计值判定，prog 用于未解锁卡背进度文案 */
export const ACHIEVEMENTS: AchievementDef[] = [
  { id: 'first', name: '初次觉醒', rarity: '普通', rc: '#6B7280', icon: 'star', desc: '完成第一次每日记录', hidden: true, revealAt: 0.5, check: (s) => s.totalDays >= 1, prog: (s) => `已记录 ${s.totalDays} / 1 天` },
  { id: 'twin', name: '双线并进', rarity: '普通', rc: '#6B7280', icon: 'twin', desc: '单日点亮 2 项以上属性', hidden: true, requires: ['first'], revealAt: 0.5, check: (s) => s.maxLitOneDay >= 2, prog: (s) => `单日最高点亮 ${Math.min(s.maxLitOneDay, 2)} / 2 项` },
  { id: 'week7', name: '七日之约', rarity: '稀有', rc: '#2F6FED', icon: 'moon', desc: '连续记录 7 天不间断', check: (s) => s.streak >= 7, prog: (s) => `当前连续 ${s.streak} / 7 天` },
  { id: 'dawn', name: '破晓之光', rarity: '稀有', rc: '#2F6FED', icon: 'sun', desc: '任意属性达到 LV 10', check: (s) => s.maxAttrLv >= 10, prog: (s) => `当前最高属性 LV ${s.maxAttrLv} / 10` },
  { id: 'd100', name: '百日筑基', rarity: '史诗', rc: '#7C3AED', icon: 'mountain', desc: '累计记录 100 天', check: (s) => s.totalDays >= 100, prog: (s) => `已记录 ${s.totalDays} / 100 天` },
  { id: 'allsix', name: '六艺俱全', rarity: '史诗', rc: '#7C3AED', icon: 'gem', desc: '单日点亮全部六项属性', hidden: true, requires: ['twin'], revealAt: 0.4, check: (s) => s.maxLitOneDay >= s.totalAttrs, prog: (s) => `历史最高 ${Math.min(s.maxLitOneDay, s.totalAttrs)} / ${s.totalAttrs}` },
  { id: 'done1', name: '完稿', rarity: '传说', rc: '#FFB020', icon: 'scroll', desc: '完成第一个项目', legendary: true, requires: ['week7', 'dawn'], check: (s) => s.projectsDone >= 1, prog: (s) => `已完成 ${s.projectsDone} / 1 个项目` },
  { id: 'grand', name: '宗师之路', rarity: '传说', rc: '#FFB020', icon: 'crown', desc: '任意技能达到宗师Ⅴ（LV 20）', legendary: true, requires: ['dawn'], check: (s) => s.maxSkillLv >= 20, prog: (s) => `当前最高 ${s.maxSkillLv} / 20` },
  { id: 'tenk', name: '万时之功', rarity: '传说', rc: '#FFB020', icon: 'scroll', desc: '某个技能累计打卡 10,000 天', legendary: true, requires: ['d100'], check: (s) => s.maxSkillCheckins >= 10000, prog: (s) => `当前最高 ${s.maxSkillCheckins.toLocaleString()} / 10,000 天` },
];

/** 种子已达成的成就（日期对齐原型演示） */
export const SEED_UNLOCKED: Record<string, string> = {
  first: '2026-06-14', twin: '2026-06-15', week7: '2026-06-20',
  dawn: '2026-07-02', d100: '2026-08-30', done1: '2025-12-20',
};

/* ---------- 84 天演示记录（确定性伪随机，今天留空供打卡） ---------- */
const CYCLE = [2, 0, 1, 3, 0, 2, 4, 1, 2, 0, 3, 1, 0, 2];
const LEAVES = ['sketch', 'oil', 'digi', 'guitar', 'fe', 'rust', 'run', 'gym'];
const ATTR_ROTATION = ['int', 'crea', 'vit', 'sen', 'will', 'cha'];

export function seedRecords(): DayRecord[] {
  const out: DayRecord[] = [];
  for (let i = 84; i >= 1; i--) {
    const lit = CYCLE[i % CYCLE.length];
    if (lit === 0) continue;
    const attrs: string[] = [];
    for (let k = 0; k < lit; k++) {
      const a = ATTR_ROTATION[(i * 3 + k * 2) % ATTR_ROTATION.length];
      if (!attrs.includes(a)) attrs.push(a);
    }
    const nSkills = Math.min(3, Math.max(1, lit - 1));
    const skillIds: string[] = [];
    for (let k = 0; k < nSkills; k++) {
      const s = LEAVES[(i * 5 + k * 3) % LEAVES.length];
      if (!skillIds.includes(s)) skillIds.push(s);
    }
    out.push({ date: daysAgoKey(i), skillIds, attrsLit: attrs });
  }
  return out;
}

export function seedState(): AppState {
  return {
    version: 1,
    attrs: JSON.parse(JSON.stringify(SEED_ATTRS)),
    skills: JSON.parse(JSON.stringify(SEED_SKILLS)),
    records: seedRecords(),
    projects: JSON.parse(JSON.stringify(SEED_PROJECTS)),
    unlocked: { ...SEED_UNLOCKED },
    settings: { ...SEED_SETTINGS },
  };
}
