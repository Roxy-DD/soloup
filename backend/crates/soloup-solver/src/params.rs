//! 生效参数解析（技能行 + 全局 param_overrides → c/f 与曲线）与参数时间线。
//! 对应 TS `@soloup/solver/params.ts`。

use soloup_core::curve::{resolve_curve, ResolvedCurve};
use soloup_core::params::category_account_params;
use soloup_core::registry::ParamOverrides;
use soloup_core::schema::{AuditLog, CurveType, DbEnum, Difficulty, Skill, SkillCategory};
use soloup_core::settle::SettleParams;

/* ---------------------------- 生效参数 ---------------------------- */

pub type EffectiveAccounts = SettleParams;

#[derive(Debug, Clone, Copy)]
pub struct EffectiveSkillParams {
    pub accounts: EffectiveAccounts,
    pub curve: ResolvedCurve,
}

/// §2.9 类别账户参数 + 覆盖（category.<c>.c/f）。
pub fn account_params_for(category: SkillCategory, overrides: &ParamOverrides) -> EffectiveAccounts {
    let (base_c, base_f) = category_account_params(category);
    let o = overrides.category.as_ref().and_then(|m| m.get(&category));
    EffectiveAccounts {
        c: o.and_then(|r| r.c).unwrap_or(base_c),
        f: o.and_then(|r| r.f).unwrap_or(base_f),
    }
}

/// 曲线参数：难度乘数并入基线后，再被 curve.<c>.{k,x0,lambda} 覆盖。
pub fn curve_params_for(
    category: SkillCategory,
    difficulty: Difficulty,
    curve_override: Option<CurveType>,
    overrides: &ParamOverrides,
) -> ResolvedCurve {
    let mut base = resolve_curve(category, curve_override, difficulty);
    let o = overrides.curve.as_ref().and_then(|m| m.get(&category));
    if let Some(o) = o {
        if base.r#type == CurveType::Saturated {
            if let Some(lambda) = o.lambda {
                base.lambda = Some(lambda);
            }
        } else {
            if let Some(k) = o.k {
                base.k = Some(k);
            }
            if let Some(x0) = o.x0 {
                base.x0 = Some(x0);
            }
        }
    }
    base
}

/// 技能行 → 结算用生效参数（账户 + 等级曲线）。
pub fn effective_skill_params(
    skill: &Skill,
    overrides: &ParamOverrides,
) -> EffectiveSkillParams {
    EffectiveSkillParams {
        accounts: account_params_for(skill.category, overrides),
        curve: curve_params_for(skill.category, skill.difficulty, skill.curve_type, overrides),
    }
}

fn effective_accounts_eq(a: &EffectiveAccounts, b: &EffectiveAccounts) -> bool {
    a.c == b.c && a.f == b.f
}

fn same_params(a: &EffectiveSkillParams, b: &EffectiveSkillParams) -> bool {
    if !effective_accounts_eq(&a.accounts, &b.accounts) {
        return false;
    }
    if a.curve.r#type != b.curve.r#type {
        return false;
    }
    match a.curve.r#type {
        CurveType::Saturated => a.curve.lambda == b.curve.lambda,
        CurveType::Sigmoid => a.curve.k == b.curve.k && a.curve.x0 == b.curve.x0,
    }
}

/* ---------------------------- 参数时间线 ---------------------------- */

#[derive(Debug, Clone, Copy)]
pub struct ProfileSnapshot {
    pub category: SkillCategory,
    pub difficulty: Difficulty,
    pub curve_type: Option<CurveType>,
}

#[derive(Debug, Clone)]
pub struct SkillParamChange {
    pub effective_at: String,
    pub category: Option<(SkillCategory, SkillCategory)>,
    pub difficulty: Option<(Difficulty, Difficulty)>,
    pub curve_type: Option<(Option<CurveType>, Option<CurveType>)>,
}

#[derive(Debug, Clone)]
pub struct TimelineSegment {
    pub from: String,
    pub params: EffectiveSkillParams,
}

