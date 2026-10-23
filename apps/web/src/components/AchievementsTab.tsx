'use client';
/* ================= 成就页：收集总览 + 卡牌墙（纸本图鉴式卡面） =================
   设计依据：docs/card-face-design.md（对应《SOLOUP 成就卡牌视觉设计规范 v1.0》）
   原型：.workbuddy/proto/achievements-cards-v5.html

   成就定义（名称 / 描述 / 稀有度 / 隐藏与前置 / 达成条件）全部来自后端 achievements 表，
   前端只保留两类展示映射：稀有度 → 工艺档、成就 id → 徽记。

   ★ 卡面是「档案标签」不是「游戏战利品」：
     卡面永远纸白，稀有度只染 徽记 / 进度条 / 一枚 10px 标签，
     不整张染色、不用彩虹箔、不用发光描边。金卡的金色面积 ≤ 卡面 8%。

   ★ 一条横向基准：所有区段共用 .acard-face 的左右 padding（14px）。
     永远不要再给单个区段单独写 left/right —— 那正是 v1.4 参差的来源。

   ★ 字号只有 5 档：19 / 15 / 12 / 10.5 / 9。
     缩到看不清的字就要删，不要留着当装饰噪点。

   ★ 纵向：.acard 固定 aspect-ratio:.78，插画窗 flex:1 吸收余量（29%~40%），
     其余区段全是 flex:none。任何区段都不要写百分比 height ——
     卡高由宽度推出，百分比 height 的基准是父级高度，两者一混就错位。
*/
import { useMemo, useState } from 'react';
import type { AchStats, AchievementDef, AppState, Rarity } from '@/lib/types';
import { achProgress, deriveStats } from '@/lib/model';
import { AchIcon } from './icons';

const RARITY_CODE: Record<Rarity, string> = { 普通: 'N', 稀有: 'R', 史诗: 'SR', 传说: 'SSR' };
const RARITY_COLOR: Record<Rarity, string> = {
  普通: 'var(--rn)',
  稀有: 'var(--rr)',
  史诗: 'var(--rsr)',
  传说: 'var(--rssr)',
};
/** 稀有度 → 卡面工艺档（CSS 里的 data-r）。稀有度决定主题色，状态决定内容可见性，两者独立。 */
const RARITY_KEY: Record<Rarity, 'common' | 'rare' | 'epic' | 'legendary'> = {
  普通: 'common',
  稀有: 'rare',
  史诗: 'epic',
  传说: 'legendary',
};
/** 稀有度符号：文字标签之外的第二编码，不能只靠颜色区分 */
const RARITY_SIGIL: Record<Rarity, string> = { 普通: '○', 稀有: '◆', 史诗: '★', 传说: '✦' };
/** 成就类型 → 卡面类型（后端 `type` 是英文枚举，自定义成就会落到「里程」） */
const TYPE_ZH: Record<string, string> = {
  skill: '技能',
  attribute: '属性',
  project: '项目',
  milestone: '里程',
};
const typeKey = (t: string) => (t in TYPE_ZH ? t : 'milestone');
/** 徽记分配：**按名字优先**。
 *
 *  库里的 22 条成就有一多半是自定义的，id 是 UUID 随机串 —— 按 id 映射必然落空。
 *  名字才是稳定语义（改 id 不改语义），所以：名字 → 内置 id → 类型兜底 → 星形兜底。
 *  自定义名称撞不上表也不会缺图，只是画成类型图示。
 */
const SIGIL_BY_NAME: Record<string, string> = {
  初次觉醒: 'spark',
  三日不退: 'dots3',
  双线并进: 'rails',
  一周之始: 'cal1',
  开锋: 'blade',
  七日之约: 'cal',
  破晓之光: 'sunrise',
  成器: 'vessel',
  十炼成钢: 'ingot',
  月满: 'moon',
  四象: 'quadrant',
  百日筑基: 'bricks',
  百炼: 'flame',
  六艺俱全: 'hex',
  独当一面: 'pillar',
  完稿: 'scroll',
  千锤: 'anvil',
  岁轮: 'rings',
  宗师之路: 'peak',
  双作: 'stack2',
  三部作: 'stack3',
  万时之功: 'hourglass',
};
/** 种子成就 id → 徽记（名字被改过时仍能对上） */
const SIGIL_BY_ID: Record<string, string> = {
  first: 'spark',
  twin: 'rails',
  week7: 'cal',
  dawn: 'sunrise',
  d100: 'bricks',
  allsix: 'hex',
  done1: 'scroll',
  grand: 'peak',
  tenk: 'hourglass',
};
/** 类型兜底：自定义成就连名字都不认识时，至少画成「它属于哪一类」 */
const SIGIL_BY_TYPE: Record<string, string> = {
  skill: 'blade',
  attribute: 'quadrant',
  project: 'stack2',
  milestone: 'cal',
};
const sigilOf = (def: AchievementDef) =>
  SIGIL_BY_NAME[def.name] ?? SIGIL_BY_ID[def.id] ?? SIGIL_BY_TYPE[typeKey(def.type)] ?? 'star';
