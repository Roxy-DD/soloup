//! §4.3 领域行结构 —— 与 SQLite 表列对齐（对应 TS `@soloup/core/schemas.ts`）。
//! 派生缓存（level / effective_exposure / current_value）不入库、不出现在此处。

use serde::{Deserialize, Serialize};

pub use crate::{CurveType, Difficulty, SkillCategory};

/* ------------------------------ 枚举 ------------------------------ */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeCategory {
    Physical,
    Mental,
    Social,
    Creative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Planned,
    Active,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievementType {
    Skill,
    Attribute,
    Project,
    Milestone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActor {
    User,
    Ai,
    System,
}

/// 各枚举的 snake_case 字符串表示 + 解析（store 层在 SQLite 文本列与 enum 之间转换用）。
pub trait DbEnum: Sized {
    fn as_str(&self) -> &'static str;
    fn parse(s: &str) -> Option<Self>;
}

macro_rules! db_enum {
    ($ty:ident, { $($name:ident => $s:literal),+ $(,)? }) => {
        impl DbEnum for $ty {
            fn as_str(&self) -> &'static str {
                match self { $($ty::$name => $s),+ }
            }
            fn parse(s: &str) -> Option<Self> {
                match s { $($s => Some($ty::$name)),+ , _ => None }
            }
        }
    };
}

db_enum!(SkillCategory, { Physical => "physical", Cognitive => "cognitive", Knowledge => "knowledge" });
db_enum!(Difficulty, { Casual => "casual", Normal => "normal", Hard => "hard", Challenge => "challenge", Legendary => "legendary" });
db_enum!(CurveType, { Saturated => "saturated", Sigmoid => "sigmoid" });
db_enum!(AttributeCategory, { Physical => "physical", Mental => "mental", Social => "social", Creative => "creative" });
db_enum!(ProjectStatus, { Planned => "planned", Active => "active", Completed => "completed", Abandoned => "abandoned" });
db_enum!(AchievementType, { Skill => "skill", Attribute => "attribute", Project => "project", Milestone => "milestone" });
db_enum!(Rarity, { Common => "common", Rare => "rare", Epic => "epic", Legendary => "legendary" });
db_enum!(AuditActor, { User => "user", Ai => "ai", System => "system" });

/* ------------------------------ 行结构 ------------------------------ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub base_value: f64,
    pub max_value: f64,
    pub alpha: f64,
    pub w0: f64,
    pub category: Option<AttributeCategory>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub category: SkillCategory,
    pub difficulty: Difficulty,
    /// NULL = 按 category 推导；可覆盖。
    pub curve_type: Option<CurveType>,
    /// 仅结算产生（§2.3-3）。
    pub c: f64,
    pub v: f64,
    pub last_settled_date: Option<String>,
    pub created_at: String,
    pub archived_at: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort: i64,
}

/// 仅叶子技能可有关联/可被打卡（§2.3-1 / §2.4）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAttributeLink {
    pub skill_id: String,
    pub attribute_id: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRecord {
    pub date: String,
    pub project_id: Option<String>,
    pub note: Option<String>,
    /// 仅提示字段（§4.3 附注）。
    pub settled: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRecordSkill {
    pub date: String,
    pub skill_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: ProjectStatus,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// §5 规则 DSL（v1.1 启用）；存储为 JSON 文本。
    pub condition_json: Option<serde_json::Value>,
    pub r#type: AchievementType,
    pub rarity: Rarity,
    pub points: i64,
    pub unlocked_at: Option<String>,
    /// 隐藏成就：未解锁时前端渐进揭示而非直接展示
    pub hidden: bool,
    /// 前置成就 id 列表（JSON 数组）：全部解锁后该卡才开始显现
    pub requires: Vec<String>,
    /// 渐进揭示阈值（0.0–1.0）：进度达到此比例时显示名称和提示
    pub reveal_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub at: String,
    pub actor: AuditActor,
    pub tool: Option<String>,
    pub params_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsRow {
    pub key: String,
    pub value: String,
}
