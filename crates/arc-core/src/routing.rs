//! Phase 4 calculators: what the profile still needs, and which POIs cover it (D11).
//! No pathing: a walkable graph doesn't exist, so this is greedy set cover. All pure.

use crate::calc::{craft_tree, raw_totals};
use crate::dataset::Dataset;
use crate::model::*;
use crate::views::{item_link, Link};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Needed {
    pub item: Link,
    /// Still to find after the stash.
    pub qty: u32,
    pub reasons: Vec<String>,
}

fn add(wanted: &mut BTreeMap<String, (u32, Vec<String>)>, id: &str, qty: u32, reason: String) {
    if qty == 0 {
        return;
    }
    let row = wanted.entry(id.to_string()).or_default();
    row.0 += qty;
    if !row.1.contains(&reason) {
        row.1.push(reason);
    }
}

/// Tracked crafts down to raw materials, tracked open quests, and the current stage of
/// tracked projects (less what's committed), minus the stash.
pub fn needed_items(ds: &Dataset, profile: &Profile) -> Vec<Needed> {
    let mut wanted = BTreeMap::new();
    for id in &profile.tracked_item_ids {
        let Some(tree) = craft_tree(ds, id, 1) else { continue };
        for (raw, qty) in raw_totals(&tree) {
            add(&mut wanted, &raw, qty, format!("Tracked: {}", ds.item_name(id)));
        }
    }
    for quest in &ds.files.quests {
        if profile.quests.get(&quest.id).is_some_and(|p| !p.done) {
            for need in &quest.required_items {
                add(&mut wanted, &need.item_id, need.qty, format!("Quest: {}", quest.name));
            }
        }
    }
    for project in &ds.files.projects {
        let Some(progress) = profile.projects.get(&project.id) else { continue };
        let Some(stage) = project.stages.iter().find(|s| s.n == progress.stage) else { continue };
        for need in &stage.inputs {
            let done = progress.committed.get(&need.item_id).copied().unwrap_or(0);
            add(&mut wanted, &need.item_id, need.qty.saturating_sub(done), format!("Project: {}", project.name));
        }
    }
    wanted.into_iter().filter_map(|(id, (qty, reasons))| {
        let left = qty.saturating_sub(profile.stash.get(&id).copied().unwrap_or(0));
        (left > 0).then(|| Needed { item: item_link(ds, &id, Some(left)), qty: left, reasons })
    }).collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoverPoi {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub covers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapCover {
    pub map_id: String,
    pub map_name: String,
    pub pois: Vec<CoverPoi>,
    pub covered_items: Vec<String>,
    pub uncovered: Vec<String>,
}

/// Per map, greedily take the POI covering the most still-uncovered items. Ties go to
/// the first POI in dataset order, so results are stable. Maps rank by coverage, then
/// by fewest stops.
pub fn poi_cover(ds: &Dataset, item_ids: &[String]) -> Vec<MapCover> {
    let wanted: BTreeSet<&str> = item_ids.iter().map(String::as_str).collect();
    let areas: BTreeMap<&str, BTreeSet<&str>> = ds.files.loot_areas.iter().map(|a| {
        (a.id.as_str(), a.item_ids.iter().map(String::as_str).filter(|i| wanted.contains(i)).collect())
    }).collect();
    let in_poi = |poi: &Poi| -> BTreeSet<&str> {
        poi.loot_areas.iter().filter_map(|a| areas.get(a.as_str())).flatten().copied().collect()
    };

    let mut covers: Vec<MapCover> = ds.files.maps.iter().map(|map| {
        let candidates: Vec<(&Poi, BTreeSet<&str>)> = ds.files.pois.iter()
            .filter(|p| p.map_id == map.id)
            .map(|p| (p, in_poi(p)))
            .filter(|(_, items)| !items.is_empty())
            .collect();
        let mut left: BTreeSet<&str> = candidates.iter().flat_map(|(_, i)| i.iter().copied()).collect();
        let reachable = left.clone();
        let mut pois = Vec::new();
        while !left.is_empty() {
            let gain = |items: &BTreeSet<&str>| items.intersection(&left).count();
            let Some((poi, items)) = candidates.iter().rev().max_by_key(|(_, i)| gain(i)) else { break };
            let covered: Vec<&str> = items.intersection(&left).copied().collect();
            if covered.is_empty() {
                break;
            }
            left.retain(|i| !covered.contains(i));
            pois.push(CoverPoi {
                id: poi.id.clone(),
                name: poi.name.clone(),
                x: poi.x,
                y: poi.y,
                covers: covered.iter().map(|i| i.to_string()).collect(),
            });
        }
        MapCover {
            map_id: map.id.clone(),
            map_name: map.name.clone(),
            pois,
            covered_items: reachable.iter().map(|i| i.to_string()).collect(),
            uncovered: wanted.difference(&reachable).map(|i| i.to_string()).collect(),
        }
    }).collect();
    covers.sort_by_key(|c| (std::cmp::Reverse(c.covered_items.len()), c.pois.len()));
    covers
}
