//! ARC pages as a bestiary: threat, a written strategy, guns suited to the armor, and where
//! each one spawns. Built from the data, so it says only what the data supports. All pure.

use crate::dataset::Dataset;
use crate::model::*;
use crate::views::{item_link, Link};
use crate::weapons::{weapon_family, weapon_family_name, weapon_stats};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Spawn {
    pub map_id: String,
    pub map_name: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BestiaryEntry {
    pub arc: Arc,
    pub drops: Vec<Link>,
    /// `Swarm`, `Light`, `Medium`, `Heavy` or `Boss`, from health; null when health is unknown.
    pub threat: Option<&'static str>,
    pub strategy: Vec<String>,
    /// Guns whose ARC armor penetration suits this armor, best first.
    pub counters: Vec<Link>,
    pub spawns: Vec<Spawn>,
}

const PENETRATION: [&str; 5] = ["very weak", "weak", "moderate", "strong", "very strong"];
const COUNTERS_SHOWN: usize = 6;

fn threat(hp: Option<f64>) -> Option<&'static str> {
    Some(match hp? {
        h if h < 50.0 => "Swarm",
        h if h < 400.0 => "Light",
        h if h < 1200.0 => "Medium",
        h if h < 2500.0 => "Heavy",
        _ => "Boss",
    })
}

/// One gun per family, at its lowest level, with the best penetration first.
fn counters(ds: &Dataset, heavy: bool) -> Vec<Link> {
    let floor = if heavy { 3 } else { 2 };
    let mut best: BTreeMap<String, (usize, &Item)> = BTreeMap::new();
    for item in ds.files.items.iter().filter(|i| i.variant_of.is_none()) {
        let Some(stats) = weapon_stats(item) else { continue };
        let Some(rank) = stats.arc_armor_penetration.as_deref().and_then(|p| PENETRATION.iter().position(|x| *x == p)) else { continue };
        if rank < floor {
            continue;
        }
        let family = weapon_family(ds, &item.id);
        let lower = best.get(&family).is_some_and(|(_, held)| held.tier <= item.tier);
        if !lower {
            best.insert(family, (rank, item));
        }
    }
    let mut rows: Vec<(usize, &Item)> = best.into_values().collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    rows.into_iter().take(COUNTERS_SHOWN)
        .map(|(rank, item)| Link {
            name: weapon_family_name(ds, &item.id),
            note: Some(format!("{} ARC armor penetration", PENETRATION[rank])),
            ..item_link(ds, &item.id, None)
        })
        .collect()
}

fn spawns(ds: &Dataset, arc_id: &str) -> Vec<Spawn> {
    let markers = &ds.files.markers;
    let Some(index) = markers.categories.iter().position(|c| c.id == format!("arc/{arc_id}")) else { return Vec::new() };
    let mut rows: Vec<Spawn> = markers.maps.iter().filter_map(|(map, list)| {
        let count = list.iter().filter(|m| usize::from(m.0) == index).count();
        let name = ds.files.maps.iter().find(|m| &m.id == map).map_or(map.as_str(), |m| m.name.as_str());
        (count > 0).then(|| Spawn { map_id: map.clone(), map_name: name.to_string(), count })
    }).collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count));
    rows
}

fn strategy(arc: &Arc, threat: Option<&str>, counters: &[Link], spawns: &[Spawn]) -> Vec<String> {
    let mut lines = Vec::new();
    match threat {
        Some("Swarm") => lines.push("Fragile but rarely alone: keep moving and clear them before they close in.".to_string()),
        Some("Boss") => lines.push("A boss-level ARC: bring a squad, full ammo and healing, and plan your cover before engaging.".to_string()),
        Some("Heavy") => lines.push("Durable: don't trade shots in the open. Fight from cover and pick your moment.".to_string()),
        _ => {}
    }
    if !arc.weak_points.is_empty() {
        lines.push(format!("Aim for the {}. Hits there do far more than body shots.", arc.weak_points.join(" and ").to_lowercase()));
    }
    let heavy = arc.armor.as_deref() == Some("Heavy");
    match (heavy, counters.first()) {
        (true, Some(_)) => lines.push(format!(
            "Heavy armor shrugs off weak rounds. Bring a gun with strong ARC armor penetration, such as {}.",
            counters.iter().take(3).map(|c| c.name.clone()).collect::<Vec<_>>().join(", "),
        )),
        (false, _) if arc.armor.is_some() => lines.push("Light armor: most guns work, so pick for range and handling.".to_string()),
        _ => {}
    }
    if let Some(attack) = &arc.attack {
        lines.push(format!("Its main attack is {}; break line of sight when it winds up.", attack.to_lowercase()));
    }
    if let Some(ability) = &arc.ability {
        lines.push(format!("Also watch for: {}.", ability.to_lowercase()));
    }
    if let Some(top) = spawns.first() {
        lines.push(format!("Most mapped spawns are on {} ({}).", top.map_name, top.count));
    }
    lines
}

pub fn bestiary(ds: &Dataset) -> Vec<BestiaryEntry> {
    ds.files.arcs.iter().map(|arc| {
        let threat = threat(arc.hp);
        let counters = counters(ds, arc.armor.as_deref() == Some("Heavy"));
        let spawns = spawns(ds, &arc.id);
        BestiaryEntry {
            strategy: strategy(arc, threat, &counters, &spawns),
            drops: arc.drops.iter().map(|d| item_link(ds, &d.item_id, None)).collect(),
            arc: arc.clone(),
            threat,
            counters,
            spawns,
        }
    }).collect()
}
