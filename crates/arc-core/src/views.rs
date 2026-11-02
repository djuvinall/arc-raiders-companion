//! Read models for the UI: ids resolved to names and icons, so a page never needs
//! a second query. HollowDeck will serve these same structs over HTTP.

use crate::calc::{self, TreeNode, UsedIn};
use crate::dataset::Dataset;
use crate::model::*;
use crate::weapons::{self, WeaponFamily};
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

pub(crate) fn item_link(ds: &Dataset, id: &str, qty: Option<u32>) -> Link {
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
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoundIn {
    pub area: Link,
    pub maps: Vec<MapPresence>,
}

fn found_in(ds: &Dataset, item: &Item) -> Vec<FoundIn> {
    item.loot_areas.iter().map(|area_id| {
        let name = ds.files.loot_areas.iter().find(|a| &a.id == area_id)
            .map_or(area_id.as_str(), |a| a.name.as_str());
        let mut per_map: BTreeMap<&str, Vec<&Poi>> = BTreeMap::new();
        for &i in ds.index.pois_by_loot_area.get(area_id).map(Vec::as_slice).unwrap_or(&[]) {
            let poi = &ds.files.pois[i];
            per_map.entry(poi.map_id.as_str()).or_default().push(poi);
        }
        let maps = per_map.into_iter().map(|(map_id, pois)| {
            let map_name = ds.files.maps.iter().find(|m| m.id == map_id)
                .map_or(map_id, |m| m.name.as_str());
            let mut places: Vec<String> = pois.iter().map(|p| p.name.clone()).collect();
            places.sort();
            places.dedup();
            MapPresence { map: entity_link("map", map_id, map_name, None), poi_count: pois.len(), places }
        }).collect();
        FoundIn { area: entity_link("loot-area", area_id, name, None), maps }
    }).collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemDetail {
    pub item: Item,
    pub crafting: Vec<RecipeView>,
    pub breakdown: Vec<RecipeView>,
    pub used_in: UsedIn,
    pub found_in: Vec<FoundIn>,
    pub sold_by: Vec<Link>,
    pub dropped_by: Vec<Link>,
    pub base: Option<Link>,
    pub family: Option<WeaponFamily>,
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
    Some(ItemDetail {
        crafting: recipes(&item.recipe_ids),
        breakdown: recipes(&item.recycle_ids),
        used_in: calc::used_in(ds, id),
        found_in: found_in(ds, &item),
        base: item.variant_of.as_ref().map(|b| item_link(ds, b, None)),
        family: weapons::weapon_family_view(ds, id),
        sold_by,
        dropped_by,
        quest_rewards,
        project_rewards,
        item,
    })
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

pub fn craft_plan(ds: &Dataset, id: &str, qty: u32) -> Option<CraftPlan> {
    let mut tree = calc::craft_tree(ds, id, qty)?;
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
