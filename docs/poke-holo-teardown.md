# 宝可梦全息卡面拆解（poke-holo）

> 学习对象：`simeydotme/pokemon-cards-css`（Simon Goellner，GPL-3.0）
> Demo：`poke-holo.simey.me`（国内需代理，`gh-proxy.com` 可拉源码）
> 动手验证：`.workbuddy/proto/poke-holo-lab.html`（逐层开关的解剖台，离线可跑）
> 日期：2026-10-23

**为什么拆它**：Soloup 的成就卡牌面要做稀有度分级，凭感觉调只会越调越花。这套是目前公开实现里把「金属箔」在屏幕上做得最像的，而且结构极简——只有 2 个效果 div。拆它的目的是拿机制，不是抄视觉。

---

## 一、架构：一个 base + 每个稀有度一个文件

`index.html` 里挂 26 个样式表：

```
css/global.css          页面
css/cards.css           全站色板 + clip-path 版面坐标 + 5 个稀有度色变量
css/cards/base.css      ★ 核心：tilt / shine / glare 的骨架
css/cards/basic.css     普通卡（无箔）
css/cards/regular-holo.css
css/cards/reverse-holo.css
css/cards/cosmos-holo.css
css/cards/v-full-art.css
css/cards/rainbow-holo.css
css/cards/secret-rare.css   …共 24 个
```

**每个稀有度文件只覆盖 `background-image` / `filter` / `mix-blend-mode` / `clip-path`，不碰结构。**

这对 Soloup 的直接启发：成就卡也应该是「一个 `AchievementCard` 组件 + 4 个档位样式」，而不是给每档写一套 JSX。

---

## 二、DOM 只有两层效果

```
.card
└ .card__rotator              旋转（rotateX/Y）
  ├ .card__front
  │ ├ img                      卡面图
  │ ├ .card__shine             ← 效果层 1（含 ::before / ::after 共 3 层）
  │ └ .card__glare             ← 效果层 2
```

**没有 9 层，没有 5 个动画。** 我上一版 SSR 叠了 9 层被你说"太花"，根因就在这里——原版只用 2 个 div。

分层不靠 `z-index`，靠真实深度：

```css
.card__shine          { transform: translateZ(1px);   }
.card__shine::before  { transform: translateZ(1px);   }
.card__shine::after   { transform: translateZ(1.2px); }
.card__glare          { transform: translateZ(1.41px); }
```

因为整张卡在 `preserve-3d` 里旋转，`z-index` 会乱，只有 3D 深度可靠。

---

## 三、JS 只做一件事：算数字

`Card.svelte` 里全部逻辑就是「鼠标位置 → 一组 CSS 变量」：

```js
--pointer-x / --pointer-y        鼠标在卡上的百分比
--pointer-from-center            到中心的归一化距离 0~1
--background-x / --background-y  压缩到 37%~63% 的背景位移
--rotate-x / --rotate-y          -(center / 3.5)，最大约 ±14°
--card-opacity                   0 → 1
--card-scale / --translate-x/y   卡片放大与位移
```

配 `svelte/motion` 的 spring（stiffness `0.066` / damping `0.25`）产生惯性。**CSS 里没有任何 JS 判断，JS 里没有任何样式。** 这条界线画得很干净。

---

## 四、五条核心机制

### 1. 位移系数 x/y 不同 → 光斜着走

```css
background-position:
  calc(((50% - var(--background-x)) * 2.6) + 50%)
  calc(((50% - var(--background-y)) * 3.5) + 50%);
```

x 乘 `2.6`、y 乘 `3.5`。两个不同系数让背景沿**斜向**位移——这就是"斜光带"的来源，不需要写任何角度。

配合 JS 端先把鼠标位置从 `0~100%` 压到 `37~63%`，位移幅度被限死在 ±34%，所以光不会追着鼠标跑。

### 2. 亮度是倾斜角的函数

```css
filter: brightness(calc(0.4 + var(--pointer-from-center) * 0.2));
```

鼠标离中心越远 = 卡倾得越狠 = 反光越强。**物理正确，不是装饰。**

### 3. 三层 shine，各用不同混合模式分工

```css
.card__shine          { mix-blend-mode: color-dodge; filter: brightness(.85) contrast(2.75) saturate(.65); }
.card__shine::before  { mix-blend-mode: hard-light;  }   /* 扫描线/竖条 */
.card__shine::after   { mix-blend-mode: luminosity;  filter: brightness(.6) contrast(4); }
```

- 主体 `color-dodge`：提亮 + 增饱和，是彩色感的来源
- `::before` `hard-light`：把 1px 黑白扫描线强行叠出对比，金属颗粒感来自这里
- `::after` `luminosity` + `contrast(4)`：**只取明暗不取颜色**，把柔和柔光压成锐利条纹

`contrast(2.75)` 和 `contrast(4)` 这两个高数值是关键——低对比度会让一切看起来像雾。

### 4. `::before` 和 `::after` 是同一纹理、色相错开一位

```css
/* ::before */ --sunpillar-clr-1: var(--sunpillar-5); --sunpillar-clr-2: var(--sunpillar-6); …
/* ::after  */ --sunpillar-clr-1: var(--sunpillar-6); --sunpillar-clr-2: var(--sunpillar-1); …
```

六色轮转一位再叠一层。同纹理异相位 → 色相不呆板。这比"一条彩虹"高级，成本却为零。

### 5. 分区靠 `clip-path` 的实测版面坐标

