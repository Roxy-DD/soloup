//! soloup-core —— 领域纯函数（零 IO）。
//! 对应 TS `@soloup/core`：数学引擎（结算/曲线/聚合/派生）、默认参数表、参数注册表、
//! ISO 日期工具、黄金表 fixture。依赖边界：仅 serde（序列化）+ chrono（本地日期）。

pub mod aggregate;
pub mod curve;
pub mod dates;
pub mod goldens;
pub mod params;
pub mod registry;
pub mod schema;
pub mod settle;

use serde::{Deserialize, Serialize};

/// 技能类别（§2.3，决定默认曲线与账户参数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Physical,
    Cognitive,
    Knowledge,
}

/// 难度档位（§3.7.3，D 乘到 λ 或 x₀）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Casual,
    Normal,
    Hard,
    Challenge,
    Legendary,
}

/// 曲线类型（§3.7 双轨制）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveType {
    Saturated,
    Sigmoid,
}

impl SkillCategory {
    pub const ALL: [SkillCategory; 3] = [
        SkillCategory::Physical,
        SkillCategory::Cognitive,
        SkillCategory::Knowledge,
    ];
}

impl Difficulty {
    pub const ALL: [Difficulty; 5] = [
        Difficulty::Casual,
        Difficulty::Normal,
        Difficulty::Hard,
        Difficulty::Challenge,
        Difficulty::Legendary,
    ];
}
