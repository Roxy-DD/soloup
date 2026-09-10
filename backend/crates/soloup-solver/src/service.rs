//! Solver 服务门面（对应 TS `@soloup/solver/service.ts` 的 createSolver）。
//! 职责：把 core 纯数学 + store 持久化编排成领域服务函数——读取前补结算、
//! 打卡（同日覆盖→全量重放）、统一写入口（mutate，事务 + 审计）、派生快照。
//! 数值 C/V 的唯一写通道是 store.skills.settle_accounts。

use std::collections::HashMap;

use chrono::{Local, NaiveDate};
use serde_json::json;

use soloup_core::dates::{add_days, compare_iso, to_iso_date, IsoDate};
use soloup_core::registry::ParamOverrides;
use soloup_core::schema::{AuditActor, DbEnum, Skill, SkillCategory};
use soloup_store::attributes::AttributeRepo;
use soloup_store::audit::{AuditQuery, AuditRepo};
use soloup_store::daily::DailyRepo;
use soloup_store::links::{LinkRepo, SkillLinkInput};
use soloup_store::skills::SkillRepo;
use soloup_store::Store;

use crate::derive::build_snapshot;
use crate::errors::{SolverError, SolverErrorCode};
use crate::ledger::{settle_range, ParamSegment};
use crate::params::{
    collect_profile_changes, profile_diff, build_param_timeline, ProfileSnapshot,
};

/* ------------------------------ 类型 ------------------------------ */

#[derive(Debug, Clone, Copy)]
pub struct AccountState {
    pub c: f64,
    pub v: f64,
}

#[derive(Debug, Clone)]
pub struct SkillEffect {
    pub skill_id: String,
    pub before: AccountState,
    pub after: AccountState,
    pub before_level: Option<f64>,
    pub after_level: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CheckinInput {
    pub date: IsoDate,
    pub skill_ids: Vec<String>,
    pub project_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Mutation {
    pub kind: MutationKind,
    pub actor: AuditActor,
}

#[derive(Debug, Clone)]
pub enum MutationKind {
    SkillCreate {
        name: String,
        category: SkillCategory,
        parent_id: Option<String>,
        difficulty: Option<soloup_core::schema::Difficulty>,
        is_branch: bool,
        links: Vec<SkillLinkInput>,
    },
    SkillUpdate {
        id: String,
        name: Option<String>,
        description: Option<Option<String>>,
        parent_id: Option<Option<String>>,
        category: Option<SkillCategory>,
        difficulty: Option<soloup_core::schema::Difficulty>,
        curve_type: Option<Option<soloup_core::schema::CurveType>>,
        is_branch: Option<bool>,
    },
    SkillMove {
        id: String,
        parent_id: Option<String>,
    },
    SkillArchive {
        id: String,
    },
    SkillRestore {
        id: String,
    },
    SkillDelete {
        id: String,
    },
    SkillLinkSet {
        id: String,
        links: Vec<SkillLinkInput>,
    },
    AttributeCreate {
        name: String,
        description: Option<String>,
        base_value: Option<f64>,
        max_value: Option<f64>,
        alpha: Option<f64>,
        w0: Option<f64>,
        category: Option<soloup_core::schema::AttributeCategory>,
        color: Option<Option<String>>,
        icon: Option<Option<String>>,
    },
    AttributeUpdate {
        id: String,
        name: Option<String>,
        description: Option<Option<String>>,
        category: Option<Option<soloup_core::schema::AttributeCategory>>,
        color: Option<Option<String>>,
        icon: Option<Option<String>>,
        sort: Option<i64>,
    },
    AttributeDelete {
        id: String,
    },
}

#[derive(Debug, Clone)]
pub struct MutateResult {
    pub detail: serde_json::Value,
}

/* ------------------------------ Solver ------------------------------ */

pub struct Solver {
    pub store: Store,
    now: Box<dyn Fn() -> NaiveDate + Send>,
}

impl Solver {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            now: Box::new(|| Local::now().date_naive()),
        }
    }

    /// 以固定"今天"构造（测试 / 服务器注入时钟用）。
    pub fn with_now(store: Store, now: NaiveDate) -> Self {
        Self {
            store,
            now: Box::new(move || now),
        }
    }

