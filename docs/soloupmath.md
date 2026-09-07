# Soloup 技术规格文档

> **版本**：v1.0（定稿）｜ **状态**：已定稿（2026-09-06）
> **定位**：个人人生 RPG 面板 —— 用可解释的数学模型跟踪技能成长、属性派生、项目轴与成就系统；提供本地 MCP 服务让 AI 作为"面板管家"参与管理。
> **本文性质**：可直接驱动编码的技术规格，含参数表、算法伪代码、MCP 工具清单、AI 协作规则与黄金测试用例。

---

## 0. 变更说明（相对初版需求稿）

| # | 变更 | 原因 |
|---|---|---|
| 1 | 技能等级模型确定**双轨制**：动作类用指数饱和，认知/知识类用 Logistic Sigmoid | 两派曲线分别对应"起步即见效、无拐点"与"前期蛰伏、有飞跃点"两种真实学习形态 |
| 2 | 结晶率 `c` 与遗忘率 `f` **解耦**，给出三档参数表 | 原稿 `c = f` 导致三类技能结晶速率都恒等于 0.5/天，分类形同虚设 |
| 3 | 属性改为**完全由技能派生**，"重复属性只增加一点"规则**废止** | 打卡不再直接加属性点；属性与技能的联动由公式自动保证，无需手工维护两套数据 |
| 4 | 引入**有效投入量 E** 统一术语，废弃 `effective_days` 字段名 | 原稿中"有效天数"一名同时指代两个不同概念 |
| 5 | 明确**逐日离散迭代为唯一权威算法**，连续形式仅用于展示 | 保证"补结算 N 天"与"连续 N 天逐日结算"结果严格一致，可测试 |
| 6 | 补齐时间语义、补结算、项目轴联动、成就 DSL、验收标准 | 原稿缺失，编码时必然产生歧义 |
| 7 | 权威数据源定为**本地 SQLite 单文件**（better-sqlite3），面板与 MCP 进程同库，浏览器不直连 | MCP 是独立本地进程，必须能直接读写同一数据源；浏览器 IndexedDB 无法被外部进程访问 |
| 8 | 部署形态：**MVP 本地一体**（Next.js 本地服务 + 可选 Tauri 壳），无云依赖 | 让本地 MCP 与面板共享一个数据源，AI 操作与人工操作所见即所得 |
| 9 | 新增 **§9 MCP 服务规格**（TypeScript `@modelcontextprotocol/sdk`）。v1 范围：只读查询 + 属性/技能/技能树/关联权重/难度/归档的管理 CRUD | 实现"AI 代为管理整个面板"的目标 |
| 10 | 新增 §4.2 分层（`core / store / solver`）与写入通道约定：**所有写操作必须走服务层**，禁止绕过不变量直接改库 | 防止 AI 或未来第三方写坏模型不变量（如直接改 C/V、破坏树结构） |
| 11 | 引入**参数三层**（公式层 / 默认层 / 语义层）与"最小表单"创建向导：实体新建的必填字段收敛为 **1~2 个**，模型参数默认隐藏 | 评审共识：多数模型参数对普通用户无意义且易设错；手动负担应降到最低（见附录 C） |
| 12 | 定义 **AI 判断型参数**（`category` / `difficulty` / 技能↔属性关联与权重 / 父节点）与三类执行模式 M1/M2/M3；默认采用"**AI 推荐 + 用户确认**"（M2） | 技能分类与属性关联是最大手动负担，又属语义知识；AI 推荐 + 确认页兼顾效率与正确性 |
| 13 | 新增 `soloup_meta` 只读工具、属性 `description` 字段、§6.6 创建向导、附录 C | AI 决策需要"枚举 + 参数元信息 + 属性语义描述"作为推断输入，并需明确的确认/审计流程 |
| 14 | **默认层参数向 AI 开放自动调优**：MCP 新增 `soloup_param_*` 工具组（`param_preview / param_set / param_remove`）+ 可调参数注册表（§9.2.1），配范围校验、影响面 dry-run 预览与审计 | "默认层"参数对普通用户无意义且主界面不展示，但恰恰应由 AI 接管——AI 可依据实际成长数据自动微调并持续优化模型曲线 |
| 15 | **技术选型定稿并落档**：§8.1 升级为正式选型表，新增附录 D 决策记录。确认：Next.js 15（保留）+ better-sqlite3 手写 SQL Repository（无 ORM）+ TanStack Query + Vitest + Zod 等 | 编码开工前冻结工具链，减少返工 |
| 16 | **前端 UI 设计选型定稿并落档**：§7 升级为 UI/UX 正式规格（7.1~7.7：视觉 token / 信息架构与路由 / 页面组件规格 / SVG 图表原语 / 主题策略 / 动效与可访问性），新增附录 E 决策记录。确认：温暖极简＋游戏化点缀、侧边栏 5 视图＋⌘K＋全局打卡、自写 SVG 4 原语、折叠缩进技能树 | UI 是高频使用界面，观感与信息架构先行冻结，避免返工 |
| 17 | **最终定档审计并落档**：附录 B 六项遗留问题逐条拍板关闭；一致性修复：技能 `description`/`sort` 落领域与 DDL、`param_overrides` 统一为嵌套 JSON＋点分路径寻址并定义 `settings` 序列化规则、确立「记录覆盖 → 全量重放」唯一纠正原语与「参数时间线」重放依据、修正附录 C.5 交叉引用等 | 消除编码歧义与「同日覆盖双算」等潜在缺陷，文档进入可开工的定稿状态 |

---

## 1. 项目定位与范围

### 1.1 一句话定位

把你投入在各项技能上的**每日行为**，通过一套可解释的数学模型，转换成**技能等级 → 属性面板 → 成就卡牌**的成长反馈；同时开放本地 MCP 服务，让 AI 能代为查询与管理这个面板。

### 1.2 设计原则

| 原则 | 说明 |
|---|---|
| **可解释** | 任何数字都能反推回"我做了什么"。等级不是黑盒经验值，而是 `E = C + V` 的确定函数 |
| **抗流失** | 遗忘会掉级，但已结晶部分永不丢失。断更后恢复比从零开始快得多 |
| **零维护** | 属性、父技能等级全部派生；界面只需维护：技能树结构 + 每日勾选（参数分层后连结构也可由 AI 辅助） |
| **本地优先** | 数据完全归用户所有，不依赖网络即可使用 |
| **最小配置** | 参数分三层收敛：普通用户只填"语义层"，默认层走内置值，公式层完全不可见（附录 C） |

### 1.3 术语表

| 术语 | 符号 | 定义 |
|---|---|---|
| 结晶账户 | `C` | 已固化的长期能力，**永不衰减**，单调递增 |
| 活性账户 | `V` | 短期熟练度，会随遗忘与结晶双重流失 |
| 有效投入量 | `E` | `E = C + V`，技能等级的唯一自变量 |
| 有效天数 | — | 已废弃术语，不再使用 |
| 结晶率 | `c` | 每日从 `V` 转入 `C` 的比例 |
| 遗忘率 | `f` | 每日从 `V` 中彻底丢失的比例 |
| 流失率 | `d` | `d = c + f`，`V` 的每日总衰减比例 |
| 拐点 | `x₀` | Sigmoid 曲线增长最快处对应的 `E` |
| 特征量 | `λ` | 指数饱和曲线的尺度参数，`E = λ` 时等级约 63 |
| 半程点 | `E₅₀` | 达到 50 级所需的 `E`，用于直观校准难度 |
| 参数分层 | — | 公式层 / 默认层 / 语义层，见附录 C |
| 执行模式 | M1/M2/M3 | 用户手填 / AI 推荐+确认 / AI 直接执行（白名单），见附录 C.3 |

### 1.4 版本规划（已定档，MVP 范围以此为准）

| 版本 | 范围 |
|---|---|
| **MVP** | 领域层 + SQLite 持久化 + Next.js 面板（技能树 CRUD、最小表单创建向导、每日打卡、双账户结算、等级曲线、属性面板、生命轴）+ **MCP v1**（查询 + `soloup_meta` + 属性/技能/关联管理 CRUD + **`soloup_param_*` 默认层参数自动调优** + 审计日志） |
| **v1.1** | 项目轴、成就规则引擎与卡牌、图表（等级曲线/E 值趋势）、数据导出、**MCP 打卡与结算工具**（默认 `confirm: true`，白名单可 M3）、**复查软提示 `soloup_insights`**、难度目标反推向导增强、技能迁移合并 |
| **v1.2** | 补打卡与历史编辑、**MCP 项目轴与导入/恢复工具**、多设备同步（可选） |
| **未来** | 里程碑复盘报告、技能推荐（基于属性缺口）、Tauri 桌面壳打包、远程 MCP（HTTP/SSE，可选） |

---

## 2. 领域模型

### 2.1 实体关系

```
Attribute ──< SkillAttributeLink >── Skill ──< DailyRecord >── Project
   (派生)                             │  (自关联 parent_id)
                                      └──< SkillSnapshot (每日快照，可选，非 MVP)
Achievement (独立，由规则引擎对全量状态求值)
LifeAxis    (全局单例配置)
Settings    (含 param_overrides / ai_execute_whitelist)
```

- **Skill → Attribute**：多对多，通过 `SkillAttributeLink` 携带权重 `w`
- **Skill → Skill**：自关联树形结构
- **DailyRecord → Project**：多对一，可为空

### 2.2 Attributes（属性）

| 字段 | 类型 | 说明 | 参数层 |
|---|---|---|---|
| `id` | string | 唯一标识 | — |
| `name` | string | 名称（力量、敏捷、体力、智力、魅力…） | **语义层（必填）** |
| `description` | string \| null | 一句话定义/同义词，供 AI 关联推荐与用户备忘（如"智力：逻辑、理解、分析、编程…"） | 语义层（可选，强烈建议填） |
| `base_value` | number | 基础值，默认 0 | 默认层（高级设置） |
| `max_value` | number | 属性上限 `S`，默认 100 | 默认层（高级设置） |
| `alpha` | number | 聚合指数 `α`，默认 1.2 | 默认层（高级设置） |
| `W0` | number | 饱和尺度，默认 400，见 §3.9 | 默认层（高级设置） |
| `category` | enum | `physical` / `mental` / `social` / `creative`（仅用于推荐与配色） | 默认层（AI 推断） |
| `color` / `icon` | string | 展示用，按 category 自动生成 | 默认层 |
| `sort` | integer | 同级展示排序，默认 0 | 默认层 |
| `current_value` | number | **派生缓存**，不可直接编辑，见 §3.9 | — |

> **默认层说明**：`alpha / W0 / max_value / base_value` 有内置全局默认（附录 A），新属性默认全部使用全局值；界面不展示，仅可在"设置 → 高级 → param_overrides"按属性覆盖。**v1 不鼓励按属性单独调参**。
>
> 属性面板**预置建议**（首次初始化时自动创建，用户可删改）：力量（physical）、敏捷（physical）、体力（physical）、智力（mental）、专注（mental）、创造力（creative）、魅力（social）。

### 2.3 Skills（技能树）

| 字段 | 类型 | 说明 | 参数层 |
|---|---|---|---|
| `id` | string | 唯一标识 | — |
| `name` | string | 名称 | **语义层（必填）** |
| `description` | string \| null | 一句话用途/投入说明：AI 推断（附录 C.4）与用户备忘的依据，可空但强烈建议填 | 语义层（可选） |
| `parent_id` | string \| null | 父节点；`null` 为根分类 | 语义层（AI 推荐） |
| `category` | enum | `physical` / `cognitive` / `knowledge`，决定默认曲线与账户参数 | 默认层（**AI 推断**，M2） |
| `difficulty` | enum | `casual` / `normal` / `hard` / `challenge` / `legendary`，默认 normal | 默认层（**AI 推断**，M2） |
| `C` | number | 结晶账户，默认 0 | —（仅结算产生） |
| `V` | number | 活性账户，默认 0 | —（仅结算产生） |
| `curve_type` | enum | `saturated` / `sigmoid`，由 `category` 推导，**允许手动覆盖** | 默认层（公式推导，高级覆盖） |
| `last_settled_date` | date | 上次结算日期，用于补结算 | — |
| `created_at` / `archived_at` | date | 归档后不再参与结算，但保留历史与派生贡献 | — |
| `color` / `icon` | string | 展示用，按 category 自动生成 | 默认层 |
| `sort` | integer | 同级展示排序，默认 0（可拖拽调整） | 默认层 |
| `level` | number | **派生缓存**，`= curve(C + V)` | — |
| `effective_exposure` | number | **派生缓存**，`= C + V` | — |

