'use client';
/* ================= 成就页：收集进度 + 卡牌墙（宝可梦式卡面 + 箔层） =================
   成就定义（名称 / 描述 / 稀有度 / 隐藏与前置 / 达成条件）全部来自后端 achievements 表，
   前端只保留两类展示映射：稀有度 → 工艺档、成就 id → 徽记。
   进度文案由条件 DSL 通用生成（见 model.achProgress），不按 id 写死。

   ★ 卡面结构照搬宝可梦实体卡，不是随便拍的：
     外框（金）→ 名字带（名字 / PTS / 类型球）→ 插画窗（徽记 + 场景）→ 属性银条
     → 条件区（进度大字 + 说明）→ 两格数据银条（进度 / 达成日）→ 页脚（ILLUS. + 稀有度符号）。
     稀有度差的是**箔的面积**（无箔 / 仅插画窗 / 插画窗外 / 整卡换材质），不是亮度数值。

   ★ 一条横向基准 --px：头 / 窗 / 银条 / 条件 / 两格 / 页脚全部对齐它。
     曾经各段各写缩进，最大差 3.4px，肉眼看就是「参差」。
   ★ 字号只有 5 档：17 / 16 / 19 / 9.5 / 7。宝可梦真卡那排 ILLUS. 是印刷级精度，
     缩到 1/3 就成噪点 —— 所以宁可少一件元素，不留一件看不清的。
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
/** 稀有度 → 卡面工艺档（CSS 里的 data-r） */
const RARITY_KEY: Record<Rarity, 'common' | 'rare' | 'epic' | 'legendary'> = {
  普通: 'common',
  稀有: 'rare',
  史诗: 'epic',
  传说: 'legendary',
};
/** 稀有度符号（页脚右端那枚） */
const RARITY_SIGIL: Record<Rarity, string> = { 普通: '○', 稀有: '◆', 史诗: '★', 传说: '✦' };
/** 成就类型 → 卡面类型色（后端 `type` 是英文枚举，自定义成就会落到「里程」） */
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

/* ---------- 指针跟随 ----------
   只写 CSS 变量，不做 JS 插值：变量在 CSS 里用 @property 注册成 <percentage> / <angle>，
   transition 就能插值。省掉一条 requestAnimationFrame 循环，也不怕组件重渲染。
   常亮基准让卡在静息态就有金属感（宝可梦卡的静息态也是亮的，不是等着被摸才亮）。 */
const MAX_TILT = 9;
const REST = { px: 38, py: 24 };
const clamp = (v: number, a: number, b: number) => Math.min(b, Math.max(a, v));

