//! MCP 工具定义和 handler 映射。
//!
//! 所有工具复用 `soloup_server::dispatch`，避免重复实现业务逻辑。
//! actor 默认为 `ai`，写入审计日志。

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use soloup_server::dispatch;
use soloup_solver::Solver;

type AppState = Arc<Mutex<Solver>>;

// ─── 工具定义 ────────────────────────────────────────────────────────────────

/// 工具定义结构
struct ToolDef {
    name: &'static str,
    description: &'static str,
    input_schema_str: &'static str,
}

/// 构建工具定义列表（运行时）
pub fn get_tool_definitions() -> Vec<Value> {
    TOOLS.iter().map(|t| {
        json!({
            "name": t.name,
            "description": t.description,
            "inputSchema": serde_json::from_str::<Value>(t.input_schema_str).unwrap_or(json!({}))
        })
    }).collect()
}

static TOOLS: &[ToolDef] = &[
    // ─── 元信息工具 ────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_meta",
        description: r#"返回全部枚举（category/difficulty/curve_type/属性 category）、默认参数表、当前 param_overrides、预置属性与成就模板。

**AI 推断前必读**：执行任何管理类操作前，先调用此工具获取枚举值和参数元信息。

**返回**：
- enum: 所有枚举类型及其合法值
- registry: 可调参数注册表（key/label/default/min/max）
- overrides: 当前参数覆盖

**使用场景**：
- 创建技能前获取 category/difficulty 枚举
- 调优参数前获取注册表和当前覆盖"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    // ─── 查询工具 ──────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_panel_overview",
        description: r#"面板总览：属性当前值 + 趋势、技能树摘要、今日是否已打卡、成就统计、生命轴。

**返回**：
- date: 当前日期
- attributes: 属性列表（含派生值、贡献占比）
- tree: 技能树结构
- records: 最近记录
- projects: 项目列表
- achievements: 成就列表
- profile: 用户配置
- life: 生命轴（出生日期/预期寿命/进度）
- stats: 统计（总天数/连续/本周/最高技能等级等）
- today: 今日打卡状态

**使用场景**：
- AI 需要了解面板整体状态时
- 用户问"我现在什么水平"时"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_attribute_list",
        description: r#"列出所有属性（含派生值、description、贡献占比）。

**返回**：属性数组，每项包含 id/name/description/category/color/value/x/contributions

**使用场景**：
- 了解当前属性面板
- 创建技能时选择关联属性"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_attribute_get",
        description: r#"获取单个属性详情 + 关联技能列表。

**参数**：
- id (必填): 属性 ID

**返回**：属性详情 + 关联技能（含权重）

**使用场景**：
- 查看某属性的详细构成
- 分析某属性由哪些技能贡献"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"属性 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_tree",
        description: r#"返回完整技能树（含等级/E/C/V/曲线参数/子节点/关联属性）。

**返回**：树形结构，每个节点包含：
- id/name/category/difficulty
- level/effective_exposure (E = C + V)
- c (结晶账户) / v (活性账户)
- curve_type (saturated/sigmoid)
- children (子节点)
- attrs (关联属性 [[attrId, weight]])

**使用场景**：
- 了解技能树整体结构
- 创建技能时选择父节点
- 分析技能成长状态"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_get",
        description: r#"获取单个技能详情 + 子节点 + 关联属性。

**参数**：
- id (必填): 技能 ID

**返回**：技能详情（id/name/description/category/difficulty/level/关联属性/子节点）

**使用场景**：
- 查看某技能的详细状态
- 分析技能的成长曲线和关联"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_daily_status",
        description: r#"获取某日打卡详情 / 结算状态。

**参数**：
- date (可选): 日期 YYYY-MM-DD，默认今天

**返回**：
- date: 日期
- skillIds: 当日打卡技能 ID 列表
- attrsLit: 当日点亮的属性
- projectId: 归属项目（如有）
- note: 备注

**使用场景**：
- 查看某天的打卡记录
- 确认今天是否已打卡"#,
        input_schema_str: r#"{"type":"object","properties":{"date":{"type":"string","description":"日期 YYYY-MM-DD","pattern":"^\\d{4}-\\d{2}-\\d{2}$"}},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_audit_log",
        description: r#"获取最近变更记录（审计追溯）。

**参数**：
- limit (可选): 返回条数，默认 50
- offset (可选): 偏移量，默认 0
- actor (可选): 过滤 actor (user/ai/system)
- tool (可选): 过滤工具名

**返回**：审计记录数组（id/at/actor/tool/params）

**使用场景**：
- 追溯最近的变更历史
- 查看 AI 或用户做了什么操作"#,
        input_schema_str: r#"{"type":"object","properties":{"limit":{"type":"integer","description":"返回条数","default":50},"offset":{"type":"integer","description":"偏移量","default":0},"actor":{"type":"string","enum":["user","ai","system"],"description":"过滤 actor"},"tool":{"type":"string","description":"过滤工具名"}},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_settings_get",
        description: r#"获取当前设置（参数覆盖 / AI 执行白名单 / 主题等）。

**返回**：
- registry: 可调参数注册表
- overrides: 当前参数覆盖

**使用场景**：
- 查看当前参数配置
- 了解哪些参数被覆盖"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_insights",
        description: r#"**分析建议**：E 增速分析、趋势缺口、推荐技能。

**返回**：
- weak_attributes: 弱项属性（值最低的 2-3 个）
- recommended_skills: 推荐技能（基于弱项属性关联）
- growth_trend: 成长趋势（最近 7/30 天的 E 值变化）
- suggestions: 具体建议（文字描述）

**使用场景**：
- 用户问"我该怎么提升"时
- AI 主动给出成长建议
- 分析成长瓶颈和突破口

**注意**：此工具为分析型，不修改任何数据"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    // ─── 属性管理 CRUD ───────────────────────────────────────────────────
    ToolDef {
        name: "soloup_attribute_create",
        description: r#"创建新属性。

**参数**：
- name (必填): 属性名称（如"力量"、"智力"）
- description (可选): 一句话定义，供 AI 关联推荐
- category (可选): physical/mental/social/creative
- color (可选): 颜色（默认按 category 自动生成）
- icon (可选): 图标
- base_value (可选): 基础值，默认 0
- max_value (可选): 上限，默认 100
- alpha (可选): 聚合指数，默认 1.2
- w0 (可选): 饱和尺度，默认 400

**返回**：创建的属性详情

**使用场景**：
- 用户说"我想新增一个 XXX 属性"时
- AI 分析后建议新增属性

**注意**：category 建议由 AI 依据属性名推断"#,
        input_schema_str: r#"{"type":"object","properties":{"name":{"type":"string","description":"属性名称"},"description":{"type":"string","description":"属性定义"},"category":{"type":"string","enum":["physical","mental","social","creative"]},"color":{"type":"string"},"icon":{"type":"string"},"base_value":{"type":"number"},"max_value":{"type":"number"},"alpha":{"type":"number"},"w0":{"type":"number"}},"required":["name"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_attribute_update",
        description: r#"更新属性。

**参数**：
- id (必填): 属性 ID
- name (可选): 新名称
- description (可选): 新定义
- category (可选): 新分类
- color (可选): 新颜色
- icon (可选): 新图标
- sort (可选): 排序

**返回**：更新后的属性详情

**使用场景**：
- 修改属性名称或描述
- 调整属性分类或颜色"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"属性 ID"},"name":{"type":"string"},"description":{"type":"string"},"category":{"type":"string","enum":["physical","mental","social","creative"]},"color":{"type":"string"},"icon":{"type":"string"},"sort":{"type":"integer"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_attribute_delete",
        description: r#"删除属性。

**参数**：
- id (必填): 属性 ID

**返回**：删除结果

**使用场景**：
- 删除不再需要的属性

**注意**：
- 会先解除所有关联
- 仅当无历史强依赖时可物理删除
- 否则建议归档（当前版本不支持属性归档，请谨慎操作）"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"属性 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    // ─── 技能管理 CRUD ────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_skill_create",
        description: r#"创建新技能。

**使用场景**：
- 用户说"我想学 XXX"时
- AI 分析用户活动后建议新增技能
- 先调用 soloup_meta 获取枚举，soloup_skill_tree 了解现有结构
- 根据用户描述推断 category/difficulty，推荐 parent_id 和关联属性

**参数**：
- name (必填): 技能名称
- category (必填): physical/cognitive/knowledge
- description (可选): 一句话用途说明
- parent_id (可选): 父技能 ID（null 为根分类）
- difficulty (可选): casual/normal/hard/challenge/legendary，默认 normal
- links (可选): 关联属性 [{attribute_id, weight}]

**返回**：创建的技能详情

**注意**：
- category/difficulty 建议由 AI 依据用户描述推断，并说明推断依据
- links 的 weight 建议主关联 1.0，次关联 0.3-0.5"#,
        input_schema_str: r#"{"type":"object","properties":{"name":{"type":"string","description":"技能名称"},"category":{"type":"string","enum":["physical","cognitive","knowledge"],"description":"技能类别"},"description":{"type":"string","description":"用途说明"},"parent_id":{"type":"string","description":"父技能 ID"},"difficulty":{"type":"string","enum":["casual","normal","hard","challenge","legendary"]},"links":{"type":"array","items":{"type":"object","properties":{"attribute_id":{"type":"string"},"weight":{"type":"number","minimum":0,"maximum":1}},"required":["attribute_id"]}}},"required":["name","category"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_update",
        description: r#"更新技能。

**参数**：
- id (必填): 技能 ID
- name (可选): 新名称
- description (可选): 新描述
- category (可选): 新类别
- difficulty (可选): 新难度
- curve_type (可选): 曲线类型 (saturated/sigmoid)

**返回**：更新后的技能详情

**使用场景**：
- 修改技能名称或描述
- 调整难度或曲线类型"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"},"name":{"type":"string"},"description":{"type":"string"},"category":{"type":"string","enum":["physical","cognitive","knowledge"]},"difficulty":{"type":"string","enum":["casual","normal","hard","challenge","legendary"]},"curve_type":{"type":"string","enum":["saturated","sigmoid"]}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_move",
        description: r#"移动技能到新的父节点。

**参数**：
- id (必填): 技能 ID
- parent_id (可选): 新父技能 ID（null 为移到根）

**返回**：移动后的技能详情

**使用场景**：
- 重组技能树结构
- 将技能归类到更合适的分类下

**注意**：服务端会执行环检测，防止循环引用"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"},"parent_id":{"type":"string","description":"新父技能 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_archive",
        description: r#"归档技能。

**参数**：
- id (必填): 技能 ID

**返回**：归档结果

**使用场景**：
- 不再练习但想保留历史的技能
- 归档后不再参与结算，但保留历史数据

**注意**：归档是软删除，可随时恢复"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_restore",
        description: r#"恢复已归档的技能。

**参数**：
- id (必填): 技能 ID

**返回**：恢复结果

**使用场景**：
- 重新开始练习之前归档的技能"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_delete",
        description: r#"删除技能（物理删除）。

**参数**：
- id (必填): 技能 ID

**返回**：删除结果

**使用场景**：
- 删除无打卡历史的技能

**注意**：
- 仅允许删除无打卡历史的叶子技能
- 有历史数据的技能应使用归档而非删除
- 此操作不可逆"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_link_set",
        description: r#"设置技能的属性关联。

**参数**：
- id (必填): 技能 ID
- links (必填): 关联列表 [{attribute_id, weight}]

**返回**：更新后的技能详情

**使用场景**：
- 建立技能与属性的关联
- 调整关联权重

**注意**：
- 关联只允许挂在叶子技能
- weight 范围 [0, 1]，主关联建议 1.0，次关联 0.3-0.5
- 此操作会替换所有现有关联"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"},"links":{"type":"array","items":{"type":"object","properties":{"attribute_id":{"type":"string","description":"属性 ID"},"weight":{"type":"number","minimum":0,"maximum":1,"description":"权重"}},"required":["attribute_id","weight"]},"description":"关联列表"}},"required":["id","links"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_skill_link_remove",
        description: r#"移除技能的属性关联。

**参数**：
- id (必填): 技能 ID
- attribute_id (必填): 要移除的属性 ID

**返回**：更新后的技能详情

**使用场景**：
- 移除技能与某属性的关联

**注意**：如需调整权重，使用 soloup_skill_link_set 重新设置"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"技能 ID"},"attribute_id":{"type":"string","description":"要移除的属性 ID"}},"required":["id","attribute_id"],"additionalProperties":false}"#,
    },

    // ─── 结算工具 ──────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_recalc",
        description: r#"触发全量补结算到今日 + 重算所有派生值。

