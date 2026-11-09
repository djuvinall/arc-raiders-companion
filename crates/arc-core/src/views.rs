//! Read models for the UI: ids resolved to names and icons, so a page never needs
//! a second query. HollowDeck will serve these same structs over HTTP.

use crate::calc::{self, TreeNode, UsedIn};
use crate::dataset::Dataset;
use crate::model::*;
use crate::weapons::{self, GunStencil, WeaponFamily};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemSummary {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub subcategory: Option<String>,
    pub rarity: Option<Rarity>,
    pub tier: Option<u8>,
    pub icon: Option<String>,
    pub loot_areas: Vec<String>,
    pub variant_of: Option<String>,
    pub value: Option<u32>,
    pub weight_kg: Option<f64>,
    /// Weapons only: every level and amplified version of a gun shares these.
    pub family: Option<String>,
    pub family_name: Option<String>,
}

pub fn item_list(ds: &Dataset) -> Vec<ItemSummary> {
    ds.files.items.iter().map(|i| ItemSummary {
        family: weapons::weapon_stats(i).map(|_| weapons::weapon_family(ds, &i.id)),
        family_name: weapons::weapon_stats(i).map(|_| weapons::weapon_family_name(ds, &i.id)),
        id: i.id.clone(),
        name: i.name.clone(),
        category: i.category,
        subcategory: i.subcategory.clone(),
        rarity: i.rarity,
        tier: i.tier,
        icon: i.icon.clone(),
        loot_areas: i.loot_areas.clone(),
        variant_of: i.variant_of.clone(),
        value: i.value,
        weight_kg: i.weight_kg,
    }).collect()
}

/// A named reference: enough to draw a linked row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Link {
    /// `item`, or an entity kind accepted by [`entity`].
    pub kind: &'static str,
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub qty: Option<u32>,
    pub note: Option<String>,
}

pub fn item_link(ds: &Dataset, id: &str, qty: Option<u32>) -> Link {
    let item = ds.item(id);
    Link {
        kind: "item",
        id: id.to_string(),
        name: ds.item_name(id).to_string(),
        icon: item.and_then(|i| i.icon.clone()),
        qty,
        note: None,
    }
}

fn entity_link(kind: &'static str, id: &str, name: &str, note: Option<String>) -> Link {
    Link { kind, id: id.to_string(), name: name.to_string(), icon: None, qty: None, note }
}

fn price_note(price: u32, currency: Option<&str>) -> String {
    match currency {
        Some(c) => format!("{price} {c}"),
        None => format!("Price {price}"),
    }
}