**关键约束**：

1. 只有**叶子节点**可被打卡，只有叶子节点持有 `C` / `V`。
2. 非叶子节点（分组）的 `level` 由子节点聚合派生，见 §3.8。
3. `C` / `V` 只能通过 §3.3 的结算算法变更，**禁止直接编辑**（调试模式除外）。

### 2.4 SkillAttributeLink（技能–属性关联）

| 字段 | 类型 | 说明 | 参数层 |
|---|---|---|---|
| `skill_id` | string | 必须是叶子节点 | 语义层（AI 推荐） |
| `attribute_id` | string | | 语义层（AI 推荐） |
| `weight` | number | `w ∈ [0, 1]`，主关联建议 1.0，次关联 0.3~0.5 | 默认层（AI 推荐初值，用户可调） |

### 2.5 DailyRecords（每日记录）

| 字段 | 类型 | 说明 |
|---|---|---|
| `date` | date | 本地自然日 `YYYY-MM-DD`，唯一 |
| `skill_ids` | string[] | 当日使用的叶子技能（落库为 `daily_record_skills`） |
| `project_id` | string \| null | 归属项目，可为空 |
| `note` | string | 可选备注 |
| `settled` | boolean | 是否已结算，防止重复计算 |

### 2.6 Projects（项目轴）

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` / `name` / `description` | string | |
| `start_date` | date | |
| `end_date` | date \| null | `null` 表示进行中 |
| `status` | enum | `planned` / `active` / `completed` / `abandoned` |
| `color` | string | |

**派生统计**（由 `DailyRecord.project_id` 聚合，不冗余存储）：

- 项目天数、打卡天数
- 涉及技能清单及各技能在项目期间的 `ΔC`、`ΔV`、`Δlevel`
- 各属性在项目期间的增长量

### 2.7 Achievements（成就卡牌）

| 字段 | 类型 | 说明 | 参数层 |
|---|---|---|---|
| `id` / `name` / `description` | string | | 语义层（预置或 **AI 提议**，用户勾选启用） |
| `condition` | object | 规则 DSL，见 §5 | 语义层（**不要求用户手写**，由预设/AI 生成） |
| `type` | enum | `skill` / `attribute` / `project` / `milestone` | — |
| `rarity` | enum | `common` / `rare` / `epic` / `legendary` | 默认层（按条件难度自动判定） |
| `points` | number | 稀有度对应 10 / 25 / 50 / 100 | 公式层 |
| `unlocked_at` | datetime \| null | | — |
| `progress` | number | 派生，进度百分比 | — |

### 2.8 LifeAxis（生命轴，全局单例）

| 字段 | 类型 | 说明 | 参数层 |
|---|---|---|---|
| `birth_date` | date | | **语义层（必填一次）** |
| `life_expectancy` | number | 默认 120（年），可配置 | 默认层 |
| `milestones` | array | 自定义里程碑 `{ date, label, color }` | 语义层（可选） |

派生：`current_age`、`progress = (now - birth) / (expectancy)`、已过天数 / 剩余天数。

### 2.9 Settings（应用设置）

| key | 类型 | 说明 |
|---|---|---|
| `theme` | string | 亮色/深色 |
| `first_day_of_week` | string | 周一/周日（影响周打卡统计） |
| `default_difficulty` | enum | 兜底难度，默认 `normal` |
| `ai_execute_whitelist` | string[] | M3 白名单：允许 AI **免确认直接执行**的工具/动作，默认空（见附录 C.3） |
| `param_overrides` | object | 全局参数覆盖（见下），默认空对象 |
| `data_version` | int | 迁移用 |

**param_overrides 结构**：存储为**嵌套 JSON 对象**（settings 行 key=`param_overrides`），以**点分路径**寻址，路径越深优先级越高（如 `attribute.<id>.w0` 覆盖 `attr_defaults.w0`）：

```jsonc
{
  "attr_defaults": { "alpha": 1.2, "w0": 400, "max_value": 100, "base_value": 0 }, // 全局默认
  "attribute":    { "<id>": { "w0": 600 } },        // 按属性覆盖（设置→高级，仅手改）
  "category":     { "knowledge": { "c": 0.006 } },  // 该类别全部技能：仅影响未来结算
  "curve":        { "cognitive": { "k": 0.006, "x0": 220 } } // 曲线参数
}
```

> **序列化约定**：`settings.value` 统一 TEXT——布尔 `"true"/"false"`、数字原样存储、数组/对象 JSON 序列化（含 `data_version`）。
> **key 一致性**：MCP `soloup_param_*`（§9.2.1）的 key 即上述对象的**点分路径**（如 `category.knowledge.c`、`curve.cognitive.x0`）。注册表只开放全局/类别/曲线类 key；`attribute.<id>.*` 等按实体覆盖仅高级模式手改，**不在 `soloup_param_*` 范围**。

> 覆盖仅影响**新建技能/新建属性**的计算起点与后续结算；历史数据（已产生的 C/V/level）不受追溯影响。所有覆盖需记审计。
>
> **所有权约定**：默认层参数对普通用户无意义，主界面不展示；**调优职责默认交给 AI**——MCP 提供 `soloup_param_preview / soloup_param_set / soloup_param_remove`（§9.2），AI 依据实际成长数据提出调整，默认需用户确认（M2），可调范围见 §9.2.1 注册表。

---

## 3. 数学模型

### 3.1 模型总览

```
每日行为（打卡技能 i）
   └─ 结算器（逐日离散，§3.3）  →  活性账户 V_i  ← 会遗忘/被提取
        │                          结晶率 c
        ▼
     结晶账户 C_i（永不衰减）
        │
   E_i = C_i + V_i  ── 代入成长曲线（§3.7，按 category 分轨）──▶  等级 L_i
        │
        ▼
   属性派生（§3.9：L_i^α 加权饱和）──▶ 属性 A_j
        │
        ▼
   成就引擎（§5：对 L/A/打卡统计求值）──▶ 卡牌解锁
```

核心不变式：**只要知道「每日打卡记录 + 参数时间线」，任何时间点的 C/V/等级/属性都是确定性可重放的函数**。参数时间线 = `audit_log` 中全部参数变更（含生效时刻），因为参数不追溯历史（§C.6），历史状态的重放必须以当时的参数为准。本模型不存储"经验值"，只存储历史行为、账户增量与参数变更审计。

### 3.2 双账户模型

每个**叶子技能**有两个内部账户：

- **`C`（结晶账户，Crystallized）**：已经内化的长期能力，`C` 单调不减（永不遗忘）。单位 ≈ "已被固化的有效练习日"。
- **`V`（活性账户，Active）**：短期熟练度，会同时被"结晶"和"遗忘"消耗，单位 ≈ "当前保持的有效练习日"。

> `C`/`V` 的单位都是「日」的当量，因此 `E = C + V` 可直接进入以"天"为自变量的成长曲线。

### 3.3 逐日结算算法（离散，唯一权威）

对**每个叶子技能**，按日期逐日推进结算，**任何跨越式结算都必须拆成逐日迭代**（这是与连续公式保持一致的保证）。

```
function settleDay(skill, hasCheckin):
    if hasCheckin:  skill.V += 1        # 每天最多 +1（不区分时长/强度，v1 约定）
    skill.C += c * skill.V               # 先结晶：从当前活性中提取比例 c
    skill.V *= (1 - c - f)               # 再衰减：活性按比例流失（c 去结晶、f 纯遗忘）
```

**执行顺序（不可调换）**：打卡加 `V` → 结晶 `C` → 衰减 `V`。若某天无打卡则跳过第 1 步，第 2、3 步仍执行。

```text
案例（cognitive，c=0.010, f=0.008，第 1 天打卡）：
  V: 0 → +1 → 1
  C: 0 → +0.010×1 = 0.010
  V: 1 × (1−0.018) = 0.982
  E = 0.010 + 0.982 = 0.992  → 等级见 §3.10 表
```

### 3.4 稳态与关键恒等式

连续每日打卡下，`V` 收敛于稳态 `V_eq`：

$$V_{eq} = \frac{1 - c - f}{c + f} = \frac{1-d}{d}, \qquad d = c + f$$

稳态下每日净结晶（`C` 的日增长）恒等于：

$$\frac{\Delta C}{\text{day}} = c\,(V_{eq}+1) = \frac{c}{c+f} = \frac{c}{d}$$

活性账户半衰期（不打卡时 V 减半所需天数）：

$$t_{1/2} = \frac{\ln 2}{-\ln(1 - d)}$$

| 类别 | d | V_eq（天） | 稳态日结晶 c/d | 活性半衰期 |
|---|---|---|---|---|
| physical | 0.019 | 51.63 | 0.789 | ≈ 36.1 天 |
| cognitive | 0.018 | 54.56 | 0.556 | ≈ 38.2 天 |
| knowledge | 0.020 | 49.00 | 0.300 | ≈ 34.3 天 |

> **语义**：`c/f` 比值表示"流失中有多少转化成了长期记忆"。动作类 `c/f = 3.75`（肌肉记忆高效固化），知识类 `c/f = 0.43`（大量流失是纯遗忘，需反复复习）。`d` 决定"停练多久活性减半"，三类接近（34–38 天），符合直觉。

### 3.5 补结算与时间推进

- **日期语义**：`date` 一律为**系统本地时区的自然日** `YYYY-MM-DD`（不做 DST 时区换算；按当天首次结算时刻所在时区取整）。
- 每个叶子技能记录 `last_settled_date`。
- 任何读取/写入前，若 `last_settled_date < today`，先对其间的每一天执行 `settleDay`（按每日记录判断是否打卡）。
- 无打卡的连续空窗可用**闭合公式**加速（等价于逐日）：

```text
空窗 n 天（期间均无打卡）：
  C += (c/d) × V × (1 − (1−d)^n)
  V  = V × (1−d)^n