**返回**：{ settled: 结算的技能数量 }

**使用场景**：
- 手动触发结算（通常自动结算）
- 修改参数后重新计算所有等级
- 修复数据不一致

**注意**：此操作可能耗时较长（技能多时）"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    // ─── 参数调优工具 ─────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_param_preview",
        description: r#"参数变更影响面分析（只读 dry-run）。

**参数**：
- key (必填): 参数 key（见 soloup_meta 返回的 registry）
- value (必填): 新值

**返回**：
- key: 参数 key
- value: 新值
- applied: false（仅预览）
- affected_skills: 受影响的技能数量
- affected_attributes: 受影响的属性数量

**使用场景**：
- 调优参数前先预览影响面
- 了解参数变更会影响哪些技能/属性

**注意**：此操作不修改任何数据"#,
        input_schema_str: r#"{"type":"object","properties":{"key":{"type":"string","description":"参数 key"},"value":{"type":"number","description":"新值"}},"required":["key","value"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_param_set",
        description: r#"设置参数（覆盖默认值）。

**参数**：
- key (必填): 参数 key（见 soloup_meta 返回的 registry）
- value (必填): 新值（必须在合法范围内）

**返回**：
- key: 参数 key
- value: 新值
- applied: true

**使用场景**：
- AI 自动调优模型参数
- 用户手动调整参数

