//! ARC Operations and boss events as guides: the curated entry plus links, where it happens,
//! and a check of the stash against its gear lines. Pure; the shell passes the profile.

use crate::dataset::Dataset;
use crate::model::*;
use crate::views::{item_link, Link};
use crate::weapons::{weapon_family_name, weapon_stats};
use serde::Serialize;

/// ARC armour penetration, weakest first, as items.json words it.
pub const PENETRATION: [&str; 5] = ["very weak", "weak", "moderate", "strong", "very strong"];

fn pen_rank(p: &str) -> Option<usize> {
    PENETRATION.iter().position(|x| *x == p)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpMap {
    pub map_id: String,
    pub map_name: String,
    /// Marked spots for it on this map (Assessor landings, Cargo drops, the Harvester).
    pub spots: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpRewardView {
    pub item: Link,
    pub chance: String,
    pub from: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GearCheck {
    pub slot: String,
    pub need: String,
    /// What counts, as names: weapon tiers fold into one name.
    pub names: Vec<String>,
    /// One link per name, for icons.
    pub examples: Vec<Link>,
    pub armor_pen: Option<String>,
    pub qty: u32,
    pub note: Option<String>,
    /// How many you have that count.
    pub have: u32,
    pub met: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OperationView {
    pub op: Operation,
    pub maps: Vec<OpMap>,
    pub enemies: Vec<Link>,
    pub rewards: Vec<OpRewardView>,
    pub gear: Vec<GearCheck>,
    /// Every minimum gear line is met by the stash.
    pub ready: bool,
}

/// Which items a gear line accepts.
fn accepts(line: &OpGear, item: &Item) -> bool {
    if line.item_ids.iter().any(|id| *id == item.id) {
        return true;
    }
    match (&line.armor_pen, weapon_stats(item)) {
        (Some(want), Some(stats)) => {
            let got = stats.arc_armor_penetration.as_deref().and_then(pen_rank);
            got.is_some_and(|g| pen_rank(want).is_some_and(|w| g >= w))
        }
        _ => false,
    }
}

/// The stash against one gear line.
pub fn check(ds: &Dataset, profile: &Profile, line: &OpGear) -> GearCheck {
    let have: u32 = profile.stash.iter()
        .filter(|(_, n)| **n > 0)
        .filter_map(|(id, n)| ds.item(id).map(|i| (i, *n)))
        .filter(|(i, _)| accepts(line, i))
        .map(|(_, n)| n)
        .sum();
    let mut names: Vec<String> = Vec::new();
    let mut examples: Vec<Link> = Vec::new();
    for id in &line.item_ids {
        let Some(item) = ds.item(id) else { continue };
        let name = if weapon_stats(item).is_some() { weapon_family_name(ds, id) } else { item.name.clone() };
        let name = name.strip_prefix("Amplified ").map(str::to_string).unwrap_or(name);
        let name = name.split(" (").next().unwrap_or(&name).to_string();
        if !names.contains(&name) {
            names.push(name);
            // The plain, lowest tier is the clearest picture of the family.
            examples.push(item_link(ds, id, None));
        }
    }
    // Families list amplified tiers first; prefer a plain one for the picture.
    for (n, link) in names.iter().zip(examples.iter_mut()) {
        let plain = line.item_ids.iter().filter_map(|id| ds.item(id))
            .filter(|i| i.variant_of.is_none() && weapon_stats(i).is_some() && weapon_family_name(ds, &i.id) == *n)
            .min_by_key(|i| i.tier.unwrap_or(9));
        if let Some(p) = plain {
            *link = item_link(ds, &p.id, None);
        }
    }
    GearCheck {
        slot: line.slot.clone(), need: line.need.clone(), names, examples, armor_pen: line.armor_pen.clone(),
        qty: line.qty.max(1), note: line.note.clone(), have, met: have >= line.qty.max(1),
    }
}

pub fn view(ds: &Dataset, profile: &Profile, op: &Operation) -> OperationView {
    let maps = op.maps.iter().map(|m| {
        let map_name = ds.files.maps.iter().find(|x| &x.id == m).map_or(m.as_str(), |x| x.name.as_str()).to_string();
        let spots = ds.files.markers.maps.get(m).map_or(0, |list| list.iter()
            .filter(|mk| ds.files.markers.categories.get(usize::from(mk.0)).is_some_and(|c| op.route.markers.contains(&c.id)))
            .count());
        OpMap { map_id: m.clone(), map_name, spots }
    }).collect();
    let enemies = op.enemies.iter().map(|id| {
        let arc = ds.files.arcs.iter().find(|a| &a.id == id);
        Link {
            kind: "arc", id: id.clone(), name: arc.map_or(id.as_str(), |a| a.name.as_str()).to_string(),
            icon: arc.and_then(|a| a.icon.clone()), qty: None, note: None,
        }
    }).collect();
    let rewards = op.rewards.iter()
        .map(|r| OpRewardView { item: item_link(ds, &r.item_id, None), chance: r.chance.clone(), from: r.from.clone() })
        .collect();
    let gear: Vec<GearCheck> = op.gear.iter().map(|g| check(ds, profile, g)).collect();
    let ready = gear.iter().filter(|g| g.need == "minimum").all(|g| g.met);
    OperationView { op: op.clone(), maps, enemies, rewards, gear, ready }
}

/// Every operation, ARC Operations first, then bosses, then events.
pub fn list(ds: &Dataset, profile: &Profile) -> Vec<OperationView> {
    let rank = |k: &str| match k { "operation" => 0, "boss" => 1, _ => 2 };
    let mut out: Vec<OperationView> = ds.files.operations.iter().map(|o| view(ds, profile, o)).collect();
    out.sort_by_key(|v| rank(&v.op.kind));
    out
}

/// The operation behind a scheduled event name, such as `ARC Frigate`.
pub fn for_condition<'a>(ds: &'a Dataset, name: &str) -> Option<&'a Operation> {
    ds.files.operations.iter().find(|o| o.condition.eq_ignore_ascii_case(name.trim()))
}