/** 插画窗场景 6 种。按索引轮转而不是随机：首屏稳定，且相邻两张必不同 */
const SCENES = ['rays', 'rings', 'wave', 'dots', 'diag', 'stars'] as const;
const RARITY_ORDER: Rarity[] = ['普通', '稀有', '史诗', '传说'];

/* ---------- 指针跟随 ----------
   只写 transform，不做 JS 插值：倾角上限 4deg（手册第 8.3 节），触屏不绑。 */
const MAX_TILT = 4;

export function AchievementsTab({ state }: { state: AppState }) {
  const stats = useMemo(() => deriveStats(state), [state]);
  // 后端按 type/rarity/name 排序下发；面板改按难度分（points）升序摆放，梯度从易到难
  const list = useMemo(
    () => [...state.achievements].sort((a, b) => a.points - b.points || a.id.localeCompare(b.id)),
    [state.achievements],
  );
  const total = list.length;
  const unlockedCount = list.filter((a) => state.unlocked[a.id]).length;

  const byRarity = (r: Rarity) => {
    const all = list.filter((a) => a.rarity === r);
    const got = all.filter((a) => state.unlocked[a.id]).length;
    return { all: all.length, got };
  };

  return (
    <section className="page" aria-label="成就卡册">
      <div className="section-title" style={{ marginTop: 4 }}>
        <h2>成就卡册</h2>
      </div>

      {/* 总览：像卡册的目录页 —— 左计数 / 中分段条 / 右百分比 / 下分档明细 */}
      <div className="coll">
        <div className="coll-top">
          <div className="coll-num">
            {unlockedCount}
            <small> / {total}</small>
          </div>
          <div className="coll-seg">
            {RARITY_ORDER.map((r) => {
              const { all, got } = byRarity(r);
              const w = total > 0 ? (all / total) * 100 : 0;
              return (
                <div className="coll-track" key={r} style={{ width: `${w}%` }}>
                  <i
                    style={{
                      width: all > 0 ? `${(got / all) * 100}%` : 0,
                      background: RARITY_COLOR[r],
                    }}
                  />
                </div>
              );
            })}
          </div>
          <div className="coll-pct">
            {total > 0 ? Math.round((unlockedCount / total) * 100) : 0}%
          </div>
        </div>
        <div className="coll-legend">
          {RARITY_ORDER.map((r) => {
            const { all, got } = byRarity(r);
            return (
              <span key={r}>
                <i style={{ background: RARITY_COLOR[r] }} />
                {RARITY_CODE[r]} {r} <b>{got}</b>/{all}
              </span>
            );
          })}
        </div>
      </div>

      <div className="card-wall">
        {list.map((a, index) => (
          <Acard
            key={a.id}
            def={a}
            no={index + 1}
            index={index}
            unlocked={!!state.unlocked[a.id]}
            date={state.unlocked[a.id]}
            stats={stats}
            unlockedMap={state.unlocked}
          />
        ))}
      </div>
    </section>
  );
}

