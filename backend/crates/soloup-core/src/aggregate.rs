//! §3.4 稳态恒等式 / §3.8 技能树聚合 / §3.9 属性派生 —— 纯函数。
//! 对应 TS `@soloup/core/aggregate.ts`。

#[derive(Debug, Clone, Copy)]
pub struct WeightedLevel {
    pub level: f64,
    pub weight: f64,
}

/// §3.8 父级 level：ℓ_parent = (Σ wᵢ·ℓᵢ^α / Σ wᵢ)^(1/α)。Σw ≤ 0 返回 0。
pub fn parent_level(children: &[WeightedLevel], alpha: f64) -> f64 {
    let sum_w: f64 = children.iter().map(|c| c.weight).sum();
    if !(sum_w > 0.0) {
        return 0.0;
    }
    let sum_wl: f64 = children
        .iter()
        .map(|c| c.weight * c.level.powf(alpha))
        .sum();
    (sum_wl / sum_w).powf(1.0 / alpha)
}

/// 权重合计（供调用方判空）。
pub fn total_weight(children: &[WeightedLevel]) -> f64 {
    children.iter().map(|c| c.weight).sum()
}

#[derive(Debug, Clone, Copy)]
pub struct AttributeParams {
    pub base_value: f64,
    pub max_value: f64,
    pub alpha: f64,
    pub w0: f64,
}

/// §3.9 属性派生：X = Σ wᵢℓᵢ^α；A = B + S(1 − e^(−X/W))。
pub fn attribute_value(p: &AttributeParams, entries: &[WeightedLevel]) -> f64 {
    let x: f64 = entries
        .iter()
        .map(|e| e.weight * e.level.powf(p.alpha))
        .sum();
    if !(x > 0.0) {
        return p.base_value;
    }
    p.base_value + p.max_value * (1.0 - (-x / p.w0).exp())
}

/// 属性从 base 起可达到的增量上限（B+S）。
pub fn attribute_ceiling(p: &AttributeParams) -> f64 {
    p.base_value + p.max_value
}

/// §3.4 V 的稳态值 V_eq = (1−d)/d。
pub fn steady_state_v(d: f64) -> f64 {
    if !(d > 0.0) || !(d < 1.0) {
        panic!("steadyStateV: d 须在 (0,1)，收到 {d}");
    }
    (1.0 - d) / d
}

/// §3.4 稳态下每日净结晶 ΔC/day = c/d。
pub fn steady_day_gain(c: f64, d: f64) -> f64 {
    c / d
}

/// §3.4 活性半衰期 t½ = ln2 / −ln(1−d)。
pub fn active_half_life_days(d: f64) -> f64 {
    if !(d > 0.0) || !(d < 1.0) {
        panic!("activeHalfLifeDays: d 须在 (0,1)，收到 {d}");
    }
    std::f64::consts::LN_2 / -(1.0 - d).ln()
}
