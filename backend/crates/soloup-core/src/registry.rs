//! §9.2.1 可调参数注册表 —— 单一来源。
//! 对应 TS `@soloup/core/param-registry.ts`。key 为点分路径，范围闭开区间由
//! min_exclusive/max_exclusive 表达。变更只影响未来结算与新建，不追溯历史。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::params::*;
use crate::SkillCategory;

/* ---------------------------- ParamOverrides ---------------------------- */

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttrDefaultsOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w0: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_value: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurveOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub k: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x0: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lambda: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attr_defaults: Option<AttrDefaultsOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<HashMap<String, AttrDefaultsOverride>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<HashMap<SkillCategory, CategoryOverride>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve: Option<HashMap<SkillCategory, CurveOverride>>,
}

/* ------------------------------ Registry ------------------------------ */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamRoot {
    AttrDefaults,
    Category,
    Curve,
}

#[derive(Debug, Clone)]
pub struct ParamPath {
    pub root: ParamRoot,
    pub category: Option<SkillCategory>,
    pub field: String,
}

pub fn parse_param_key(key: &str) -> Option<ParamPath> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return None;
    }
    if parts[0] == "attr_defaults" {
        if parts.len() != 2 {
            return None;
        }
        return Some(ParamPath {
            root: ParamRoot::AttrDefaults,
            category: None,
            field: parts[1].to_string(),
        });
    }
    if (parts[0] == "category" || parts[0] == "curve") && parts.len() == 3 {
        let cat = match parts[1] {
            "physical" => SkillCategory::Physical,
            "cognitive" => SkillCategory::Cognitive,
            "knowledge" => SkillCategory::Knowledge,
            _ => return None,
        };
        let root = if parts[0] == "category" {
            ParamRoot::Category
        } else {
            ParamRoot::Curve
        };
        return Some(ParamPath {
            root,
            category: Some(cat),
            field: parts[2].to_string(),
        });
    }
    None
}

#[derive(Debug, Clone)]
pub struct ParamRegistryEntry {
    pub key: String,
    pub label: String,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub min_exclusive: bool,
    pub max_exclusive: bool,
}

/// §9.2.1 完整注册表（顺序即文档顺序）。
pub fn param_registry() -> Vec<ParamRegistryEntry> {
    let mut out: Vec<ParamRegistryEntry> = vec![
        ParamRegistryEntry {
            key: "attr_defaults.alpha".to_string(),
            label: "属性聚合指数 α".to_string(),
            default: ATTRIBUTE_DEFAULTS.alpha,
            min: 1.0,
            max: 1.8,
            min_exclusive: false,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "attr_defaults.w0".to_string(),
            label: "属性饱和尺度 W0".to_string(),
            default: ATTRIBUTE_DEFAULTS.w0,
            min: 100.0,
            max: 2000.0,
            min_exclusive: false,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "attr_defaults.max_value".to_string(),
            label: "新建属性上限默认".to_string(),
            default: ATTRIBUTE_DEFAULTS.max_value,
            min: 20.0,
            max: 1000.0,
            min_exclusive: false,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "attr_defaults.base_value".to_string(),
            label: "新建属性基础值默认".to_string(),
            default: ATTRIBUTE_DEFAULTS.base_value,
            min: 0.0,
            max: 50.0,
            min_exclusive: false,
            max_exclusive: false,
        },
    ];
    for cat in SkillCategory::ALL {
        let (c, f) = category_account_params(cat);
        let name = match cat {
            SkillCategory::Physical => "physical",
            SkillCategory::Cognitive => "cognitive",
            SkillCategory::Knowledge => "knowledge",
        };
        out.push(ParamRegistryEntry {
            key: format!("category.{name}.c"),
            label: format!("category.{name} 结晶率"),
            default: c,
            min: 0.0,
            max: 0.04,
            min_exclusive: true,
            max_exclusive: false,
        });
        out.push(ParamRegistryEntry {
            key: format!("category.{name}.f"),
            label: format!("category.{name} 遗忘率"),
            default: f,
            min: 0.0,
            max: 0.04,
            min_exclusive: true,
            max_exclusive: false,
        });
    }
    let sig_c = sigmoid_default_params(SkillCategory::Cognitive);
    let sig_k = sigmoid_default_params(SkillCategory::Knowledge);
    out.extend([
        ParamRegistryEntry {
            key: "curve.physical.lambda".to_string(),
            label: "指数饱和特征量 λ".to_string(),
            default: SATURATED_DEFAULT_LAMBDA,
            min: 100.0,
            max: 3000.0,
            min_exclusive: false,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "curve.cognitive.k".to_string(),
            label: "cognitive sigmoid 陡峭度 k".to_string(),
            default: sig_c.k,
            min: 0.0,
            max: 0.03,
            min_exclusive: true,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "curve.cognitive.x0".to_string(),
            label: "cognitive sigmoid 拐点 x0".to_string(),
            default: sig_c.x0,
            min: 50.0,
            max: 1000.0,
            min_exclusive: false,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "curve.knowledge.k".to_string(),
            label: "knowledge sigmoid 陡峭度 k".to_string(),
            default: sig_k.k,
            min: 0.0,
            max: 0.03,
            min_exclusive: true,
            max_exclusive: false,
        },
        ParamRegistryEntry {
            key: "curve.knowledge.x0".to_string(),
            label: "knowledge sigmoid 拐点 x0".to_string(),
            default: sig_k.x0,
            min: 50.0,
            max: 1000.0,
            min_exclusive: false,
            max_exclusive: false,
        },
    ]);
    out
}