    /// 变更"今天"（测试推进日期用）。
    pub fn set_now(&mut self, now: NaiveDate) {
        self.now = Box::new(move || now);
    }

    pub fn today(&self) -> IsoDate {
        to_iso_date((self.now)())
    }

    /* ------------------------------ 辅助 ------------------------------ */

    fn read_overrides(&self) -> Result<ParamOverrides, SolverError> {
        Ok(self
            .store
            .settings()
            .get_json::<ParamOverrides>("param_overrides")?
            .unwrap_or_default())
    }

    fn member_map(&self, to: &str) -> Result<HashMap<String, Vec<String>>, SolverError> {
        let members = self
            .store
            .daily()
            .list_members(None, Some(to))?;
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for m in members {
            map.entry(m.date).or_default().push(m.skill_id);
        }
        Ok(map)
    }

    fn profile_of(skill: &Skill) -> ProfileSnapshot {
        ProfileSnapshot {
            category: skill.category,
            difficulty: skill.difficulty,
            curve_type: skill.curve_type,
        }
    }

    fn leaf_level_of(&self, skill: &Skill, overrides: &ParamOverrides) -> Option<f64> {
        if skill.archived_at.is_some() {
            return None;
        }
        if let Ok(kids) = self.store.skills().children_of(&skill.id) {
            if !kids.is_empty() {
                return None;
            }
        }
        Some(crate::derive::leaf_level(skill, overrides))
    }

    /// 增量补结算：从上次结算次日推进到 to（含）。无缺口返回 None。
    fn incremental_next(
        &self,
        skill: &Skill,
        to: &str,
        overrides: &ParamOverrides,
        members: &HashMap<String, Vec<String>>,
    ) -> Result<Option<(f64, f64, String)>, SolverError> {
        if skill.archived_at.is_some() {
            return Ok(None);
        }
        if let Some(last) = &skill.last_settled_date {
            if compare_iso(last, to) != std::cmp::Ordering::Less {
                return Ok(None);
            }
        }
        let from = match &skill.last_settled_date {
            Some(d) => add_days(d, 1),
            None => skill.created_at.clone(),
        };
        let params = crate::params::effective_skill_params(skill, overrides);
        let is_checkin = |d: &str| {
            members
                .get(d)
                .map(|ids| ids.contains(&skill.id))
                .unwrap_or(false)
        };
        let accounts = settle_range(
            AccountState { c: skill.c, v: skill.v }.into(),
            &from,
            to,
            &is_checkin,
            params.accounts,
        )?;
        Ok(Some((accounts.c, accounts.v, to.to_string())))
    }

    /* ------------------------------ API ------------------------------ */

    /// 读取前补结算全部活跃叶子到今天。
    pub fn settle_all(&mut self) -> Result<Vec<SkillEffect>, SolverError> {
        let to = self.today();
        let overrides = self.read_overrides()?;
        let members = self.member_map(&to)?;
        let mut effects = Vec::new();
        for skill in self.store.skills().list()? {
            if !self.store.skills().children_of(&skill.id)?.is_empty() {
                continue; // 父级无 C/V（§3.8）
            }
            if let Some((c, v, _)) = self.incremental_next(&skill, &to, &overrides, &members)? {
                let after = self.store.skills().settle_accounts(&skill.id, c, v, &to)?;
                effects.push(SkillEffect {
                    skill_id: skill.id.clone(),
                    before: AccountState { c: skill.c, v: skill.v },
                    after: AccountState { c: after.c, v: after.v },
                    before_level: self.leaf_level_of(&skill, &overrides),
                    after_level: self.leaf_level_of(&after, &overrides),
                });
            }
        }
        Ok(effects)
    }

