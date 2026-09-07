//! soloup-server —— HTTP JSON-RPC 出口（`POST /api/rpc`）。
//! 协议与前端 `apps/web/src/lib/api.ts` 一致：请求 `{op,args}` → `{ok:true,data}` / `{ok:false,error:{code,message}}`。
//! 前端不重算：所有结算/派生都在本进程完成（§4.2）。

pub mod seed;

use std::collections::{HashMap, HashSet};

use chrono::Datelike;
use serde_json::{json, Value};

use soloup_core::dates::{add_days, today_iso};
use soloup_core::schema::{AuditActor, DbEnum, Project, ProjectStatus, Skill};
use soloup_solver::service::{CheckinInput, MutationKind, Solver};
use soloup_solver::SolverError;
use soloup_store::links::SkillLinkInput;

pub struct RpcError {
    pub code: String,
    pub message: String,
}

impl From<SolverError> for RpcError {
    fn from(e: SolverError) -> Self {
        RpcError {
            code: e.code.as_str().to_string(),
            message: e.message,
        }
    }
}

impl From<soloup_store::StoreError> for RpcError {
    fn from(e: soloup_store::StoreError) -> Self {
        RpcError {
            code: e.code.as_str().to_string(),
            message: e.message,
        }
    }
}

type Args<'a> = &'a Value;

fn s<'a>(a: Args<'a>, k: &str) -> Option<&'a str> {
    a.get(k).and_then(|v| v.as_str())
}
fn f(a: Args, k: &str) -> Option<f64> {
    a.get(k).and_then(|v| v.as_f64())
}
fn i(a: Args, k: &str) -> Option<i64> {
    a.get(k).and_then(|v| v.as_i64())
}
fn b(a: Args, k: &str) -> bool {
    a.get(k).and_then(|v| v.as_bool()).unwrap_or(false)
}
fn arr(a: Args, k: &str) -> Vec<Value> {
    a.get(k)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}
fn req_str(a: Args, k: &str) -> Result<String, RpcError> {
    s(a, k)
        .map(|x| x.to_string())
        .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: format!("缺少参数 {k}") })
}
fn opt_string(a: Args, k: &str) -> Option<Option<String>> {
    if a.get(k).is_none() {
        return None;
    }
    Some(a.get(k).and_then(|v| v.as_str()).map(|x| x.to_string()))
}

/// 技能 → 关联属性 id（供记录/成就推导点亮属性）。
fn link_maps(solver: &Solver) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
    let mut skill_to_attrs: HashMap<String, Vec<String>> = HashMap::new();
    let mut attr_to_skills: HashMap<String, Vec<String>> = HashMap::new();
    if let Ok(attrs) = solver.store.attributes().list() {
        for at in attrs {
            if let Ok(links) = solver.store.links().list_for_attribute(&at.id) {
                for l in links {
                    skill_to_attrs
                        .entry(l.skill_id.clone())
                        .or_default()
                        .push(at.id.clone());
                    attr_to_skills
                        .entry(at.id.clone())
                        .or_default()
                        .push(l.skill_id.clone());
                }
            }
        }
    }
    (skill_to_attrs, attr_to_skills)
}

fn lit_attrs_of(ids: &[String], skill_to_attrs: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut set: HashSet<String> = HashSet::new();
    for id in ids {
        if let Some(v) = skill_to_attrs.get(id) {
            for a in v {
                set.insert(a.clone());
            }
        }
    }
    let mut out: Vec<String> = set.into_iter().collect();
    out.sort();
    out
}

fn actor_of(a: Args) -> AuditActor {
    match s(a, "actor") {
        Some("ai") => AuditActor::Ai,
        Some("system") => AuditActor::System,
        _ => AuditActor::User,
    }
}