pub fn find_registry_entry(key: &str) -> Option<ParamRegistryEntry> {
    param_registry().into_iter().find(|e| e.key.as_str() == key)
}

pub fn registry_default(key: &str) -> Option<f64> {
    find_registry_entry(key).map(|e| e.default)
}

/// 值是否落在该条目的合法区间。
pub fn value_in_range(entry: &ParamRegistryEntry, value: f64) -> bool {
    if value < entry.min || value > entry.max {
        return false;
    }
    if entry.min_exclusive && value == entry.min {
        return false;
    }
    if entry.max_exclusive && value == entry.max {
        return false;
    }
    true
}

/// 读取某 key 的显式覆盖值（未覆盖返回 None）。
pub fn read_override_value(overrides: &ParamOverrides, key: &str) -> Option<f64> {
    let p = parse_param_key(key)?;
    match p.root {
        ParamRoot::AttrDefaults => match &overrides.attr_defaults {
            Some(a) => match p.field.as_str() {
                "alpha" => a.alpha,
                "w0" => a.w0,
                "max_value" => a.max_value,
                "base_value" => a.base_value,
                _ => None,
            },
            None => None,
        },
        ParamRoot::Category => overrides
            .category
            .as_ref()
            .and_then(|m| p.category.and_then(|c| m.get(&c)))
            .and_then(|r| match p.field.as_str() {
                "c" => r.c,
                "f" => r.f,
                _ => None,
            }),
        ParamRoot::Curve => overrides
            .curve
            .as_ref()
            .and_then(|m| p.category.and_then(|c| m.get(&c)))
            .and_then(|r| match p.field.as_str() {
                "k" => r.k,
                "x0" => r.x0,
                "lambda" => r.lambda,
                _ => None,
            }),
    }
}

/// 某 key 当前生效值（覆盖优先，缺省取注册表默认）。
pub fn read_effective_value(overrides: &ParamOverrides, key: &str) -> f64 {
    read_override_value(overrides, key)
        .or_else(|| registry_default(key))
        .unwrap_or(f64::NAN)
}

