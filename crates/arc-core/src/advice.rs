//! One-line answers for mid-raid: keep it, sell it or recycle it. All pure.
//!
//! The order is what's needed first (tracked goals, the next workbench level, unfinished
//! quests), then what the item is worth. It's a suggestion: it can't know your plans.

use crate::calc::{craft_tree_from, raw_totals};
use crate::dataset::Dataset;
use crate::model::*;
use crate::routing::{needed_items, Needed};
use crate::tracking::requirement_met;
use crate::views::{item_link, Link};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Verdict {
    /// `keep`, `recycle` or `sell`.
    pub action: &'static str,
    pub headline: String,
    pub reasons: Vec<String>,
    pub sell_value: Option<u32>,
    pub recycle_value: Option<u32>,
    pub gives: Vec<Link>,
}

/// Gear and collectibles are about your loadout, so only say something if a goal needs them.
fn is_loot(category: Category) -> bool {
    !matches!(
        category,
        Category::Weapon | Category::WeaponMod | Category::Cosmetic | Category::Furniture | Category::Design
            | Category::Stencil | Category::Blueprint | Category::Key | Category::QuestItem
            | Category::Augment | Category::Shield
    )
}

fn recycle_outputs<'a>(ds: &'a Dataset, item: &Item) -> &'a [ItemQty] {
    item.recycle_ids.iter()
        .filter_map(|id| ds.recipe(id))
        .find(|r| r.kind == RecipeKind::Recycle)
        .map_or(&[], |r| &r.outputs)
}