fn bench_name<'a>(ds: &'a Dataset, id: &'a str) -> &'a str {
    ds.files.workbenches.iter().find(|b| b.id == id).map_or(id, |b| b.name.as_str())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecipeView {
    pub id: String,
    pub kind: RecipeKind,
    pub inputs: Vec<Link>,
    pub outputs: Vec<Link>,
    pub stations: Vec<Link>,
    pub blueprint: Option<Link>,
    pub requires_blueprint: bool,
    pub required_skill: Option<String>,
    pub provenance: Provenance,
}

fn station_link(ds: &Dataset, id: &str, level: Option<u8>) -> Link {
    let note = level.map(|l| format!("Level {l}"));
    entity_link("workbench", id, bench_name(ds, id), note)
}

fn recipe_view(ds: &Dataset, r: &Recipe) -> RecipeView {
    let primary = r.workbench_id.iter().map(|id| station_link(ds, id, r.workbench_level));
    let others = r.alt_stations.iter().map(|s| station_link(ds, &s.workbench_id, s.level));
    RecipeView {
        id: r.id.clone(),
        kind: r.kind,
        inputs: r.inputs.iter().map(|q| item_link(ds, &q.item_id, Some(q.qty))).collect(),
        outputs: r.output.iter().chain(&r.outputs)
            .map(|q| item_link(ds, &q.item_id, Some(q.qty))).collect(),
        stations: primary.chain(others).collect(),
        blueprint: r.blueprint_item_id.as_ref().map(|id| item_link(ds, id, None)),
        requires_blueprint: r.requires_blueprint,
        required_skill: r.required_skill.clone(),
        provenance: r.provenance,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapPresence {
    pub map: Link,
    pub poi_count: usize,
    /// Distinct named spots, for the "Generator, Locker" line.
    pub places: Vec<String>,
    /// The spots counted, so the map shows the same ones.
    pub poi_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoundIn {
    pub area: Link,
    pub maps: Vec<MapPresence>,
}

/// `ARC Surveyor` and `Rollbot` both name `rollbot`; spawn spots say `Arc Courier`, `surveyor`,
/// and source ids `containers/wasp_husk`.
fn arc_word(name: &str) -> String {
    let n = name.trim().to_lowercase().replace('_', " ");
    let n = n.rsplit('/').next().unwrap_or(&n).to_string();
    n.strip_prefix("arc ").unwrap_or(&n).to_string()
}

/// The ARC loot area's spots are ARC spawns and wrecks. An item is only at the ones it comes
/// from: the ARCs that drop it and the husks, couriers or probes the app lists it in.
fn arc_spot_names(ds: &Dataset, item: &Item) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for id in &item.drops_from {
        names.push(arc_word(id));
        if let Some(arc) = ds.files.arcs.iter().find(|a| &a.id == id) {
            names.push(arc_word(&arc.name));
        }
    }
    for source in crate::raid::item_sources(ds, &item.id).iter().filter(|s| s.chance != "unlikely") {
        names.push(arc_word(&source.name));
        names.push(arc_word(&source.id));
    }
    names
}

/// A spot named `Wasp Husk` counts for the generic `husk` too.
fn arc_spot_matches(names: &[String], poi: &str) -> bool {
    let word = arc_word(poi);
    names.iter().any(|n| *n == word || (n == "husk" && word.ends_with(" husk")))
}

fn found_in(ds: &Dataset, item: &Item) -> Vec<FoundIn> {
    let arc_names = arc_spot_names(ds, item);
    item.loot_areas.iter().map(|area_id| {
        let name = ds.files.loot_areas.iter().find(|a| &a.id == area_id)
            .map_or(area_id.as_str(), |a| a.name.as_str());
        let mut per_map: BTreeMap<&str, Vec<&Poi>> = BTreeMap::new();
        for &i in ds.index.pois_by_loot_area.get(area_id).map(Vec::as_slice).unwrap_or(&[]) {
            let poi = &ds.files.pois[i];
            if area_id == "arc" && !arc_spot_matches(&arc_names, &poi.name) {
                continue;
            }
            per_map.entry(poi.map_id.as_str()).or_default().push(poi);
        }
        let maps = per_map.into_iter().map(|(map_id, pois)| {
            let map_name = ds.files.maps.iter().find(|m| m.id == map_id)
                .map_or(map_id, |m| m.name.as_str());
            let mut places: Vec<String> = pois.iter().map(|p| p.name.clone()).collect();
            places.sort();
            places.dedup();
            let poi_ids = pois.iter().map(|p| p.id.clone()).collect();
            MapPresence { map: entity_link("map", map_id, map_name, None), poi_count: pois.len(), places, poi_ids }
        }).collect();
        FoundIn { area: entity_link("loot-area", area_id, name, None), maps }
    }).collect()
}

/// The loot areas' maps merged, so a map with spots from two areas shows once.
fn found_maps(found: &[FoundIn]) -> Vec<MapPresence> {
    let mut out: Vec<MapPresence> = Vec::new();
    for presence in found.iter().flat_map(|f| &f.maps) {
        match out.iter_mut().find(|m| m.map.id == presence.map.id) {
            Some(m) => {
                m.poi_count += presence.poi_count;
                m.places.extend(presence.places.iter().cloned());
                m.places.sort();
                m.places.dedup();
                m.poi_ids.extend(presence.poi_ids.iter().cloned());
            }
            None => out.push(presence.clone()),
        }
    }
    out.sort_by(|a, b| b.poi_count.cmp(&a.poi_count).then_with(|| a.map.name.cmp(&b.map.name)));
    out
}

/// Where a key's door is on one map: marker indexes into that map's marker list.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DoorPresence {
    pub map: Link,
    pub markers: Vec<usize>,
    pub places: Vec<String>,
}

/// Words that say nothing about which door it is.
const DOOR_NOISE: &[&str] = &["key", "keys", "keycard", "room", "keyroom", "no", "the", "locked", "door", "of"];
/// A door marked with one of these needs something else, such as a fuel cell, not a key.
const OTHER_LOCKS: &[&str] = &["fuel", "cell", "puzzle", "capacitor", "power", "printer", "code"];

fn door_words(text: &str, drop: &[String]) -> Vec<String> {
    text.to_lowercase().replace(['\'', '\u{2019}'], "")
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !DOOR_NOISE.contains(w) && !drop.iter().any(|d| d == w))
        .map(str::to_string)
        .collect()
}

/// The doors a key opens, matched by name against the map's locked-room markers. Marker
/// labels are hand-typed upstream (with typos), so a door matches when every word that
/// names the key's room is in the label. Keys for a kind of thing (every Raider Hatch)
/// point at that marker type instead.
fn key_doors(ds: &Dataset, item: &Item) -> Vec<DoorPresence> {
    if item.category != Category::Key {
        return Vec::new();
    }
    let markers = &ds.files.markers;
    let category = |id: &str| markers.categories.iter().position(|c| c.id == id);
    let lower = item.name.to_lowercase();
    let whole_type = if lower.contains("hatch") {
        category("locations/hatch")
    } else if lower.contains("bunker") {
        category("events/bunker")
    } else {
        None
    };
    // Security codes open the locked gates, which are named after their place.
    let code = lower.contains("security code");
    let doors = if code { [category("events/locked_gate_key")] } else { [category("locations/locked_room")] };
    let home = ds.files.maps.iter().find(|m| {
        let name = m.name.to_lowercase();
        let short = name.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
        lower.starts_with(&short) || lower.starts_with(&m.id.replace('-', " "))
    });
    let map_words: Vec<String> = ds.files.maps.iter()
        .flat_map(|m| door_words(&format!("{} {}", m.name, m.id), &[]))
        .collect();
    let mut wanted = door_words(&item.name, &map_words);
    if code {
        wanted.retain(|w| w != "security" && w != "code");
    }
    if wanted.is_empty() && whole_type.is_none() {
        return Vec::new();
    }
    markers.maps.iter()
        .filter(|(map_id, _)| whole_type.is_some() || code || home.is_none_or(|h| &h.id == *map_id))
        .filter_map(|(map_id, list)| {
            let mut hits = Vec::new();
            let mut places = Vec::new();
            for (i, MapMarker(index, _, _, _, label)) in list.iter().enumerate() {
                let index = Some(*index as usize);
                let matched = if whole_type.is_some() {
                    index == whole_type
                } else if doors.contains(&index) {
                    let Some(label) = label else { continue };
                    let words = door_words(label, &[]);
                    wanted.iter().all(|w| words.contains(w))
                        && !words.iter().any(|w| OTHER_LOCKS.contains(&w.as_str()) && !wanted.contains(w))
                } else {
                    false
                };
                if matched {
                    hits.push(i);
                    if let Some(label) = label.as_deref().filter(|l| !l.trim().is_empty()) {
                        places.push(label.trim().to_string());
                    }
                }
            }
            if hits.is_empty() {
                return None;
            }
            places.sort();
            places.dedup();
            let name = ds.files.maps.iter().find(|m| &m.id == map_id).map_or(map_id.as_str(), |m| m.name.as_str());
            Some(DoorPresence { map: entity_link("map", map_id, name, None), markers: hits, places })
        })
        .collect()
}

/// What a weapon mod does and which guns take it, worded like the in-game tooltip.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModInfo {
    pub slot: String,
    /// Gun families, by display name ("Il Toro"), whose slot takes this mod's tier.
    pub compatible: Vec<String>,
    /// "40% Reduced Per-Shot Dispersion", "+4 Magazine Size".
    pub effects: Vec<String>,
}

