//! 默认参数表 —— 单一来源（附录 A / §3.4 / §3.7.3 / §2.x 默认层）。
//! 对应 TS `@soloup/core/params.ts`。

use crate::{Difficulty, SkillCategory};

/// 等级上限（§3.7 值域 [0,100)）。
pub const LEVEL_CAP: f64 = 100.0;

/// 难度系数（§3.7.3）：作用于 λ（saturated）或 x₀（sigmoid），k 不变。
pub fn difficulty_multiplier(d: Difficulty) -> f64 {
    match d {
        Difficulty::Casual => 0.5,
        Difficulty::Normal => 1.0,
        Difficulty::Hard => 2.0,
        Difficulty::Challenge => 4.0,
        Difficulty::Legendary => 8.0,
    }
}

pub const DEFAULT_DIFFICULTY: Difficulty = Difficulty::Normal;

/// 技能类别账户参数（附录 A：c 结晶率 / f 遗忘率）。
pub fn category_account_params(cat: SkillCategory) -> (f64, f64) {
    match cat {
        SkillCategory::Physical => (0.015, 0.004),
        SkillCategory::Cognitive => (0.01, 0.008),
        SkillCategory::Knowledge => (0.006, 0.014),
    }
}

/// 流失率 d = c + f（§3.4）。
pub fn loss_rate(cat: SkillCategory) -> f64 {
    let (c, f) = category_account_params(cat);
    c + f
}

/// 类别 → 默认曲线类型（§3.7：physical→saturated，cognitive/knowledge→sigmoid）。
pub fn category_curve_type(cat: SkillCategory) -> crate::CurveType {
    match cat {
        SkillCategory::Physical => crate::CurveType::Saturated,
        SkillCategory::Cognitive | SkillCategory::Knowledge => crate::CurveType::Sigmoid,
    }
}

/// 指数饱和默认 λ（§3.7.1，physical λ=480）。
pub const SATURATED_DEFAULT_LAMBDA: f64 = 480.0;

/// Sigmoid 曲线默认参数（§3.7.2）。
#[derive(Debug, Clone, Copy)]
pub struct SigmoidParams {
    pub k: f64,
    pub x0: f64,
}

pub fn sigmoid_default_params(cat: SkillCategory) -> SigmoidParams {
    match cat {
        SkillCategory::Cognitive => SigmoidParams { k: 0.006, x0: 220.0 },
        SkillCategory::Knowledge => SigmoidParams { k: 0.01, x0: 175.0 },
        // physical 用 saturated，不会走 sigmoid 默认；给个兜底。
        SkillCategory::Physical => SigmoidParams { k: 0.006, x0: 220.0 },
    }
}

/// 属性派生默认参数（§3.9 / 附录 A）。
#[derive(Debug, Clone, Copy)]
pub struct AttributeDefaults {
    pub base_value: f64,
    pub max_value: f64,
    pub alpha: f64,
    pub w0: f64,
}

pub const ATTRIBUTE_DEFAULTS: AttributeDefaults = AttributeDefaults {
    base_value: 0.0,
    max_value: 100.0,
    alpha: 1.2,
    w0: 400.0,
};

/// 技能树父级聚合 α（§3.8，与属性派生一致）。
pub const AGGREGATE_ALPHA: f64 = 1.2;

/// 生命轴默认期望寿命（§2.8，年）。
pub const LIFE_DEFAULT_EXPECTANCY: f64 = 120.0;

/// 关联权重语义（§2.4）。
pub const LINK_WEIGHT_MIN: f64 = 0.0;
pub const LINK_WEIGHT_MAX: f64 = 1.0;
pub const LINK_PRIMARY_WEIGHT: f64 = 1.0;
pub const LINK_SECONDARY_WEIGHT_MIN: f64 = 0.3;
pub const LINK_SECONDARY_WEIGHT_MAX: f64 = 0.5;

/// 单次补结算防呆上限（§3.5：3650 天）。
pub const CATCH_UP_DAY_CAP: i64 = 3650;