pub fn item_verdict(ds: &Dataset, profile: &Profile, id: &str) -> Option<Verdict> {
    let item = ds.item(id)?;
    let needed = needed_items(ds, profile);
    let have = profile.stash.get(id).copied().unwrap_or(0);
    let mut reasons = Vec::new();
    let mine = needed.iter().find(|n| n.item.id == id);
    if let Some(row) = mine {
        reasons.extend(row.reasons.iter().cloned());
    }
    for bench in &ds.files.workbenches {
        let next = profile.workbenches.get(&bench.id).copied().unwrap_or(0) + 1;
        let need = bench.levels.iter().find(|l| l.level == next)
            .and_then(|l| l.upgrade_inputs.iter().find(|q| q.item_id == id));
        if let Some(need) = need.filter(|n| n.qty > have) {
            reasons.push(format!("{} level {next} needs {}", bench.name, need.qty));
        }
    }
    for quest in ds.files.quests.iter().filter(|q| !profile.quests.get(&q.id).is_some_and(|p| p.done)) {
        let tracked = profile.quests.contains_key(&quest.id);
        if !tracked && quest.required_items.iter().any(|q| q.item_id == id && q.qty > have) {
            reasons.push(format!("Quest later: {}", quest.name));
        }
    }
    reasons.dedup();

    let outputs = recycle_outputs(ds, item);
    let recycle_value = (!outputs.is_empty()).then(|| {
        outputs.iter().map(|q| q.qty * ds.item(&q.item_id).and_then(|i| i.value).unwrap_or(0)).sum()
    });
    let gives: Vec<Link> = outputs.iter().map(|q| item_link(ds, &q.item_id, Some(q.qty))).collect();
    let verdict = |action, headline: String, reasons| Verdict {
        action, headline, reasons, sell_value: item.value, recycle_value, gives: gives.clone(),
    };

    if !reasons.is_empty() {
        let headline = mine.map_or("Keep it for later".to_string(), |row| format!("Keep: {} more needed", row.qty));
        return Some(verdict("keep", headline, reasons));
    }
    if !is_loot(item.category) {
        return None;
    }
    // Selling a staple only to buy it back later is the mistake worth preventing.
    if matches!(item.category, Category::MaterialBasic | Category::MaterialTopside | Category::MaterialRefined) {
        let recipes = crate::calc::used_in(ds, id).recipes.len();
        if recipes > 0 {
            return Some(verdict("keep", format!("Keep: crafting material, used in {recipes} recipes"), Vec::new()));
        }
    }
    if let Some(part) = gives.iter().find(|g| needed.iter().any(|n| n.item.id == g.id)) {
        return Some(verdict("recycle", format!("Recycle: gives {} you need", part.name), Vec::new()));
    }
    let sell = item.value?;
    match recycle_value.filter(|r| *r > sell) {
        Some(parts) => Some(verdict("recycle", format!("Recycle: parts are worth {parts}, selling gets {sell}"), Vec::new())),
        None => Some(verdict("sell", format!("Sell: worth {sell}"), Vec::new())),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Goal {
    /// `craft`, `quest`, `project` or `bench`.
    pub kind: &'static str,
    pub id: String,
    pub name: String,
    pub have: u32,
    pub need: u32,
    /// Items still short, with how many are missing.
    pub missing: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Home {
    /// Goals the stash fully covers.
    pub ready: Vec<Goal>,
    /// Goals at least half covered, closest first.
    pub almost: Vec<Goal>,
    pub quests_active: usize,
    pub quests_done: usize,
    pub quests_total: usize,
    pub items_needed: usize,
    pub blueprints_found: usize,
    pub blueprints_total: usize,
    pub bench_levels: u32,
    pub bench_levels_total: u32,
    /// Tracked quests with the step to do next, in the order they were tracked.
    pub quest_next: Vec<QuestNext>,
    /// The items still to find, most wanted first.
    pub needed_top: Vec<Needed>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuestNext {
    pub id: String,
    pub name: String,
    pub trader: Option<String>,
    /// Null when every step is ticked but the quest isn't turned in.
    pub step: Option<String>,
    pub maps: Vec<String>,
    pub done: usize,
    pub total: usize,
}

/// The first unticked step in log order. Done groups are skipped, and an either-or group
/// follows the option already started, else its first option.
pub fn next_step<'a>(objectives: &'a [Objective], id: &str, done: &[String], depth: usize) -> Option<&'a Objective> {
    let node = objectives.iter().find(|o| o.id == id)?;
    if depth > 16 {
        return None;
    }
    match node.kind.as_str() {
        "atomic" => (!done.contains(&node.id)).then_some(node),
        "optional" => None,
        "any" => {
            let started = node.children.iter().find(|c| {
                let mut ids = Vec::new();
                collect_steps(objectives, c, &mut ids, 0);
                ids.iter().any(|i| done.contains(i))
            });
            started.or(node.children.first()).and_then(|c| next_step(objectives, c, done, depth + 1))
        }
        _ => node.children.iter().find_map(|c| next_step(objectives, c, done, depth + 1)),
    }
}

fn collect_steps(objectives: &[Objective], id: &str, out: &mut Vec<String>, depth: usize) {
    let Some(node) = objectives.iter().find(|o| o.id == id) else { return };
    if node.kind == "atomic" {
        out.push(node.id.clone());
    } else if depth < 16 {
        node.children.iter().for_each(|c| collect_steps(objectives, c, out, depth + 1));
    }
}

const ALMOST_SHOWN: usize = 8;
const NEEDED_SHOWN: usize = 6;

fn goal(ds: &Dataset, profile: &Profile, kind: &'static str, id: &str, name: String, needs: &[(String, u32)]) -> Option<Goal> {
    if needs.is_empty() {
        return None;
    }
    let (mut have, mut need, mut missing) = (0, 0, Vec::new());
    for (item, qty) in needs {
        let stash = profile.stash.get(item).copied().unwrap_or(0);
        have += stash.min(*qty);
        need += qty;
        if stash < *qty {
            missing.push(item_link(ds, item, Some(qty - stash)));
        }
    }
    Some(Goal { kind, id: id.to_string(), name, have, need, missing })
}

/// Every open goal, each judged alone: two goals wanting the same stack both look covered.
pub fn goals(ds: &Dataset, profile: &Profile) -> Vec<Goal> {
    let mut rows = Vec::new();
    for id in &profile.tracked_item_ids {
        let owned = profile.tracked_from.get(id).map(String::as_str);
        let Some(tree) = craft_tree_from(ds, id, 1, owned) else { continue };
        let needs: Vec<(String, u32)> = raw_totals(&tree).into_iter().filter(|(raw, _)| raw != id).collect();
        rows.extend(goal(ds, profile, "craft", id, ds.item_name(id).to_string(), &needs));
    }
    for quest in ds.files.quests.iter().filter(|q| profile.quests.get(&q.id).is_some_and(|p| !p.done)) {
        let needs: Vec<_> = quest.required_items.iter().map(|q| (q.item_id.clone(), q.qty)).collect();
        rows.extend(goal(ds, profile, "quest", &quest.id, quest.name.clone(), &needs));
    }
    for project in &ds.files.projects {
        let Some(progress) = profile.projects.get(&project.id) else { continue };
        let Some(stage) = project.stages.iter().find(|s| s.n == progress.stage) else { continue };
        let needs: Vec<_> = stage.inputs.iter().filter_map(|q| {
            let left = q.qty.saturating_sub(progress.committed.get(&q.item_id).copied().unwrap_or(0));
            (left > 0).then(|| (q.item_id.clone(), left))
        }).collect();
        rows.extend(goal(ds, profile, "project", &project.id, format!("{} stage {}", project.name, stage.n), &needs));
    }
    for bench in &ds.files.workbenches {
        let next = profile.workbenches.get(&bench.id).copied().unwrap_or(0) + 1;
        let Some(level) = bench.levels.iter().find(|l| l.level == next) else { continue };
        if level.requires.as_deref().is_some_and(|r| !requirement_met(profile, r)) {
            continue;
        }
        let needs: Vec<_> = level.upgrade_inputs.iter().map(|q| (q.item_id.clone(), q.qty)).collect();
        rows.extend(goal(ds, profile, "bench", &bench.id, format!("{} level {next}", bench.name), &needs));
    }
    rows
}

pub fn home(ds: &Dataset, profile: &Profile) -> Home {
    let (ready, open): (Vec<Goal>, Vec<Goal>) = goals(ds, profile).into_iter().partition(|g| g.have >= g.need);
    let share = |g: &Goal| f64::from(g.have) / f64::from(g.need);
    let mut almost: Vec<Goal> = open.into_iter().filter(|g| share(g) >= 0.5).collect();
    almost.sort_by(|a, b| share(b).total_cmp(&share(a)));
    almost.truncate(ALMOST_SHOWN);
    let blueprints: Vec<&Item> = ds.files.items.iter().filter(|i| i.category == Category::Blueprint).collect();
    let levels: BTreeMap<&str, u8> = ds.files.workbenches.iter().map(|b| (b.id.as_str(), b.max_level)).collect();
    let needed = needed_items(ds, profile);
    let mut needed_top = needed.clone();
    needed_top.sort_by_key(|n| std::cmp::Reverse((n.reasons.len(), n.qty)));
    needed_top.truncate(NEEDED_SHOWN);
    let quest_next = ds.files.quests.iter().filter_map(|q| {
        let progress = profile.quests.get(&q.id).filter(|p| !p.done)?;
        let total = q.objectives.iter().filter(|o| o.kind == "atomic").count();
        let step = next_step(&q.objectives, &q.objective_root, &progress.objectives_done, 0);
        let trader = q.trader.as_ref().map(|t| ds.files.traders.iter().find(|x| &x.id == t).map_or(t.clone(), |x| x.name.clone()));
        Some(QuestNext {
            id: q.id.clone(),
            name: q.name.clone(),
            trader,
            step: step.and_then(|o| o.description.clone()),
            maps: step.map_or_else(Vec::new, |o| o.maps.clone()),
            done: progress.objectives_done.len().min(total),
            total,
        })
    }).collect();
    Home {
        quest_next,
        needed_top,
        ready,
        almost,
        quests_active: profile.quests.values().filter(|p| !p.done).count(),
        quests_done: profile.quests.values().filter(|p| p.done).count(),
        quests_total: ds.files.quests.len(),
        items_needed: needed.len(),
        blueprints_found: blueprints.iter().filter(|b| profile.blueprints_found.contains(&b.id)).count(),
        blueprints_total: blueprints.len(),
        bench_levels: profile.workbenches.iter()
            .filter_map(|(id, level)| levels.get(id.as_str()).map(|max| u32::from((*level).min(*max))))
            .sum(),
        bench_levels_total: levels.values().map(|l| u32::from(*l)).sum(),
    }
}