    /// 打卡（date 必须=今天；同日覆盖 → 受影响技能全量重放）。
    pub fn checkin(&mut self, input: &CheckinInput, actor: AuditActor) -> Result<Vec<SkillEffect>, SolverError> {
        let today = self.today();
        let date = &input.date;
        if compare_iso(date, &today) == std::cmp::Ordering::Greater {
            return Err(SolverError::new(SolverErrorCode::FutureDate, format!("打卡日期 {date} 晚于今天 {today}")));
        }
        if compare_iso(date, &today) != std::cmp::Ordering::Equal {
            return Err(SolverError::new(SolverErrorCode::PastEdit, "历史日期编辑属 v1.2，今天以外不可写"));
        }
        let mut deduped = input.skill_ids.clone();
        deduped.dedup();
        if deduped.is_empty() {
            return Err(SolverError::new(SolverErrorCode::Validation, "打卡技能列表不能为空"));
        }
        for id in &deduped {
            let skill = self
                .store
                .skills()
                .get(id)?
                .ok_or_else(|| SolverError::new(SolverErrorCode::NotFound, format!("技能不存在：{id}")))?;
            if !self.store.skills().children_of(id)?.is_empty() {
                return Err(SolverError::new(SolverErrorCode::NonLeaf, format!("技能 {id} 非叶子，不能打卡")));
            }
            if skill.archived_at.is_some() {
                return Err(SolverError::new(SolverErrorCode::Archived, format!("技能 {id} 已归档，不能打卡")));
            }
        }
        let previous = self.store.daily().skill_ids_on(date)?;
        let affected: Vec<String> = {
            let mut set = std::collections::BTreeSet::new();
            for id in previous.iter().chain(deduped.iter()) {
                set.insert(id.clone());
            }
            set.into_iter().collect()
        };
        let overrides = self.read_overrides()?;
        let members = self.member_map(&today)?;
        let effects = self.store.tx(|c| -> Result<Vec<SkillEffect>, SolverError> {
            let daily = DailyRepo { db: c };
            daily.save(
                date,
                &soloup_store::daily::DailySaveInput {
                    project_id: input.project_id.clone(),
                    note: input.note.clone(),
                    skill_ids: deduped.clone(),
                },
            )?;
            let mut out = Vec::new();
            for id in &affected {
                let skill = SkillRepo { db: c }.get(id)?;
                if let Some(skill) = skill {
                    if skill.archived_at.is_none() {
                        let eff = Self::replay_and_persist_conn(c, id, &today, &overrides, &members)?;
                        out.push(eff);
                    }
                }
            }
            let _ = AuditRepo { db: c }.append(
                actor,
                Some("daily.checkin"),
                Some(&json!({ "date": date, "skill_ids": deduped, "project_id": input.project_id, "note": input.note })),
            )?;
            Ok(out)
        })?;
        Ok(effects)
    }