function Acard({
  def,
  no,
  index,
  unlocked,
  date,
  stats,
  unlockedMap,
}: {
  def: AchievementDef;
  no: number;
  index: number;
  unlocked: boolean;
  date?: string;
  stats: AchStats;
  unlockedMap: Record<string, string>;
}) {
  const [flip, setFlip] = useState(false);

  const depsMet = def.requires.length === 0 || def.requires.every((id) => !!unlockedMap[id]);
  const prog = achProgress(stats, def.condition);
  const truePct = prog ? Math.round(prog.ratio * 100) : 0;

  // 隐藏成就的三段式揭示：完全遮住 → 只露名字 → 全露
  let showName: boolean;
  let showDesc: boolean;
  let deepLocked = false;

  if (unlocked) {
    showName = true;
    showDesc = true;
  } else if (def.hidden) {
    if (!depsMet) {
      showName = false;
      showDesc = false;
      deepLocked = true;
    } else if (def.revealAt != null && prog) {
      if (prog.ratio >= def.revealAt) {
        showName = true;
        showDesc = true;
      } else {
        showName = true;
        showDesc = false;
      }
    } else {
      showName = false;
      showDesc = false;
    }
  } else {
    showName = true;
    showDesc = true;
  }

  /** 名字都没露出来的卡，进度、日期、百分比一律不给 —— 遮一半比不遮更漏 */
  const masked = !showName;
  /** 进度条至少留 2%：0% 会让槽看起来是空的，读作「没有进度」而不是「刚开始」。
   *  但底部那个数字只报真实值 —— 辅助文本不许谎报进度。 */
  const barPct = masked || !prog ? 0 : Math.max(2, truePct);
  const pctText = masked || !prog ? '—' : `${truePct}%`;
  /** 条件进度原文（"已记录 14 / 1 天" 这类）。它有 10~18 个字，塞不进进度条右端那种位置，
   *  所以放在描述下面一行；条右端只放百分比。 */
  const progText = masked || !prog ? '' : prog.text;
  const dateText = unlocked ? (date ?? '—') : '—';
  // 深锁（前置没点亮）与「隐藏但前置已满足」是两种不同的少话程度，别用同一句占位
  const descText = showDesc
    ? def.desc
    : showName
      ? '似乎快要浮出水面…'
      : deepLocked
        ? '与某张卡牌存在关联…'
        : '达成后揭晓';
  const stateText = unlocked ? '已达成' : masked ? '未揭示' : '进行中';
  const backCond = deepLocked ? '前置成就尚未点亮' : masked ? '尚未解锁，内容暂不揭示' : def.desc;

  return (
    <div className="acard-stage">
      <div
        className={`acard${flip ? ' flip' : ''}${unlocked ? '' : ' is-locked'}${
          masked ? ' is-masked' : ''
        }`}
        data-r={RARITY_KEY[def.rarity]}
        data-sc={SCENES[index % SCENES.length]}
        data-state={stateText === '已达成' ? 'unlocked' : masked ? 'hidden' : 'progress'}
        onClick={() => setFlip((f) => !f)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            setFlip((f) => !f);
          }
        }}
        onPointerMove={(e) => {
          if (!window.matchMedia('(hover: hover) and (pointer: fine)').matches) return;
          const el = e.currentTarget;
          const r = el.getBoundingClientRect();
          if (!r.width || !r.height) return;
          const rx = ((e.clientY - r.top) / r.height - 0.5) * -MAX_TILT;
          const ry = ((e.clientX - r.left) / r.width - 0.5) * MAX_TILT;
          // 抬升写在这里而不是 CSS：.acard 的 transform 已被倾斜占用，两处写必互相覆盖
          el.style.transform = `perspective(1200px) translateY(-2px) rotateX(${rx.toFixed(
            2,
          )}deg) rotateY(${ry.toFixed(2)}deg)`;
        }}
        onPointerLeave={(e) => {
          e.currentTarget.style.transform = '';
        }}
        role="button"
        tabIndex={0}
        aria-label={`成就卡牌：${showName ? def.name : '未揭示'}，${def.rarity}，${
          unlocked ? '已解锁' : '未解锁'
        }`}
        aria-pressed={flip}
      >
        <div className="acard-3d">
          <div className="acard-face front">
            <div className="af-meta">
              <span className="af-no">NO.{String(no).padStart(3, '0')}</span>
              <span className="af-sep" />
              <span>{TYPE_ZH[typeKey(def.type)]}</span>
              <span className="af-tag">
                <s>{RARITY_SIGIL[def.rarity]}</s>
                {def.rarity}
              </span>
            </div>

            <div className="af-title">
              <div className="af-name">{showName ? def.name : '？？？'}</div>
              <div className="af-pts">
                PTS<b>{def.points}</b>
              </div>
            </div>

            <div className="af-art">
              <AchIcon name={masked ? 'seal' : sigilOf(def)} />
            </div>

            <div className="af-cond">
              <div className="af-main">{descText}</div>
              <div className="af-sub">{progText || '—'}</div>
            </div>

            <div className="af-bar">
              <div className="af-track">
                <i className="af-fill" style={{ width: `${barPct}%` }} />
              </div>
              <div className="af-ratio">{pctText}</div>
            </div>

            <div className="af-data">
              <span>{stateText}</span>
              <span>{dateText}</span>
            </div>

            <div className="af-foot">
              <span>SOLOUP</span>
              <span className="af-glyph">{RARITY_SIGIL[def.rarity]}</span>
            </div>
          </div>

          {/* 背面：档案卡式数据表。手册第 5.6 节要求「不能只放一个大星号和大片空白」 */}
          <div className="acard-face back">
            <div className="ab-top">
              <i />
              {RARITY_SIGIL[def.rarity]} {def.rarity} · NO.{String(no).padStart(3, '0')}
            </div>
            <div className="ab-name">{showName ? def.name : '？？？'}</div>
            <div className="ab-rule" />
            <div className="ab-rows">
              <div className="ab-row">
                <span>状态</span>
                <b>{stateText}</b>
              </div>
              <div className="ab-row">
                <span>条件</span>
                <b className="mut">{backCond}</b>
              </div>
              <div className="ab-row">
                <span>进度</span>
                <b>{masked || !prog ? '—' : prog.text}</b>
              </div>
              <div className="ab-row">
                <span>达成</span>
                <b className="mut">{dateText}</b>
              </div>
            </div>
            <div className="ab-bottom">
              <div className="ab-track">
                <i className="ab-fill" style={{ width: `${barPct}%` }} />
              </div>
              <div className="ab-mark">SOLOUP · ACHIEVEMENT</div>
            </div>
          </div>
        </div>
      </div>
      <div className="acard-hint">点击翻面</div>
    </div>
  );
}