/// 应用一个覆盖（值须已通过注册表范围校验）。返回新 overrides（不可变）。
pub fn apply_override_value(overrides: &ParamOverrides, key: &str, value: f64) -> ParamOverrides {
    let mut out = overrides.clone();
    let Some(p) = parse_param_key(key) else {
        return out;
    };
    match p.root {
        ParamRoot::AttrDefaults => {
            let mut a = out.attr_defaults.clone().unwrap_or_default();
            match p.field.as_str() {
                "alpha" => a.alpha = Some(value),
                "w0" => a.w0 = Some(value),
                "max_value" => a.max_value = Some(value),
                "base_value" => a.base_value = Some(value),
                _ => return out,
            }
            out.attr_defaults = Some(a);
        }
        ParamRoot::Category => {
            let Some(cat) = p.category else { return out };
            let mut m = out.category.clone().unwrap_or_default();
            let mut r = m.get(&cat).cloned().unwrap_or_default();
            match p.field.as_str() {
                "c" => r.c = Some(value),
                "f" => r.f = Some(value),
                _ => return out,
            }
            m.insert(cat, r);
            out.category = Some(m);
        }
        ParamRoot::Curve => {
            let Some(cat) = p.category else { return out };
            let mut m = out.curve.clone().unwrap_or_default();
            let mut r = m.get(&cat).cloned().unwrap_or_default();
            match p.field.as_str() {
                "lambda" => r.lambda = Some(value),
                "k" => r.k = Some(value),
                "x0" => r.x0 = Some(value),
                _ => return out,
            }
            m.insert(cat, r);
            out.curve = Some(m);
        }
    }
    out
}

/// 移除某 key 的覆盖（所属记录/映射为空时一并清理）。
pub fn remove_override_key(overrides: &ParamOverrides, key: &str) -> ParamOverrides {
    let mut out = overrides.clone();
    let Some(p) = parse_param_key(key) else {
        return out;
    };
    match p.root {
        ParamRoot::AttrDefaults => {
            if let Some(mut a) = out.attr_defaults.clone() {
                match p.field.as_str() {
                    "alpha" => a.alpha = None,
                    "w0" => a.w0 = None,
                    "max_value" => a.max_value = None,
                    "base_value" => a.base_value = None,
                    _ => return out,
                }
                out.attr_defaults = if a.alpha.is_none() && a.w0.is_none() && a.max_value.is_none() && a.base_value.is_none() { None } else { Some(a) };
            }
        }
        ParamRoot::Category => {
            let Some(cat) = p.category else { return out };
            if let Some(mut m) = out.category.clone() {
                if let Some(mut r) = m.get(&cat).cloned() {
                    match p.field.as_str() {
                        "c" => r.c = None,
                        "f" => r.f = None,
                        _ => return out,
                    }
                    if r.c.is_none() && r.f.is_none() { m.remove(&cat); } else { m.insert(cat, r); }
                }
                out.category = if m.is_empty() { None } else { Some(m) };
            }
        }
        ParamRoot::Curve => {
            let Some(cat) = p.category else { return out };
            if let Some(mut m) = out.curve.clone() {
                if let Some(mut r) = m.get(&cat).cloned() {
                    match p.field.as_str() {
                        "lambda" => r.lambda = None,
                        "k" => r.k = None,
                        "x0" => r.x0 = None,
                        _ => return out,
                    }
                    if r.lambda.is_none() && r.k.is_none() && r.x0.is_none() { m.remove(&cat); } else { m.insert(cat, r); }
                }
                out.curve = if m.is_empty() { None } else { Some(m) };
            }
        }
    }
    out
}

/// §9.2.1 约束：category.<c> 生效的 c+f ≤ 0.05。
pub fn category_pair_within_bound(overrides: &ParamOverrides, cat: SkillCategory) -> bool {
    let (dc, df) = category_account_params(cat);
    let explicit = overrides.category.as_ref().and_then(|m| m.get(&cat));
    let eff_c = explicit.and_then(|r| r.c).unwrap_or(dc);
    let eff_f = explicit.and_then(|r| r.f).unwrap_or(df);
    eff_c + eff_f <= 0.05
}

/// 判断某 key 变更后影响的语义描述（供 audit/preview 计数实现）。
pub fn describe_key(key: &str) -> String {
    find_registry_entry(key)
        .map(|e| e.label.to_string())
        .unwrap_or_else(|| key.to_string())
}
