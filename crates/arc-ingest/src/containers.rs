//! `containers.json`: one row per loot marker type, from `overlays/containers_overlay.json`
//! plus the husk, probe and courier drop lists in `overlays/arc_loot.json`. Also gives the
//! map's marker types their display names.

use crate::arc_loot::{self, Overlay as ArcLoot};
use anyhow::{bail, Context, Result};
use arc_core::model::{Container, ContainerItem, GameMap, Item, MapCondition, MapMarkers, PoolWeight};
use serde::Deserialize;

#[derive(Deserialize)]
struct Overlay {
    containers: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    marker: String,
    name: String,
    group: String,
    #[serde(default = "yes")]
    loot: bool,
    #[serde(default)]
    pool: Vec<PoolWeight>,
    #[serde(default)]
    items: Vec<String>,
    #[serde(default)]
    from_arc_loot: Option<String>,
    #[serde(default)]
    breach: Option<bool>,
    #[serde(default)]
    needs: Option<String>,
    #[serde(default)]
    zone_dependent: bool,
    #[serde(default)]
    conditions: Vec<String>,
    confidence: String,
    #[serde(default)]
    wiki: Option<String>,
    #[serde(default)]
    check_name: bool,
    #[serde(default)]
    notes: Option<String>,
}

fn yes() -> bool {
    true
}

const CONFIDENCE: &[&str] = &["confirmed", "likely", "unverified", "in-game"];
const GROUPS: &[&str] = &["normal", "raider", "arc", "condition"];

pub fn build(overlay: &str, arc_loot: &ArcLoot, items: &[Item], markers: &mut MapMarkers) -> Result<Vec<Container>> {
    let overlay: Overlay = serde_json::from_str(overlay).context("containers_overlay.json")?;
    let names = arc_loot::names(items);
    let mut out = Vec::new();
    for row in overlay.containers {
        let whose = format!("containers_overlay.json: {}", row.marker);
        if !CONFIDENCE.contains(&row.confidence.as_str()) || !GROUPS.contains(&row.group.as_str()) {
            bail!("{whose}: confidence must be one of {CONFIDENCE:?} and group one of {GROUPS:?}");
        }
        let Some(category) = markers.categories.iter_mut().find(|c| c.id == row.marker) else {
            println!("note: {whose}: no such marker type on any map");
            continue;
        };
        category.name = row.name.clone();
        let mut listed = Vec::new();
        for name in &row.items {
            match names.get(&name.to_lowercase()) {
                Some(id) => listed.push(ContainerItem { item_id: id.clone(), sure: row.confidence == "confirmed" }),
                None => println!("note: {whose}: no item called {name:?}"),
            }
        }
        let mut breach = row.breach.unwrap_or(false);
        let mut notes = row.notes.clone();
        if let Some(arc_name) = &row.from_arc_loot {
            let entry = arc_loot.arcs.iter().find(|e| &e.name == arc_name)
                .with_context(|| format!("{whose}: arc_loot.json has no {arc_name}"))?;
            for drop in &entry.drops {
                let id = arc_loot::item_id(&names, &drop.item, arc_name)?;
                if !listed.iter().any(|l: &ContainerItem| l.item_id == id) {
                    listed.push(ContainerItem { item_id: id, sure: drop.sure });
                }
            }
            breach = row.breach.or(entry.breach).unwrap_or(false);
            if notes.is_none() {
                notes = entry.notes.clone();
            }
        }
        if row.loot && row.pool.is_empty() && listed.is_empty() {
            bail!("{whose}: a loot container needs a pool or items");
        }
        out.push(Container {
            marker: row.marker,
            name: row.name,
            group: row.group,
            loot: row.loot,
            pool: row.pool,
            items: listed,
            breach,
            needs: row.needs,
            zone_dependent: row.zone_dependent,
            conditions: row.conditions,
            confidence: row.confidence,
            wiki: row.wiki,
            check_name: row.check_name,
            notes,
        });
    }
    out.sort_by(|a, b| a.marker.cmp(&b.marker));
    Ok(out)
}

/// `conditions.json` from `overlays/conditions_overlay.json`, checked against the maps and
/// marker types it names.
pub fn conditions(overlay: &str, maps: &[GameMap], markers: &MapMarkers) -> Result<Vec<MapCondition>> {
    #[derive(Deserialize)]
    struct Loose {
        name: String,
        major: Option<bool>,
        maps: Vec<String>,
        loot_value: f64,
        boosts: Vec<arc_core::model::MarkerBoost>,
        loot_effects: Vec<String>,
        arc_effects: Vec<String>,
        confidence: String,
    }
    #[derive(Deserialize)]
    struct File {
        conditions: Vec<Loose>,
    }
    let file: File = serde_json::from_str(overlay).context("conditions_overlay.json")?;
    file.conditions.into_iter().map(|c| {
        for m in &c.maps {
            if !maps.iter().any(|g| &g.id == m) {
                bail!("conditions_overlay.json: {} names unknown map {m}", c.name);
            }
        }
        for b in &c.boosts {
            if !markers.categories.iter().any(|k| k.id == b.marker) {
                bail!("conditions_overlay.json: {} boosts unknown marker {}", c.name, b.marker);
            }
        }
        if !CONFIDENCE.contains(&c.confidence.as_str()) {
            bail!("conditions_overlay.json: {} confidence {}", c.name, c.confidence);
        }
        Ok(MapCondition {
            name: c.name, major: c.major, maps: c.maps, loot_value: c.loot_value, boosts: c.boosts,
            loot_effects: c.loot_effects, arc_effects: c.arc_effects, confidence: c.confidence,
        })
    }).collect()
}