**注意**：
- key 必须在注册表中，否则返回 ERR_UNKNOWN_PARAM
- value 必须在合法范围内，否则返回 ERR_PARAM_RANGE
- 变更只影响未来结算与新建实体，不追溯历史
- 默认需用户确认（M2 模式）"#,
        input_schema_str: r#"{"type":"object","properties":{"key":{"type":"string","description":"参数 key"},"value":{"type":"number","description":"新值"}},"required":["key","value"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_param_remove",
        description: r#"移除参数覆盖，恢复注册表默认值。

**参数**：
- key (必填): 参数 key

**返回**：
- key: 参数 key
- applied: true

**使用场景**：
- 撤销之前的参数覆盖
- 恢复某参数的默认值"#,
        input_schema_str: r#"{"type":"object","properties":{"key":{"type":"string","description":"参数 key"}},"required":["key"],"additionalProperties":false}"#,
    },

    // ─── 项目 CRUD ──────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_project_list",
        description: r#"列出所有项目（含状态、日期、颜色）。

**返回**：
- projects: 项目列表 [{id, name, description, startDate, endDate, status, color}]

**使用场景**：
- 查看当前所有项目
- 了解项目进度和状态"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_project_create",
        description: r#"创建新项目。

**参数**：
- name (必填): 项目名称
- description: 项目描述
- start_date: 开始日期（默认今天）
- end_date: 结束日期（留空 = 进行中）
- color: 颜色标识