```
- 有打卡的区间**禁止闭合公式**，必须逐日迭代。
- 防呆：单次补结算上限 3650 天，超出提示人工核对日期是否录入错误。
- **记录覆盖与重放**：改写/覆盖某日 `daily_records` 后（§6.2），不增量"打补丁"修复账户，而是把**受影响技能**从创建日起以「记录 + 参数时间线」全量重放（等价于从未写错，天然幂等）；成本远低于 §10.4 预算。非当日的日期编辑属 v1.2 历史编辑范围。

### 3.6 连续时间近似（仅展示/估算，不作为计算权威）

供趋势图、说明文案使用，与逐日结果存在小幅误差，禁止用于状态计算：

$$V(t) \approx V_{eq} + \left(V_0 - V_{eq}\right)e^{-d\,t}, \qquad E(t) \approx \frac{c}{d}\,t + V(t)$$

### 3.7 成长曲线（双轨制）

等级是 `E` 的单调增函数，值域 `[0, 100)`。由技能 `category` 决定轨道（可手动覆盖 `curve_type`）：

- **physical（动作/肌肉记忆）** → 指数饱和（无拐点，起步即见效、递减趋缓）
- **cognitive / knowledge（认知/知识）** → 归一化 Sigmoid（有拐点：蛰伏期 → 快速上升 → 趋缓）

#### 3.7.1 指数饱和曲线（`saturated`）

$$\ell(E) = 100\left(1 - e^{-E/\lambda}\right)$$

- `λ`：特征投入量，`E=λ` 时约达 63 级。
- 半程点：`E_{50} = λ·ln2`。
- 默认值：physical `λ = 480` → `E₅₀ ≈ 332.7`。

#### 3.7.2 归一化 Sigmoid 曲线（`sigmoid`）

先定义零起点基线 `s₀`（保证 `E=0` 时等级严格为 0）：

$$s_0 = \sigma(-k\,x_0) = \frac{1}{1 + e^{\,k x_0}}$$

$$\ell(E) = 100\;\frac{\sigma\big(k(E-x_0)\big) - s_0}{1 - s_0}, \qquad \sigma(z)=\frac{1}{1+e^{-z}}$$

- `x₀`：**拐点**（增长最快处）对应的 `E`。
- `k`：陡峭系数。乘积 `k·x₀` 决定曲线形态：越大越接近"延迟启动+陡峭跃迁"。
- 半程点（达到 50 级的 `E`）：

$$E_{50} = x_0 + \frac{1}{k}\ln\frac{1+s_0}{1-s_0}$$

默认值：

| category | k | x₀ | k·x₀ | s₀ | E₅₀ |
|---|---|---|---|---|---|
| cognitive | 0.006 | 220 | 1.32 | 0.211 | ≈ 291 |
| knowledge | 0.010 | 175 | 1.75 | 0.148 | ≈ 205 |

#### 3.7.3 难度系数（difficulty）

`x₀`（sigmoid）或 `λ`（saturated）乘以下表系数（`k` 不变）：

| difficulty | casual | normal | hard | challenge | legendary |
|---|---|---|---|---|---|
| 系数 D | 0.5 | 1.0 | 2.0 | 4.0 | 8.0 |

> 语义：难度只平移"需要积累到爆发点的投入量"，不改变陡峭度与账户参数。
> 难度对应"用户希望投入多久才见明显进展"是个人化的——因此 **v1 建议把难度交给 AI 依据用户一句话推断**（附录 C.4），不确定一律 `normal`。

### 3.8 技能树聚合（父级派生）

- **叶子节点**：拥有自己的 `C/V`，是唯一可打卡对象。
- **父级节点**：无 `C/V`，其 `level` 由直接子节点聚合：

$$\ell_{parent} = \Big(\frac{\sum_i w_i\, \ell_i^{\alpha}}{\sum_i w_i}\Big)^{1/\alpha}, \qquad w_i \ge 0$$

- `α = 1.2`（默认，与属性派生一致），子节点 `w_i` 可调（默认 1）。
- 属性只能关联**叶子技能**（见 §3.9），避免同一条成长被父/子重复计入。

### 3.9 属性派生公式

属性不由打卡直接增加，而是对**关联叶子技能等级**做加权聚合后再"饱和"：

$$A_j = B_j + S_j\left(1 - e^{-X_j / W_j}\right), \qquad X_j = \sum_{i \in \text{leaf}(j)} w_{ij}\,\ell_i^{\alpha}$$

- `B_j = base_value`（默认 0），`S_j = max_value`（默认 100，作为可增长上限）。
- `α = 1.2`（默认），`W_j = 400`（默认）。`w_{ij}` 即关联权重（主关联 1.0，次关联 0.3~0.5）。
- `A_j` 严格位于 `(B_j, B_j + S_j)`，**永不溢出**；技能升级 → 属性单调不降。

| 场景 | X | 属性增量 |
|---|---|---|
| 1 个技能 60 级（w=1） | 60^1.2 ≈ 136.1 | +28.8 |
| 3 个技能 60 级（w=1） | 3×136.1 ≈ 408.2 | +64.0 |
| 3 个技能 100 级 | 3×251.2 ≈ 753.6 | +84.8 |

> 效果：一个技能撑不起满属性；同属性多条技能共同耕耘才能接近上限——鼓励"复利式广度 + 深度"。

### 3.10 黄金数值表（测试基准）

以下为**普通难度、连续每日打卡**（从第 1 天起每天打卡）的权威期望值，用作 solver 单元测试快照。允许误差：`C/V` 相对误差 1e-9，等级相对误差 1e-6。

**physical**（c=0.015, f=0.004, saturated λ=480）

| 天数 | C | V | E | 等级 |
|---|---|---|---|---|
| 0 | 0.000 | 0.000 | 0.0 | 0.00 |
| 1 | 0.015 | 0.981 | 1.0 | 0.21 |
| 7 | 0.404 | 6.488 | 6.9 | 1.43 |
| 30 | 5.848 | 22.592 | 28.4 | 5.75 |
| 100 | 44.172 | 44.049 | 88.2 | 16.79 |
| 365 | 247.433 | 51.585 | 299.0 | 46.36 |
| 1095 | 823.712 | 51.632 | 875.3 | 83.86 |
| 1825 | 1400.028 | 51.632 | 1451.7 | 95.14 |

**cognitive**（c=0.010, f=0.008, sigmoid k=0.006, x₀=220）

| 天数 | C | V | E | 等级 |
|---|---|---|---|---|
| 0 | 0.000 | 0.000 | 0.0 | 0.00 |
| 1 | 0.010 | 0.982 | 1.0 | 0.13 |
| 7 | 0.270 | 6.514 | 6.8 | 0.87 |
| 30 | 3.934 | 22.919 | 26.9 | 3.55 |
| 100 | 30.175 | 45.684 | 75.9 | 10.84 |
| 365 | 172.509 | 54.484 | 227.0 | 37.97 |
| 1095 | 578.025 | 54.556 | 632.6 | 90.17 |
| 1825 | 983.580 | 54.556 | 1038.1 | 99.07 |

**knowledge**（c=0.006, f=0.014, sigmoid k=0.010, x₀=175）

| 天数 | C | V | E | 等级 |
|---|---|---|---|---|
| 0 | 0.000 | 0.000 | 0.0 | 0.00 |
| 1 | 0.006 | 0.980 | 1.0 | 0.15 |
| 7 | 0.161 | 6.462 | 6.6 | 1.00 |
| 30 | 2.319 | 22.271 | 24.6 | 3.96 |
| 100 | 17.250 | 42.502 | 59.8 | 10.80 |
| 365 | 94.809 | 48.969 | 143.8 | 32.22 |
| 1095 | 313.800 | 49.000 | 362.8 | 84.43 |
| 1825 | 532.800 | 49.000 | 581.8 | 98.03 |

**断更行为测试**（cognitive，打卡 100 天后断更 60 天）：

| 时刻 | C | V | E | 等级 |
|---|---|---|---|---|
| 第 100 天（打卡后） | 30.175 | 45.684 | 75.9 | 10.84 |
| 第 160 天（断更 60 天） | 47.021 | 15.362 | 62.4 | 8.73 |

> 断更 60 天：活性从 45.7 衰减到 15.4，等级回落到 8.7；但结晶从 30.2 增至 47.0——已固化的部分仍在积累，恢复时只需补活性。这正是"抗流失"设计的可测试表现。

---

## 4. 数据存储与领域层

### 4.1 存储形态

- **单一权威数据源**：SQLite 单文件（better-sqlite3），WAL 模式，外键开启。
- 默认路径：`SOLOUP_DB` 环境变量指定；未指定时使用 `~/.soloup/soloup.db`。
- 浏览器**不直连数据库**；Next.js 仅通过本地 API 路由（Node runtime）访问，MCP 服务以独立进程读写**同一文件**。
- 两个进程并发写由 SQLite WAL + `busy_timeout`（默认 5s）保证一致性；所有写操作走事务。

### 4.2 包结构与共享领域逻辑（防绕过）

单仓库（pnpm workspace）分层，核心规则只实现一次：

```
soloup/
├─ packages/core/       领域类型 + 纯函数（曲线、参数表、AI 推断词表，零 IO）
├─ packages/store/      SQLite schema + Repository + 迁移
├─ packages/solver/     结算/属性派生引擎（消费 store，或纯入参纯输出）
├─ packages/mcp-server/ MCP server（stdio，复用 core/store/solver）
├─ apps/web/            Next.js 面板（服务端调用 solver）
└─ apps/desktop/        Tauri 壳（可选，打包 web 构建）
```

**写入通道约定**：任何写操作（人工界面、MCP、脚本）都必须经过 `solver` 的服务函数（`recordCheckin / settleSkill / applyMutation` 等），**禁止直接对表 UPDATE `C/V` 或绕过校验改树结构**。违反约定视为 bug。

### 4.3 SQLite Schema（核心表）

```sql
attributes(id TEXT PK, name TEXT, description TEXT NULL,
  base_value REAL DEFAULT 0, max_value REAL DEFAULT 100,
  alpha REAL DEFAULT 1.2, w0 REAL DEFAULT 400, category TEXT,
  color TEXT, icon TEXT, sort INTEGER DEFAULT 0);

skills(id TEXT PK, name TEXT, description TEXT NULL, parent_id TEXT NULL REFERENCES skills(id),
  category TEXT NOT NULL,            -- physical|cognitive|knowledge
  difficulty TEXT DEFAULT 'normal',  -- casual|normal|hard|challenge|legendary
  curve_type TEXT NULL,              -- NULL=按 category 推导；可覆盖 saturated|sigmoid
  c REAL DEFAULT 0, v REAL DEFAULT 0,
  last_settled_date TEXT NULL,       -- YYYY-MM-DD
  created_at TEXT, archived_at TEXT NULL, color TEXT, icon TEXT, sort INTEGER DEFAULT 0);

skill_attributes(skill_id TEXT REFERENCES skills(id) ON DELETE CASCADE,
  attribute_id TEXT REFERENCES attributes(id) ON DELETE CASCADE,
  weight REAL NOT NULL DEFAULT 1.0,
  PRIMARY KEY(skill_id, attribute_id));

daily_records(date TEXT PRIMARY KEY, project_id TEXT NULL, note TEXT, settled INT DEFAULT 0);
daily_record_skills(date TEXT REFERENCES daily_records(date) ON DELETE CASCADE,
  skill_id TEXT REFERENCES skills(id) ON DELETE CASCADE,
  PRIMARY KEY(date, skill_id));

projects(id TEXT PK, name TEXT, description TEXT, start_date TEXT,
  end_date TEXT NULL, status TEXT, color TEXT);
achievements(id TEXT PK, name TEXT, description TEXT, condition_json TEXT,
  type TEXT, rarity TEXT, points INT, unlocked_at TEXT NULL);
audit_log(id INTEGER PK AUTOINCREMENT, at TEXT, actor TEXT,   -- user|ai|system
  tool TEXT, params_json TEXT);
settings(key TEXT PK, value TEXT);
```

> `daily_records.settled` 仅作提示字段；权威结算状态由 `skills.last_settled_date` + 逐日算法保证。归档技能（`archived_at` 非空）保留账户与历史，不再推进结算。

---

## 5. 成就规则引擎 DSL

成就 `condition_json` 使用可嵌套的 JSON 规则树。任一顶层规则 `is_active` 为 `true` 即解锁；解锁结果记录 `unlocked_at` 并追加审计。

**文法（BNF 摘要）**：

```
condition_json := { "rules": [ rule... ] }            // 顶层：任一规则满足即解锁（隐式 OR）
rule           := leaf | { "op": "AND"|"OR", "rules": [ rule... ] }
               |  { "op": "NOT", "rules": [ rule ] }
leaf           := { "metric": M, "operator": "gte"|"lte"|"eq", "value": N,
                    ["skill_id"|"attribute_id"|"project_id"], ["count_gte": N] }
