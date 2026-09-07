/* ================= 人生 RPG 面板 · 类型定义 ================= */

/** 属性（体质/智力/创造…）：ep 为累计经验，LV 由公式从 ep 推导，不直接存储 */
export interface Attr {
  id: string;
  name: string;
  en: string;
  color: string; // 深字色
  tint: string; // 淡底色（WCAG AA 配对）
  ep: number;
}

/** 属性色板项：[深字色, 淡底色] */
export type PalettePair = [string, string];

/** 叶子技能：可被打卡，uses 为累计使用次数 */
export interface SkillLeaf {
  id: string;
  name: string;
  uses: number;
  /** 关联属性 [attrId, 权重]，1–3 项，权重和为 1 */
  attrs: [string, number][];
  archived?: boolean;
}

/** 技能树节点：大类/子类（只有 ch）；id 由后端技能行提供（新增/移动时需回传） */
export interface SkillBranch {
  name: string;
  ch: SkillTreeNode[];
  id?: string;
}

export type SkillTreeNode = SkillBranch | SkillLeaf;
/** 顶层大类带 id */
export interface SkillGroup extends SkillBranch {
  id: string;
}

export function isLeaf(n: SkillTreeNode): n is SkillLeaf {
  return !('ch' in n);
}

/** 单日记录：date 为本地 YYYY-MM-DD */
export interface DayRecord {
  date: string;
  skillIds: string[];
  /** 当日点亮的属性（去重，每日每属性封顶 +1 EP） */
  attrsLit: string[];
}

export interface ProjectEvent {
  date: string;
  title: string;
  tags: string[];
  /** 事件带来的属性展示 [attrId, +n] */
  gains: [string, number][];
}

export interface Project {
  id: string;
  name: string;
  start: string;
  end?: string;
  status: 'doing' | 'done';
  events: ProjectEvent[];
}

export interface Settings {
  nickname: string;
  avatar: string;
  birth: string; // YYYY-MM-DD
  lifeExp: number;
  remind: string; // HH:mm
  motion: boolean;
  lockHistory: boolean;
}

export type Rarity = '普通' | '稀有' | '史诗' | '传说';

/** 成就判定输入：从全局状态推导的统计值 */
export interface AchStats {
  totalDays: number;
  streak: number;
  maxLitOneDay: number;
  maxAttrLv: number;
  maxSkillLv: number;
  maxSkillCheckins: number;
  projectsDone: number;
  totalAttrs: number;
}

export interface AchievementDef {
  id: string;
  name: string;
  rarity: Rarity;
  rc: string;
  icon: string; // icons.tsx 中的 key
  desc: string;
  legendary?: boolean;
  hidden?: boolean; // 隐藏成就：未解锁时显示 ???，已解锁后揭示
  /** 前置成就 id 列表：全部解锁后该卡才开始显现 */
  requires?: string[];
  /** 隐藏卡渐进揭示阈值（0–1）：进度达到此比例时显示名称和提示 */
  revealAt?: number;
  /** 达成条件（基于统计值判定） */
  check: (s: AchStats) => boolean;
  /** 未解锁时卡背进度文案 */
  prog?: (s: AchStats) => string;
}

export interface AppState {
  version: 1;
  attrs: Attr[];
  skills: SkillGroup[];
  records: DayRecord[];
  projects: Project[];
  /** 成就解锁表：id → 解锁日期 */
  unlocked: Record<string, string>;
  settings: Settings;
}