fn title_words(snake: &str) -> String {
    snake.split('_').map(|w| {
        let mut c = w.chars();
        c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
    }).collect::<Vec<_>>().join(" ")
}

fn mod_info(ds: &Dataset, item: &Item) -> Option<ModInfo> {
    let m = ds.files.weapon_mods.iter().find(|m| m.id == item.id)?;
    let mut compatible: Vec<String> = Vec::new();
    for gun in ds.files.items.iter().filter(|i| i.category == Category::Weapon && i.variant_of.is_none()) {
        let Some(Stats::Weapon(stats)) = &gun.stats else { continue };
        if stats.mod_slots.iter().any(|slot| weapons::fits(ds, &gun.id, slot, m)) {
            let family = weapons::weapon_family_name(ds, &gun.id);
            if !compatible.contains(&family) {
                compatible.push(family);
            }
        }
    }
    compatible.sort();
    let mut effects: Vec<String> = m.modifiers.iter().map(|x| {
        let key = serde_json::to_value(x.stat).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
        // The game's own wording where it differs from the stat's name.
        let stat = match key.as_str() {
            "per_shot_dispersion" => "Per-Shot Dispersion".to_string(),
            "falloff_distance" => "Distance Until Damage Falloff".to_string(),
            _ => title_words(&key),
        };
        format!("{}% {} {stat}", x.pct.abs(), if x.pct < 0.0 { "Reduced" } else { "Increased" })
    }).collect();
    if m.flat.magazine_size != 0 {
        effects.push(format!("{:+} Magazine Size", m.flat.magazine_size));
    }
    Some(ModInfo { slot: title_words(&m.slot), compatible, effects })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemDetail {
    pub item: Item,
    pub crafting: Vec<RecipeView>,
    pub breakdown: Vec<RecipeView>,
    pub used_in: UsedIn,
    pub found_in: Vec<FoundIn>,
    /// Container types and ARCs that can hold it, best chance first.
    pub sources: Vec<crate::raid::ItemSource>,
    /// Each map once, with every named spot from all its loot areas.
    pub found_maps: Vec<MapPresence>,
    /// Keys only: the doors it opens.
    pub opens: Vec<DoorPresence>,
    /// Weapon mods only.
    pub mod_info: Option<ModInfo>,
    /// Weapons only: the ammo it takes, and one tile per mod slot (the slot name, with the
    /// icon of the lowest-tier mod that fits, like the game's empty-slot silhouettes).
    pub ammo: Option<Link>,
    pub mod_slots: Vec<Link>,
    /// Hand-entered where-to-find note, for items no source places.
    pub note: Option<ItemNote>,
    pub sold_by: Vec<Link>,
    pub dropped_by: Vec<Link>,
    pub base: Option<Link>,
    pub family: Option<WeaponFamily>,
    pub stencil_guns: Vec<Link>,
    pub gun_stencils: Vec<GunStencil>,
    pub quest_rewards: Vec<Link>,
    pub project_rewards: Vec<Link>,
}

pub fn item_detail(ds: &Dataset, id: &str) -> Option<ItemDetail> {
    let item = ds.item(id)?.clone();
    let recipes = |ids: &[String]| -> Vec<RecipeView> {
        ids.iter().filter_map(|r| ds.recipe(r)).map(|r| recipe_view(ds, r)).collect()
    };
    let sold_by = item.sold_by.iter().map(|s| {
        let name = ds.files.traders.iter().find(|t| t.id == s.trader).map_or("", |t| t.name.as_str());
        entity_link("trader", &s.trader, name, Some(price_note(s.price, s.currency.as_deref())))
    }).collect();
    let dropped_by = item.drops_from.iter().map(|a| {
        let name = ds.files.arcs.iter().find(|x| &x.id == a).map_or(a.as_str(), |x| x.name.as_str());
        entity_link("arc", a, name, None)
    }).collect();
    let quest_rewards = ds.files.quests.iter()
        .filter(|q| q.rewards.iter().any(|r| r.item_id == id))
        .map(|q| entity_link("quest", &q.id, &q.name, None))
        .collect();
    let project_rewards = ds.files.projects.iter()
        .flat_map(|p| p.stages.iter().map(move |s| (p, s)))
        .filter(|(_, s)| s.rewards.iter().any(|r| r.item_id == id))
        .map(|(p, s)| entity_link("project", &p.id, &p.name, Some(s.title.clone())))
        .collect();
    let found = found_in(ds, &item);
    Some(ItemDetail {
        crafting: recipes(&item.recipe_ids),
        breakdown: recipes(&item.recycle_ids),
        used_in: calc::used_in(ds, id),
        found_maps: found_maps(&found),
        sources: crate::raid::item_sources(ds, id),
        found_in: found,
        opens: key_doors(ds, &item),
        mod_info: mod_info(ds, &item),
        ammo: item.ammo_type.as_ref().map(|a| format!("{}-ammo", a.to_lowercase()))
            .filter(|a| ds.item(a).is_some()).map(|a| item_link(ds, &a, None)),
        mod_slots: weapons::slot_options(ds, id).unwrap_or_default().into_iter().map(|s| {
            let pick = s.mods.iter().min_by_key(|m| m.tier).filter(|m| ds.item(&m.id).is_some());
            let mut link = pick.map_or_else(
                || Link { kind: "item", id: String::new(), name: String::new(), icon: None, qty: None, note: None },
                |m| item_link(ds, &m.id, None),
            );
            link.name = slot_label(&s.slot);
            link.note = Some(s.slot);
            link
        }).collect(),
        note: ds.files.notes.items.get(id).cloned(),
        base: item.variant_of.as_ref().map(|b| item_link(ds, b, None)),
        family: weapons::weapon_family_view(ds, id),
        gun_stencils: weapons::gun_stencils(ds, id),
        stencil_guns: item.stencil.as_ref().map_or(Vec::new(), |s| weapons::stencil_guns(ds, s)),
        sold_by,
        dropped_by,
        quest_rewards,
        project_rewards,
        item,
    })
}

/// "underbarrel" or "shotgun-muzzle" as the game names the slot.
fn slot_label(slot: &str) -> String {
    slot.split(['-', '_']).map(|w| {
        let mut c = w.chars();
        c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
    }).collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CraftPlan {
    pub tree: TreeNode,
    pub totals: Vec<Link>,
    pub root: Link,
}

fn annotate(ds: &Dataset, node: &mut TreeNode) {
    if let Some(id) = node.workbench_id.clone() {
        node.workbench_id = Some(bench_name(ds, &id).to_string());
    }
    node.children.iter_mut().for_each(|c| annotate(ds, c));
}

pub fn craft_plan(ds: &Dataset, id: &str, qty: u32, owned: Option<&str>) -> Option<CraftPlan> {
    let mut tree = calc::craft_tree_from(ds, id, qty, owned)?;
    let mut totals: Vec<Link> = calc::raw_totals(&tree).into_iter()
        .map(|(item, n)| item_link(ds, &item, Some(n)))
        .collect();
    totals.sort_by(|a, b| b.qty.cmp(&a.qty).then_with(|| a.name.cmp(&b.name)));
    annotate(ds, &mut tree);
    Some(CraftPlan { root: item_link(ds, id, Some(qty)), totals, tree })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntitySection {
    pub title: String,
    pub text: Vec<String>,
    pub links: Vec<Link>,
}

/// Generic page for what has no dedicated route before Phases 4–5, so that every
/// name on an item page is a link.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntityPage {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub sections: Vec<EntitySection>,
}

fn section(title: impl Into<String>, links: Vec<Link>) -> EntitySection {
    EntitySection { title: title.into(), text: Vec::new(), links }
}

pub(crate) fn lines(ds: &Dataset, rows: &[ItemQty]) -> Vec<Link> {
    rows.iter().map(|q| item_link(ds, &q.item_id, Some(q.qty))).collect()
}

pub fn entity(ds: &Dataset, kind: &str, id: &str) -> Option<EntityPage> {
    let f = &ds.files;
    let (name, subtitle, sections) = match kind {
        "workbench" => {
            let b = f.workbenches.iter().find(|x| x.id == id)?;
            let sections = b.levels.iter().flat_map(|l| {
                let unlocks = l.unlocks_recipe_ids.iter()
                    .filter_map(|r| ds.recipe(r))
                    .filter_map(|r| r.output.as_ref())
                    .map(|o| item_link(ds, &o.item_id, None))
                    .collect();
                [
                    section(format!("Level {} upgrade", l.level), lines(ds, &l.upgrade_inputs)),
                    section(format!("Level {} recipes", l.level), unlocks),
                ]
            }).filter(|s| !s.links.is_empty()).collect();
            (b.name.clone(), format!("Workbench, {} levels", b.max_level), sections)
        }
        "loot-area" => {
            let a = f.loot_areas.iter().find(|x| x.id == id)?;
            let items = a.item_ids.iter().map(|i| item_link(ds, i, None)).collect();
            (a.name.clone(), "Loot area".to_string(), vec![section("Items", items)])
        }
        "map" => {
            let m = f.maps.iter().find(|x| x.id == id)?;
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            for p in f.pois.iter().filter(|p| p.map_id == m.id) {
                for a in &p.loot_areas {
                    *counts.entry(a.as_str()).or_default() += 1;
                }
            }
            let areas = counts.into_iter().map(|(a, n)| {
                let name = f.loot_areas.iter().find(|x| x.id == a).map_or(a, |x| x.name.as_str());
                entity_link("loot-area", a, name, Some(format!("{n} spots")))
            }).collect();
            let events = EntitySection {
                title: "Events".into(), text: m.event_pool.clone(), links: Vec::new(),
            };
            (m.name.clone(), "Map".to_string(), vec![section("Loot areas", areas), events])
        }
        "trader" => {
            let t = f.traders.iter().find(|x| x.id == id)?;
            let stock = t.inventory.iter().map(|s| {
                let mut link = item_link(ds, &s.item_id, None);
                link.note = Some(price_note(s.price, s.currency.as_deref()));
                link
            }).collect();
            (t.name.clone(), "Trader".to_string(), vec![section("Sells", stock)])
        }
        "arc" => {
            let a = f.arcs.iter().find(|x| x.id == id)?;
            let drops = a.drops.iter().map(|d| item_link(ds, &d.item_id, None)).collect();
            let subtitle = a.class.clone().unwrap_or_else(|| "ARC".to_string());
            (a.name.clone(), subtitle, vec![section("Drops", drops)])
        }
        "quest" => {
            let q = f.quests.iter().find(|x| x.id == id)?;
            let text = q.objectives.iter().filter_map(|o| o.description.clone()).collect();
            let prereqs = q.prerequisites.iter().filter_map(|p| {
                f.quests.iter().find(|x| x.id == p.quest_id)
            }).map(|x| entity_link("quest", &x.id, &x.name, None)).collect();
            let sections = vec![
                EntitySection { title: "Objectives".into(), text, links: Vec::new() },
                section("Requires", lines(ds, &q.required_items)),
                section("Rewards", lines(ds, &q.rewards)),
                section("Prerequisites", prereqs),
            ];
            let trader = q.trader.clone().unwrap_or_default().to_uppercase();
            (q.name.clone(), format!("Quest, {trader}"), sections)
        }
        "project" => {
            let p = f.projects.iter().find(|x| x.id == id)?;
            let sections = p.stages.iter().flat_map(|s| [
                section(format!("{}: needs", s.title), lines(ds, &s.inputs)),
                section(format!("{}: rewards", s.title), lines(ds, &s.rewards)),
            ]).collect();
            let subtitle = p.deadline_days.map_or("Project".to_string(), |d| format!("Project, {d} days"));
            (p.name.clone(), subtitle, sections)
        }
        _ => return None,
    };
    let sections = sections.into_iter()
        .filter(|s: &EntitySection| !s.links.is_empty() || !s.text.is_empty())
        .collect();
    Some(EntityPage { kind: kind.to_string(), id: id.to_string(), name, subtitle, sections })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Overview {
    pub dataset_version: String,
    pub game_patch: Option<String>,
    pub item_count: usize,
    pub recipe_count: usize,
    pub loot_areas: Vec<LootArea>,
}

pub fn overview(ds: &Dataset) -> Overview {
    Overview {
        dataset_version: ds.files.version.dataset_version.clone(),
        game_patch: ds.files.version.game_patch.clone(),
        item_count: ds.files.items.len(),
        recipe_count: ds.files.recipes.len(),
        loot_areas: ds.files.loot_areas.clone(),
    }
}