```

进度条 = 叶子规则的加权通过率（`NOT` 取反，`AND/OR` 递归合并）。

**示例**（复合成就：素描 50 级 且 30 天连续 且 创造力 ≥ 40 且 总打卡 ≥ 500）：

```jsonc
{
  "op": "AND",
  "rules": [
    { "metric": "skill.level",   "skill_id": "sketching", "gte": 50 },
    { "metric": "skill.streak",  "skill_id": "sketching", "gte": 30 },
    { "metric": "attribute.value", "attribute_id": "creativity", "gte": 40 },
    { "metric": "checkin.total", "gte": 500 },
    { "metric": "project.completed_count", "gte": 5 }
  ]
}
```

### 5.1 指标字典

| metric | 求值目标 | 说明 |
|---|---|---|
| `skill.level` | 指定技能等级 | 支持 `parent_id`（聚合值）或全树 max/avg |
| `skill.crystallized` | 指定技能 `C` | 累计固化的长期投入 |
| `skill.streak` | 连续打卡天数 | 漏卡清零 |
| `skill.any_level` | 任意技能 ≥ 阈值 | `count_gte` 可要求数量 |
| `attribute.value` | 属性当前值 | |
| `checkin.total` | 历史总打卡天次 | |
| `checkin.week_streak` | 周内打卡 ≥ N 天 | |
| `project.completed_count` | 已完成项目数 | |

**算子**：`gte` / `lte` / `eq`，可再套 `AND` / `OR` / `NOT`。

> **成就创建减负**：系统提供一套**预置成就模板**（覆盖常用里程碑：首次打卡、首技能 10/50 级、任意属性破 40/70、7/30/100 天连击等）。v1.1 起，AI 可依据用户当前数据**提议个性化成就**（DSL 由 AI 生成），用户只在面板/对话中"启用"或"忽略"——**不要求用户手写 DSL**。

### 5.2 触发时机

- 每次结算或属性重算完成后，对**全部**成就求值一次（个人应用量级小，不做增量索引）。
- 解锁动画一次性播放，随后在卡牌墙常驻展示。

---

## 6. 功能规格（核心交互）

### 6.1 技能树管理

- 新增/重命名/移动（`parent_id`）/归档/删除叶子技能。新建走 **§6.6 最小表单向导**：用户仅需输入名称与一句话说明，`category` / `difficulty` / `curve_type` / 配色 由默认值或 AI 推荐填充，确认后落库。
- 父级分组自动派生等级（§3.8），不单独打卡。
- 校验：禁止把节点移动为自己的后代（防环）、禁止给归档技能打卡、删除必须先归档或确认无历史记录。

### 6.2 每日打卡

- 入口选择"今天" + 勾选叶子技能（支持多选），提交后写 `daily_records` 并触发该日结算。
- 打卡日期只允许 ≤ 今天。**同日覆盖**：同一天可重复提交、以最后一次为准（记审计）；实现为先删该日旧行再写新行，并对**受影响技能**按 §3.5 全量重放结算，杜绝重复计算；对"今天"以外日期的编辑属 v1.2 历史编辑。
- 打卡后即时反馈：每个技能的 `V` 变化与等级、受影响属性的增量。

### 6.3 项目轴（v1.1）

- 打卡时可挂靠 `project_id`；项目页聚合统计：项目区间内各技能 `ΔC/ΔV/Δlevel`、属性增量、每日打卡分布。
- 项目关闭时生成"项目收获"摘要（含成就）。

### 6.4 属性面板与技能详情

- 属性页展示派生值构成：各技能对属性贡献占比（`w·ℓ^α / X`），高亮最近一次变化的来源。
- 技能详情页展示：`E/C/V` 分解条、等级曲线预测（连续近似 §3.6）、距下一里程碑所需天数估算、复习提示（`V < 0.5·V_eq` 时提示"活性偏低，建议恢复练习"）。

### 6.5 生命轴

- 出生日期 → 年龄进度条（出生→ `life_expectancy`，默认 120），里程碑可按日龄标记。
- 展示：已用 %、剩余年数、按 5 岁粒度刻度。

### 6.6 最小表单创建向导（参数分层落地）

**目标**：每个实体新建时，用户面对的表单只含"语义层必填项"；其余字段以"AI 推荐候选 + 默认值"预填，确认页逐项可改。详见附录 C。

| 实体 | 首屏字段（必填） | 预填区（可展开编辑） |
|---|---|---|
| 技能 | `name`、`description`（一句话用途，可空） | category / difficulty / parent / 关联属性与权重 / curve_type / 配色 |
| 属性 | `name`、`description`（一句话定义，可空） | base / max / alpha / W0 / category / 配色 |
| 项目 | `name`、`start_date` | description / end_date / 配色 |
| 成就 | 从预置模板 + AI 提议中**勾选启用** | 不提供 DSL 手写入口（v1） |

交互流程（以技能为例，M2 模式）：

1. 用户输入：名称 + 一句话说明（如"每天早读 30 分钟英语外刊"）。
2. 系统/AI 生成候选卡片（每项附**置信度与理由**），见附录 C.4 推断规则。
3. 用户逐项确认或修改；`category` 置信度为 low 时**强制停留**此步。
4. 点击"创建" → `solver.applyMutation` 原子落库（技能 + 关联 + 初始参数），写入 `audit_log`（actor=`ai`/`user`，params 含候选 diff）。
5. 若候选关联的属性不存在 → 向导提示"顺手创建默认属性"或跳过关联，不阻塞。

> 面板内同样提供"高级模式"开关：展开全部默认层字段直接手填（M1），供想精细控制的用户使用。

---

## 7. UI/UX 设计规格（正式稿 v1.0）

> 2026-09 定稿，拍板记录与候选对比见附录 E。底层技术已由 §8.1/附录 D 锁定（Tailwind + shadcn/ui + motion + dnd-kit + TanStack Query）；本页定义**观感、信息架构、页面规格与可视化原语**，组件级变量以代码内 token 为唯一实现来源（以本页为准）。

### 7.1 视觉气质：温暖极简 + 游戏化点缀

- **底**：低饱和中性（白 `#FFFFFF` / 浅灰面 `#F8F9FA`）＋大留白＋克制卡片（中等圆角、柔和投影）；品牌橙 `#FF6B35` 为**唯一主强调**。
- **游戏化只出现在高光瞬间**：等级提升动效、里程碑达成辉光（MVP）；成就卡翻转/解锁（v1.1）。日常界面保持安静，保证数值可读、不疲劳。
- **情绪护栏（延续原稿）**：Sigmoid 前期慢的补偿——等级低时 UI 突出 `E` 值、`C` 结晶进度与"距下一级还需 X"而非"又没升级"，避免挫败感。
- 卡片式容器惯例：属性卡、技能树卡、项目卡（v1.1）、成就卡（v1.1）、生命轴进度条。

### 7.2 设计 token（单源，预留深色）

- 全部以 CSS 自定义属性表达（`--brand-*`/`--surface-*`/`--text-*`/`--status-*`/`--data-*`），根节点 `data-theme="light"`；深色只需补一套变量覆盖（§7.6）。
- 语义色只表状态：success / warning / danger / info（置信度徽标 high=绿/med=黄/low=红、`Δ` 正负、打卡态）。
- 数据系列色：`#FF6B35`（E/主数据）、`#4ECDC4`、`#45B7D1`、`#96CEB4` 降级为**辅助系列色**，仅用于多系列堆叠（`E/C/V`）与分类图标，不并列抢焦点。
- 字体：系统字体栈（含 PingFang SC / Noto Sans SC / 微软雅黑），**不引 web font**（本地优先）；所有数值默认 `font-variant-numeric: tabular-nums`（等宽对齐，表格/进度/坐标轴必备）。
- 字号基线：正文桌面 15px、移动端 ≥14px；层级 = 页面标题 / 卡标题 / 正文 / 标注 / 数值。
- 圆角/阴影/间距：`radius` token（sm/md/lg/xl）、柔和投影 token、4px 间距基准。

### 7.3 信息架构与导航

- 桌面：**固定左侧边栏**，5 个一级视图；移动端收为**底部 tab**（同 5 项）。
- 全局动作（任何视图可用）：悬浮「打卡」按钮 + **⌘K 命令面板**（跳转视图、新建技能/属性、搜索技能）。
- 路由表：

| 路径 | 视图 | MVP 内容 | 备注 |
|---|---|---|---|
| `/` | 今日看板 | 打卡勾选（按父分组折叠）＋即时结算反馈条＋AI 今日建议卡＋空态引导 | 打卡最高频，默认落点 |
| `/skills` | 技能树 | 折叠缩进列表＋拖拽调层＋行内迷你进度 | 详情 `/skills/[id]`：宽屏抽屉、窄屏整页 |
| `/attributes` | 属性面板 | 派生构成分解条、最近变化来源高亮 | |
| `/life` | 生命轴 | 年龄进度条＋里程碑刻度 | |
| `/settings` | 设置 | 参数分层浏览/高级模式开关、MCP 状态、数据目录/备份、关于 | |
| v1.1 | `/projects` `/achievements` 及图表聚合页 | — | 路由预留，不阻塞 |

### 7.4 关键页面 × 组件规格

- **今日看板**：打卡列表行 = 名称 + 今日时态点，按父分组折叠；勾选提交走 TanStack Query 乐观更新，结算回包后行内展示 `ΔV`/`Δlevel` 徽章（正橙负灰），页尾属性增量条；数值滚动用 `AnimatedNumber`。
- **技能树行组件**：图标（按 category）+ 名称 + 迷你进度条（`E/C/V` 堆叠色带）+ 打卡状态点；行内菜单（新建子级/重命名/移动/归档/删除，进 §6.6 向导）；拖拽用 dnd-kit sortable tree，防环/禁用态由 store 校验兜底。
- **技能详情**：`E/C/V` 分解条、等级曲线预览（§7.5 原语）、距下一里程碑所需天数、活性低提示（`V < 0.5·V_eq`，§6.4）。
- **属性面板**：各技能贡献占比条 `w·ℓ^α / X`，最近一次变化来源高亮。
- **向导确认页（§6.6）**：候选卡带置信度徽标＋可折叠"为什么这么填"；low 置信度强制停留；每字段可回退默认值/改手动；「高级模式」开关展开默认层直填（M1）。
- **空态**：各视图首次进入给出引导动作（如"添加第一个技能"），不展示裸空页面。

### 7.5 数据可视化：自写 SVG 原语（零依赖）

- 仅 4 类原语组件（`apps/web` 内 charts 目录，约 300 行）：
  1. `LineCurve`：折线＋可选面积，含**预测虚线段**（连续近似 §3.6）与**里程碑点**；
  2. `StackBreakdown`：`E/C/V` 及属性贡献的堆叠分解条；
  3. `MiniBar`：行内迷你进度条；
  4. `MilestoneDot`：与折线联动的刻度点/下一目标标记。
- tooltip 统一呈现原始语义（"2026-06-03 打卡 30min → E 240"），强调**可解释**而非裸数字。
- 动画：折线 `pathLength` 生长（motion）、数值 `AnimatedNumber`；尊重 `prefers-reduced-motion`。
- 数据只由 core/solver 派生的点数组直出，前端**不重算任何数值**。
- 若 v1.1 图表数量激增 → 评估 Recharts（可回退，附录 E.4）。

### 7.6 主题策略与响应式

- v1 只做亮色；token 已变量化并为 `data-theme="dark"` 预留整套覆盖位，深色 v1.1（可跟随系统）落地。
- 响应式断点：桌面多栏网格（`lg`+）/ 平板 / 移动端单列＋底部 tab；移动端触控目标 ≥44px。

### 7.7 动效与可访问性

- 动效预算：常规 transition 120–200ms；**长动效只留给高光瞬间**；尊重 `prefers-reduced-motion`。
- 键盘可达：树操作全键控、⌘K 面板、焦点环可见；勾选用原生 checkbox 语义；正文对比度满足 WCAG AA（≥4.5:1）。

---

## 8. 技术方案与部署

### 8.1 技术选型（正式表；逐项论证与备选对比见附录 D）

| 层 | 最终选型 | 关键说明 |
|---|---|---|
| 语言/运行时 | TypeScript（strict）＋ Node.js | engine ≥ 22，建议 24 LTS |
| 仓库/包管理 | pnpm workspace 单仓库 | pnpm ≥ 9；MVP 不引入 turbo |
| Web 框架 | Next.js 15（App Router），**仅本地服务** | Route Handler 于 Node runtime 直连 SQLite（`serverExternalPackages: ['better-sqlite3']`）；`output: 'standalone'` 供 Tauri 复用 |
| UI | Tailwind CSS + shadcn/ui | 动画 `motion`（原 Framer Motion）、树拖拽 `dnd-kit`、图标 `lucide-react` |
| 客户端数据 | TanStack Query | 查询缓存/失效、打卡乐观更新、向导确认回填 |
| 数据访问 | better-sqlite3 + 手写 SQL Repository（无 ORM） | WAL；同步事务贴合逐日结算 |
| Schema 迁移 | 自写版本化脚本（`PRAGMA user_version`） | `migrations/*.sql` + 递增版本 |
| 输入校验 | Zod | MCP 工具参数、API body、参数注册表（§9.2.1）统一 schema；可导出 JSON Schema |
| ID / 日期 | `crypto.randomUUID()`／core 内置 `YYYY-MM-DD` 本地日工具 | 日期语义见 §3.5 |
| 计算层 | `packages/core`（纯函数/参数表/AI 词表）＋ `packages/solver`（结算/派生/成就） | UI 与 MCP 共享同一实现 |
| 测试 | Vitest | §3.10 黄金表快照 + MCP `InMemoryTransport` 集成测试（§10.3） |
| 开发脚本 | `tsx` | 迁移/种子/结算 CLI 直接跑 TS |
| MCP | `@modelcontextprotocol/sdk`，stdio | 预留 SSE Transport |
| 桌面壳（未来） | Tauri v2 | 复用 `apps/web` standalone 产物 |
| 代码质量 | ESLint 9（typescript-eslint）＋ Prettier | `better-sqlite3` 仅允许被 `packages/store` 引用 |