/// 统一 op 分发。
pub fn dispatch(solver: &mut Solver, op: &str, args: &Value) -> Result<Value, RpcError> {
    match op {
        "meta" => Ok(json!({
            "app": "soloup", "version": "0.1.0-rust", "date": today_iso(),
            "enum": {
                "skill_category": ["physical","cognitive","knowledge"],
                "difficulty": ["casual","normal","hard","challenge","legendary"],
                "curve_type": ["saturated","sigmoid"],
                "attribute_category": ["physical","mental","social","creative"],
                "project_status": ["planned","active","completed","abandoned"],
                "rarity": ["common","rare","epic","legendary"],
                "actor": ["user","ai","system"]
            }
        })),

        "seed.status" => {
            let seeded = solver.store.settings().get("seeded")?.is_some();
            Ok(json!({ "seeded": seeded }))
        }
        "seed.demo" => {
            seed::seed_demo(&mut solver.store)?;
            Ok(json!({ "ok": true }))
        }

        "overview" => {
            let snap = solver.snapshot(None)?;
            let today = today_iso();
            let checked = solver.store.daily().skill_ids_on(&today)?;
            let (skill_to_attrs, _) = link_maps(solver);
            let lit = lit_attrs_of(&checked, &skill_to_attrs);
            let attrs: Vec<Value> = snap
                .attributes
                .iter()
                .map(|a| {
                    json!({
                        "id": a.attribute.id, "name": a.attribute.name,
                        "description": a.attribute.description, "category": a.attribute.category,
                        "color": a.attribute.color, "value": a.value, "x": a.x,
                        "contributions": a.entries.iter().map(|e| json!({
                            "skill_id": e.skill_id, "weight": e.weight, "level": e.level, "share": e.share
                        })).collect::<Vec<_>>()
                    })
                })
                .collect();
            let skills = solver.store.skills().list_all()?;
            let leaves = skills.iter().filter(|s| is_leaf(&skills, &s.id)).count();
            Ok(json!({
                "date": snap.date,
                "counts": { "skills": skills.len(), "leaves": leaves, "attributes": attrs.len() },
                "attributes": attrs,
                "today": { "date": today, "checked": checked, "litAttrs": lit }
            }))
        }

        "stats" => {
            let snap = solver.snapshot(None)?;
            let (skill_to_attrs, _) = link_maps(solver);
            let recs = solver.store.daily().list_records(None, None)?;
            let mut max_lit = 0usize;
            for r in &recs {
                let ids = solver.store.daily().skill_ids_on(&r.date)?;
                let n = lit_attrs_of(&ids, &skill_to_attrs).len();
                if n > max_lit {
                    max_lit = n;
                }
            }
            let dates: HashSet<String> = recs.iter().map(|r| r.date.clone()).collect();
            let mut streak = 0i64;
            let mut cursor = today_iso();
            if !dates.contains(&cursor) {
                cursor = add_days(&cursor, -1);
            }
            while dates.contains(&cursor) {
                streak += 1;
                cursor = add_days(&cursor, -1);
            }
            // 本周（周一至今）
            let chrono_now = chrono::Local::now().date_naive();
            let dow = chrono_now.weekday().num_days_from_monday() as i64;
            let monday = add_days(&today_iso(), -dow);
            let week_days = (0..=dow)
                .map(|i| add_days(&monday, i))
                .filter(|d| dates.contains(d))
                .count();
            let mut top: Option<(&soloup_solver::derive::DerivedLeaf, f64)> = None;
            for l in &snap.leaves {
                if top.is_none() || l.level > top.unwrap().1 {
                    top = Some((l, l.level));
                }
            }
            let attrs = solver.store.attributes().list()?;
            let projects = solver.store.projects().list()?;
            let done = projects
                .iter()
                .filter(|p| matches!(p.status, soloup_core::schema::ProjectStatus::Completed))
                .count();
            let max_lv = snap.leaves.iter().map(|l| l.level).fold(0f64, f64::max);
            // 万时成就：单个技能最多打卡天数
            let mut skill_checkin_counts: HashMap<String, usize> = HashMap::new();
            for r in &recs {
                let ids = solver.store.daily().skill_ids_on(&r.date)?;
                for sid in ids {
                    *skill_checkin_counts.entry(sid).or_insert(0) += 1;
                }
            }
            let max_skill_checkins = skill_checkin_counts.values().copied().max().unwrap_or(0);
            Ok(json!({
                "totalDays": dates.len(),
                "streak": streak,
                "weekDays": week_days,
                "maxLitOneDay": max_lit,
                "maxSkillLv": max_lv,
                "maxSkillCheckins": max_skill_checkins,
                "topSkill": top.map(|(l, lv)| json!({ "id": l.skill.id, "name": l.skill.name, "level": lv })),
                "totalAttrs": attrs.len(),
                "projectsCompleted": done
            }))
        }

        "tree" => {
            let skills = solver.store.skills().list_all()?;
            let snap = solver.snapshot(None)?;
            let levels: HashMap<String, f64> = snap
                .leaves
                .iter()
                .map(|l| (l.skill.id.clone(), l.level))
                .chain(snap.parents.iter().map(|p| (p.skill.id.clone(), p.level)))
                .collect();
            // 叶子 → 关联属性 [attrId, weight]
            let mut link_map: HashMap<String, Vec<(String, f64)>> = HashMap::new();
            for s in &skills {
                let ls = solver.store.links().list_for_skill(&s.id)?;
                if !ls.is_empty() {
                    link_map.insert(
                        s.id.clone(),
                        ls.iter().map(|l| (l.attribute_id.clone(), l.weight)).collect(),
                    );
                }
            }
            fn build(
                skills: &[Skill],
                parent: Option<&str>,
                levels: &HashMap<String, f64>,
                links: &HashMap<String, Vec<(String, f64)>>,
            ) -> Vec<Value> {
                let mut out = Vec::new();
                for s in skills.iter().filter(|x| x.parent_id.as_deref() == parent) {
                    let kids = build(skills, Some(&s.id), levels, links);
                    let leaf = kids.is_empty();
                    let attrs: Vec<Value> = links
                        .get(&s.id)
                        .map(|v| v.iter().map(|(a, w)| json!([a, w])).collect())
                        .unwrap_or_default();
                    out.push(json!({
                        "id": s.id, "name": s.name, "category": s.category,
                        "difficulty": s.difficulty, "leaf": leaf, "archived": s.archived_at.is_some(),
                        "level": levels.get(&s.id).copied().unwrap_or(0.0),
                        "e": s.c + s.v, "c": s.c, "v": s.v,
                        "attrs": attrs,
                        "children": kids
                    }));
                }
                out
            }
            Ok(json!(build(&skills, None, &levels, &link_map)))
        }

        "attributes" => {
            let snap = solver.snapshot(None)?;
            Ok(json!(snap
                .attributes
                .iter()
                .map(|a| json!({
                    "id": a.attribute.id, "name": a.attribute.name,
                    "description": a.attribute.description,
                    "category": a.attribute.category, "color": a.attribute.color,
                    "value": a.value, "x": a.x,
                    "contributions": a.entries.iter().map(|e| json!({
                        "skill_id": e.skill_id, "weight": e.weight, "level": e.level, "share": e.share
                    })).collect::<Vec<_>>()
                }))
                .collect::<Vec<_>>()))
        }

        "records.list" => {
            let limit = i(args, "limit").unwrap_or(120) as usize;
            let recs = solver.store.daily().list_records(None, None)?;
            let (skill_to_attrs, _) = link_maps(solver);
            let mut out: Vec<Value> = Vec::new();
            for r in recs.iter().rev().take(limit) {
                let ids = solver.store.daily().skill_ids_on(&r.date)?;
                out.push(json!({
                    "date": r.date,
                    "skillIds": ids.iter().cloned().collect::<Vec<_>>(),
                    "attrsLit": lit_attrs_of(&ids, &skill_to_attrs),
                    "projectId": r.project_id, "note": r.note
                }));
            }
            Ok(json!(out))
        }

        "checkin" => {
            let date = s(args, "date").map(|x| x.to_string()).unwrap_or_else(today_iso);
            let ids: Vec<String> = arr(args, "skillIds")
                .into_iter()
                .chain(arr(args, "skill_ids"))
                .filter_map(|v| v.as_str().map(|x| x.to_string()))
                .collect();
            let effects = solver.checkin(
                &CheckinInput {
                    date,
                    skill_ids: ids,
                    project_id: s(args, "projectId").map(|x| x.to_string()),
                    note: s(args, "note").map(|x| x.to_string()),
                },
                actor_of(args),
            )?;
            Ok(json!({
                "ok": true,
                "effects": effects.iter().map(|e| json!({
                    "skillId": e.skill_id,
                    "before": { "c": e.before.c, "v": e.before.v, "level": e.before_level },
                    "after": { "c": e.after.c, "v": e.after.v, "level": e.after_level }
                })).collect::<Vec<_>>()
            }))
        }

        "checkin.clear" => {
            let date = s(args, "date").map(|x| x.to_string()).unwrap_or_else(today_iso);
            solver.store.tx(|c| -> Result<(), soloup_store::StoreError> {
                soloup_store::daily::DailyRepo { db: c }.clear(&date)?;
                Ok(())
            })?;
            let _ = solver.settle_all()?;
            Ok(json!({ "ok": true }))
        }

        "skill.create" => {
            let name = req_str(args, "name")?;
            let cat = soloup_core::schema::SkillCategory::parse(&req_str(args, "category")?)
                .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 category".into() })?;
            let diff = s(args, "difficulty")
                .and_then(soloup_core::schema::Difficulty::parse);
            let links: Vec<SkillLinkInput> = arr(args, "links")
                .into_iter()
                .filter_map(|v| {
                    Some(SkillLinkInput {
                        attribute_id: v.get("attributeId").or(v.get("attribute_id"))?.as_str()?.to_string(),
                        weight: v.get("weight").and_then(|w| w.as_f64()).unwrap_or(1.0),
                    })
                })
                .collect();
            let res = solver.mutate(
                MutationKind::SkillCreate {
                    name,
                    category: cat,
                    parent_id: s(args, "parentId").or(s(args, "parent_id")).map(|x| x.to_string()),
                    difficulty: diff,
                    links,
                },
                actor_of(args),
            )?;
            Ok(res.detail)
        }

        "skill.updateProfile" | "skill.update" => {
            let id = req_str(args, "id")?;
            let cat = s(args, "category").map(|c| {
                soloup_core::schema::SkillCategory::parse(c)
                    .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 category".into() })
            }).transpose()?;
            let diff = s(args, "difficulty").map(|c| {
                soloup_core::schema::Difficulty::parse(c)
                    .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 difficulty".into() })
            }).transpose()?;
            let ct = if args.get("curveType").is_some() || args.get("curve_type").is_some() {
                let raw = s(args, "curveType").or(s(args, "curve_type"));
                Some(raw.and_then(soloup_core::schema::CurveType::parse))
            } else {
                None
            };
            let res = solver.mutate(
                MutationKind::SkillUpdate {
                    id,
                    name: s(args, "name").map(|x| x.to_string()),
                    description: opt_string(args, "description"),
                    parent_id: opt_string(args, "parentId").or_else(|| opt_string(args, "parent_id")),
                    category: cat,
                    difficulty: diff,
                    curve_type: ct,
                },
                actor_of(args),
            )?;
            Ok(res.detail)
        }

        "skill.move" => {
            let res = solver.mutate(
                MutationKind::SkillMove {
                    id: req_str(args, "id")?,
                    parent_id: s(args, "parentId").or(s(args, "parent_id")).map(|x| x.to_string()),
                },
                actor_of(args),
            )?;
            Ok(res.detail)
        }
        "skill.archive" => {
            let res = solver.mutate(MutationKind::SkillArchive { id: req_str(args, "id")? }, actor_of(args))?;
            Ok(res.detail)
        }
        "skill.restore" => {
            let res = solver.mutate(MutationKind::SkillRestore { id: req_str(args, "id")? }, actor_of(args))?;
            Ok(res.detail)
        }
        "skill.delete" | "skill.remove" => {
            let res = solver.mutate(MutationKind::SkillDelete { id: req_str(args, "id")? }, actor_of(args))?;
            Ok(res.detail)
        }
        "skill.linkSet" => {
            let links: Vec<SkillLinkInput> = arr(args, "links")
                .into_iter()
                .filter_map(|v| {
                    Some(SkillLinkInput {
                        attribute_id: v.get("attributeId").or(v.get("attribute_id"))?.as_str()?.to_string(),
                        weight: v.get("weight").and_then(|w| w.as_f64()).unwrap_or(1.0),
                    })
                })
                .collect();
            let res = solver.mutate(
                MutationKind::SkillLinkSet { id: req_str(args, "id")?, links },
                actor_of(args),
            )?;
            Ok(res.detail)
        }

        "skillDetail" => {
            let id = req_str(args, "id")?;
            let skill = solver
                .store
                .skills()
                .get(&id)?
                .ok_or_else(|| RpcError { code: "ERR_NOT_FOUND".into(), message: format!("技能不存在：{id}") })?;
            let snap = solver.snapshot(None)?;
            let level = snap
                .leaves
                .iter()
                .find(|l| l.skill.id == id)
                .map(|l| l.level)
                .or_else(|| snap.parents.iter().find(|p| p.skill.id == id).map(|p| p.level))
                .unwrap_or(0.0);
            let links = solver.store.links().list_for_skill(&id)?;
            Ok(json!({
                "id": skill.id, "name": skill.name, "description": skill.description,
                "category": skill.category, "difficulty": skill.difficulty,
                "parentId": skill.parent_id, "archived": skill.archived_at.is_some(),
                "level": level, "e": skill.c + skill.v, "c": skill.c, "v": skill.v,
                "linkedAttributes": links.iter().map(|l| json!({
                    "attributeId": l.attribute_id, "weight": l.weight
                })).collect::<Vec<_>>()
            }))
        }

        "attribute.create" => {
            let cat = s(args, "category").map(|c| {
                soloup_core::schema::AttributeCategory::parse(c)
                    .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 category".into() })
            }).transpose()?;
            let res = solver.mutate(
                MutationKind::AttributeCreate {
                    name: req_str(args, "name")?,
                    description: s(args, "description").map(|x| x.to_string()),
                    base_value: f(args, "baseValue").or(f(args, "base_value")),
                    max_value: f(args, "maxValue").or(f(args, "max_value")),
                    alpha: f(args, "alpha"),
                    w0: f(args, "w0"),
                    category: cat,
                    color: opt_string(args, "color"),
                    icon: opt_string(args, "icon"),
                },
                actor_of(args),
            )?;
            Ok(res.detail)
        }
        "attribute.update" => {
            let cat = if args.get("category").is_some() {
                let raw = s(args, "category");
                Some(raw.and_then(soloup_core::schema::AttributeCategory::parse))
            } else {
                None
            };
            let res = solver.mutate(
                MutationKind::AttributeUpdate {
                    id: req_str(args, "id")?,
                    name: s(args, "name").map(|x| x.to_string()),
                    description: opt_string(args, "description"),
                    category: cat,
                    color: opt_string(args, "color"),
                    icon: opt_string(args, "icon"),
                    sort: i(args, "sort"),
                },
                actor_of(args),
            )?;
            Ok(res.detail)
        }
        "attribute.delete" => {
            let res = solver.mutate(MutationKind::AttributeDelete { id: req_str(args, "id")? }, actor_of(args))?;
            Ok(res.detail)
        }

        "settings" => {
            let overrides = solver
                .store
                .settings()
                .get_json::<soloup_core::registry::ParamOverrides>("param_overrides")?
                .unwrap_or_default();
            Ok(json!({
                "registry": soloup_core::registry::param_registry().iter().map(|e| json!({
                    "key": e.key, "label": e.label, "default": e.default,
                    "min": e.min, "max": e.max,
                    "minExclusive": e.min_exclusive, "maxExclusive": e.max_exclusive
                })).collect::<Vec<_>>(),
                "overrides": overrides
            }))
        }

        "param.preview" | "param.set" => {
            let key = req_str(args, "key")?;
            let value = f(args, "value")
                .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "缺少 value".into() })?;
            let entry = soloup_core::registry::find_registry_entry(&key).ok_or_else(|| RpcError {
                code: "ERR_UNKNOWN_PARAM".into(),
                message: format!("未知参数：{key}"),
            })?;
            if !soloup_core::registry::value_in_range(&entry, value) {
                return Err(RpcError {
                    code: "ERR_PARAM_RANGE".into(),
                    message: format!("{key} 越界：{value} 不在 [{},{}]", entry.min, entry.max),
                });
            }
            if op == "param.preview" {
                return Ok(json!({ "key": key, "value": value, "applied": false }));
            }
            let mut overrides = solver
                .store
                .settings()
                .get_json::<soloup_core::registry::ParamOverrides>("param_overrides")?
                .unwrap_or_default();
            overrides = soloup_core::registry::apply_override_value(&overrides, &key, value);
            solver.store.settings().set_json("param_overrides", &serde_json::to_value(&overrides).unwrap())?;
            Ok(json!({ "key": key, "value": value, "applied": true }))
        }
        "param.remove" => {
            let key = req_str(args, "key")?;
            let mut overrides = solver
                .store
                .settings()
                .get_json::<soloup_core::registry::ParamOverrides>("param_overrides")?
                .unwrap_or_default();
            overrides = soloup_core::registry::remove_override_key(&overrides, &key);
            solver.store.settings().set_json("param_overrides", &serde_json::to_value(&overrides).unwrap())?;
            Ok(json!({ "key": key, "applied": true }))
        }

        "audit" => {
            let limit = i(args, "limit").unwrap_or(50);
            let list = solver.store.audit().list(&soloup_store::audit::AuditQuery {
                limit,
                offset: i(args, "offset").unwrap_or(0),
                actor: s(args, "actor").and_then(AuditActor::parse),
                tool: s(args, "tool").map(|x| x.to_string()),
            })?;
            Ok(json!(list
                .iter()
                .map(|a| json!({
                    "id": a.id, "at": a.at, "actor": a.actor, "tool": a.tool, "params": a.params_json
                }))
                .collect::<Vec<_>>()))
        }

        "life.view" => {
            let axis = solver
                .store
                .settings()
                .get_json::<Value>("life_axis")?
                .unwrap_or_else(|| json!({ "born": "2002-05-20", "expectancy": 120 }));
            let born = axis.get("born").and_then(|v| v.as_str()).unwrap_or("2002-05-20").to_string();
            let exp = axis.get("expectancy").and_then(|v| v.as_f64()).unwrap_or(120.0);
            let pct = life_pct(&born, exp);
            Ok(json!({ "born": born, "expectancy": exp, "pct": pct }))
        }
        "life.save" => {
            let born = req_str(args, "born")?;
            let exp = f(args, "expectancy").unwrap_or(120.0);
            solver
                .store
                .settings()
                .set_json("life_axis", &json!({ "born": born, "expectancy": exp }))?;
            Ok(json!({ "born": born, "expectancy": exp }))
        }

        "profile.load" => {
            let p = solver.store.settings().get_json::<Value>("ui_profile")?;
            Ok(json!(p.unwrap_or_else(|| json!({ "nickname": "人生玩家", "avatar": "人", "remind": "20:00", "motion": true, "lockHistory": false }))))
        }
        "profile.save" => {
            solver.store.settings().set_json("ui_profile", args)?;
            Ok(json!({ "ok": true }))
        }

        "projects.list" => {
            let list = solver.store.projects().list()?;
            Ok(json!(list
                .iter()
                .map(|p| json!({
                    "id": p.id, "name": p.name, "description": p.description,
                    "startDate": p.start_date, "endDate": p.end_date, "status": p.status, "color": p.color
                }))
                .collect::<Vec<_>>()))
        }

        "project.create" => {
            let row = Project {
                id: String::new(),
                name: req_str(args, "name")?,
                description: s(args, "description").map(|x| x.to_string()),
                start_date: s(args, "start").or(s(args, "startDate")).map(|x| x.to_string()).unwrap_or_else(today_iso),
                end_date: opt_string(args, "end").or_else(|| opt_string(args, "endDate")).flatten(),
                status: ProjectStatus::Active,
                color: opt_string(args, "color").flatten(),
            };
            let created = solver.store.projects().create(&row)?;
            Ok(json!({ "id": created.id, "name": created.name }))
        }
        "project.finish" => {
            let id = req_str(args, "id")?;
            let end = s(args, "end").map(|x| x.to_string()).unwrap_or_else(today_iso);
            let mut p = solver
                .store
                .projects()
                .get(&id)?
                .ok_or_else(|| RpcError { code: "ERR_NOT_FOUND".into(), message: format!("项目不存在：{id}") })?;
            p.status = ProjectStatus::Completed;
            p.end_date = Some(end);
            solver.store.projects().update(&p)?;
            Ok(json!({ "id": id }))
        }
        "project.update" => {
            let id = req_str(args, "id")?;
            let mut p = solver
                .store
                .projects()
                .get(&id)?
                .ok_or_else(|| RpcError { code: "ERR_NOT_FOUND".into(), message: format!("项目不存在：{id}") })?;
            if let Some(name) = s(args, "name") { p.name = name.to_string(); }
            if let Some(desc) = opt_string(args, "description") { p.description = desc; }
            if let Some(color) = opt_string(args, "color") { p.color = color; }
            if let Some(start) = s(args, "start").or(s(args, "startDate")) { p.start_date = start.to_string(); }
            if args.get("end").is_some() || args.get("endDate").is_some() {
                p.end_date = opt_string(args, "end").or_else(|| opt_string(args, "endDate")).flatten();
            }
            if let Some(status_str) = s(args, "status") {
                p.status = ProjectStatus::parse(status_str)
                    .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 status".into() })?;
            }
            solver.store.projects().update(&p)?;
            Ok(json!({ "id": p.id, "name": p.name, "status": p.status }))
        }
        "project.delete" => {
            let id = req_str(args, "id")?;
            solver.store.projects().delete(&id)?;
            Ok(json!({ "id": id, "deleted": true }))
        }

        "achievements" => {
            let list = solver.store.achievements().list()?;
            let stats = dispatch(solver, "stats", &json!({}))?;
            let total_days = stats.get("totalDays").and_then(|v| v.as_i64()).unwrap_or(0);
            let streak = stats.get("streak").and_then(|v| v.as_i64()).unwrap_or(0);
            let max_lv = stats.get("maxSkillLv").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max_lit = stats.get("maxLitOneDay").and_then(|v| v.as_i64()).unwrap_or(0);
            let total_attrs = stats.get("totalAttrs").and_then(|v| v.as_i64()).unwrap_or(0);
            let done = stats.get("projectsCompleted").and_then(|v| v.as_i64()).unwrap_or(0);
            let max_skill_checkins = stats.get("maxSkillCheckins").and_then(|v| v.as_i64()).unwrap_or(0);
            let unlocked = |id: &str, cond: &Option<Value>| match id {
                "first" => total_days >= 1,
                "twin" => max_lit >= 2,
                "week7" => streak >= 7,
                "dawn" => max_lv >= 4.0,
                "d100" => total_days >= 100,
                "allsix" => total_attrs > 0 && max_lit >= total_attrs,
                "done1" => done >= 1,
                "grand" => max_lv >= 20.0,
                "tenk" => max_skill_checkins >= 10000,
                _ => eval_condition(cond, total_days, streak, max_lv, max_lit, total_attrs, done, max_skill_checkins),
            };
            Ok(json!(list
                .iter()
                .map(|a| json!({
                    "id": a.id, "name": a.name, "description": a.description,
                    "type": a.r#type, "rarity": a.rarity, "points": a.points,
                    "condition": a.condition_json,
                    "hidden": a.hidden, "requires": a.requires, "reveal_at": a.reveal_at,
                    "unlocked": unlocked(&a.id, &a.condition_json)
                }))
                .collect::<Vec<_>>()))
        }
        "achievement.create" => {
            let requires = args.get("requires")
                .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
                .unwrap_or_default();
            let a = soloup_core::schema::Achievement {
                id: String::new(),
                name: req_str(args, "name")?,
                description: s(args, "description").map(|x| x.to_string()),
                condition_json: args.get("condition").cloned(),
                r#type: s(args, "type").and_then(soloup_core::schema::AchievementType::parse)
                    .unwrap_or(soloup_core::schema::AchievementType::Milestone),
                rarity: s(args, "rarity").and_then(soloup_core::schema::Rarity::parse)
                    .unwrap_or(soloup_core::schema::Rarity::Common),
                points: i(args, "points").unwrap_or(10),
                unlocked_at: None,
                hidden: b(args, "hidden"),
                requires,
                reveal_at: args.get("reveal_at").and_then(|v| v.as_f64()),
            };
            let created = solver.store.achievements().create(&a)?;
            Ok(json!({ "id": created.id, "name": created.name }))
        }
        "achievement.update" => {
            let id = req_str(args, "id")?;
            let mut a = solver
                .store
                .achievements()
                .get(&id)?
                .ok_or_else(|| RpcError { code: "ERR_NOT_FOUND".into(), message: format!("成就不存在：{id}") })?;
            if let Some(name) = s(args, "name") { a.name = name.to_string(); }
            if let Some(desc) = opt_string(args, "description") { a.description = desc; }
            if args.get("condition").is_some() { a.condition_json = args.get("condition").cloned(); }
            if let Some(type_str) = s(args, "type") {
                a.r#type = soloup_core::schema::AchievementType::parse(type_str)
                    .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 type".into() })?;
            }
            if let Some(rarity_str) = s(args, "rarity") {
                a.rarity = soloup_core::schema::Rarity::parse(rarity_str)
                    .ok_or_else(|| RpcError { code: "ERR_VALIDATION".into(), message: "非法 rarity".into() })?;
            }
            if let Some(pts) = i(args, "points") { a.points = pts; }
            if args.get("hidden").is_some() { a.hidden = b(args, "hidden"); }
            if let Some(req_arr) = args.get("requires") {
                if let Ok(v) = serde_json::from_value::<Vec<String>>(req_arr.clone()) {
                    a.requires = v;
                }
            }
            if let Some(rv) = args.get("reveal_at").and_then(|v| v.as_f64()) {
                a.reveal_at = Some(rv);
            }
            solver.store.achievements().update(&a)?;
            Ok(json!({ "id": a.id, "name": a.name }))
        }
        "achievement.delete" => {
            let id = req_str(args, "id")?;
            solver.store.achievements().delete(&id)?;
            Ok(json!({ "id": id, "deleted": true }))
        }

        // 首屏聚合：一次请求拿齐渲染所需的全部后端派生数据（前端不重算）。
        "bootstrap" => {
            let overview = dispatch(solver, "overview", &json!({}))?;
            Ok(json!({
                "date": today_iso(),
                "attributes": dispatch(solver, "attributes", &json!({}))?,
                "tree": dispatch(solver, "tree", &json!({}))?,
                "records": dispatch(solver, "records.list", &json!({ "limit": 400 }))?,
                "projects": dispatch(solver, "projects.list", &json!({}))?,
                "achievements": dispatch(solver, "achievements", &json!({}))?,
                "profile": dispatch(solver, "profile.load", &json!({}))?,
                "life": dispatch(solver, "life.view", &json!({}))?,
                "stats": dispatch(solver, "stats", &json!({}))?,
                "today": overview.get("today").cloned().unwrap_or(json!(null))
            }))
        }

        "export.all" => {
            let attrs = solver.store.attributes().list()?;
            let skills = solver.store.skills().list_all()?;
            let mut links: Vec<Value> = Vec::new();
            for s in &skills {
                for l in solver.store.links().list_for_skill(&s.id)? {
                    links.push(json!({ "skill_id": l.skill_id, "attribute_id": l.attribute_id, "weight": l.weight }));
                }
            }
            let recs = solver.store.daily().list_records(None, None)?;
            let members = solver.store.daily().list_members(None, None)?;
            let projects = solver.store.projects().list()?;
            let achievements = solver.store.achievements().list()?;
            let settings_rows = solver.store.settings().list()?;
            Ok(json!({
                "version": 1,
                "exportedAt": today_iso(),
                "attributes": attrs,
                "skills": skills,
                "links": links,
                "records": recs,
                "recordSkills": members,
                "projects": projects,
                "achievements": achievements,
                "settings": settings_rows,
            }))
        }

        "import.all" => {
            let attrs = arr(args, "attributes");
            let skills = arr(args, "skills");
            let links = arr(args, "links");
            let records = arr(args, "records");
            let record_skills = arr(args, "recordSkills");
            let projects = arr(args, "projects");
            let achievements = arr(args, "achievements");
            let settings_rows = arr(args, "settings");

            solver.store.tx(|c| -> Result<(), soloup_store::StoreError> {
                c.execute_batch(
                    "DELETE FROM skill_attributes;
                     DELETE FROM daily_record_skills;
                     DELETE FROM daily_records;
                     DELETE FROM skills;
                     DELETE FROM attributes;
                     DELETE FROM projects;
                     DELETE FROM achievements;
                     DELETE FROM settings;",
                )?;

                for a in &attrs {
                    let row = serde_json::from_value::<soloup_core::schema::Attribute>(a.clone())
                        .map_err(|e| soloup_store::StoreError::new(soloup_store::StoreErrorCode::Validation, e.to_string()))?;
                    soloup_store::attributes::AttributeRepo { db: c }.create(&row)?;
                }
                for s in &skills {
                    let row = serde_json::from_value::<soloup_core::schema::Skill>(s.clone())
                        .map_err(|e| soloup_store::StoreError::new(soloup_store::StoreErrorCode::Validation, e.to_string()))?;
                    soloup_store::skills::SkillRepo { db: c }.create(&row)?;
                }
                for l in &links {
                    let skill_id = l.get("skill_id").and_then(|v| v.as_str()).unwrap_or("");
                    let attribute_id = l.get("attribute_id").and_then(|v| v.as_str()).unwrap_or("");
                    let weight = l.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0);
                    c.execute(
                        "INSERT INTO skill_attributes (skill_id, attribute_id, weight) VALUES (?1, ?2, ?3)",
                        rusqlite::params![skill_id, attribute_id, weight],
                    )?;
                }
                for r in &records {
                    let date = r.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let project_id = r.get("project_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let note = r.get("note").and_then(|v| v.as_str()).map(|s| s.to_string());
                    c.execute(
                        "INSERT INTO daily_records (date, project_id, note, settled) VALUES (?1, ?2, ?3, 0)",
                        rusqlite::params![date, project_id, note],
                    )?;
                }
                for m in &record_skills {
                    let date = m.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let skill_id = m.get("skill_id").and_then(|v| v.as_str()).unwrap_or("");
                    c.execute(
                        "INSERT INTO daily_record_skills (date, skill_id) VALUES (?1, ?2)",
                        rusqlite::params![date, skill_id],
                    )?;
                }
                for p in &projects {
                    let row = serde_json::from_value::<soloup_core::schema::Project>(p.clone())
                        .map_err(|e| soloup_store::StoreError::new(soloup_store::StoreErrorCode::Validation, e.to_string()))?;
                    soloup_store::projects::ProjectRepo { db: c }.create(&row)?;
                }
                for a in &achievements {
                    let row = serde_json::from_value::<soloup_core::schema::Achievement>(a.clone())
                        .map_err(|e| soloup_store::StoreError::new(soloup_store::StoreErrorCode::Validation, e.to_string()))?;
                    soloup_store::achievements::AchievementRepo { db: c }.create(&row)?;
                }
                for s in &settings_rows {
                    let key = s.get("key").and_then(|v| v.as_str()).unwrap_or("");
                    let value = s.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    c.execute(
                        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                        rusqlite::params![key, value],
                    )?;
                }
                Ok(())
            })?;

            Ok(json!({ "ok": true, "imported": {
                "attributes": attrs.len(),
                "skills": skills.len(),
                "links": links.len(),
                "records": records.len(),
                "projects": projects.len(),
                "achievements": achievements.len(),
                "settings": settings_rows.len(),
            }}))
        }

        "recalc" => {
            let effects = solver.settle_all()?;
            Ok(json!({ "settled": effects.len() }))
        }

        _ => Err(RpcError {
            code: "ERR_UNKNOWN_OP".into(),
            message: format!("未知 op：{op}"),
        }),
    }
}

