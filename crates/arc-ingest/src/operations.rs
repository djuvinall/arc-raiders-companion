//! `operations.json`: ARC Operations, bosses and payload events, from
//! `overlays/operations_overlay.json`. Names in the overlay (items, ARCs, maps, marker types)
//! must all resolve, so a typo fails the build instead of shipping a dead link.

use anyhow::{bail, Context, Result};
use arc_core::model::{
    Arc, GameMap, Item, MapCondition, MapMarkers, OpExpect, OpGear, OpPatch, OpReward, OpRoute, OpSource, OpStep, OpTeam,
    Operation,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Overlay {
    operations: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: String,
    name: String,
    kind: String,
    condition: String,
    introduced: String,
    maps: Vec<String>,
    summary: String,
    starts: String,
    map_effects: Vec<String>,
    steps: Vec<OpStep>,
    enemies: Vec<String>,
    enemy_notes: Vec<String>,
    rewards: Vec<RewardRow>,
    reward_notes: Vec<String>,
    team: OpTeam,
    gear: Vec<GearRow>,
    gear_notes: Vec<String>,
    expect: OpExpect,
    mistakes: Vec<String>,
    route: OpRoute,
    patches: Vec<OpPatch>,
    conflicts: Vec<String>,
    checks: Vec<String>,
    confidence: String,
    sources: Vec<OpSource>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RewardRow {
    item: String,
    chance: String,
    from: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GearRow {
    slot: String,
    need: String,
    #[serde(default)]
    any_of: Vec<String>,
    #[serde(default)]
    armor_pen: Option<String>,
    #[serde(default = "one")]
    qty: u32,
    #[serde(default)]
    note: Option<String>,
}

fn one() -> u32 {
    1
}

const KINDS: &[&str] = &["operation", "boss", "event"];
const CONFIDENCE: &[&str] = &["sourced", "conflicting", "unverified", "in-game"];
const CHANCES: &[&str] = &["guaranteed", "likely", "possible"];
const NEEDS: &[&str] = &["minimum", "recommended"];
const SLOTS: &[&str] = &["weapon", "shield", "healing", "utility", "augment"];
/// ARC armour penetration, weakest first, as items.json words it.
use arc_core::operations::PENETRATION;

/// `Anvil I` and `Amplified Anvil (X-Rounds)` are both Anvils.
fn family_of<'a>(items: &'a [Item], item: &'a Item) -> &'a str {
    let mut root = item;
    for _ in 0..4 {
        match root.variant_of.as_deref().and_then(|v| items.iter().find(|i| i.id == v)) {
            Some(up) => root = up,
            None => break,
        }
    }
    let name = root.name.as_str();
    let name = name.strip_prefix("Amplified ").unwrap_or(name);
    let name = name.split(" (").next().unwrap_or(name);
    name.trim_end_matches(|c: char| c == ' ' || c == 'I' || c == 'V')
}

/// Every item a gear name means: the item itself, or every tier of a weapon family.
fn gear_ids(items: &[Item], name: &str, whose: &str) -> Result<Vec<String>> {
    let lower = name.to_lowercase();
    let exact: Vec<String> = items.iter().filter(|i| i.name.to_lowercase() == lower).map(|i| i.id.clone()).collect();
    let family: Vec<String> = items.iter()
        .filter(|i| i.category == arc_core::model::Category::Weapon && family_of(items, i).eq_ignore_ascii_case(name))
        .map(|i| i.id.clone())
        .collect();
    let mut ids: Vec<String> = exact.into_iter().chain(family).collect();
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        bail!("{whose}: no item or weapon is called {name:?}");
    }
    Ok(ids)
}

fn arc_id(arcs: &[Arc], name: &str, whose: &str) -> Result<String> {
    arcs.iter()
        .find(|a| a.name.eq_ignore_ascii_case(name) || a.id.eq_ignore_ascii_case(name)
            || a.name.strip_prefix("ARC ").is_some_and(|n| n.eq_ignore_ascii_case(name)))
        .map(|a| a.id.clone())
        .with_context(|| format!("{whose}: no ARC is called {name:?}"))
}

pub fn build(
    overlay: &str, items: &[Item], arcs: &[Arc], maps: &[GameMap], markers: &MapMarkers, conditions: &[MapCondition],
) -> Result<Vec<Operation>> {
    let overlay: Overlay = serde_json::from_str(overlay).context("operations_overlay.json")?;
    let mut out = Vec::new();
    for row in overlay.operations {
        let whose = format!("operations_overlay.json: {}", row.id);
        if !KINDS.contains(&row.kind.as_str()) || !CONFIDENCE.contains(&row.confidence.as_str()) {
            bail!("{whose}: kind must be one of {KINDS:?} and confidence one of {CONFIDENCE:?}");
        }
        if !conditions.iter().any(|c| c.name == row.condition) {
            bail!("{whose}: no map condition is called {:?}", row.condition);
        }
        for map in &row.maps {
            if !maps.iter().any(|m| &m.id == map) {
                bail!("{whose}: no map {map}");
            }
        }
        for marker in &row.route.markers {
            if !markers.categories.iter().any(|c| &c.id == marker) {
                bail!("{whose}: no marker type {marker}");
            }
        }
        if row.sources.is_empty() {
            bail!("{whose}: needs at least one source");
        }
        let enemies = row.enemies.iter().map(|e| arc_id(arcs, e, &whose)).collect::<Result<Vec<_>>>()?;
        let rewards = row.rewards.iter().map(|r| {
            if !CHANCES.contains(&r.chance.as_str()) {
                bail!("{whose}: reward chance must be one of {CHANCES:?}");
            }
            let item_id = items.iter().find(|i| i.name.eq_ignore_ascii_case(&r.item)).map(|i| i.id.clone())
                .with_context(|| format!("{whose}: no item is called {:?}", r.item))?;
            Ok(OpReward { item_id, chance: r.chance.clone(), from: r.from.clone() })
        }).collect::<Result<Vec<_>>>()?;
        let gear = row.gear.iter().map(|g| {
            if !NEEDS.contains(&g.need.as_str()) || !SLOTS.contains(&g.slot.as_str()) {
                bail!("{whose}: gear need must be one of {NEEDS:?} and slot one of {SLOTS:?}");
            }
            if let Some(pen) = &g.armor_pen {
                if !PENETRATION.contains(&pen.as_str()) {
                    bail!("{whose}: armour penetration must be one of {PENETRATION:?}");
                }
            }
            if g.any_of.is_empty() && g.armor_pen.is_none() {
                bail!("{whose}: a gear line needs any_of or armor_pen");
            }
            let mut item_ids = Vec::new();
            for name in &g.any_of {
                item_ids.extend(gear_ids(items, name, &whose)?);
            }
            Ok(OpGear { slot: g.slot.clone(), need: g.need.clone(), item_ids, armor_pen: g.armor_pen.clone(), qty: g.qty.max(1), note: g.note.clone() })
        }).collect::<Result<Vec<_>>>()?;
        out.push(Operation {
            id: row.id, name: row.name, kind: row.kind, condition: row.condition, introduced: row.introduced,
            maps: row.maps, summary: row.summary, starts: row.starts, map_effects: row.map_effects, steps: row.steps,
            enemies, enemy_notes: row.enemy_notes, rewards, reward_notes: row.reward_notes, team: row.team, gear,
            gear_notes: row.gear_notes, expect: row.expect, mistakes: row.mistakes, route: row.route,
            patches: row.patches, conflicts: row.conflicts, checks: row.checks, confidence: row.confidence,
            sources: row.sources,
        });
    }
    Ok(out)
}