### 8.2 进程与数据流

```
┌────────────┐   HTTP(localhost)   ┌────────────────────┐
│  浏览器/UI  │◀──────────────────▶│ apps/web (Next)     │
└────────────┘                     │  Route Handlers     │
                                   └─────────┬──────────┘
                                             │ solver / store
┌────────────┐  stdio(MCP)  ┌───────────────▼─────────┐
│ CodeBuddy/ │◀────────────▶│ packages/mcp-server      │──▶ SQLite 文件
│ AI 客户端   │              │ (meta/查询/CRUD/param) │   (~/.soloup/soloup.db)
└────────────┘              └─────────────────────────┘
```

- 前端不直连 DB；所有页面数据来自本地 API。
- MCP 与 Web 使用同一 `solver` 层，保证 AI 与人操作的数值一致。

---

## 9. MCP 服务规格（AI 接入）

> 目标：让 AI 作为"面板管家"——替用户查询状态、管理属性与技能的增删改查、维护技能间关联与难度/分类配置。所有变更走同一 `solver`，可审计、可回滚、不破坏数值不变量。

### 9.1 传输与运行

- **传输**：`stdio`（本地），MCP server 通过 `node packages/mcp-server/dist/index.js` 启动。
- HTTP/SSE 等传输属可选后续扩展：仅限本机监听；若未来开放网络访问，须先补鉴权（不在 MVP）。
- 每个工具执行在**独立事务**内；读取前自动将相关技能结算到"今天"（§3.5），保证返回即最新。

### 9.2 工具清单 —— v1（元信息 + 查询 + 管理 CRUD + 参数调优）

**命名空间约定**：`soloup_*`。

| 工具 | 方向 | 说明 / 关键参数 |
|---|---|---|
| `soloup_meta` | 元信息 | 返回全部枚举（category/difficulty/curve_type/属性 category）、默认参数表（附录 A）、当前 `param_overrides`、预置属性与成就模板 —— **AI 推断前必读** |
| `soloup_panel_overview` | 查询 | 面板总览：属性当前值+趋势、技能树摘要、今日是否已打卡、成就统计、生命轴 |
| `soloup_attribute_list` | 查询 | 列出属性（含派生值、description、贡献占比） |
| `soloup_attribute_get` | 查询 | 单属性详情 + 关联技能 |
| `soloup_attribute_create` | 管理 | name, description?, base_value?, max_value?, alpha?, w0?, category?, color?, icon? |
| `soloup_attribute_update` | 管理 | id + 可改字段 |
| `soloup_attribute_delete` | 管理 | id；**先解除关联**，仅当无历史强依赖时可物理删除，否则建议归档 |
| `soloup_skill_tree` | 查询 | 返回整棵树（含等级/E/C/V/曲线参数） |
| `soloup_skill_get` | 查询 | 单技能详情 + 子节点 + 关联属性 |
| `soloup_skill_create` | 管理 | name, description?, parent_id?, category*, difficulty?, curve_type?, color?, icon?<br>※ `category/difficulty` 建议由 AI 依据用户描述推断（附录 C.4），服务端校验枚举与树约束 |
| `soloup_skill_update` | 管理 | id + name / category / difficulty / curve_type / icon 等 |
| `soloup_skill_move` | 管理 | id, new_parent_id；执行环检测 |
| `soloup_skill_archive` / `_restore` | 管理 | id；归档后不再结算 |
| `soloup_skill_delete` | 管理 | id；仅允许删除无打卡历史叶子，否则引导归档 |
| `soloup_skill_link_set` | 管理 | skill_id, attribute_id, weight；建立/更新关联 |
| `soloup_skill_link_remove` | 管理 | skill_id, attribute_id |
| `soloup_daily_status` | 查询 | date?；返回某日打卡详情 / 是否已结算 |
| `soloup_recalc` | 管理 | 触发全部技能补结算到今日 + 重算派生值 |
| `soloup_audit_log` | 查询 | 最近变更记录（工具+参数+actor+时间） |
| `soloup_settings_get` | 查询 | 当前 Settings（param_overrides / ai_execute_whitelist / 主题等） |
| `soloup_param_preview` | 查询 | key, value；**只读 dry-run**：返回该参数生效后影响的技能/属性数量、新曲线参数与等级差示例（不落库） |
| `soloup_param_set` | 管理 | key, value；按 §9.2.1 注册表校验 key 与范围；返回影响面；**默认需用户确认（M2）** |
| `soloup_param_remove` | 管理 | key；移除单个覆盖，恢复注册表默认值 |

> **AI 使用约定**：执行"管理类"写操作前，先调用 `soloup_meta` + `soloup_skill_tree` + `soloup_attribute_list` 获取枚举、现有结构与属性描述，再进行推断与写入；写入参数应自证推断依据（工具描述内注明置信度）。
>
> v1 **不含**打卡新增/删除（避免"AI 替你打卡"在无人确认时产生歧义），打卡写操作规划在 v1.1 并默认带 `confirm: true` 参数。若希望 AI 直接代填打卡，请在 v1.1 评审时确认。

### 9.2.1 可调参数注册表（默认层，AI 自动调优范围）

`soloup_param_set / remove` 只接受下列 key（定义于 `packages/core`，为单一来源；`soloup_meta` 返回同源数据）：

| key | 默认 | 允许范围 | 含义 |
|---|---|---|---|
| `attr_defaults.alpha` | 1.2 | [1.0, 1.8] | 属性聚合指数 α（§3.9） |
| `attr_defaults.w0` | 400 | [100, 2000] | 属性饱和尺度（§3.9） |
| `attr_defaults.max_value` | 100 | [20, 1000] | 新建属性上限默认 |
| `attr_defaults.base_value` | 0 | [0, 50] | 新建属性基础值默认 |
| `category.<c>.c` | physical 0.015 / cognitive 0.010 / knowledge 0.006 | (0, 0.04] | 该类别全部技能的结晶率 |
| `category.<c>.f` | physical 0.004 / cognitive 0.008 / knowledge 0.014 | (0, 0.04] | 该类别全部技能的遗忘率 |
| `curve.physical.lambda` | 480 | [100, 3000] | 指数饱和特征量 |
| `curve.cognitive.k` / `x0` | 0.006 / 220 | k ∈ (0, 0.03]，x0 ∈ [50, 1000] | sigmoid 陡峭度 / 拐点 |
| `curve.knowledge.k` / `x0` | 0.010 / 175 | k ∈ (0, 0.03]，x0 ∈ [50, 1000] | 同上 |

约束：`category.<c>.c + f ≤ 0.05`；`x0` 范围配合难度乘数后须满足 §3.7 单调性要求。

- 服务端强制：key 不在注册表 → `ERR_UNKNOWN_PARAM`；值越界 → `ERR_PARAM_RANGE`（返回合法区间）。
- 变更**只影响未来结算与新建实体**，不追溯改写历史 C/V/level（§C.6）。
- 每次成功变更追加审计：`{ key, old, new, affected_skills, affected_attributes }`。
- 单次调用只能改 1 个 key（避免 AI 一次埋入多个耦合改动）；多 key 调整请分次并各自 preview。

### 9.3 后续版本工具（规划）

| 版本 | 工具 |
|---|---|
| v1.1 | `soloup_checkin_set(date, skill_ids, note?, confirm)`、`soloup_checkin_clear(date, skill_ids?)`、`soloup_project_*`（项目 CRUD 与统计）、`soloup_insights`（复查软提示：E 增速显著低于同类别预期；趋势/缺口建议）、`soloup_export` |
| v1.2 | `soloup_import`、`soloup_skill_merge` |

### 9.4 提示词示例（客户端侧用法）

> "按今天的实际活动帮我打卡：晨跑 30 分钟挂到『跑步』，晚上学一节《神经网络》挂到『深度学习』。先读总览确认技能存在，再写入并核对数值变化。"

> 创建技能（M2 风格）：
> "帮我新建技能：每天早读 30 分钟英语外刊，练的是理解和词汇。先 meta + 树 + 属性，推断 category/难度并推荐它喂给哪个属性，列出来等我确认。"

### 9.5 不变量与安全约束（MCP 服务端强制）

1. **禁改账户**：不暴露任何直接修改 `C/V` 的工具；数值只经结算/重算产生。
2. **树结构安全**：`parent_id` 变更做环检测；父节点不可打卡。
3. **关联校验**：link 只允许挂在叶子技能；attribute 必须先存在。
4. **软删优先**：有历史数据的实体不物理删除，引导归档。
5. **全量审计**：所有管理写操作写入 `audit_log`（actor=`ai`/`user`，含参数快照）。
6. **幂等与重放**：同一工具重复调用结果一致；每日记录以"最后一次提交"为准并记录审计。
7. **数值结果由 solver 统一输出**，服务端仅透传，不自行计算等级/属性。
8. **确认优先**：M2 推荐类写操作要求 AI 先向用户展示候选并取得确认；M3 免确认仅限 `ai_execute_whitelist` 允许的动作（附录 C.3）。
9. **参数调优护栏**：`soloup_param_set / remove` 只接受 §9.2.1 注册表 key 且在合法区间，越界/未知 key 直接拒绝；写前建议（服务端不强制的提示）先跑 `soloup_param_preview`；变更不追溯历史 C/V；默认需用户确认（M2），仅当用户将 `soloup_param_set` 加入 `ai_execute_whitelist` 才允许免确认（M3）。
10. **参数时间线与记录重放**：`audit_log` 即参数时间线（含生效时刻与 old→new）；任何记录覆盖/导入后按 §3.5 全量重放受影响技能，服务端禁止增量"打补丁"式修数。

---

## 10. 验收标准与测试用例

### 10.1 数值正确性（solver 单元测试）

| 用例 | 输入 | 期望 |
|---|---|---|
| 黄金表 | §3.10 三类技能连续打卡表 | C/V 相对误差 ≤ 1e-9，等级 ≤ 1e-6 |
| 断更一致性 | 打卡 100 天断 60 天 | 结果 == §3.10 断更表（逐日与闭合公式一致） |
| 补结算等价 | 一次结算到 160 天 vs 逐日 160 次 | 两路结果完全一致 |
| 零起点 | E=0 | 等级恒 0，C/V 不变 |
| 单调性 | 任意不含撤销的操作序列 | C 永不下降；等级随 E 单调不减 |
| 收敛性 | 连续打卡 2000 天 | V → V_eq（±0.01），日结晶 → c/d |
| 饱和边界 | 长期打卡超大 E | 等级 < 100；属性 < base+max |

### 10.2 不变量测试

- 归档技能不再推进结算；恢复后从 `last_settled_date` 起补结算。
- 父级移动成自己后代 → 被拒绝（环检测）。
- 对非叶子执行打卡/关联属性 → 被拒绝。
- 直接改 `C/V`（绕过 solver）→ 无法通过 store 接口完成（服务层校验）。
- 删除有历史的技能 → 默认归档而非物理删除。
- param_overrides 仅影响新建/后续结算，历史 `audit_log` 可解释当时用的参数。
- 同日覆盖（含移除已结算技能）→ 受影响技能数值与「自始按最终记录全量重放」完全一致，无重复结算（§3.5/§6.2）。
- 按参数时间线重放 → 可复现任意历史日期的 C/V/level（§9.5-10）。

### 10.3 MCP 集成测试

- 通过 MCP 创建属性 → 面板列表可见；创建技能挂父级 → 树图刷新。
- `soloup_meta` 返回的枚举与默认参数表 == `packages/core` 定义（单一来源校验）。
- MCP 修改权重 → 属性派生值按 §3.9 变化且审计可查。
- 乱序/重复调用（幂等）→ 结果一致；非法参数返回结构化错误而非崩溃。
- 面板进行中的操作与 MCP 写操作并发 → 无数据损坏（WAL）。
- 创建向导确认流程 → `audit_log` 记录候选 diff；修改 category/difficulty/关联后派生曲线/属性按新参数变化。
- `soloup_param_preview` → 不产生任何写操作与审计记录；返回值与 `param_set` 生效后的实际影响面一致。
- `soloup_param_set` 越界/未知 key → `ERR_PARAM_RANGE` / `ERR_UNKNOWN_PARAM`；成功后旧技能 C/V/level 不变，新建技能使用新参数，审计含 old→new。
- `soloup_param_remove` 后 → 参数恢复注册表默认，后续结算按默认进行。