fn is_leaf(skills: &[Skill], id: &str) -> bool {
    !skills.iter().any(|s| s.parent_id.as_deref() == Some(id))
}

/// 评估自定义成就条件（简化版 stat_threshold DSL）。
fn eval_condition(
    cond: &Option<Value>,
    total_days: i64,
    streak: i64,
    max_lv: f64,
    max_lit: i64,
    total_attrs: i64,
    done: i64,
    max_skill_checkins: i64,
) -> bool {
    let cond = match cond {
        Some(c) => c,
        None => return false,
    };
    let stat = cond.get("stat").and_then(|v| v.as_str()).unwrap_or("");
    let operator = cond.get("operator").and_then(|v| v.as_str()).unwrap_or(">=");
    let value = cond.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let actual: f64 = match stat {
        "totalDays" => total_days as f64,
        "streak" => streak as f64,
        "maxSkillLv" => max_lv,
        "maxLitOneDay" => max_lit as f64,
        "totalAttrs" => total_attrs as f64,
        "projectsCompleted" => done as f64,
        "maxSkillCheckins" => max_skill_checkins as f64,
        _ => return false,
    };
    match operator {
        ">=" => actual >= value,
        ">" => actual > value,
        "==" => actual == value,
        "<=" => actual <= value,
        "<" => actual < value,
        _ => false,
    }
}

fn life_pct(born: &str, expectancy: f64) -> f64 {
    let parts: Vec<&str> = born.split('-').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let y: i32 = parts[0].parse().unwrap_or(2000);
    let m: u32 = parts[1].parse().unwrap_or(1);
    let d: u32 = parts[2].parse().unwrap_or(1);
    let b = chrono::NaiveDate::from_ymd_opt(y, m, d);
    let Some(b) = b else { return 0.0 };
    let now = chrono::Local::now().date_naive();
    let elapsed = (now - b).num_days() as f64;
    let total = expectancy * 365.25;
    ((elapsed / total) * 100.0).clamp(0.0, 100.0)
}