**返回**：
- id: 项目 ID
- name: 项目名称

**使用场景**：
- 用户开始新学习/工作目标
- 创建时间范围，周期内打卡自动归入"#,
        input_schema_str: r#"{"type":"object","properties":{"name":{"type":"string","description":"项目名称"},"description":{"type":"string","description":"项目描述"},"start_date":{"type":"string","description":"开始日期 YYYY-MM-DD"},"end_date":{"type":"string","description":"结束日期 YYYY-MM-DD"},"color":{"type":"string","description":"颜色标识"}},"required":["name"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_project_update",
        description: r#"更新项目信息。

**参数**：
- id (必填): 项目 ID
- name: 新名称
- description: 新描述
- color: 新颜色
- start_date: 新开始日期
- end_date: 新结束日期
- status: 新状态（planned/active/completed/abandoned）

**返回**：
- id: 项目 ID
- name: 项目名称
- status: 新状态

**使用场景**：
- 修改项目信息
- 更改项目状态"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"项目 ID"},"name":{"type":"string","description":"新名称"},"description":{"type":"string","description":"新描述"},"color":{"type":"string","description":"新颜色"},"start_date":{"type":"string","description":"新开始日期"},"end_date":{"type":"string","description":"新结束日期"},"status":{"type":"string","enum":["planned","active","completed","abandoned"],"description":"新状态"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_project_delete",
        description: r#"删除项目（物理删除，不可恢复）。

**参数**：
- id (必填): 项目 ID

**返回**：
- id: 项目 ID
- deleted: true

**使用场景**：
- 删除测试/无效项目
- 清理旧项目数据

**注意**：删除前建议先确认，此操作不可撤销"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"项目 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    // ─── 打卡操作 ─────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_checkin_set",
        description: r#"记录某日打卡（触发技能结算）。

**参数**：
- date: 日期（默认今天）
- skill_ids (必填): 技能 ID 列表
- note: 备注
- project_id: 关联项目 ID

**返回**：
- ok: true
- effects: 每个技能的前后变化 [{skillId, before, after}]

**使用场景**：
- AI 代用户记录每日活动
- 批量补打卡

**注意**：默认需用户确认（M2 模式）"#,
        input_schema_str: r#"{"type":"object","properties":{"date":{"type":"string","description":"日期 YYYY-MM-DD"},"skill_ids":{"type":"array","items":{"type":"string"},"description":"技能 ID 列表"},"note":{"type":"string","description":"备注"},"project_id":{"type":"string","description":"关联项目 ID"}},"required":["skill_ids"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_checkin_clear",
        description: r#"清除某日打卡并重新结算。

**参数**：
- date: 日期（默认今天）

**返回**：
- ok: true

