//! 派生快照（§3.8 聚合 / §3.9 属性派生）。派生值不入库，一律现算 —— solver 只读面。
//! 对应 TS `@soloup/solver/derive.ts`。

use std::collections::HashMap;

use soloup_core::aggregate::{attribute_value, AttributeParams, WeightedLevel};
use soloup_core::curve::level_of;
use soloup_core::params::AGGREGATE_ALPHA;
use soloup_core::registry::ParamOverrides;
use soloup_core::schema::{Attribute, Skill, SkillAttributeLink};

use crate::params::effective_skill_params;

/// 叶子等级 = ℓ(c+v)（按生效曲线）。
pub fn leaf_level(skill: &Skill, overrides: &ParamOverrides) -> f64 {
    level_of(skill.c + skill.v, &effective_skill_params(skill, overrides).curve)
}

struct LevelMap<'a> {
    skills: &'a [Skill],
    overrides: &'a ParamOverrides,
    cache: HashMap<String, f64>,
}

impl<'a> LevelMap<'a> {
    fn level_of_id(&mut self, id: &str) -> f64 {
        if let Some(&v) = self.cache.get(id) {
            return v;
        }
        let skill = self.skills.iter().find(|s| s.id == id);
        let Some(skill) = skill else {
            return 0.0;
        };
        let kids: Vec<&Skill> = self
            .skills
            .iter()
            .filter(|s| s.parent_id.as_deref() == Some(&skill.id))
            .collect();
        let lvl = if kids.is_empty() {
            leaf_level(skill, self.overrides)
        } else {
            let mut sum_w = 0.0;
            let mut sum_wl = 0.0;
            for k in &kids {
                let lv = self.level_of_id(&k.id);
                sum_w += 1.0;
                sum_wl += lv.powf(AGGREGATE_ALPHA);
            }
            (sum_wl / sum_w).powf(1.0 / AGGREGATE_ALPHA)
        };
        self.cache.insert(id.to_string(), lvl);
        lvl
    }
}

/// 整棵技能树逐节点等级（叶子用 ℓ(E)，父级用 α 广义均值）。含已归档。
pub fn build_level_map(skills: &[Skill], overrides: &ParamOverrides) -> HashMap<String, f64> {
    let mut lm = LevelMap {
        skills,
        overrides,
        cache: HashMap::new(),
    };
    let ids: Vec<String> = skills.iter().map(|s| s.id.clone()).collect();
    for id in &ids {
        lm.level_of_id(id);
    }
    lm.cache
}

/// 属性生效参数：行值 + 按属性覆盖 attribute.<id>。
pub fn effective_attribute_params(
    attribute: &Attribute,
    overrides: &ParamOverrides,
) -> AttributeParams {
    let o = overrides.attribute.as_ref().and_then(|m| m.get(&attribute.id));
    AttributeParams {
        base_value: o.and_then(|x| x.base_value).unwrap_or(attribute.base_value),
        max_value: o.and_then(|x| x.max_value).unwrap_or(attribute.max_value),
        alpha: o.and_then(|x| x.alpha).unwrap_or(attribute.alpha),
        w0: o.and_then(|x| x.w0).unwrap_or(attribute.w0),
    }
}

#[derive(Debug, Clone)]
pub struct AttributeContribution {
    pub skill_id: String,
    pub weight: f64,
    pub level: f64,
    pub share: f64,
}

#[derive(Debug, Clone)]
pub struct AttributeDerived {
    pub attribute: Attribute,
    pub value: f64,
    pub x: f64,
    pub params: AttributeParams,
    pub entries: Vec<AttributeContribution>,
}

pub fn derive_attribute(
    attribute: &Attribute,
    links: &[SkillAttributeLink],
    levels: &HashMap<String, f64>,
    overrides: &ParamOverrides,
) -> AttributeDerived {
    let params = effective_attribute_params(attribute, overrides);
    let mut x = 0.0;
    let mut entries: Vec<AttributeContribution> = Vec::new();
    for link in links {
        let level = levels.get(&link.skill_id).copied().unwrap_or(0.0);
        x += link.weight * level.powf(params.alpha);
        entries.push(AttributeContribution {
            skill_id: link.skill_id.clone(),
            weight: link.weight,
            level,
            share: 0.0,
        });
    }
    if x > 0.0 {
        for e in &mut entries {
            e.share = (e.weight * e.level.powf(params.alpha)) / x;
        }
    }
    let weighted: Vec<WeightedLevel> = links
        .iter()
        .map(|l| WeightedLevel {
            level: levels.get(&l.skill_id).copied().unwrap_or(0.0),
            weight: l.weight,
        })
        .collect();
    let value = attribute_value(&params, &weighted);
    AttributeDerived {
        attribute: attribute.clone(),
        value,
        x,
        params,
        entries,
    }
}

#[derive(Debug, Clone)]
pub struct DerivedLeaf {
    pub skill: Skill,
    pub level: f64,
}

#[derive(Debug, Clone)]
pub struct DerivedChild {
    pub skill: Skill,
    pub level: f64,
}

#[derive(Debug, Clone)]
pub struct DerivedParent {
    pub skill: Skill,
    pub level: f64,
    pub children: Vec<DerivedChild>,
}

#[derive(Debug, Clone)]
pub struct DerivedSnapshot {
    pub date: String,
    pub leaves: Vec<DerivedLeaf>,
    pub parents: Vec<DerivedParent>,
    pub attributes: Vec<AttributeDerived>,
}

pub struct SnapshotInput<'a> {
    pub date: String,
    pub skills: &'a [Skill],
    pub attributes: &'a [Attribute],
    /// attribute_id → 全部关联（store 守卫保证只挂叶子）。
    pub links_by_attribute: &'a HashMap<String, Vec<SkillAttributeLink>>,
    pub overrides: &'a ParamOverrides,
}

pub fn build_snapshot(input: &SnapshotInput) -> DerivedSnapshot {
    let levels = build_level_map(input.skills, input.overrides);

    let mut leaves: Vec<DerivedLeaf> = Vec::new();
    let mut parents: Vec<DerivedParent> = Vec::new();
    for skill in input.skills {
        let kids: Vec<&Skill> = input
            .skills
            .iter()
            .filter(|s| s.parent_id.as_deref() == Some(&skill.id))
            .collect();
        if kids.is_empty() {
            leaves.push(DerivedLeaf {
                skill: skill.clone(),
                level: levels.get(&skill.id).copied().unwrap_or(0.0),
            });
        } else {
            parents.push(DerivedParent {
                skill: skill.clone(),
                level: levels.get(&skill.id).copied().unwrap_or(0.0),
                children: kids
                    .iter()
                    .map(|k| DerivedChild {
                        skill: (*k).clone(),
                        level: levels.get(&k.id).copied().unwrap_or(0.0),
                    })
                    .collect(),
            });
        }
    }

    let attributes: Vec<AttributeDerived> = input
        .attributes
        .iter()
        .map(|a| {
            derive_attribute(
                a,
                input.links_by_attribute.get(&a.id).map(|v| v.as_slice()).unwrap_or(&[]),
                &levels,
                input.overrides,
            )
        })
        .collect();

    leaves.sort_by(|a, b| b.level.partial_cmp(&a.level).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.skill.name.cmp(&b.skill.name)));
    parents.sort_by(|a, b| b.level.partial_cmp(&a.level).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.skill.name.cmp(&b.skill.name)));
    let mut attributes = attributes;
    attributes.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.attribute.name.cmp(&b.attribute.name)));

    DerivedSnapshot {
        date: input.date.clone(),
        leaves,
        parents,
        attributes,
    }
}