/// 由「当前技能行参数」沿变更事件倒推各历史段（§C.6）。全局 overrides 视为恒定。
pub fn build_param_timeline(
    created_at: &str,
    current: ProfileSnapshot,
    overrides: &ParamOverrides,
    changes: &[SkillParamChange],
) -> Vec<TimelineSegment> {
    let mut sorted: Vec<&SkillParamChange> = changes.iter().collect();
    sorted.sort_by(|a, b| a.effective_at.cmp(&b.effective_at));

    let mut profile = current;
    let mut out: Vec<(String, ProfileSnapshot)> = Vec::new();
    for ch in sorted.iter().rev() {
        out.push((ch.effective_at.clone(), profile));
        if let Some((old, _)) = ch.category {
            profile.category = old;
        }
        if let Some((old, _)) = ch.difficulty {
            profile.difficulty = old;
        }
        if let Some((old, _)) = ch.curve_type {
            profile.curve_type = old;
        }
    }
    out.push((created_at.to_string(), profile));
    out.reverse();

    let mut segments: Vec<TimelineSegment> = Vec::new();
    for (from, p) in out {
        let params = EffectiveSkillParams {
            accounts: account_params_for(p.category, overrides),
            curve: curve_params_for(p.category, p.difficulty, p.curve_type, overrides),
        };
        if let Some(last) = segments.last() {
            if same_params(&last.params, &params) {
                continue;
            }
        }
        segments.push(TimelineSegment { from, params });
    }
    segments
}

/// 变化前后 Profile 的 diff（值变化才列字段）。
pub fn profile_diff(before: ProfileSnapshot, after: ProfileSnapshot) -> SkillParamChange {
    let mut change = SkillParamChange {
        effective_at: String::new(),
        category: None,
        difficulty: None,
        curve_type: None,
    };
    if before.category != after.category {
        change.category = Some((before.category, after.category));
    }
    if before.difficulty != after.difficulty {
        change.difficulty = Some((before.difficulty, after.difficulty));
    }
    if before.curve_type != after.curve_type {
        change.curve_type = Some((before.curve_type, after.curve_type));
    }
    change
}

pub fn has_param_diff(change: &SkillParamChange) -> bool {
    change.category.is_some() || change.difficulty.is_some() || change.curve_type.is_some()
}

/// 从审计中取某技能的参数变更（tool='skill.update' 且带 skill_id/effective_at/changed）。
pub fn collect_profile_changes(
    audits: &[AuditLog],
    skill_id: &str,
) -> Vec<SkillParamChange> {
    let mut out: Vec<SkillParamChange> = Vec::new();
    for entry in audits {
        if entry.tool.as_deref() != Some("skill.update") {
            continue;
        }
        let Some(raw) = entry.params_json.as_ref() else { continue };
        let Some(p) = raw.as_object() else { continue };
        if p.get("skill_id").and_then(|v| v.as_str()) != Some(skill_id) {
            continue;
        }
        let Some(effective_at) = p.get("effective_at").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(changed) = p.get("changed").and_then(|v| v.as_object()) else { continue };
        let mut change = SkillParamChange {
            effective_at: effective_at.to_string(),
            category: None,
            difficulty: None,
            curve_type: None,
        };
        let mut any = false;
        if let Some(c) = changed.get("category") {
            let old = c.get("old").and_then(|v| v.as_str()).and_then(SkillCategory::parse);
            let new = c.get("new").and_then(|v| v.as_str()).and_then(SkillCategory::parse);
            if let (Some(o), Some(n)) = (old, new) {
                if o != n {
                    change.category = Some((o, n));
                    any = true;
                }
            }
        }
        if let Some(c) = changed.get("difficulty") {
            let old = c.get("old").and_then(|v| v.as_str()).and_then(Difficulty::parse);
            let new = c.get("new").and_then(|v| v.as_str()).and_then(Difficulty::parse);
            if let (Some(o), Some(n)) = (old, new) {
                if o != n {
                    change.difficulty = Some((o, n));
                    any = true;
                }
            }
        }
        if let Some(c) = changed.get("curve_type") {
            let old = c.get("old").and_then(|v| v.as_str()).and_then(CurveType::parse);
            let new = c.get("new").and_then(|v| v.as_str()).and_then(CurveType::parse);
            if old != new {
                change.curve_type = Some((old, new));
                any = true;
            }
        }
        if any {
            out.push(change);
        }
    }
    out
}