### 10.4 性能预算（个人应用）

- 全量补结算（10 年 × 200 技能）< 1s；单次打卡反馈 < 200ms；SQLite 文件 < 20MB/年。

---

## 附录 A：默认参数速查

| 参数 | physical | cognitive | knowledge |
|---|---|---|---|
| c（结晶率） | 0.015 | 0.010 | 0.006 |
| f（遗忘率） | 0.004 | 0.008 | 0.014 |
| d = c+f | 0.019 | 0.018 | 0.020 |
| V_eq（天） | 51.6 | 54.6 | 49.0 |
| 稳态日结晶 | 0.789 | 0.556 | 0.300 |
| 活性半衰期 | ≈36 天 | ≈38 天 | ≈34 天 |
| 曲线 | saturated | sigmoid | sigmoid |
| 曲线参数 | λ=480 | k=0.006, x₀=220 | k=0.010, x₀=175 |
| E₅₀ | ≈333 | ≈291 | ≈205 |

全局默认：等级上限 100；属性 α=1.2、W₀=400、max_value=100、base_value=0；树聚合 α=1.2；`default_difficulty=normal`。
> 上述全部属于**公式层/默认层**（附录 C），除 param_overrides 外不暴露给普通用户。

## 附录 B：定档决议（原开放问题，2026-09-06 全部关闭）

> 评审遗留的 6 条开放问题已在最终定档逐条拍板，结论同步落入正文相应章节；后续如需变更走版本演进，不再作为悬置问题。

| # | 问题 | 定档决议 | 正文落点 |
|---|---|---|---|
| 1 | 难度是否支持"目标反推" | **后置 v1.1**。MVP 保持"AI 由投入描述推断 + normal 兜底"（§C.4.2），难度不与目标耦合；v1.1 增加向导"期望 X 时间达熟练 → difficulty 反推"增强。理由：反推需可靠映射表与数据验证，MVP 先行简化 | §1.4 v1.1 / §C.4.2 |
| 2 | 属性 `base_value` 边界 | **高级可改、不设系统解锁**。注册表合法区间 [0, 50] 内可经 `param_overrides`（设置→高级）手改；不引入"天赋成就"类解锁机制（本地个人应用无数值经济顾虑） | §2.9 / §9.2.1 |
| 3 | v1.1 打卡工具默认权限 | 维持 M2：`soloup_checkin_set / clear` 默认 `confirm: true`；仅当工具被用户加入 `ai_execute_whitelist` 才允许 M3 免确认 | §1.4 v1.1 / §9.3 |
| 4 | MCP 是否加"批量建树" | **不引入** `build_subtree`。整棵子树一次传入价值低，且与"单事务全量校验"冲突；批量场景由 v1.2 `soloup_import` 覆盖 | §9.3 v1.2 |
| 5 | Tauri 是否进 MVP | 维持"未来"，不进 MVP。MVP = 本地一体（Next 本地服务），`standalone` 输出保留将来内嵌空间 | §1.4 / §8.1 |
| 6 | 运行期分类复查软校验 | **v1.1 软提示**。MVP 不做服务端校验（low 置信度强制确认已兜底，§C.7）；v1.1 由 `soloup_insights` 输出"E 增速显著低于同类别预期" + 面板"建议复查分类/难度"软提示 | §1.4 v1.1 / §9.3 v1.1 |

## 附录 C：参数分层与自动化策略

> 本文档配套决策记录。目标：**每个实体新建时，用户必填 ≤ 2 个语义字段**，其余由默认值、公式或 AI 推荐自动产生，且每个自动值都可解释、可推翻、可审计。

### C.1 三层参数总表

| 层 | 定义 | 处理方式 | 界面可见性 |
|---|---|---|---|
| **公式层** | 模型常量与全部派生值；由类别与参数表唯一确定 | c/f/d、V_eq、curve_type（默认）、C/V/E/level/current_value、成就 points | 不可编辑；只读展示（可解释文案） |
| **默认层** | 有合理内置默认值、绝大多数用户无需修改的模型参数 | alpha、W0、max_value、base_value、difficulty（兜底 normal）、配色/图标、life_expectancy、curve_type 覆盖 | 主界面隐藏；设置 → 高级 → param_overrides |
| **语义层** | 只有用户知道的语义或事实；AI 只做推荐、**不代决** | name、description、parent_id、属性定义与技能↔属性关联、weight、birth_date、项目时间、打卡记录 | 必填/确认页 |

### C.2 逐实体参数处理策略

| 实体 | 字段 | 层 | 用户必填? | 处理策略 |
|---|---|---|---|---|
| 属性 | name / description | 语义 | 是（仅 name） | 手填；description 推荐填（AI 匹配依据） |
| 属性 | category | 默认 | 否 | AI 按 name+description 推荐，可改 |
| 属性 | base / max / alpha / W0 | 默认 | 否 | 全局默认；param_overrides 覆盖 |
| 属性 | color/icon/sort | 默认 | 否 | 按 category 自动 |
| 技能 | name / description | 语义 | 是（仅 name） | 手填 |
| 技能 | category | 默认 | 否 | **AI 推断**（置信度 high/med/low） |
| 技能 | difficulty | 默认 | 否 | **AI 推断**，兜底 normal |
| 技能 | parent_id | 语义 | 否 | AI 按名称 token 推荐挂点，可改 |
| 技能 | curve_type | 公式/默认 | 否 | category 推导；高级可覆盖 |
| 技能 | C/V/level | 公式 | — | 仅结算产生，禁手改 |
| 技能 | color/icon | 默认 | 否 | 按 category 自动 |
| 关联 | skill↔attribute | 语义 | 否（AI 推荐） | **AI 推荐候选**，确认后落库 |
| 关联 | weight | 默认 | 否 | 主 1.0 / 次 0.3~0.5，确认页可调 |
| 成就 | 规则本体 | 语义 | 否 | 预置模板 / AI 提议，用户只"启用" |
| 成就 | rarity/points | 默认/公式 | 否 | 按条件难度自动 |
| 生命轴 | birth_date | 语义 | 是（一次） | 手填 |
| 生命轴 | life_expectancy | 默认 | 否 | 默认 120 |
| 项目 | name/start | 语义 | 是 | 手填 |
| 每日打卡 | 技能勾选/日期 | 语义 | 是（每天） | 人工或 v1.1 起 AI（confirm） |

**结论**：长期维护中，用户每天只做一件事（勾选打卡）；偶尔的新增操作每个实体只填 1 个名称字段，其余交给默认与 AI。

### C.3 执行模式（M1/M2/M3）

| 模式 | 说明 | 适用 | 审计要求 |
|---|---|---|---|
| **M1 用户手填** | 面板"高级模式"，全部字段人工指定 | 离线、精确调参、AI 不可用 | actor=`user` |
| **M2 AI 推荐 + 确认**（默认） | AI 按规则生成候选卡片（值 + 置信度 + 理由），确认页逐项可改，确认后才落库 | 技能/属性新建、关联建立、难度/分类修改 | actor=`ai`，params 含候选 diff + 确认结果 |
| **M3 AI 直接执行** | 白名单机制：仅 `ai_execute_whitelist` 中列出的动作可免确认直写 | v1.1 的打卡代填（confirm 机制）、用户明确授权的重复性维护 | actor=`ai`，强制审计；可一键回滚（打卡以当日最后一次为准） |

约束：任何模式下都**不允许**直接改 `C/V`、删除有历史的实体、破坏树结构。M2 中 `category` 置信度为 **low** 时服务端返回"需确认"错误码，AI 不得绕过。

### C.4 AI 推断规则（v1 词表，`packages/core` 中实现，可扩展）

#### C.4.1 category 推断（决定曲线与账户参数）

| 信号 | physical | cognitive | knowledge |
|---|---|---|---|
| 动作词 | 跑、练、举、游、跳、打(球)、拉伸 | 学、练(脑力)、刷题、读、写、编程 | 读、背、记忆、复述、理解概念 |
| 对象词 | 力量、体能、肌肉、身体、协调 | 语言、乐器、棋、数学、代码、课程 | 史、理论、公式、学科名 |
| 一句话模式 | "练/训练 + 身体部位或运动" | "学/练习 + 技能名 + 每天" | "记忆/背诵/搞懂 + 知识领域" |

判定优先级：**动作对象词 > 一句话模式 > 名称**。示例："每天早读 30 分钟英语外刊" → cognitive（阅读对象 + 每日固定投入）；"晨跑 5 公里" → physical；"背 GRE 单词" → knowledge（记忆型）。给出建议时附带判定依据，便于用户在确认页校对。

#### C.4.2 difficulty 推断

| 用户投入描述（示例） | 建议 |
|---|---|
| "随便玩玩 / 偶尔 / 想起来才练" | casual |
| "每天 / 固定 X 分钟 / 常规练习" | normal |
| "每天 ≥1~2 小时 / 系统训练 / 刻意练习" | hard |
| "近乎每天数小时 / 为比赛或考试" | challenge / legendary |

规则：从 description 提取强度词 → 映射；提取不到一律 `normal`。MVP 不提供"目标反推"难度输入（已定档，附录 B#1）；"期望时长 → difficulty"反推留待 v1.1 向导增强（§1.4）。

#### C.4.3 关联推荐（技能 → 属性候选）

先匹配 `attribute.description` 的同义词，再退化为按技能 `category` 的候选表（v1 内置）：

| 技能类别 | 主属性候选（w=1.0） | 次属性候选（w=0.3~0.5） |
|---|---|---|
| physical | 体力 | 力量 / 敏捷 |
| cognitive | 智力 | 专注 |
| knowledge | 智力 | 专注 |
| 名称含"口才/演讲/沟通/社交" | 魅力 | 智力 |

- 推荐条数 ≤ 3；无候选时不硬推，提示"此技能暂不关联属性或先创建属性"。
- 用户确认后落库；权重一律可在确认页调整。
- **准确性增强机制**：属性应填写 `description`（含同义词列表），AI 的命中优先级 = 同义词命中 > 类别表。推荐结果计入 `audit_log`，可据此迭代词表。

#### C.4.4 父节点推荐

名称 token 与现有树匹配：如"英语外刊" → 命中已存在的 `语言 / 英语` 分组则挂入，否则建议挂到最接近的分类或置根。仅给出 1 个候选（避免选择负担），可改。

### C.5 端到端示例（技能新建，M2）

用户输入："新增技能：每天早读 30 分钟英语外刊，主要练理解和词汇。"

1. AI 读取 `soloup_meta` + `soloup_skill_tree` + `soloup_attribute_list`。
2. 生成候选并自证：
   - `category = cognitive`（阅读/理解对象词，high）
   - `difficulty = normal`（每天固定 30 分钟）
   - `parent_id = 语言 > 英语`（若树中存在该分组，med）
   - 关联：智力 w=1.0（词义命中"理解"）、专注 w=0.5（"每天固定阅读"）
   - 配色/图标：按 cognitive 默认
3. 确认页逐项展示（值 + 置信度 + 理由）。用户把专注改为 w=0.3 并确认。
4. 落库：`skills` + `skill_attributes` 一次事务；`audit_log` 记 actor=`ai`、params 含最终值与修改项。
5. 面板技能树立即出现该技能，随后可正常打卡结算。

### C.6 取消/纠错路径

- 任何 M2 确认前可整体放弃，不产生脏数据。
- 创建后纠错：category / difficulty / curve_type / parent / 关联 均可修改（`skill_update` / `skill_move` / `link_set`）；修改**不追溯历史**——`C/V/level` 是历史行为与"当前曲线参数"的函数，改变参数后仅影响**未来展示与后续结算**，文档在技能详情页注明"曲线参数变更于 X 日生效"。
- 若认为历史行为也应按新曲线重算，属破坏性操作，需显式 `recalc(force_params=...)` + 二次确认 + 审计（v1.2 评估）。
- **参数时间线即权威重放依据**：`audit_log` 记录每次参数变更（`soloup_param_set`、技能 `update` 的 difficulty/curve_type/category、关联权重）及生效时刻（§9.5-10）；§3.1 的确定性重放以「记录 + 时间线」为输入，保证任何历史状态可精确复现。

### C.7 与 MCP 的配合