```css
--clip:         inset(9.85% 8% 52.85% 8%);   /* 插画区 */
--clip-invert:  polygon(…);                   /* 插画区以外（反向闪） */
--clip-stage:   polygon(…);                   /* 进化卡，左上多一块 */
--clip-trainer: inset(14.5% 8.5% 48.2% 8.5%); /* 人物卡 */
--clip-borders: inset(2.8% 4% round 2.55% / 1.5%); /* 边框（金卡） */
```

这些百分比是从真实卡面量出来的，不是估的。

---

## 五、最重要的发现：稀有度差的是「箔的面积」

对比三个文件，一眼看清：

| 档位 | 文件 | 分区 | 效果范围 |
|---|---|---|---|
| 普通 | `basic.css` | — | 无箔 |
| 闪（Holo） | `regular-holo.css` | `clip-path: var(--clip)` | 只在插画区 |
| 反向闪 | `reverse-holo.css` | `clip-path: var(--clip-invert)` | 插画区**以外** |
| 金 / 秘稀 | `secret-rare.css` | `mask-image: none !important` | **整卡全箔，包括边框** |

**三档用的是同一套 5 色、同一套层。** 差别只是裁到哪里、以及金卡换了材质（真实箔纸图 + glitter 颗粒 + `saturate(2.7)`）。

这一条直接否掉了我上一版的做法：我原来靠"光带宽度"分档（26% / 34% / 46% / 60%），那是**量变**；原版是**面积从局部到全卡**，是质变。而且原版把"反向闪"单独做成一档——箔在插画区之外，这个反差比任何亮度差别都明显。

---

## 六、实测踩到的坑（都在这台机器上跑过）

### 坑 1：混合模式是单向的，选错永远出不来颜色

做金卡预设时连错三轮，第一版几乎看不出变化：

| 混合模式 | 实际行为 | 结果 |
|---|---|---|
| `color-dodge` / `screen` | 只会变亮 | 金色被提成纯白 |
| `multiply` | 只会变暗 | 永远没有高光 |
| `overlay` | 强化底图**原有的**色相 | 橙卡上叠金 → 更橙 |
| **`color`** | 取源色的色相 + 饱和度，保留底的明暗 | ✅ 真的变金了 |

**要给一张图整体换金属色，只认 `color`。** 这条同样适用于 Soloup 的 SSR 银卡。

### 坑 2：`--card-opacity` 从 0 起，静态下什么都没

原版设计是「悬停才有光」——因为 demo 里一屏几十张卡，全亮会花。

但**卡册是静态一屏卡**，这个设计直接搬过来就会得到"一屏灰卡"。我给解剖台加了常亮基准 `HOLD = {o: .6}`，悬停才升到 `.82`。

这条和上一轮给 Soloup SSR 加静息箔光是同一个结论，现在有了第二份证据。

### 坑 3：`clip-path` 只裁父元素，光带不能靠子元素突破

`.card__shine` 加了 `clip-path` 后，`::before` / `::after` 一起被裁——这是想要的行为。但反过来，如果你想让某一层"漏出框外"，`clip-path` 会挡住，必须把它提到父级或换 `mask-image`。

### 坑 4：底色亮度决定 `color-dodge` 能不能用

`color-dodge` 的数学是 `结果 = 底 / (1 - 源)`：底越暗、提亮越猛；**底是纯白时恒等于白**。

宝可梦卡底是有颜色的插画，所以能出彩。Soloup 的卡是白底黑字，直接套 `color-dodge` 等于没画——这是上一轮已经踩过的坑，现在知道了原因。

---

## 七、迁到 Soloup：能抄 / 不能抄

### 能直接抄的（机制层）

| 机制 | 用在 Soloup 的哪里 |
|---|---|
| 两层 div 结构（shine + glare） | 替换现在 4 层的 SSR，砍掉 5 层高频纹理 |
| `background-position` 的 x/y 不同系数 | 现在我的斜光带是写死角度的，换成这个更自然 |
| `brightness` 挂在 `--pointer-from-center` 上 | 鼠标离卡中心越远越亮，比固定亮度真 |
| `::before` 色相错开一位 | 让稀疏的彩色不呆板，成本为零 |
| `luminosity + contrast(3~4)` 做锐利条纹 | 现在我的条纹偏软，就是缺这个 |
| 稀有度 = 箔的面积（含"反向"这一档） | 4 档改成：无 → 局部 → 局部反向 → 全卡 |
| `color` 混合模式染金属色 | SSR 银卡、金卡统一走这条 |

### 不能抄的

| 它的做法 | 为什么不行 |
|---|---|
| 整卡铺彩色插画作底 | Soloup 卡面是文字块，没有插画承担色彩 |
| 全卡全箔 | 会把文字一起糊掉，必须给文字留不透明底 |
| 悬停才有 `opacity` | 卡册静态一屏，必须常亮基准 |
| 固定版面百分比坐标 | Soloup 是流式排版，没有"插画区"，分区要按元素自身框来算 |
| Svelte spring | Soloup 是 React，用 lerp 或 `useSpring` 近似即可 |

---

## 八、下一步（等你定）

1. 用这套机制重写 Soloup 的 SSR（银底 + `color` 染银 + 光带按 x/y 系数斜走 + 只留 1 个动画）
2. 把稀有度分级从"光带宽度"改成"箔的面积"，并考虑加一档"反向"（箔只在文字区外/边框）
3. 成就卡补 `unlocked_at` 落库等机制（原计划 P0）

**没有你点头我不动现有卡面。**