**使用场景**：
- 撤销某日打卡
- 修正打卡记录"#,
        input_schema_str: r#"{"type":"object","properties":{"date":{"type":"string","description":"日期 YYYY-MM-DD"}},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_records_list",
        description: r#"查询历史打卡记录。

**参数**：
- date: 筛选日期（可选）
- limit: 返回条数（默认 120）

**返回**：
- records: 记录列表 [{date, skillIds, attrsLit, projectId, note}]

**使用场景**：
- 查看历史打卡
- 分析打卡趋势"#,
        input_schema_str: r#"{"type":"object","properties":{"date":{"type":"string","description":"筛选日期 YYYY-MM-DD"},"limit":{"type":"integer","description":"返回条数"}},"additionalProperties":false}"#,
    },

    // ─── 成就 CRUD ──────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_achievement_list",
        description: r#"列出所有成就（含解锁状态）。

**返回**：
- achievements: 成就列表 [{id, name, description, type, rarity, points, condition, hidden, requires, reveal_at, unlocked}]

**使用场景**：
- 查看成就进度
- 分析解锁条件
- 查看依赖链和渐进揭示配置"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_achievement_create",
        description: r#"创建自定义成就。

**参数**：
- name (必填): 成就名称
- description: 成就描述
- type: 类型（skill/attribute/project/milestone）
- rarity: 稀有度（common/rare/epic/legendary）
- points: 积分值（默认 10）
- condition: 解锁条件 JSON（简化 DSL）
- hidden: 是否隐藏成就（默认 false）
- requires: 前置成就 ID 列表（全部解锁后该卡才开始显现）
- reveal_at: 渐进揭示阈值 0–1（隐藏卡进度达到此比例时显示名称和描述）

**条件 DSL 示例**：
```json
{"type":"stat_threshold","stat":"totalDays","operator":">=","value":100}
```

**成就设计规则**：
1. **隐藏成就**（hidden=true）：卡牌初始不可见，满足前置条件后逐步显现
2. **依赖链**（requires）：卡牌可要求前置成就全部解锁后才开始显现
3. **渐进揭示**（revealAt）：0–1 的阈值，当进度达到该比例时：
   - 先显示名称（revealAt 阈值）
   - 再显示描述（接近达成时）
4. **卡牌状态流转**：
   - deep-locked（前置未满足）→ 完全不可见
   - name-only（达到 revealAt）→ 仅显示名称
   - name+desc（接近达成）→ 显示名称和描述
   - unlocked（达成条件）→ 完全解锁

**设计示例**（初次觉醒→双线并进→六艺俱全）：
```json
{"name":"初次觉醒","hidden":true,"revealAt":0.5,"requires":[]}
{"name":"双线并进","hidden":true,"revealAt":0.5,"requires":["first"]}
{"name":"六艺俱全","hidden":true,"revealAt":0.4,"requires":["twin"]}
```

**返回**：
- id: 成就 ID
- name: 成就名称

**使用场景**：
- 创建个性化成就
- 构建成就依赖链
- 设计渐进揭示体验"#,
        input_schema_str: r#"{"type":"object","properties":{"name":{"type":"string","description":"成就名称"},"description":{"type":"string","description":"成就描述"},"type":{"type":"string","enum":["skill","attribute","project","milestone"],"description":"类型"},"rarity":{"type":"string","enum":["common","rare","epic","legendary"],"description":"稀有度"},"points":{"type":"integer","description":"积分值"},"condition":{"type":"object","description":"解锁条件 JSON"},"hidden":{"type":"boolean","description":"是否隐藏成就"},"requires":{"type":"array","items":{"type":"string"},"description":"前置成就 ID 列表"},"reveal_at":{"type":"number","description":"渐进揭示阈值 0–1"}},"required":["name"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_achievement_update",
        description: r#"更新成就信息。

**参数**：
- id (必填): 成就 ID
- name: 新名称
- description: 新描述
- type: 新类型
- rarity: 新稀有度
- points: 新积分值
- condition: 新解锁条件
- hidden: 是否隐藏成就
- requires: 前置成就 ID 列表
- reveal_at: 渐进揭示阈值 0–1

**成就设计规则**：
1. **隐藏成就**（hidden=true）：卡牌初始不可见，满足前置条件后逐步显现
2. **依赖链**（requires）：卡牌可要求前置成就全部解锁后才开始显现
3. **渐进揭示**（reveal_at）：0–1 的阈值，当进度达到该比例时显示名称和描述
4. **卡牌状态流转**：deep-locked → name-only → name+desc → unlocked

**返回**：
- id: 成就 ID
- name: 成就名称