    /// 统一写入口：领域变更 + 审计同一事务。
    pub fn mutate(&mut self, mutation: MutationKind, actor: AuditActor) -> Result<MutateResult, SolverError> {
        let today = self.today();
        let overrides = self.read_overrides()?;
        let detail = self.store.tx(|c| -> Result<serde_json::Value, SolverError> {
            let skills = SkillRepo { db: c };
            let attrs = AttributeRepo { db: c };
            let links = LinkRepo { db: c };
            let audit = AuditRepo { db: c };
            match &mutation {
                MutationKind::SkillCreate { name, category, parent_id, difficulty, is_branch, links: input_links } => {
                    let row = Skill {
                        id: String::new(),
                        name: name.clone(),
                        description: None,
                        parent_id: parent_id.clone(),
                        category: *category,
                        difficulty: difficulty.unwrap_or(soloup_core::schema::Difficulty::Normal),
                        curve_type: None,
                        c: 0.0,
                        v: 0.0,
                        last_settled_date: None,
                        created_at: today.clone(),
                        archived_at: None,
                        color: None,
                        icon: None,
                        sort: 0,
                        is_branch: *is_branch,
                    };
                    let created = skills.create(&row)?;
                    if !input_links.is_empty() {
                        links.set_for_skill(&created.id, input_links)?;
                    }
                    audit.append(
                        actor,
                        Some("skill.create"),
                        Some(&json!({ "skill_id": created.id })),
                    )?;
                    Ok(json!({ "id": created.id, "name": created.name }))
                }
                MutationKind::SkillUpdate { id, name, description, parent_id, category, difficulty, curve_type, is_branch } => {
                    let existing = skills.get(id)?
                        .ok_or_else(|| SolverError::new(SolverErrorCode::NotFound, format!("技能不存在：{id}")))?;
                    let mut row = existing.clone();
                    if let Some(n) = name { row.name = n.clone(); }
                    if let Some(d) = description { row.description = d.clone(); }
                    if let Some(p) = parent_id { row.parent_id = p.clone(); }
                    if let Some(c) = category { row.category = *c; }
                    if let Some(d) = difficulty { row.difficulty = *d; }
                    if let Some(ct) = curve_type { row.curve_type = *ct; }
                    if let Some(ib) = is_branch { row.is_branch = *ib; }
                    let changed = profile_diff(Self::profile_of(&existing), Self::profile_of(&row));
                    let has_changed = crate::params::has_param_diff(&changed);
                    let effective_at = if has_changed { Some(add_days(&today, 1)) } else { None };
                    skills.update(&row)?;
                    audit.append(
                        actor,
                        Some("skill.update"),
                        Some(&json!({
                            "skill_id": id,
                            "effective_at": effective_at,
                            "changed": {
                                "category": changed.category.map(|(o, n)| json!({ "old": o.as_str(), "new": n.as_str() })),
                                "difficulty": changed.difficulty.map(|(o, n)| json!({ "old": o.as_str(), "new": n.as_str() })),
                                "curve_type": changed.curve_type.map(|(o, n)| json!({ "old": o.map(|x| x.as_str()), "new": n.map(|x| x.as_str()) })),
                            },
                        })),
                    )?;
                    Ok(json!({ "id": id }))
                }
                MutationKind::SkillMove { id, parent_id } => {
                    let existing = skills.get(id)?
                        .ok_or_else(|| SolverError::new(SolverErrorCode::NotFound, format!("技能不存在：{id}")))?;
                    let mut row = existing.clone();
                    row.parent_id = parent_id.clone();
                    skills.update(&row)?;
                    audit.append(actor, Some("skill.move"), Some(&json!({ "skill_id": id, "parent_id": parent_id })))?;
                    Ok(json!({ "id": id }))
                }
                MutationKind::SkillArchive { id } => {
                    skills.archive(id, Some(&today))?;
                    audit.append(actor, Some("skill.archive"), Some(&json!({ "skill_id": id })))?;
                    Ok(json!({ "id": id }))
                }
                MutationKind::SkillRestore { id } => {
                    skills.restore(id)?;
                    audit.append(actor, Some("skill.restore"), Some(&json!({ "skill_id": id })))?;
                    Ok(json!({ "id": id }))
                }
                MutationKind::SkillDelete { id } => {
                    skills.delete(id)?;
                    audit.append(actor, Some("skill.delete"), Some(&json!({ "skill_id": id })))?;
                    Ok(json!({ "deleted": id }))
                }
                MutationKind::SkillLinkSet { id, links: input_links } => {
                    links.set_for_skill(id, input_links)?;
                    audit.append(actor, Some("skill.link.set"), Some(&json!({ "skill_id": id })))?;
                    Ok(json!({ "id": id }))
                }
                MutationKind::AttributeCreate { name, description, base_value, max_value, alpha, w0, category, color, icon } => {
                    let defaults = overrides.attr_defaults.clone();
                    let row = soloup_core::schema::Attribute {
                        id: String::new(),
                        name: name.clone(),
                        description: description.clone(),
                        base_value: base_value.or_else(|| defaults.as_ref().and_then(|d| d.base_value)).unwrap_or(0.0),
                        max_value: max_value.or_else(|| defaults.as_ref().and_then(|d| d.max_value)).unwrap_or(100.0),
                        alpha: alpha.or_else(|| defaults.as_ref().and_then(|d| d.alpha)).unwrap_or(1.2),
                        w0: w0.or_else(|| defaults.as_ref().and_then(|d| d.w0)).unwrap_or(400.0),
                        category: *category,
                        color: color.clone().flatten(),
                        icon: icon.clone().flatten(),
                        sort: 0,
                    };
                    let created = attrs.create(&row)?;
                    audit.append(actor, Some("attribute.create"), Some(&json!({ "attribute_id": created.id })))?;
                    Ok(json!({ "id": created.id, "name": created.name }))
                }
                MutationKind::AttributeUpdate { id, name, description, category, color, icon, sort } => {
                    let existing = attrs.get(id)?
                        .ok_or_else(|| SolverError::new(SolverErrorCode::NotFound, format!("属性不存在：{id}")))?;
                    let mut row = existing.clone();
                    if let Some(n) = name { row.name = n.clone(); }
                    if let Some(d) = description { row.description = d.clone(); }
                    if let Some(c) = category { row.category = *c; }
                    if let Some(c) = color { row.color = c.clone(); }
                    if let Some(i) = icon { row.icon = i.clone(); }
                    if let Some(sv) = sort { row.sort = *sv; }
                    attrs.update(&row)?;
                    audit.append(actor, Some("attribute.update"), Some(&json!({ "attribute_id": id })))?;
                    Ok(json!({ "id": id }))
                }
                MutationKind::AttributeDelete { id } => {
                    attrs.delete(id)?;
                    audit.append(actor, Some("attribute.delete"), Some(&json!({ "attribute_id": id })))?;
                    Ok(json!({ "deleted": id }))
                }
            }
        })?;
        Ok(MutateResult { detail })
    }

