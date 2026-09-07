//! §3.7 成长曲线（双轨制）—— 等级 ℓ(E) 的纯函数。
//! 对应 TS `@soloup/core/curve.ts`。值域 [0,100)。

use crate::params::*;
use crate::{CurveType, Difficulty, SkillCategory};

/// 解析后的有效曲线参数（难度已并入 λ/x₀）。
#[derive(Debug, Clone, Copy)]
pub struct ResolvedCurve {
    pub r#type: CurveType,
    pub lambda: Option<f64>,
    pub k: Option<f64>,
    pub x0: Option<f64>,
}

fn sigma(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// §3.7.1 指数饱和：ℓ(E)=100(1−e^(−E/λ))。
pub fn saturated_level(e: f64, lambda: f64) -> f64 {
    LEVEL_CAP * (1.0 - (-e / lambda).exp())
}

/// §3.7.2 归一化 Sigmoid（零起点基线 s₀ 保证 E=0 时严格为 0）。
pub fn sigmoid_level(e: f64, k: f64, x0: f64) -> f64 {
    let s0 = sigma(-k * x0);
    let se = sigma(k * (e - x0));
    LEVEL_CAP * ((se - s0) / (1.0 - s0))
}

/// 曲线类型 + 难度 → 有效参数（§3.7.3：D 乘到 λ 或 x₀）。
pub fn resolve_curve(
    category: SkillCategory,
    override_type: Option<CurveType>,
    difficulty: Difficulty,
) -> ResolvedCurve {
    let r#type = override_type.unwrap_or_else(|| category_curve_type(category));
    let d = difficulty_multiplier(difficulty);
    match r#type {
        CurveType::Saturated => ResolvedCurve {
            r#type,
            lambda: Some(SATURATED_DEFAULT_LAMBDA * d),
            k: None,
            x0: None,
        },
        CurveType::Sigmoid => {
            let base = if category == SkillCategory::Knowledge {
                sigmoid_default_params(SkillCategory::Knowledge)
            } else {
                sigmoid_default_params(SkillCategory::Cognitive)
            };
            ResolvedCurve {
                r#type,
                lambda: None,
                k: Some(base.k),
                x0: Some(base.x0 * d),
            }
        }
    }
}

/// E → 等级（按已解析曲线）。
pub fn level_of(e: f64, curve: &ResolvedCurve) -> f64 {
    if e <= 0.0 {
        return 0.0;
    }
    match curve.r#type {
        CurveType::Saturated => saturated_level(e, curve.lambda.unwrap_or(SATURATED_DEFAULT_LAMBDA)),
        CurveType::Sigmoid => sigmoid_level(
            e,
            curve.k.unwrap_or_else(|| sigmoid_default_params(SkillCategory::Cognitive).k),
            curve.x0.unwrap_or(0.0),
        ),
    }
}

/// 便捷入口：类别（+可选覆盖与难度）→ 等级。
pub fn level_from_category(
    e: f64,
    category: SkillCategory,
    difficulty: Difficulty,
    override_type: Option<CurveType>,
) -> f64 {
    level_of(e, &resolve_curve(category, override_type, difficulty))
}

/// 半程点 E₅₀（达到 50 级所需 E）。
pub fn half_way_point(curve: &ResolvedCurve) -> f64 {
    match curve.r#type {
        CurveType::Saturated => curve.lambda.unwrap_or(SATURATED_DEFAULT_LAMBDA) * std::f64::consts::LN_2,
        CurveType::Sigmoid => {
            let k = curve.k.unwrap_or_else(|| sigmoid_default_params(SkillCategory::Cognitive).k);
            let x0 = curve.x0.unwrap_or(0.0);
            let s0 = sigma(-k * x0);
            x0 + (1.0 / k) * ((1.0 + s0) / (1.0 - s0)).ln()
        }
    }
}