function applyTilt(el: HTMLElement, px: number, py: number) {
  el.style.setProperty('--pointer-x', `${px.toFixed(1)}%`);
  el.style.setProperty('--pointer-y', `${py.toFixed(1)}%`);
  el.style.setProperty('--background-x', `${(37 + (px / 100) * 26).toFixed(1)}%`);
  el.style.setProperty('--background-y', `${(33 + (py / 100) * 27).toFixed(1)}%`);
  el.style.setProperty('--rotate-x', `${(-((px - 50) / 50) * MAX_TILT).toFixed(2)}deg`);
  el.style.setProperty('--rotate-y', `${(((py - 50) / 50) * MAX_TILT).toFixed(2)}deg`);
  el.style.setProperty(
    '--pointer-from-center',
    clamp(Math.hypot(px - 50, py - 50) / 50, 0, 1).toFixed(3),
  );
}

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
      <div className="card" style={{ marginBottom: 18 }}>
        <div className="coll-head">
          <div className="coll-num">
            {unlockedCount}
            <small> / {total}</small>
          </div>
          <div className="coll-bar">
            {(['普通', '稀有', '史诗', '传说'] as Rarity[]).map((r) => {
              const { all, got } = byRarity(r);
              const w = total > 0 ? (all / total) * 100 : 0;
              return (
                <i
                  key={r}
                  style={{ width: `${w}%`, background: RARITY_COLOR[r], position: 'relative' }}
                >
                  {got > 0 && all > 0 && (
                    <i
                      style={{
                        position: 'absolute',
                        inset: 0,
                        width: `${(got / all) * 100}%`,
                        background: RARITY_COLOR[r],
                        borderRight: '2px solid var(--card)',
                      }}
                    />
                  )}
                </i>
              );
            })}
          </div>
          <div className="coll-pct">
            {total > 0 ? Math.round((unlockedCount / total) * 100) : 0}%
          </div>
        </div>
        <div className="coll-legend">
          {(['普通', '稀有', '史诗', '传说'] as Rarity[]).map((r) => {
            const { all, got } = byRarity(r);
            return (
              <span key={r}>
                <i style={{ background: RARITY_COLOR[r] }} />
                {RARITY_CODE[r]} {r} ×{got}/{all}
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
  const ratio = prog?.ratio ?? 0;
  /** 进度条至少留 2%：0% 会让槽看起来是空的，读作「没有进度」而不是「刚开始」 */
  const pct = Math.max(2, Math.round(ratio * 100));

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

  const descText = showDesc
    ? def.desc
    : showName
      ? '似乎快要浮出水面…'
      : deepLocked
        ? '与某张卡牌存在关联…'
        : '达成后揭晓';

  return (
    <div className="acard-stage">
      <div
        className={`acard${flip ? ' flip' : ''}${unlocked ? '' : ' locked'}${
          showName ? '' : ' masked'
        }${deepLocked ? ' deep' : ''}`}
        data-r={RARITY_KEY[def.rarity]}
        data-t={typeKey(def.type)}
        data-sc={SCENES[index % SCENES.length]}
        onClick={() => setFlip((f) => !f)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            setFlip((f) => !f);
          }
        }}
        onPointerMove={(e) => {
          const el = e.currentTarget;
          const r = el.getBoundingClientRect();
          if (!r.width || !r.height) return;
          applyTilt(
            el,
            clamp(((e.clientX - r.left) / r.width) * 100, 0, 100),
            clamp(((e.clientY - r.top) / r.height) * 100, 0, 100),
          );
        }}
        onPointerLeave={(e) => applyTilt(e.currentTarget, REST.px, REST.py)}
        role="button"
        tabIndex={0}
        aria-label={`成就卡牌：${showName ? def.name : '未揭示'}，${def.rarity}`}
        aria-pressed={flip}
      >
        <div className="acard-inner">
          <div className="acard-face front">
            <div className="acard-field">
              <div className="acard-gold" />
              <div className="acard-shine" />
              <div className="acard-art" />
              <div className="acard-glare" />
              <div className="zone-emb">
                <AchIcon name={sigilOf(def)} />
              </div>

              <div className="zone-head">
                <div className="acard-nm">{showName ? def.name : '？？？'}</div>
                <div className="acard-pts">
                  <s>PTS</s>
                  <b>{def.points}</b>
                </div>
                <i className="acard-orb" />
              </div>

              <div className="zone-band acard-silver">
                <span>
                  NO.<b>{String(no).padStart(3, '0')}</b>
                </span>
                <span>{TYPE_ZH[typeKey(def.type)]}</span>
                <span className="acard-rar">{def.rarity}</span>
              </div>

              <div className="zone-cond">
                <div className="acard-big">{showDesc && prog ? prog.text : '???'}</div>
                <div className="acard-text">{descText}</div>
              </div>

              <div className="zone-wrr acard-silver">
                <div className="acell">
                  <span className="acell-label">进度</span>
                  <span className="acell-val">{prog ? `${pct}%` : '—'}</span>
                </div>
                <div className="acell">
                  <span className="acell-label">达成日</span>
                  <span className="acell-val">{unlocked ? (date ?? '—') : '—'}</span>
                </div>
              </div>
            </div>

            <div className="zone-foot">
              <span className="acard-fnote">ILLUS.&nbsp;SOLOUP</span>
              <span className="acard-glyph">{RARITY_SIGIL[def.rarity]}</span>
            </div>
          </div>

          <div className="acard-face back">
            <div className="acard-backmark">{unlocked ? '★' : deepLocked ? '⚿' : '？'}</div>
            <div className="acard-btxt">
              {unlocked ? (
                <>
                  <b>已经拿到</b>
                  <br />
                  {def.desc}
                </>
              ) : deepLocked ? (
                <>前置成就还没点亮，这张卡暂时看不见内容</>
              ) : (
                <>
                  {prog ? prog.text : '条件尚未达成'}
                  <br />
                  <span style={{ opacity: 0.65 }}>还差一点</span>
                </>
              )}
            </div>
            {unlocked ? (
              date && <div className="acard-bdate">{date}</div>
            ) : (
              <div className="acard-bplit">
                <i style={{ width: `${pct}%` }} />
              </div>
            )}
            <div className="acard-bmark">{unlocked ? 'ACHIEVED' : 'LOCKED'}</div>
          </div>
        </div>
      </div>
      <div className="acard-hint">点击翻面</div>
    </div>
  );
}