- AI 执行推断前必须读 `soloup_meta`（枚举与词表版本），避免用过期枚举建数据。
- `soloup_skill_create` / `soloup_skill_update` / `soloup_skill_link_set` 返回结构化结果时附 `suggestion_confidence` 字段；服务端在 low 置信度下返回 `ERR_CONFIRM_REQUIRED`，客户端需先展示给用户。
- 面板确认页与 MCP 流程共用同一份**候选 diff 结构**（schema 定义于 `packages/core`），保证"对话框里确认"和"面板里确认"体验等价。
- **默认层参数自动调优**：`soloup_param_set` 是 AI 修改注册表参数（§9.2.1）的唯一入口。AI 先 `param_preview` 展示影响面（受影响技能数、等级差示例），再提交变更；默认 M2 需用户确认。典型场景："最近 60 天『知识』类技能等级涨得比预期慢，建议把 curve.knowledge.x0 从 175 降到 150，会影响 3 个技能，预览如下…"

---

## 附录 D：技术选型决策记录

> 2026-09 定稿。评估原则：**单语言全栈、领域纯函数与 IO 分离、本地优先、少依赖、写路径同步事务、所有数字可由 solver 确定性重放**。MVP 无云、无用户系统、数据规模个人级——候选均在此约束下评估，不为"大规模/多团队"场景买单。

### D.1 决策总表（结论快照）

| 决策项 | 结论 | 备选（否决） |
|---|---|---|
| 语言 | TypeScript（strict） | —（全栈共享 core 类型的唯一合理选择） |
| 运行时 | Node ≥ 22，建议 24 LTS | Deno / Bun（better-sqlite3、Next、MCP SDK 兼容风险） |
| 仓库/包管理 | pnpm workspace | npm workspaces（慢/隔离弱）、turborepo（MVP 包少，缓存收益低）、rush |
| Web 框架 | **Next.js 15（App Router）** | Vite + Express、SvelteKit、纯静态 + 自建 API |
| UI | Tailwind CSS + shadcn/ui + motion + dnd-kit | MUI / AntD / Chakra（组件重、视觉定制成本高） |
| 客户端数据 | **TanStack Query** | SWR（能力子集）、原生 fetch（需手写失效/乐观逻辑） |
| 数据访问 | **better-sqlite3 + 手写 SQL Repository** | Drizzle、Kysely、Prisma、node:sqlite |
| Schema 迁移 | 自写 `user_version` 脚本 | Drizzle migrate（MVP 迁移量极小，依赖多余） |
| 输入校验 | Zod | 手写 JSON Schema（难复用）、Valibot（心智不互通） |
| ID | `crypto.randomUUID()` | nanoid / ulid / uuid（本地单机无排序/短码需求） |
| 日期 | core 内置 `YYYY-MM-DD` 本地日纯函数 | date-fns、dayjs、Temporal（语义极简，引库徒增时区坑） |
| 测试 | Vitest | Jest（ESM 配置繁琐） |
| MCP | `@modelcontextprotocol/sdk` | 自研协议（成本高、无生态） |
| 桌面壳 | Tauri v2（未来） | Electron（包体/内存大） |
| 脚本/质量 | tsx；ESLint 9 + typescript-eslint + Prettier | ts-node（ESM 配置繁琐） |

### D.2 关键论证

1. **Web 框架：Next.js 15，否决 Vite+Express**
   需求实质：本地**单进程**同时提供页面与 JSON API、Node 端直连 SQLite、未来可被 Tauri 内嵌。
   - Next 以 Route Handler 提供 API（Node runtime），页面同为该进程产物，与 §8.2 架构一致；本地场景页面可用 Client Components + TanStack Query 拉取。
   - `output: 'standalone'` 的产物可整体被桌面壳复用，桌面化不重写。
   - Vite+Express 更轻、HMR 更快，但需自建服务端目录结构并改写 §8.2，桌面化需重新接线——收益不足以抵消一致性成本。记录为**可回退条款**（见 D.3），领域层/store/MCP 均不受影响。

2. **数据访问：better-sqlite3 + 手写 SQL，否决 ORM**
   - better-sqlite3 同步 API：结算这类"读状态 → 计算 → 多表写"流程可在单个同步事务内线性执行，无异步编排心智，且吞吐对个人数据量绰绰有余。
   - 领域规则与数值全部在 `core`/`solver`，`store` 仅是表↔内存的薄映射 + 事务包裹。ORM 的类型安全、迁移、关系加载在此几乎无用武之地。
   - Prisma：引擎二进制 + 代码生成 + 异步模型，与同步结算配合繁琐 → 否决。Drizzle/Kysely：SQL 本身短且数量少，多一层抽象不值 → 否决；**若未来 SQL 复杂度上升，优先补 Drizzle 而非手写扩展**（可回退条款）。
   - 迁移：`migrations/*.sql` + `PRAGMA user_version` 递增，约 20 行脚本覆盖全生命周期。

3. **客户端数据：TanStack Query**
   页面虽少，但"打卡 → 即时数值反馈、向导确认 → 列表刷新、参数调优 → 曲线预览"全部依赖**缓存失效与乐观更新**；手写 fetch 的失效逻辑恰是该类应用最易出错处。SWR 是能力子集；原生 fetch 最简但需自维护。

4. **校验：Zod**
   core 中一处定义、三处消费：MCP 工具参数（可导出 JSON Schema）、HTTP API body、param 注册表（§9.2.1）范围校验。Valibot 体积更小但与现有生态/心智不互通。

5. **日期：不引库**
   §3.5 语义只有"本地自然日 `YYYY-MM-DD`"的当日、加 N 天、间隔三种操作。在 `packages/core/dates` 内置 4 个纯函数（toKey / toDate / addDays / diff）即覆盖，避开 date-fns/dayjs 的体积与时区陷阱；Temporal 仍未成为默认稳定 API。

6. **运行时：Node ≥ 22（建议 24 LTS）**
   Next 15 与 MCP SDK 的长期支持基线；better-sqlite3 预编译覆盖活跃 LTS。Node 20 已近 EOL；内置 `node:sqlite` 尚不稳定，不作为生产依赖。

### D.3 可回退条款（决策不锁死，降低反悔成本）

| 情形 | 回退路径 | 影响面 |
|---|---|---|
| 本地 Web 热更/体积成为痛点 | apps/web 平替为 Vite + Express | 仅 apps/web；领域层/store/MCP 零改动 |
| Repository SQL 复杂度明显上升 | 引入 Drizzle 逐步替换手写 SQL | 仅 packages/store |
| 自写迁移不够用 | 切 Drizzle migrate | 仅 packages/store |
| 页面交互需要 E2E | 补 Playwright | 新增，无迁移成本 |

> MVP 明确**不引入**：turbo、Playwright、Tauri、CI 流水线、遥测/分析、权限系统。

### D.4 落库约束（编码时强制）

1. 根 `package.json`：`engine-strict=true`，`engines.node >= 22`；`packageManager: "pnpm@>=9"`；锁文件提交。
2. 依赖边界 lint：`better-sqlite3` 只允许被 `packages/store` import；core/solver 保持**纯函数、无 IO**（可单测、可被任意进程复用）。运行时第三方依赖：core 仅允许 `zod`（D.4-3 schema 单一来源所需），solver 不允许任何第三方。*（2026-09-06 执行期微调：原「零外部依赖」与 D.4-3「core 内定义 Zod schema」冲突，按 schema 单源优先修订为 zod 例外。）*
3. 所有工具/API 参数 schema 只由 `packages/core` 的 Zod 定义，MCP 与 HTTP 层不得另写一套。
4. 时间表示统一走 `packages/core/dates`；禁止在业务代码里直接散用 `new Date()` 格式化。

## 附录 E：UI 设计决策记录

> 2026-09 定稿。方法：把 UI 决策拆成「技术层（已由附录 D 定）」「观感 / 信息架构 / 可视化实现 / 树形态渲染」四层，仅后四项需拍板；其余不影响用户决策自由的设计默认直接采纳（见 E.3），不再逐项确认。

### E.1 决策总表

| 决策项 | 结论 | 备选（否决） |
|---|---|---|
| 视觉气质 | **温暖极简 + 游戏化点缀** | 强游戏化仪表盘（Habitica/Duolingo 型）、数据科学极客风（Linear/Strava 型）、杂志手账 |
| 信息架构 | **侧边栏 5 视图 + ⌘K + 全局打卡** | 顶栏 Tab、单页大看板流 |
| 可视化实现 | **自写 SVG 4 原语（零依赖）** | Recharts、ECharts |
| 技能树形态 | **折叠缩进列表（sortable tree）** | 卡片嵌套树、节点连线树图 |

### E.2 关键论证

1. **视觉气质**：产品 = 每日高频 + 数值密集 + 「抗流失 / 防挫败」原则（§1.2）。大面积高饱和 → 视觉疲劳与数值噪声；纯数据极客风与"成长反馈"的情绪价值冲突，且推翻已定的亮色基调。折衷：中性底 + 品牌橙唯一强调保持克制，游戏化动效**只挂载在升级/达成/解锁这类高光瞬间**——成本低且契合奖励时机（v1.1 成就卡才引入，不阻塞 MVP）。
2. **信息架构**：MVP 视图仅 5 个，"今日打卡"频次远超其它 → 固定侧栏（移动端底 tab）最稳、纵向空间最大；打卡应"任何页面都能发起" → 悬浮按钮 + ⌘K。不做单页大看板：图表与成就未到（v1.1），过早聚合对新手是心智负担。
3. **可视化**：MVP 数学输出可穷举为 4 类原语，且点数组已由 core 算好，前端只需"画"。手写 SVG 零新依赖（契合 D.4 少依赖）、tooltip 与"可解释"文案完全可控；Recharts 的缩放/坐标系能力用不上，自定义里程碑点与主题 token 绑定反而贵；ECharts 体积与默认样式与 token 化冲突。
4. **技能树形态**：主视图需高频遍历 + 拖拽调层 + 信息密度。折叠缩进列表天然支持深层级、行内可挂迷你进度与参数徽标、拖拽最不易误触；卡片嵌套浪费横向空间；连线树图自动布局成本高且对拖拽不友好，仅留待"成长全景"类页面（未来）单独评估。

### E.3 直接采纳的默认项（不再逐项确认）

- 亮色 v1（深色 v1.1）；原四色保留但**降级为 token 体系内的数据/语义色**（不再视觉并列抢焦点）。
- 系统字体栈 + `tabular-nums`；不引 web font；移动端字号 ≥14px、触控目标 ≥44px。
- 桌面多栏 / 移动单列 + 底部 tab（呼应原 §7 响应式约定）。
- 全局 ⌘K、打卡悬浮按钮、向导置信度徽标配色、Sigmoid 补偿文案、卡片式容器。

### E.4 可回退条款

| 情形 | 回退路径 | 影响面 |
|---|---|---|
| v1.1 图表数量/复杂度激增 | 图表原语切换 Recharts | 仅 apps/web 内 charts |
| 气质拿不准、要更强游戏化 | 提升点缀层级（动效密度/渐变卡） | 仅 token 与高光组件 |
| 树层级极深、折叠列表臃肿 | 卡片嵌套或新增"成长全景"连线图 | 仅技能树页 |
| 深色诉求提前 | v1.1 提前补暗色变量 | token 层已预留覆盖位 |

### E.5 v1.1 才定的 UI 细节（不阻塞 MVP 开工）

成就墙排版与解锁动画、项目轴页、图表聚合页、暗色变量、Web font 评估（本地打包是否可选字体内嵌）。

---

## 附录 F：长期工程路线图（执行基线 v1，2026-09-06）

> 说明：本文为**工程执行配套**，非产品规格，不改变已定稿范围；MVP 范围以 §1.4 为准，验收锚点全部引用 §10。状态在本表维护（☐→✔），跨会话开工前先读本附录续接进度；每里程碑收尾须更新本表状态与版本历史。

