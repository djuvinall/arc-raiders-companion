//! What each ARC drops, from `overlays/arc_loot.json` (researched from the wiki's per-ARC
//! pages and MetaForge). It replaces RaidTheory's drop lists, which stopped updating in
//! August 2026. Husks, probes and couriers in the same file become loot containers.

use anyhow::{bail, Context, Result};
use arc_core::model::{ArcDrop, Item};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize)]
pub struct Overlay {
    pub arcs: Vec<Entry>,
}

/// Fields beyond those read here are kept for reviewers of the overlay.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Entry {
    pub name: String,
    pub kind: String,
    /// An id in arcs.json, for live ARC.
    pub arc: Option<String>,
    /// A map marker type, for static ARC containers.
    pub marker: Option<String>,
    pub breach: Option<bool>,
    pub drops: Vec<Drop>,
    pub notes: Option<String>,
    pub disputes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Drop {
    pub item: String,
    pub sure: bool,
    pub source: String,
    pub qty: Option<String>,
    pub rate: Option<String>,
    pub note: Option<String>,
}

pub fn load(text: &str) -> Result<Overlay> {
    serde_json::from_str(text).context("arc_loot.json")
}

/// Item ids by lower-case name.
pub fn names(items: &[Item]) -> HashMap<String, String> {
    items.iter().map(|i| (i.name.to_lowercase(), i.id.clone())).collect()
}

pub fn item_id(names: &HashMap<String, String>, name: &str, whose: &str) -> Result<String> {
    match names.get(&name.to_lowercase()) {
        Some(id) => Ok(id.clone()),
        None => bail!("arc_loot.json: {whose} drops {name:?}, which no item is called"),
    }
}

/// Sets each live ARC's drops from the overlay and rebuilds every item's `drops_from`.
pub fn apply(arcs: &mut [arc_core::model::Arc], items: &mut [Item], overlay: &Overlay) -> Result<()> {
    let names = names(items);
    for entry in &overlay.arcs {
        let Some(id) = &entry.arc else { continue };
        let Some(arc) = arcs.iter_mut().find(|a| &a.id == id) else {
            bail!("arc_loot.json: no ARC {id} for {}", entry.name);
        };
        if entry.drops.is_empty() {
            continue;
        }
        arc.drops.clear();
        for drop in &entry.drops {
            let item_id = item_id(&names, &drop.item, &entry.name)?;
            if !arc.drops.iter().any(|d| d.item_id == item_id) {
                arc.drops.push(ArcDrop { item_id, chance: None });
            }
        }
    }
    let mut from: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for arc in arcs.iter() {
        for d in &arc.drops {
            from.entry(d.item_id.as_str()).or_default().push(arc.id.clone());
        }
    }
    for item in items.iter_mut() {
        item.drops_from = from.get(item.id.as_str()).cloned().unwrap_or_default();
    }
    Ok(())
}