**使用场景**：
- 修改成就内容
- 调整解锁条件
- 修改依赖链或渐进揭示阈值"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"成就 ID"},"name":{"type":"string","description":"新名称"},"description":{"type":"string","description":"新描述"},"type":{"type":"string","enum":["skill","attribute","project","milestone"],"description":"新类型"},"rarity":{"type":"string","enum":["common","rare","epic","legendary"],"description":"新稀有度"},"points":{"type":"integer","description":"新积分值"},"condition":{"type":"object","description":"新解锁条件"},"hidden":{"type":"boolean","description":"是否隐藏成就"},"requires":{"type":"array","items":{"type":"string"},"description":"前置成就 ID 列表"},"reveal_at":{"type":"number","description":"渐进揭示阈值 0–1"}},"required":["id"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_achievement_delete",
        description: r#"删除成就（物理删除）。

**参数**：
- id (必填): 成就 ID

**返回**：
- id: 成就 ID
- deleted: true

**使用场景**：
- 删除无效成就
- 清理测试数据"#,
        input_schema_str: r#"{"type":"object","properties":{"id":{"type":"string","description":"成就 ID"}},"required":["id"],"additionalProperties":false}"#,
    },

    // ─── 个人设置 ──────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_life_view",
        description: r#"查看生命轴设置。

**返回**：
- born: 出生日期
- life_expectancy: 预期寿命
- pct: 已度过百分比

**使用场景**：
- 查看生命进度
- 了解人生阶段"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_life_save",
        description: r#"保存生命轴设置。

**参数**：
- born (必填): 出生日期（YYYY-MM-DD）
- life_expectancy: 预期寿命（默认 120）

**返回**：
- ok: true

**使用场景**：
- 设置/修改出生日期
- 调整预期寿命"#,
        input_schema_str: r#"{"type":"object","properties":{"born":{"type":"string","description":"出生日期 YYYY-MM-DD"},"life_expectancy":{"type":"number","description":"预期寿命（年）"}},"required":["born"],"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_profile_load",
        description: r#"加载用户配置。

**返回**：
- nickname: 昵称
- avatar: 头像 URL
- remind_time: 提醒时间
- motion: 动画开关
- lock_history: 历史锁定

**使用场景**：
- 读取用户偏好设置"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },

    ToolDef {
        name: "soloup_profile_save",
        description: r#"保存用户配置。

**参数**：
- nickname: 昵称
- avatar: 头像 URL
- remind_time: 提醒时间
- motion: 动画开关
- lock_history: 历史锁定

**返回**：
- ok: true

**使用场景**：
- 更新用户偏好"#,
        input_schema_str: r#"{"type":"object","properties":{"nickname":{"type":"string","description":"昵称"},"avatar":{"type":"string","description":"头像 URL"},"remind_time":{"type":"string","description":"提醒时间 HH:MM"},"motion":{"type":"boolean","description":"动画开关"},"lock_history":{"type":"boolean","description":"历史锁定"}},"additionalProperties":false}"#,
    },

    // ─── 统计查询 ──────────────────────────────────────────────────────────
    ToolDef {
        name: "soloup_stats",
        description: r#"查询统计数据。

**返回**：
- totalDays: 总打卡天数
- streak: 当前连续天数
- weekDays: 本周打卡天数
- maxLitOneDay: 单日最多点亮属性数
- maxSkillLv: 最高技能等级
- topSkill: 最高技能信息
- totalAttrs: 属性总数
- projectsCompleted: 已完成项目数

**使用场景**：
- 查看成长统计
- 分析打卡趋势
- 成就解锁判断"#,
        input_schema_str: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
    },
];

// ── 工具调用分发 ────────────────────────────────────────────────────────────