| # | 里程碑 | 版本 | 关键交付 | 出口标准 / 验收锚点 | 依赖 | 状态 |
|---|---|---|---|---|---|---|
| 0 | **仓库与工具链基线** | MVP | pnpm workspace（root + `core/store/solver/mcp-server` + `apps/web`，desktop 预留）、TS strict、tsx、ESLint 9/Prettier、Vitest、engine ≥22 + `engine-strict`、依赖边界 lint（`better-sqlite3` 仅 store）、schema 单源目录约定 | §8.1 选型表 / D.4 落库约束落地可校验 | — | ✔ |
| 1 | **core 领域核心** | MVP | 全实体类型与 Zod schema（与 §4.3 对齐）、dates 时间工具（§3.5）、默认参数表（附录 A）、§3 结算/曲线/聚合/派生数学纯函数、AI 推断词表（§C.4）、§3.10 黄金表测试 fixture | §10.1 数值正确性全绿（黄金表 / 断更 / 补结算等价 / 单调性 / 收敛性 / 饱和边界） | 0 | ✔ |
| 2 | **store 持久化** | MVP | SQLite 单文件 WAL + `PRAGMA user_version` 迁移 + Repository（全部 CRUD）；直改 `C/V` 被服务层拒绝 | §10.2 不变量中 store 接口层面用例 | 1 | ✔ |
| 3 | **solver 引擎** | MVP | `recordCheckin / settleSkill / applyMutation`、父级聚合、属性派生、归档/恢复、环检测、非叶子打卡拒绝、同日覆盖→全量重放、参数时间线重放 | §10.1 + §10.2 全部单测绿 | 1, 2 | ✔ |
| 4 | **MCP server v1** | MVP | stdio server、§9.2 v1 工具全量（含 `soloup_meta`、`param_preview/set/remove`、`audit_log`）、读前结算到"今日"、每工具独立事务、Zod 参数校验与结构化错误码、审计全覆盖 | §10.3 集成测试全绿；CodeBuddy 实连可用（§8.2 双进程） | 3 | ✔ |
| 5 | **Web 面板 MVP** | MVP | Next 15 本地服务、Route Handler（`serverExternalPackages` / `standalone`）、TanStack Query、shadcn/ui + motion + dnd-kit、5 视图 + ⌘K + 悬浮打卡、创建向导（M2 确认）、SVG 4 原语、移动断点 / a11y | §6 功能自检清单 + §7 UI 走查 + §10.4 单次打卡 <200ms | 3（+2） | ☐ |
| 6 | **MVP 集成验收** | MVP | 面板 + MCP 双进程同库并发、全量补结算性能预算、端到端验收 | §10.1–10.4 全绿 → **MVP 完成** | 4, 5 | ☐ |
| 7 | **v1.1** | v1.1 | 项目轴、成就 DSL 引擎与卡牌、MCP 打卡/结算（默认 `confirm`）、`soloup_insights` 复查软提示、图表聚合页、数据导出、难度目标反推向导、技能迁移合并、暗色 | §1.4 v1.1 行逐项落地 + 各章对应验收 | 6 | ☐ |
| 8 | **v1.2** | v1.2 | `soloup_import`、历史编辑与补打卡、多设备同步（可选） | §1.4 v1.2 行 | 7 | ☐ |
| 9 | **未来** | — | Tauri 壳（复用 standalone）、里程碑复盘报告、技能推荐、远程 MCP（HTTP/SSE）、成长全景连线图 | §1.4「未来」行 | 按序 | ☐ |

**实施顺序**：#0→#6 为 MVP 主链，严格执行"上一链出口标准全绿才进入下一链"；#1 core 先行（Web/MCP/测试全部消费它）；依赖允许 4 与 5 部分并行（共用 solver 契约）。

**横切长期约束**（每个里程碑都须守住，违反即退回）：
- core/solver 纯函数、无 IO（D.4-2；core 运行时第三方依赖仅 `zod`，solver 零第三方）；Zod schema 单一来源，MCP 与 HTTP 层不另写（D.4-3）；时间一律走 `core/dates`（D.4-4）。
- 一切写入经 solver 服务函数，审计全覆盖；数值只由结算/重放产生；任何历史状态可确定性重放（§3.1 / §9.5）。
- 每个里程碑收尾 = 对应 §10 用例全绿 + 更新本表与版本历史，留下完工摘要。

> **M0 ✔ 完工摘要（2026-09-06）**：pnpm workspace（root + 4 包）骨架就绪；实测 Node 22.22 / pnpm 10.24；tsc build/typecheck、Vitest（依赖边界护栏测试 ×2）、ESLint、Prettier 全绿；`better-sqlite3` 预编译绑定可用（`onlyBuiltDependencies` + rebuild）。

> **M1 ✔ 完工摘要（2026-09-06）**：core 领域核心落地（`schemas` 实体/enum/param_overrides Zod 单源、`dates`、`params` 默认参数表、`curve`/`settle`/`aggregate` §3 数学、`vocab` §C.4 词表、`goldens` §3.10 fixture）。测试 47 全绿：三类别黄金表 round3 逐日精确复现、断更/闭合等价 ≤1e-9、收敛 V_eq±0.01、sigmoid 零起点/半程点/难度平移、属性派生 §3.9 表场景。工程口径：core 运行时第三方依赖仅 zod（D.4-2 微调已落档）、NodeNext + 相对 `.js` 导入（d.ts 产物可被下游解析，Node ESM 冒烟通过）。下一步：#2 store。

> **M2 ✔ 完工摘要（2026-09-06）**：store 持久化落地（`packages/store`）：`openDatabase`/`openStore` 门面——单文件 SQLite + WAL（面板/MCP 双进程同库，§8.2）、`foreign_keys=ON` + `busy_timeout`、`PRAGMA user_version` 增量迁移与"库比程序新即拒开"（NewerSchema）版本保护、事务入口（嵌套 savepoint 自动回滚）；v1 迁移 9 表（`attributes`/`skills`/`skill_attributes`/`daily_records`/`daily_record_skills`/`projects`/`achievements`/`audit_log`/`settings`）；8 组 Repository 全部 CRUD：技能树不变量（递归 CTE 子树/祖先、父级移动环检测、有子/有历史删除守卫、archive/restore 幂等）、档案更新白名单合并（`c/v/last_settled_date` 不在写入口，即使混入 patch 也不生效——直改 C/V 经 store 不可达，§10.2）、技能↔属性关联、同日期打卡记录原子 upsert（含技能明细）、项目/成就（`condition_json` 往返）、审计（`params_json` 快照往返）、settings（JSON 键值）。测试 +30 全绿（db 迁移/重开/WAL/版本守卫/事务 + repos 行为与不变量），全仓 8 文件 77 用例；行数据一律经 core Zod schema 校验出包（D.4-3 单一来源）。工程口径：better-sqlite3 原生绑定（`onlyBuiltDependencies` + prebuilt）实测可用；NodeNext + 相对 `.js` 导入，`tsc` build 后 dist ESM 冒烟通过。下一步：#3 solver。

> **M3 ✔ 完工摘要（2026-09-06）**：solver 引擎落地（`packages/solver`）：可注入时钟 `createSolver(store, {now})`，服务层面覆盖 `checkin / clearCheckin / settleAll / archive / restore / mutate（skill.create·skill.update·attribute.create·link.set 等原子写入）/ snapshot / replaySkill`；store 侧增补 C/V 专用结算通道 `skills.settleAccounts`（白名单外直改 C/V 仍不可达，§10.2）、`dailyRecords.listMembers`、非叶技能不可关联属性守卫。结算/重放数学在 `ledger.ts`：`settleRange`（跨参数段按段闭合式、防呆上限）与 `replayFull`（基于参数时间线分段的确定性全量重放）；参数时间线（§C.6）由 `params.ts` 读取 `audit_log(skill.update)` 构建（store 以 TEXT 落库，读回按需 JSON 解码），变更自 `effective_at`（修改当日先结算再生效）生效、不追溯历史；`derive.ts` 纯派生快照（父级聚合 §3.9 一般平均、属性 `base+S·(1−e^(−X/W))`）。测试 23 全绿（ledger 6 + service 17，覆盖 §10.1 三类别黄金表 round3 逐日复现 / 断更 60 天 / 结算幂等、§10.2 服务层拒绝直改 C/V 等全部不变量 / 同日覆盖→全量重放 / 参数覆盖只影响后续 / 时间线分段生效 / 派生对照）；全仓 100 用例（core 47 + store 30 + solver 23）+ solver `tsc` build / eslint / prettier 全绿。工程口径教训：solver 测试 harness `openStore()` 默认落在 `~/.soloup/soloup.db` 文件库 → 用例互污染，一律 `{ path: ':memory:' }` 独立内存库。下一步：#4 MCP server v1。

> **M4 ✔ 完工摘要（2026-09-06）**：MCP server v1 落地（`packages/mcp-server`）：stdio + JSON-RPC 2.0 逐行服务（`server.ts` `SoloupMcpServer`）——`initialize` 协议协商（支持 2024-11-05 / 2025-03-26 / 2025-06-18，缺省 2025-06-18）、`ping`、`tools/list`、`tools/call`；每个 `tools/call` 在独立 `store.tx` 内先 `solver.settleAll()` 读前结算到"今日"（§9.1-4），再进 handler，返回统一 `{content:[{type:'text'}]}` 包装（`ok/data` 或结构化 `{code,message}`）。§9.2 v1 全量 **24 工具**（元信息 + 查询 + 管理 CRUD + 参数调优），参数 schema 单一来源 `packages/core/src/mcp.ts`（`mcpToolArgSchemas`，24 zod 对象，`packages/mcp-server/src/zjson.ts` 做 zod→JSON-Schema 供 `tools/list`）；§9.2.1 参数注册表与覆盖函数落 `packages/core/src/param-registry.ts`（key 白名单、开闭区间、category `c+f≤0.05` 横切约束、`preview/set/remove` 同源的 `computeImpact` 影响面估算 + 等级差采样）。领域错误 → 结构化 `ToolErrCodes`（UNKNOWN_TOOL / VALIDATION / NOT_FOUND / CONSTRAINT / LINKED / CYCLE / UNKNOWN_PARAM / PARAM_RANGE / INTERNAL），未知一律 INTERNAL 绝不裸抛。入口 `index.ts`：`openStore()`（可用 `SOLOUP_DB_PATH` 指定库）→ 服务循环。测试 5 全绿（`server.test.ts`：初始化协商与工具清单 = 24、属性→技能→关联→结算→树可见 CRUD 冒烟、参数约束链 unknown/越界/c+f 拒绝 + set 审计 old→new、缺必填/坏枚举写前 VALIDATION、写操作审计 actor=ai 全覆盖 + 读前结算可见性）；**全仓 105 用例** + build / typecheck / eslint / prettier 全绿。工程口径教训：① 本机 node_modules 的 `@soloup/*` junction 因上次中断安装而损坏且无法原地重建 → 编译期以 `tsconfig.base.json` 的 `paths` 把 `@soloup/*` 映射到各包 `dist/*.d.ts`，测试期由 Vitest `resolve.alias` 直指各包 `src`；**进程级 stdio 冒烟 / CodeBuddy 实连（§10.3 出口）须待重装 `pnpm install` 修复 `node_modules/@soloup` 链接后执行**；② `solver.mutate` 返回 `{ audit, detail }`，工具层统一取 `detail` 解包并透传。下一步：#5 Web 面板 MVP（依赖 3；按附录 F 与 4 可部分并行）。

---

## 版本历史

| 版本 | 说明 |
|---|---|
| v1.0（完整稿） | 三轮评审共识全量落地：双轨模型 + 参数解耦 + 属性纯派生 + MCP 架构 + 参数三层与 AI 协作规则（附录 C）+ 默认层参数对 AI 开放自动调优（§9.2.1 注册表 + `soloup_param_*`）+ 技术选型（§8.1 正式表 + 附录 D）+ UI 设计（§7 正式规格 v1.0 + 附录 E）+ 完整验收体系。 |
| v1.0（定稿） | 在完整稿之上完成最终审计与定档：附录 B 六项开放问题全部拍板关闭（目标反推后置 v1.1、base_value 高级可改、打卡工具默认 confirm、不引批量建树、Tauri 维持未来、复查软提示入 v1.1）；一致性修复落档（技能 `description`/`sort`、`param_overrides` 结构与 settings 序列化、记录覆盖→全量重放、参数时间线、交叉引用修正）。本文为定稿，可按 §4.2 包结构开工；MVP 范围以 §1.4 为准。定稿后（2026-09-06）执行微调：D.4-2 明确 zod 为 core 唯一运行时第三方依赖（schema 单源所需），附录 F 横切约束同步。 |

*已定稿（2026-09-06）。后续变更走版本历史演进；开工入口见 §4.2 包结构与 §10 验收体系。*