    /// 派生只读快照（先结算再算）。
    pub fn snapshot(&mut self, to_date: Option<&str>) -> Result<crate::derive::DerivedSnapshot, SolverError> {
        let date = to_date.map(|s| s.to_string()).unwrap_or_else(|| self.today());
        self.settle_all()?;
        let overrides = self.read_overrides()?;
        let skills = self.store.skills().list_all()?;
        let attributes = self.store.attributes().list()?;
        let mut links_by_attribute: HashMap<String, Vec<soloup_core::schema::SkillAttributeLink>> = HashMap::new();
        for a in &attributes {
            links_by_attribute.insert(a.id.clone(), self.store.links().list_for_attribute(&a.id)?);
        }
        Ok(build_snapshot(&crate::derive::SnapshotInput {
            date,
            skills: &skills,
            attributes: &attributes,
            links_by_attribute: &links_by_attribute,
            overrides: &overrides,
        }))
    }

    fn replay_and_persist_conn(
        c: &rusqlite::Connection,
        skill_id: &str,
        to: &str,
        overrides: &ParamOverrides,
        members: &HashMap<String, Vec<String>>,
    ) -> Result<SkillEffect, SolverError> {
        let skills = SkillRepo { db: c };
        let skill = skills
            .get(skill_id)?
            .ok_or_else(|| SolverError::new(SolverErrorCode::NotFound, format!("技能不存在：{skill_id}")))?;
        let is_leaf = skills.children_of(skill_id)?.is_empty();
        let before_level = if skill.archived_at.is_none() && is_leaf {
            Some(crate::derive::leaf_level(&skill, overrides))
        } else {
            None
        };
        let audits = AuditRepo { db: c }.list(&AuditQuery { limit: 1_000_000, ..Default::default() })?;
        let changes = collect_profile_changes(&audits, skill_id);
        let timeline = build_param_timeline(&skill.created_at, Self::profile_of(&skill), overrides, &changes);
        let segments: Vec<ParamSegment> = timeline
            .into_iter()
            .map(|s| ParamSegment { from: s.from, accounts: s.params.accounts })
            .collect();
        let is_checkin = |d: &str| {
            members
                .get(d)
                .map(|ids| ids.iter().any(|s| s == skill_id))
                .unwrap_or(false)
        };
        let final_accounts = crate::ledger::replay_full(crate::ledger::ReplayOptions {
            initial: soloup_core::settle::ZERO_ACCOUNTS,
            created_at: &skill.created_at,
            to_date: to,
            segments: &segments,
            is_checkin: &is_checkin,
        })?;
        let after = skills.settle_accounts(skill_id, final_accounts.c, final_accounts.v, to)?;
        let after_level = if after.archived_at.is_none() && is_leaf {
            Some(crate::derive::leaf_level(&after, overrides))
        } else {
            None
        };
        Ok(SkillEffect {
            skill_id: skill_id.to_string(),
            before: AccountState { c: skill.c, v: skill.v },
            after: AccountState { c: after.c, v: after.v },
            before_level,
            after_level,
        })
    }
}

impl From<AccountState> for soloup_core::settle::Accounts {
    fn from(a: AccountState) -> Self {
        Self { c: a.c, v: a.v }
    }
}
