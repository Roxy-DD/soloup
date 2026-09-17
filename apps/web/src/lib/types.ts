/* ================= 人生 RPG 面板 · 类型定义 ================= */

/** 属性（体质/智力/创造…）：数值全部由后端派生，前端只负责展示，不再本地推导 */
export interface Attr {
  id: string;
  name: string;
  en: string;
  color: string; // 深字色
  tint: string; // 淡底色（WCAG AA 配对）
  /** 后端派生的连续值（0–100） */
  value: number;
  /** 面板等级（0–50），由 value 折半取整得到 */
  lv: number;
  /** 当前等级内的进度（0–1） */
  frac: number;
}

/** 属性色板项：[深字色, 淡底色] */
export type PalettePair = [string, string];

/** 叶子技能：可被打卡；等级由后端曲线派生，前端不再自行推导 */
export interface SkillLeaf {
  id: string;
  name: string;
  /** 后端派生的连续等级（0–100） */
  level: number;
  /** 面板等级（整数，= ⌊level⌋） */
  lv: number;
  /** 当前等级内的进度（0–1） */
  frac: number;
  /** 累计打卡天数（后端按每日记录统计） */
  checkins: number;
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

/** 成就条件 DSL（后端 achievements.condition_json）：左侧取一个统计量，右侧取常量阈值或另一个统计量 */
export interface AchCondition {
  /** 左侧统计量名（后端 stats 口径：totalDays / streak / maxLitOneDay / maxSkillLv / maxSkillCheckins / projectsCompleted / totalAttrs） */
  stat: string;
  /** 比较符：>= > == <= < */
  operator: string;
  /** 常量阈值；与 ref 二选一 */
  value?: number;
  /** 右值改取另一个统计量（如 allsix：单日点亮数 ≥ 属性总数） */
  ref?: string;
}

/** 成就定义：全部字段来自后端 achievements 表（名称/描述/稀有度/揭示/条件），前端不再持有定义常量。
 *  图标与配色属于展示资源，由页面按 id / rarity 映射，不入此结构。 */
export interface AchievementDef {
  id: string;
  name: string;
  desc: string;
  rarity: Rarity;
  points: number;
  /** 后端枚举字符串：skill / attribute / project / milestone */
  type: string;
  condition: AchCondition | null;
  /** 隐藏成就：未解锁时按 revealAt 渐进揭示，而非直接展示 */
  hidden: boolean;
  /** 前置成就 id 列表：全部解锁后该卡才开始显现 */
  requires: string[];
  /** 渐进揭示阈值（0–1）：进度达到此比例时显示名称和提示 */
  revealAt: number | null;
}

export interface AppState {
  version: 1;
  attrs: Attr[];
  skills: SkillGroup[];
  records: DayRecord[];
  projects: Project[];
  /** 成就定义（后端 achievements 表下发） */
  achievements: AchievementDef[];
  /** 成就解锁表：id → 解锁日期 */
  unlocked: Record<string, string>;
  settings: Settings;
}