/// 调用 MCP 工具，返回 JSON 结果
pub fn call_tool(state: &AppState, name: &str, args: &Value) -> Result<Value, ToolError> {
    let mut solver = state.lock().unwrap();

    // 所有 MCP 请求的 actor 默认为 ai
    let args_with_actor = if args.is_object() {
        let mut m = args.clone();
        if !m.as_object().unwrap().contains_key("actor") {
            m["actor"] = json!("ai");
        }
        m
    } else {
        args.clone()
    };

    let op = match name {
        "soloup_meta" => "meta",
        "soloup_panel_overview" => "bootstrap",
        "soloup_attribute_list" => "attributes",
        "soloup_attribute_get" => "attribute.get",
        "soloup_attribute_create" => "attribute.create",
        "soloup_attribute_update" => "attribute.update",
        "soloup_attribute_delete" => "attribute.delete",
        "soloup_skill_tree" => "tree",
        "soloup_skill_get" => "skillDetail",
        "soloup_skill_create" => "skill.create",
        "soloup_skill_update" => "skill.update",
        "soloup_skill_move" => "skill.move",
        "soloup_skill_archive" => "skill.archive",
        "soloup_skill_restore" => "skill.restore",
        "soloup_skill_delete" => "skill.delete",
        "soloup_skill_link_set" => "skill.linkSet",
        "soloup_skill_link_remove" => "skill.linkRemove",
        "soloup_daily_status" => "daily.status",
        "soloup_audit_log" => "audit",
        "soloup_settings_get" => "settings",
        "soloup_insights" => "insights",
        "soloup_recalc" => "recalc",
        "soloup_param_preview" => "param.preview",
        "soloup_param_set" => "param.set",
        "soloup_param_remove" => "param.remove",

        // ── 项目 CRUD ──
        "soloup_project_list" => "projects.list",
        "soloup_project_create" => "project.create",
        "soloup_project_update" => "project.update",
        "soloup_project_delete" => "project.delete",

        // ── 打卡操作 ──
        "soloup_checkin_set" => "checkin",
        "soloup_checkin_clear" => "checkin.clear",
        "soloup_records_list" => "records.list",

        // ── 成就 CRUD ──
        "soloup_achievement_list" => "achievements",
        "soloup_achievement_create" => "achievement.create",
        "soloup_achievement_update" => "achievement.update",
        "soloup_achievement_delete" => "achievement.delete",

        // ── 个人设置 ──
        "soloup_life_view" => "life.view",
        "soloup_life_save" => "life.save",
        "soloup_profile_load" => "profile.load",
        "soloup_profile_save" => "profile.save",

        // ── 统计 ──
        "soloup_stats" => "stats",

        _ => {
            return Err(ToolError {
                code: "ERR_UNKNOWN_TOOL".into(),
                message: format!("未知工具：{}", name),
            });
        }
    };

    // 特殊处理：attribute.get 和 skill.linkRemove 需要额外逻辑
    match name {
        "soloup_attribute_get" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| ToolError {
                code: "ERR_VALIDATION".into(),
                message: "缺少参数 id".into(),
            })?;
            let snap = solver.snapshot(None).map_err(|e| ToolError::from_solver(e))?;
            let derived = snap.attributes.iter().find(|a| a.attribute.id == id).ok_or_else(|| ToolError {
                code: "ERR_NOT_FOUND".into(),
                message: format!("属性不存在：{}", id),
            })?;
            let links = solver.store.links().list_for_attribute(&id).map_err(|e| ToolError::from_store(e))?;
            Ok(json!({
                "id": derived.attribute.id,
                "name": derived.attribute.name,
                "description": derived.attribute.description,
                "category": derived.attribute.category,
                "color": derived.attribute.color,
                "value": derived.value,
                "x": derived.x,
                "linkedSkills": links.iter().map(|l| json!({
                    "skillId": l.skill_id,
                    "weight": l.weight
                })).collect::<Vec<_>>(),
                "contributions": derived.entries.iter().map(|e| json!({
                    "skill_id": e.skill_id, "weight": e.weight, "level": e.level, "share": e.share
                })).collect::<Vec<_>>()
            }))
        }
        "soloup_skill_link_remove" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| ToolError {
                code: "ERR_VALIDATION".into(),
                message: "缺少参数 id".into(),
            })?;
            let attribute_id = args.get("attribute_id").and_then(|v| v.as_str()).ok_or_else(|| ToolError {
                code: "ERR_VALIDATION".into(),
                message: "缺少参数 attribute_id".into(),
            })?;
            // 获取现有 links，移除目标，重新设置
            let existing = solver.store.links().list_for_skill(&id).map_err(|e| ToolError::from_store(e))?;
            let new_links: Vec<soloup_store::links::SkillLinkInput> = existing
                .into_iter()
                .filter(|l| l.attribute_id != attribute_id)
                .map(|l| soloup_store::links::SkillLinkInput {
                    attribute_id: l.attribute_id,
                    weight: l.weight,
                })
                .collect();
            let res = solver.mutate(
                soloup_solver::service::MutationKind::SkillLinkSet {
                    id: id.to_string(),
                    links: new_links,
                },
                soloup_core::schema::AuditActor::Ai,
            ).map_err(|e| ToolError::from_solver(e))?;
            Ok(res.detail)
        }
        "soloup_daily_status" => {
            let date = args.get("date").and_then(|v| v.as_str()).unwrap_or(&soloup_core::dates::today_iso()).to_string();
            let recs = solver.store.daily().list_records(None, None).map_err(|e| ToolError::from_store(e))?;
            let rec = recs.into_iter().find(|r| r.date == date);
            match rec {
                Some(r) => {
                    let ids = solver.store.daily().skill_ids_on(&date).map_err(|e| ToolError::from_store(e))?;
                    Ok(json!({
                        "date": r.date,
                        "skillIds": ids,
                        "projectId": r.project_id,
                        "note": r.note
                    }))
                }
                None => Ok(json!({
                    "date": date,
                    "skillIds": [],
                    "projectId": null,
                    "note": null,
                    "checked": false
                })),
            }
        }
        "soloup_insights" => {
            // 分析建议：弱项属性 + 推荐技能
            let snap = solver.snapshot(None).map_err(|e| ToolError::from_solver(e))?;
            let mut sorted_attrs: Vec<_> = snap.attributes.iter().collect();
            sorted_attrs.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal));
            let weak_attrs: Vec<_> = sorted_attrs.iter().take(2).map(|a| json!({
                "id": a.attribute.id,
                "name": a.attribute.name,
                "value": a.value
            })).collect();

            // 推荐技能：基于弱项属性关联
            let skills = solver.store.skills().list_all().map_err(|e| ToolError::from_store(e))?;
            let mut recommended = Vec::new();
            for attr in &sorted_attrs[..2.min(sorted_attrs.len())] {
                let links = solver.store.links().list_for_attribute(&attr.attribute.id).map_err(|e| ToolError::from_store(e))?;
                for link in links {
                    if let Some(skill) = skills.iter().find(|s| s.id == link.skill_id) {
                        recommended.push(json!({
                            "skillId": skill.id,
                            "skillName": skill.name,
                            "attributeId": attr.attribute.id,
                            "attributeName": attr.attribute.name,
                            "weight": link.weight
                        }));
                    }
                }
            }

            Ok(json!({
                "weakAttributes": weak_attrs,
                "recommendedSkills": recommended,
                "suggestions": format!("建议优先提升 {} 相关技能", sorted_attrs.first().map(|a| a.attribute.name.as_str()).unwrap_or("弱项"))
            }))
        }
        "soloup_project_create" => {
            let mut a = args_with_actor.clone();
            if let Some(v) = a.get("start_date").cloned() { a["startDate"] = v; }
            if let Some(v) = a.get("end_date").cloned() { a["endDate"] = v; }
            dispatch(&mut *solver, op, &a).map_err(|e| ToolError { code: e.code, message: e.message })
        }
        "soloup_project_update" => {
            let mut a = args_with_actor.clone();
            if let Some(v) = a.get("start_date").cloned() { a["startDate"] = v; }
            if let Some(v) = a.get("end_date").cloned() { a["endDate"] = v; }
            dispatch(&mut *solver, op, &a).map_err(|e| ToolError { code: e.code, message: e.message })
        }
        "soloup_checkin_set" => {
            let mut a = args_with_actor.clone();
            if let Some(v) = a.get("skill_ids").cloned() { a["skillIds"] = v; }
            if let Some(v) = a.get("project_id").cloned() { a["projectId"] = v; }
            dispatch(&mut *solver, op, &a).map_err(|e| ToolError { code: e.code, message: e.message })
        }
        "soloup_life_save" => {
            let mut a = args_with_actor.clone();
            if let Some(v) = a.get("life_expectancy").cloned() { a["expectancy"] = v; }
            dispatch(&mut *solver, op, &a).map_err(|e| ToolError { code: e.code, message: e.message })
        }
        "soloup_profile_save" => {
            let mut a = args_with_actor.clone();
            if let Some(v) = a.get("remind_time").cloned() { a["remind"] = v; }
            if let Some(v) = a.get("lock_history").cloned() { a["lockHistory"] = v; }
            dispatch(&mut *solver, op, &a).map_err(|e| ToolError { code: e.code, message: e.message })
        }
        _ => {
            dispatch(&mut *solver, op, &args_with_actor).map_err(|e| ToolError {
                code: e.code,
                message: e.message,
            })
        }
    }
}

// ─── 错误类型 ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct ToolError {
    pub code: String,
    pub message: String,
}

impl ToolError {
    fn from_store(e: soloup_store::StoreError) -> Self {
        ToolError {
            code: e.code.as_str().to_string(),
            message: e.message,
        }
    }

    fn from_solver(e: soloup_solver::SolverError) -> Self {
        ToolError {
            code: e.code.as_str().to_string(),
            message: e.message,
        }
    }
}
