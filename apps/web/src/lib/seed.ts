/* ================= 种子数据（后端不可用时的兜底占位） =================
   这里的字段口径与后端派生结果保持一致（属性 value 0–100、技能 level 0–100 连续值），
   但不需要复刻成长曲线——后端可用时整份数据会被 bootstrap 直接覆盖。 */
import type { AchievementDef, AppState, Attr, DayRecord, Project, Settings, SkillGroup, SkillLeaf } from './types';
import { daysAgoKey } from './model';

/** 由面板等级 + 级内进度构造属性（面板折半展示：0–50 级）。 */
function mkAttr(id: string, name: string, en: string, color: string, tint: string, lv: number, frac: number): Attr {
  return { id, name, en, color, tint, value: (lv + frac) * 2, lv, frac };
}

/** 由后端口径的连续等级构造叶子技能（面板等级 = 向下取整）。 */
function mkLeaf(id: string, name: string, level: number, checkins: number, attrs: [string, number][]): SkillLeaf {
  const lv = Math.floor(level);
  return { id, name, level, lv, frac: level - lv, checkins, attrs };
}

export const SEED_ATTRS: Attr[] = [
  mkAttr('vit', '体质', 'VIT', '#C6352B', '#FDE8E6', 24, 0.49),
  mkAttr('int', '智力', 'INT', '#2456C4', '#E8EFFF', 35, 0.91),
  mkAttr('crea', '创造', 'CREA', '#6D28D9', '#F1E9FF', 29, 0.55),
  mkAttr('will', '意志', 'WILL', '#B45309', '#FFF1D6', 18, 0.23),
  mkAttr('cha', '魅力', 'CHA', '#BE185D', '#FDE9F3', 12, 0.03),
  mkAttr('sen', '感知', 'SEN', '#047857', '#DFF7EE', 21, 0.11),
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
      mkLeaf('sketch', '素描', 41.3, 62, [['crea', .5], ['sen', .5]]),
      mkLeaf('oil', '油画', 18.7, 11, [['crea', .6], ['sen', .4]]),
      mkLeaf('digi', '数字绘画', 33.5, 38, [['crea', .7], ['int', .3]]),
    ] },
    { name: '音乐', ch: [
      mkLeaf('guitar', '吉他', 27.9, 29, [['crea', .5], ['cha', .5]]),
    ] },
  ] },
  { id: 'tech', name: '技术', ch: [
    { name: '编程', ch: [
      mkLeaf('fe', '前端开发', 58.4, 71, [['int', .6], ['crea', .4]]),
      mkLeaf('rust', 'Rust', 22.1, 24, [['int', .8], ['will', .2]]),
    ] },
  ] },
  { id: 'body', name: '身体', ch: [
    { name: '运动', ch: [
      mkLeaf('run', '跑步', 44.6, 45, [['vit', .7], ['will', .3]]),
      mkLeaf('gym', '力量训练', 15.2, 15, [['vit', .8], ['will', .2]]),
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

/** 成就兜底数据（后端不可用时展示）。
 *  字段口径与后端 achievements 表一致：条件为 stat 阈值 DSL，判定与展示都交给同一套通用逻辑，
 *  这里不再携带 check / prog 函数——加了新成就也只需照抄后端那一行。 */
export const SEED_ACHIEVEMENTS: AchievementDef[] = [
  { id: 'first', name: '初次觉醒', desc: '完成第一次每日记录', rarity: '普通', points: 10, type: 'milestone', condition: { stat: 'totalDays', operator: '>=', value: 1 }, hidden: true, requires: [], revealAt: 0.5 },
  { id: 'twin', name: '双线并进', desc: '单日点亮 2 项以上属性', rarity: '普通', points: 20, type: 'attribute', condition: { stat: 'maxLitOneDay', operator: '>=', value: 2 }, hidden: true, requires: ['first'], revealAt: 0.5 },
  { id: 'week7', name: '七日之约', desc: '连续记录 7 天不间断', rarity: '稀有', points: 30, type: 'milestone', condition: { stat: 'streak', operator: '>=', value: 7 }, hidden: false, requires: [], revealAt: null },
  { id: 'dawn', name: '破晓之光', desc: '任一技能达到 4 级', rarity: '稀有', points: 40, type: 'skill', condition: { stat: 'maxSkillLv', operator: '>=', value: 4 }, hidden: false, requires: [], revealAt: null },
  { id: 'd100', name: '百日筑基', desc: '累计记录 100 天', rarity: '史诗', points: 100, type: 'milestone', condition: { stat: 'totalDays', operator: '>=', value: 100 }, hidden: false, requires: [], revealAt: null },
  { id: 'allsix', name: '六艺俱全', desc: '单日点亮全部属性', rarity: '史诗', points: 120, type: 'attribute', condition: { stat: 'maxLitOneDay', operator: '>=', ref: 'totalAttrs' }, hidden: true, requires: ['twin'], revealAt: 0.4 },
  { id: 'done1', name: '完稿', desc: '完成第一个项目', rarity: '传说', points: 150, type: 'project', condition: { stat: 'projectsCompleted', operator: '>=', value: 1 }, hidden: false, requires: ['week7', 'dawn'], revealAt: null },
  { id: 'grand', name: '宗师之路', desc: '任一技能达到 20 级', rarity: '传说', points: 200, type: 'skill', condition: { stat: 'maxSkillLv', operator: '>=', value: 20 }, hidden: false, requires: ['dawn'], revealAt: null },
  { id: 'tenk', name: '万时之功', desc: '某个技能累计打卡 10,000 天', rarity: '传说', points: 300, type: 'skill', condition: { stat: 'maxSkillCheckins', operator: '>=', value: 10000 }, hidden: false, requires: ['d100'], revealAt: null },
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
    achievements: JSON.parse(JSON.stringify(SEED_ACHIEVEMENTS)),
    unlocked: { ...SEED_UNLOCKED },
    settings: { ...SEED_SETTINGS },
  };
}
