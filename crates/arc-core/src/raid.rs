//! Raid plans: which parts of a map are worth heading for, given what you're after.
//!
//! **The spawn model.** No source publishes drop rates, so every container gets a relative
//! score per item from three things: its type (what that kind of container holds), its zone
//! (the loot-area tags of the named spots around it) and the item's rarity. The scores rank
//! places against each other; they are never a promise that an item is there.
//!
//! **Target areas, not routes from a spawn.** You don't know your spawn until you're in, so
//! a plan is a ranked list of areas with a short loop through each and its nearest exits.
//! Distances are straight lines: there is no walkable map to route around cliffs. All pure.

use crate::calc::{craft_tree_from, raw_totals};
use crate::dataset::Dataset;
use crate::model::*;
use crate::views::{item_link, Link};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Tagged spots within this distance give a container their zone.
const ZONE_RADIUS: f64 = 200.0;
/// A container in a matching zone counts this much more.
const ZONE_BONUS: f64 = 1.6;
/// Scores at or above this read as "likely", at or above `POSSIBLE` as "possible".
const LIKELY: f64 = 0.9;
const POSSIBLE: f64 = 0.4;
/// Area centres tried per map, best containers first.
const CENTRES: usize = 1500;
/// Each quest stop inside an area adds this much to its value.
const QUEST_STOP_VALUE: f64 = 0.75;

/// An item a container's own loot table names, when the table is sure of it.
const LISTED: f64 = 1.5;
/// Named, but the source flags it unconfirmed or only one source has it.
const LISTED_UNSURE: f64 = 1.0;
/// Containers tied to a map condition are only there some raids.
const CONDITION_ONLY: f64 = 0.35;

/// Plants and the item they give, where the names differ.
const NATURE_ITEMS: &[(&str, &str)] = &[("olive", "olives")];
const PLANT: f64 = 2.0;
const ARC_DROP: f64 = 1.5;

fn rarity_factor(r: Option<Rarity>) -> f64 {
    match r {
        Some(Rarity::Common) => 1.0,
        Some(Rarity::Uncommon) => 0.7,
        Some(Rarity::Rare) => 0.45,
        Some(Rarity::Epic) => 0.25,
        Some(Rarity::Legendary) => 0.12,
        None => 0.8,
    }
}

fn category_tag(c: Category) -> String {
    serde_json::to_value(c).ok().and_then(|v| v.as_str().map(|s| format!("cat:{s}"))).unwrap_or_default()
}

/// One thing a raid is for, with the items it wants.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Source {
    /// `quest:<id>`, `craft:<id>`, `project:<id>`, `bench:<id>` or `extra:<id>`.
    pub key: String,
    /// `quest`, `craft`, `project`, `bench` or `extra`.
    pub kind: &'static str,
    pub name: String,
    pub items: Vec<Link>,
    pub on: bool,
}

/// A source as (key, kind, name, items with quantities).
type Raw = (String, &'static str, String, Vec<(String, u32)>);

/// Everything tracked, by source, before the stash is taken off.
fn sources(ds: &Dataset, profile: &Profile, extra: &[String]) -> Vec<Raw> {
    let mut out = Vec::new();
    for id in &profile.tracked_item_ids {
        let owned = profile.tracked_from.get(id).map(String::as_str);
        let Some(tree) = craft_tree_from(ds, id, 1, owned) else { continue };
        let items = if tree.children.is_empty() { vec![(id.clone(), 1)] } else { raw_totals(&tree).into_iter().collect() };
        out.push((format!("craft:{id}"), "craft", ds.item_name(id).to_string(), items));
    }
    for quest in ds.files.quests.iter().filter(|q| profile.quests.get(&q.id).is_some_and(|p| !p.done)) {
        let items = quest.required_items.iter().map(|q| (q.item_id.clone(), q.qty)).collect();
        out.push((format!("quest:{}", quest.id), "quest", quest.name.clone(), items));
    }
    for project in &ds.files.projects {
        let Some(progress) = profile.projects.get(&project.id) else { continue };
        let Some(stage) = project.stages.iter().find(|s| s.n == progress.stage) else { continue };
        let items: Vec<_> = stage.inputs.iter().filter_map(|q| {
            let left = q.qty.saturating_sub(progress.committed.get(&q.item_id).copied().unwrap_or(0));
            (left > 0).then(|| (q.item_id.clone(), left))
        }).collect();
        if !items.is_empty() {
            out.push((format!("project:{}", project.id), "project", format!("{} stage {}", project.name, stage.n), items));
        }
    }
    for bench in ds.files.workbenches.iter().filter(|b| profile.tracked_benches.contains(&b.id)) {
        let next = profile.workbenches.get(&bench.id).copied().unwrap_or(0) + 1;
        let Some(level) = bench.levels.iter().find(|l| l.level == next) else { continue };
        let items = level.upgrade_inputs.iter().map(|q| (q.item_id.clone(), q.qty)).collect();
        out.push((format!("bench:{}", bench.id), "bench", format!("{} level {next}", bench.name), items));
    }
    for id in extra.iter().filter(|i| ds.item(i).is_some()) {
        out.push((format!("extra:{id}"), "extra", ds.item_name(id).to_string(), vec![(id.clone(), 1)]));
    }
    out
}

/// An item the plan looks for.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Want {
    pub item: Link,
    /// Still to find after the stash.
    pub qty: u32,
    pub on: bool,
    /// False when nothing in the model can hold it: no loot area, plant or ARC drop.
    pub placeable: bool,
}

struct Wanted<'a> {
    item: &'a Item,
    qty: u32,
    tags: BTreeSet<String>,
}

fn tags(ds: &Dataset, item: &Item) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = item.loot_areas.iter().cloned().collect();
    out.extend(ds.files.loot_areas.iter().filter(|a| a.item_ids.contains(&item.id)).map(|a| a.id.clone()));
    out.insert(category_tag(item.category));
    out
}

/// How well one container type suits one item, before zone and rarity. Container types,
/// what they hold and how strongly come from containers.json.
fn affinity(ds: &Dataset, category: &str, w: &Wanted, live: &[&MapCondition]) -> f64 {
    if let Some(plant) = category.strip_prefix("nature/") {
        let id = NATURE_ITEMS.iter().find(|(p, _)| *p == plant).map_or(plant, |(_, i)| i);
        return if w.item.id == id { PLANT } else { 0.0 };
    }
    if let Some(arc) = category.strip_prefix("arc/") {
        return if w.item.drops_from.iter().any(|a| a == arc) { ARC_DROP } else { 0.0 };
    }
    let Some(container) = ds.files.containers.iter().find(|c| c.marker == category && c.loot) else { return 0.0 };
    let on_now = container.conditions.iter().any(|c| live.iter().any(|l| l.name.eq_ignore_ascii_case(c)));
    let condition = if container.conditions.is_empty() || container.group != "condition" || on_now { 1.0 } else { CONDITION_ONLY };
    if let Some(listed) = container.items.iter().find(|i| i.item_id == w.item.id) {
        return condition * if listed.sure { LISTED } else { LISTED_UNSURE };
    }
    let spawns = w.tags.iter().any(|t| !t.starts_with("cat:"));
    condition * container.pool.iter().map(|p| match p.tag.as_str() {
        "*" if spawns => p.weight,
        t if w.tags.contains(t) => p.weight,
        _ => 0.0,
    }).fold(0.0, f64::max)
}

/// The item is named in this source's own short list: an ARC's drops or a container's loot
/// table. A short list makes even a rare item a fair bet, so rarity counts for less.
fn named(ds: &Dataset, category: &str, item: &Item) -> bool {
    if let Some(arc) = category.strip_prefix("arc/") {
        return item.drops_from.iter().any(|a| a == arc);
    }
    ds.files.containers.iter().any(|c| c.marker == category && c.items.iter().any(|i| i.item_id == item.id))
}

fn rarity_at(ds: &Dataset, category: &str, item: &Item) -> f64 {
    let r = rarity_factor(item.rarity);
    if named(ds, category, item) { r.sqrt() } else { r }
}

fn modelled(ds: &Dataset, category: &str) -> bool {
    category.starts_with("nature/") || category.starts_with("arc/")
        || ds.files.containers.iter().any(|c| c.marker == category && c.loot)
}

/// One kind of place an item can come from: a container type or an ARC.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemSource {
    /// The marker category, as `containers/locker` or `arc/hornet`.
    pub id: String,
    pub name: String,
    /// `containers`, `arc`, `nature` or `locations`.
    pub group: String,
    /// `likely`, `possible` or `unlikely`, in the best case: inside one of the item's loot areas.
    pub chance: &'static str,
    pub score: f64,
    /// Maps with at least one of these.
    pub maps: Vec<String>,
    /// "Breach", "Needs the Security Breach skill", "During Cold Snap", "Named in its loot table".
    pub notes: Vec<String>,
}

/// Everything the spawn model says can hold an item, best chance first. Same scores as the
/// raid planner, so the item page and a plan never disagree.
pub fn item_sources(ds: &Dataset, item_id: &str) -> Vec<ItemSource> {
    let Some(item) = ds.item(item_id) else { return Vec::new() };
    let w = Wanted { item, qty: 1, tags: tags(ds, item) };
    let zoned = if w.tags.iter().any(|t| !t.starts_with("cat:")) { ZONE_BONUS } else { 1.0 };
    let mut out: Vec<ItemSource> = ds.files.markers.categories.iter().enumerate().filter_map(|(i, c)| {
        let base = affinity(ds, &c.id, &w, &[]);
        if base == 0.0 {
            return None;
        }
        // ARC drops and plants don't depend on the zone around them.
        let zone = if c.id.starts_with("arc/") || c.id.starts_with("nature/") { 1.0 } else { zoned };
        let score = base * zone * rarity_at(ds, &c.id, item);
        let maps: Vec<String> = ds.files.maps.iter()
            .filter(|m| ds.files.markers.maps.get(&m.id).is_some_and(|list| list.iter().any(|mk| usize::from(mk.0) == i)))
            .map(|m| m.id.clone())
            .collect();
        let mut notes = Vec::new();
        if let Some(box_) = ds.files.containers.iter().find(|k| k.marker == c.id) {
            if box_.items.iter().any(|l| l.item_id == item.id) {
                notes.push("Named in its loot table".to_string());
            }
            if let Some(needs) = &box_.needs {
                notes.push(format!("Needs the {needs}"));
            } else if box_.breach {
                notes.push("Breach to open".to_string());
            }
            if !box_.conditions.is_empty() {
                let when = if box_.group == "condition" { "During" } else { "More during" };
                notes.push(format!("{when} {}", box_.conditions.join(" or ")));
            }
        } else if c.id.starts_with("arc/") {
            notes.push("Drops when destroyed".to_string());
        }
        (!maps.is_empty()).then(|| ItemSource {
            id: c.id.clone(), name: c.name.clone(), group: c.group.clone(), chance: chance(score), score, maps, notes,
        })
    }).collect();
    out.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    out
}

/// The label a score reads as.
pub fn chance(score: f64) -> &'static str {
    if score >= LIKELY {
        "likely"
    } else if score >= POSSIBLE {
        "possible"
    } else {
        "unlikely"
    }
}

/// Buckets points by cell so neighbours within one radius are a 3×3 lookup.
struct Grid {
    cell: f64,
    cells: HashMap<(i64, i64), Vec<usize>>,
}

impl Grid {
    fn new(points: impl Iterator<Item = (usize, f64, f64)>, cell: f64) -> Self {
        let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (i, x, y) in points {
            cells.entry(((x / cell).floor() as i64, (y / cell).floor() as i64)).or_default().push(i);
        }
        Self { cell, cells }
    }

    fn near(&self, x: f64, y: f64) -> impl Iterator<Item = usize> + '_ {
        let (cx, cy) = ((x / self.cell).floor() as i64, (y / self.cell).floor() as i64);
        (-1..=1).flat_map(move |dx| (-1..=1).map(move |dy| (cx + dx, cy + dy)))
            .filter_map(|c| self.cells.get(&c)).flatten().copied()
    }
}

/// A container or plant worth opening, scored per wanted item.
struct Spot<'a> {
    key: String,
    category: &'a MarkerCategory,
    x: f64,
    y: f64,
    floors: Option<u32>,
    scores: Vec<f64>,
}

/// The conditions live on a map, matched by name against the schedule's event names.
fn live_on<'a>(ds: &'a Dataset, map_id: &str, live: &[String]) -> Vec<&'a MapCondition> {
    ds.files.conditions.iter()
        .filter(|c| c.maps.iter().any(|m| m == map_id))
        .filter(|c| live.iter().any(|l| l.trim().eq_ignore_ascii_case(&c.name)))
        .collect()
}

fn rare_or_better(r: Option<Rarity>) -> bool {
    matches!(r, Some(Rarity::Rare | Rarity::Epic | Rarity::Legendary))
}

fn spots<'a>(ds: &'a Dataset, map_id: &str, wants: &[Wanted], off_boxes: &BTreeSet<&str>, live: &[&MapCondition]) -> Vec<Spot<'a>> {
    let markers = ds.files.markers.maps.get(map_id).map_or(&[][..], |m| &m[..]);
    let zone_pois: Vec<&Poi> = ds.files.pois.iter().filter(|p| p.map_id == map_id && !p.loot_areas.is_empty()).collect();
    let zones = Grid::new(zone_pois.iter().enumerate().map(|(i, p)| (i, p.x, p.y)), ZONE_RADIUS);
    markers.iter().enumerate().filter_map(|(i, m)| {
        let category = ds.files.markers.categories.get(usize::from(m.0))?;
        if !modelled(ds, &category.id) || off_boxes.contains(category.id.as_str()) {
            return None;
        }
        let zone: BTreeSet<&str> = zones.near(m.1, m.2)
            .filter(|&p| (zone_pois[p].x - m.1).hypot(zone_pois[p].y - m.2) <= ZONE_RADIUS)
            .flat_map(|p| zone_pois[p].loot_areas.iter().map(String::as_str)).collect();
        let scores: Vec<f64> = wants.iter().map(|w| {
            let base = affinity(ds, &category.id, w, live)
                * live.iter().flat_map(|c| &c.boosts).filter(|b| b.marker == category.id).map(|b| b.factor).product::<f64>();
            if base == 0.0 {
                return 0.0;
            }
            let zoned = if w.tags.iter().any(|t| zone.contains(t.as_str())) { ZONE_BONUS } else { 1.0 };
            let value = if rare_or_better(w.item.rarity) { live.iter().map(|c| c.loot_value).product::<f64>() } else { 1.0 };
            base * zoned * rarity_at(ds, &category.id, w.item) * value
        }).collect();
        scores.iter().any(|s| *s > 0.0).then(|| Spot {
            key: format!("{map_id}:{i}"), category, x: m.1, y: m.2, floors: m.3, scores,
        })
    }).collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stop {
    pub key: String,
    /// `container`, `plant`, `arc`, `quest` or `custom`.
    pub kind: &'static str,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub floors: Option<u32>,
    /// What it might give, as "Rubber Parts (likely)".
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemCover {
    pub item_id: String,
    pub name: String,
    pub likely: u32,
    pub possible: u32,
    /// Enough chances for the confidence setting.
    pub enough: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Exit {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TargetArea {
    /// A, B, C…
    pub label: String,
    /// The nearest named place.
    pub name: String,
    pub map_id: String,
    pub map_name: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    /// Share of what you want this area gives the chances for, 0 to 1 (more with quest stops).
    pub value: f64,
    /// Chances against what's wanted, uncapped: 2 means twice the chances asked for.
    pub depth: f64,
    pub stops: Vec<Stop>,
    pub covers: Vec<ItemCover>,
    pub exits: Vec<Exit>,
    /// Straight-line length of the loop through the stops.
    pub walk: f64,
    /// With a primary objective: how far past the area's edge it is, in map units.
    pub detour: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BoxType {
    /// `box:<marker type>`.
    pub key: String,
    pub name: String,
    pub count: usize,
    pub on: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RaidAuto {
    pub sources: Vec<Source>,
    pub wants: Vec<Want>,
    pub boxes: Vec<BoxType>,
    pub areas: Vec<TargetArea>,
    /// Quest steps on this map, for quests that are switched on.
    pub quest_stops: Vec<Stop>,
    /// The plan's primary objective as a stop, when it has one on this map.
    pub primary: Option<Stop>,
    /// Conditions live on this map now that the plan counted.
    pub conditions: Vec<ConditionNote>,
}

/// One map's case for a raid: its best area and what else is there for you.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapPick {
    pub map_id: String,
    pub map_name: String,
    /// Ranking score: the best area's worth plus a little for every other quest step.
    pub score: f64,
    pub best: Option<TargetArea>,
    /// Quest steps on this map for quests that are switched on.
    pub quest_steps: usize,
    /// Items with enough chances somewhere on the map.
    pub items_here: Vec<String>,
    /// Wanted items nothing on this map can hold.
    pub items_missing: Vec<String>,
    /// Conditions live there now that the ranking counted.
    pub conditions: Vec<ConditionNote>,
}

/// A live condition and what it changes, for showing beside a plan.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConditionNote {
    pub name: String,
    pub effects: Vec<String>,
}

fn condition_note(c: &MapCondition) -> ConditionNote {
    let mut effects = Vec::new();
    if c.loot_value > 1.0 {
        effects.push("Increased loot value: rare and better items count for more".to_string());
    } else if c.loot_value < 1.0 {
        effects.push("Decreased loot value".to_string());
    }
    for b in &c.boosts {
        effects.push(format!("More {} than usual", b.marker.rsplit('/').next().unwrap_or(&b.marker).replace('_', " ")));
    }
    effects.extend(c.loot_effects.iter().take(2).cloned());
    ConditionNote { name: c.name.clone(), effects }
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Nearest-neighbour from `start`, then 2-opt until no swap shortens the path.
pub fn order(points: &[(f64, f64)], start: usize) -> Vec<usize> {
    if points.len() < 3 {
        let mut all: Vec<usize> = (0..points.len()).collect();
        all.sort_by_key(|&i| usize::from(i != start));
        return all;
    }
    let mut path = vec![start];
    let mut left: Vec<usize> = (0..points.len()).filter(|&i| i != start).collect();
    while !left.is_empty() {
        let last = points[*path.last().expect("not empty")];
        let (k, _) = left.iter().enumerate()
            .min_by(|a, b| dist(last, points[*a.1]).total_cmp(&dist(last, points[*b.1])))
            .expect("not empty");
        path.push(left.remove(k));
    }
    let mut better = true;
    let mut rounds = 0;
    while better && rounds < 50 {
        better = false;
        rounds += 1;
        for i in 0..path.len() - 2 {
            for j in i + 2..path.len() - 1 {
                let (a, b, c, d) = (points[path[i]], points[path[i + 1]], points[path[j]], points[path[j + 1]]);
                if dist(a, c) + dist(b, d) + 1e-9 < dist(a, b) + dist(c, d) {
                    path[i + 1..=j].reverse();
                    better = true;
                }
            }
        }
    }
    path
}

fn walk(points: &[(f64, f64)]) -> f64 {
    points.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn exits(ds: &Dataset, map_id: &str, at: (f64, f64)) -> Vec<Exit> {
    let mut all: Vec<Exit> = ds.files.pois.iter()
        .filter(|p| p.map_id == map_id && p.kind == PoiKind::Extract)
        .map(|p| Exit { name: p.name.clone(), x: p.x, y: p.y, distance: dist(at, (p.x, p.y)).round() })
        .collect();
    all.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    all.truncate(2);
    all
}

/// A spot named after a key, keycard or floor ("Dam Staff Key Room" is fine; "Buried City JKV
/// Employee Access Card" and "first floor" aren't places anyone would name an area by).
fn key_named(ds: &Dataset, name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(" key") || lower.ends_with(" card") || lower.contains("keycard") || lower.ends_with(" floor")
        || ds.files.items.iter().any(|i| i.category == Category::Key && i.name.eq_ignore_ascii_case(name))
}

/// A marker label fit to name an area by, with the map's name taken off the front. Labels
/// are typed by hand upstream, so this skips spawns, crates, ziplines, notes and sentences.
fn place_label(ds: &Dataset, map_name: &str, name: &str) -> Option<String> {
    const NOISE: &[&str] = &["spawn", "field crate", "zipline", "player", "button", "nest", "snow pile", "supply station", "mast lift"];
    let lower = name.to_lowercase();
    let generic = ds.files.markers.categories.iter().any(|c| c.name.eq_ignore_ascii_case(name));
    if generic || key_named(ds, name) || name.len() > 40 || name.contains('"') || name.ends_with('.')
        || NOISE.iter().any(|n| lower.contains(n)) || !name.chars().next().is_some_and(char::is_uppercase) {
        return None;
    }
    let short = map_name.trim_start_matches("The ");
    let first = short.split_whitespace().next().unwrap_or(short);
    let trimmed = [map_name, short, first].iter()
        .find_map(|p| name.strip_prefix(p).filter(|rest| rest.starts_with(' ')))
        .unwrap_or(name).trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// The nearest well-named place.
fn place_name(ds: &Dataset, map_id: &str, at: (f64, f64)) -> String {
    let map_name = ds.files.maps.iter().find(|m| m.id == map_id).map_or("", |m| m.name.as_str());
    ds.files.pois.iter()
        .filter(|p| p.map_id == map_id && matches!(p.kind, PoiKind::Landmark | PoiKind::Building | PoiKind::Extract))
        .filter_map(|p| place_label(ds, map_name, &p.name).map(|n| (n, dist(at, (p.x, p.y)))))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or_else(String::new, |(n, _)| n)
}

/// How much an area's worth is kept at a distance from the primary objective: full beside
/// it, half at `DETOUR` radii away.
const DETOUR: f64 = 3.0;

fn detour_factor(primary: Option<&Stop>, at: (f64, f64), radius: f64) -> f64 {
    primary.map_or(1.0, |p| {
        let d = dist(at, (p.x, p.y)) / (radius * DETOUR);
        1.0 / (1.0 + d * d)
    })
}

/// The best non-overlapping areas on one map for these wants. With a primary objective,
/// areas near it rank higher and each loop passes through it.
fn search(
    ds: &Dataset, map_id: &str, wants: &[Wanted], spots: &[Spot], quest_stops: &[Stop], settings: &RaidSettings,
    primary: Option<&Stop>,
) -> Vec<TargetArea> {
    let radius = settings.radius.clamp(60.0, 1500.0);
    let confidence = f64::from(settings.confidence.clamp(1, 10));
    let targets: Vec<f64> = wants.iter().map(|w| f64::from(w.qty.max(1)) * confidence).collect();
    let grid = Grid::new(spots.iter().enumerate().map(|(i, s)| (i, s.x, s.y)), radius);
    let quest_grid = Grid::new(quest_stops.iter().enumerate().map(|(i, s)| (i, s.x, s.y)), radius);
    let value_of = |s: &Spot| -> f64 { s.scores.iter().zip(&targets).map(|(v, t)| v / t).sum() };

    let mut centres: Vec<usize> = (0..spots.len()).collect();
    centres.sort_by(|a, b| value_of(&spots[*b]).total_cmp(&value_of(&spots[*a])));
    centres.truncate(CENTRES);
    // Quest stops are centres too: a quest step alone can make an area worth the trip.
    let points: Vec<(f64, f64)> = centres.iter().map(|&c| (spots[c].x, spots[c].y))
        .chain(quest_stops.iter().map(|q| (q.x, q.y))).collect();
    let mut scored: Vec<(f64, f64, f64, (f64, f64))> = points.iter().map(|&(cx, cy)| {
        let mut sums = vec![0.0; wants.len()];
        for n in grid.near(cx, cy).filter(|&n| dist((cx, cy), (spots[n].x, spots[n].y)) <= radius) {
            for (sum, v) in sums.iter_mut().zip(&spots[n].scores) {
                *sum += v;
            }
        }
        let loot: f64 = sums.iter().zip(&targets).map(|(s, t)| (s / t).min(1.0)).sum::<f64>() / wants.len().max(1) as f64;
        let quests = quest_grid.near(cx, cy).filter(|&q| dist((cx, cy), (quest_stops[q].x, quest_stops[q].y)) <= radius).count();
        let depth = sums.iter().zip(&targets).map(|(s, t)| s / t).sum::<f64>() / wants.len().max(1) as f64;
        let value = loot + (quests as f64 * QUEST_STOP_VALUE).min(3.0);
        // Ranked on value against the detour to the primary; shown as the plain value.
        (value * detour_factor(primary, (cx, cy), radius), value, depth, (cx, cy))
    }).collect();
    // Areas that cover the same share rank by how deep their chances go.
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.2.total_cmp(&a.2)));

    let map_name = ds.files.maps.iter().find(|m| m.id == map_id).map_or(map_id, |m| m.name.as_str()).to_string();
    let mut areas: Vec<TargetArea> = Vec::new();
    let mut taken: Vec<(f64, f64)> = Vec::new();
    for (rank, value, depth, at) in scored {
        if areas.len() >= settings.areas.clamp(1, 8) as usize || rank <= 0.0 {
            break;
        }
        if taken.iter().any(|t| dist(*t, at) < radius * 2.0) {
            continue;
        }
        taken.push(at);
        areas.push(area(ds, map_id, &map_name, at, (value, depth), wants, spots, &grid, quest_stops, &quest_grid, &targets, settings, areas.len(), primary));
    }
    areas
}

#[allow(clippy::too_many_arguments)]
fn area(
    ds: &Dataset, map_id: &str, map_name: &str, at: (f64, f64), (value, depth): (f64, f64), wants: &[Wanted], spots: &[Spot],
    grid: &Grid, quest_stops: &[Stop], quest_grid: &Grid, targets: &[f64], settings: &RaidSettings, index: usize,
    primary: Option<&Stop>,
) -> TargetArea {
    let radius = settings.radius.clamp(60.0, 1500.0);
    let inside: Vec<usize> = grid.near(at.0, at.1).filter(|&n| dist(at, (spots[n].x, spots[n].y)) <= radius).collect();
    let covers: Vec<ItemCover> = wants.iter().enumerate().map(|(i, w)| {
        let likely = inside.iter().filter(|&&n| spots[n].scores[i] >= LIKELY).count() as u32;
        let possible = inside.iter().filter(|&&n| (POSSIBLE..LIKELY).contains(&spots[n].scores[i])).count() as u32;
        let sum: f64 = inside.iter().map(|&n| spots[n].scores[i]).sum();
        ItemCover { item_id: w.item.id.clone(), name: w.item.name.clone(), likely, possible, enough: sum >= targets[i] }
    }).collect();
    let mut best = inside.clone();
    let worth = |n: usize| -> f64 { spots[n].scores.iter().zip(targets).map(|(v, t)| v / t).sum() };
    best.sort_by(|a, b| worth(*b).total_cmp(&worth(*a)));
    best.truncate(settings.stops.clamp(1, 30) as usize);
    let mut stops: Vec<Stop> = best.iter().map(|&n| {
        let s = &spots[n];
        let mut items: Vec<(f64, String)> = s.scores.iter().zip(wants)
            .filter(|(v, _)| **v > 0.0)
            .map(|(v, w)| (*v, format!("{} ({})", w.item.name, chance(*v)))).collect();
        items.sort_by(|a, b| b.0.total_cmp(&a.0));
        let kind = if s.category.id.starts_with("nature/") { "plant" } else if s.category.id.starts_with("arc/") { "arc" } else { "container" };
        Stop { key: s.key.clone(), kind, name: s.category.name.clone(), x: s.x, y: s.y, floors: s.floors, items: items.into_iter().map(|i| i.1).collect() }
    }).collect();
    stops.extend(quest_grid.near(at.0, at.1)
        .filter(|&q| dist(at, (quest_stops[q].x, quest_stops[q].y)) <= radius)
        .map(|q| quest_stops[q].clone()));
    let detour = primary.map(|p| (dist(at, (p.x, p.y)) - radius).max(0.0).round());
    if let Some(p) = primary {
        // A quest step or marker picked as the primary is the same place as its own stop.
        let same = |s: &Stop| s.key == p.key || p.key.strip_prefix("primary:") == Some(s.key.as_str()) || dist((s.x, s.y), (p.x, p.y)) < 1.0;
        stops.retain(|s| !same(s));
        stops.insert(0, p.clone());
    }
    let centre = if stops.is_empty() {
        at
    } else {
        let n = stops.len() as f64;
        (stops.iter().map(|s| s.x).sum::<f64>() / n, stops.iter().map(|s| s.y).sum::<f64>() / n)
    };
    let out = exits(ds, map_id, centre);
    // Start at the stop farthest from the nearest exit, so the loop ends close to it.
    let points: Vec<(f64, f64)> = stops.iter().map(|s| (s.x, s.y)).collect();
    let exit = out.first().map_or(centre, |e| (e.x, e.y));
    let start = (0..points.len()).max_by(|a, b| dist(points[*a], exit).total_cmp(&dist(points[*b], exit))).unwrap_or(0);
    let path = order(&points, start);
    let stops: Vec<Stop> = path.iter().map(|&i| stops[i].clone()).collect();
    let length = walk(&path.iter().map(|&i| points[i]).collect::<Vec<_>>());
    TargetArea {
        label: char::from(b'A' + (index as u8 % 26)).to_string(),
        name: place_name(ds, map_id, centre),
        map_id: map_id.to_string(),
        map_name: map_name.to_string(),
        x: centre.0,
        y: centre.1,
        radius,
        value: (value * 100.0).round() / 100.0,
        depth: (depth * 10.0).round() / 10.0,
        stops,
        covers,
        exits: out,
        walk: length.round(),
        detour,
    }
}

fn wanted<'a>(ds: &'a Dataset, profile: &Profile, items: &BTreeMap<String, u32>) -> Vec<Wanted<'a>> {
    items.iter().filter_map(|(id, qty)| {
        let have = profile.stash.get(id).copied().unwrap_or(0);
        let left = qty.saturating_sub(have);
        let item = ds.item(id)?;
        (left > 0).then(|| Wanted { item, qty: left, tags: tags(ds, item) })
    }).collect()
}

fn placeable(ds: &Dataset, w: &Wanted) -> bool {
    ds.files.markers.categories.iter().any(|c| affinity(ds, &c.id, w, &[]) > 0.0)
}

fn quest_stops(ds: &Dataset, profile: &Profile, map_id: &str, off: &BTreeSet<&str>) -> Vec<Stop> {
    if off.contains("cat:quest") {
        return Vec::new();
    }
    ds.files.pois.iter().filter(|p| p.map_id == map_id).filter_map(|p| {
        let quest = p.quest_id.as_ref()?;
        let open = profile.quests.get(quest).is_some_and(|q| !q.done) && !off.contains(format!("quest:{quest}").as_str());
        let q = ds.files.quests.iter().find(|q| &q.id == quest)?;
        open.then(|| Stop {
            key: p.id.clone(), kind: "quest", name: p.name.clone(), x: p.x, y: p.y, floors: p.zlayers,
            items: vec![q.name.clone()],
        })
    }).collect()
}

/// What a plan is for: every source, and the items switched on, after the stash.
fn plan_wants<'a>(ds: &'a Dataset, profile: &Profile, plan: &RaidPlan) -> (Vec<Raw>, BTreeMap<String, u32>) {
    let off: BTreeSet<&str> = plan.off.iter().map(String::as_str).collect();
    let all = sources(ds, profile, &plan.extra_items);
    let source_on = |key: &str, kind: &str| !off.contains(key) && !off.contains(format!("cat:{kind}").as_str());
    let mut totals: BTreeMap<String, u32> = BTreeMap::new();
    for (key, kind, _, items) in &all {
        if source_on(key, kind) {
            for (id, qty) in items {
                *totals.entry(id.clone()).or_default() += qty;
            }
        }
    }
    totals.retain(|id, _| !off.contains(format!("item:{id}").as_str()));
    (all, totals)
}

/// Where a primary objective sits, as a stop on the plan's map.
fn primary_stop(plan: &RaidPlan) -> Option<Stop> {
    let p = plan.primary.as_ref()?;
    Some(Stop {
        key: format!("primary:{}", p.id), kind: "primary", name: p.name.clone(), x: p.x, y: p.y, floors: p.floors,
        items: vec!["Primary objective".into()],
    })
}

/// Every map ranked for this plan's wants and quest steps, best first.
pub fn suggest_maps(ds: &Dataset, profile: &Profile, plan: &RaidPlan) -> Vec<MapPick> {
    suggest_maps_live(ds, profile, plan, &BTreeMap::new())
}

/// The same, counting the conditions live on each map now (map id to event names).
pub fn suggest_maps_live(ds: &Dataset, profile: &Profile, plan: &RaidPlan, live: &BTreeMap<String, Vec<String>>) -> Vec<MapPick> {
    let (_, totals) = plan_wants(ds, profile, plan);
    let wants = wanted(ds, profile, &totals);
    let off: BTreeSet<&str> = plan.off.iter().map(String::as_str).collect();
    let boxes_off: BTreeSet<&str> = off.iter().filter_map(|k| k.strip_prefix("box:")).collect();
    let one = RaidSettings { areas: 1, ..plan.settings.clone() };
    let mut picks: Vec<MapPick> = ds.files.maps.iter().map(|m| {
        let now = live_on(ds, &m.id, live.get(&m.id).map_or(&[][..], Vec::as_slice));
        let spots_here = spots(ds, &m.id, &wants, &boxes_off, &now);
        let quests = quest_stops(ds, profile, &m.id, &off);
        let best = search(ds, &m.id, &wants, &spots_here, &quests, &one, None).into_iter().next();
        let mut sums = vec![0.0; wants.len()];
        for s in &spots_here {
            for (sum, v) in sums.iter_mut().zip(&s.scores) {
                *sum += v;
            }
        }
        let (mut here, mut missing) = (Vec::new(), Vec::new());
        for (w, sum) in wants.iter().zip(&sums) {
            if *sum >= f64::from(w.qty.max(1)) * f64::from(plan.settings.confidence.clamp(1, 10)) {
                here.push(w.item.name.clone());
            } else if *sum == 0.0 {
                missing.push(w.item.name.clone());
            }
        }
        // Quest steps outside the best area still pull a map up: they're needed anyway.
        let in_best = best.as_ref().map_or(0, |b| b.stops.iter().filter(|s| s.kind == "quest").count());
        let coverage = if wants.is_empty() { 0.0 } else { here.len() as f64 / wants.len() as f64 };
        let score = best.as_ref().map_or(0.0, |b| b.value) + coverage
            + (quests.len().saturating_sub(in_best) as f64 * QUEST_STOP_VALUE * 0.5).min(2.0);
        MapPick {
            map_id: m.id.clone(), map_name: m.name.clone(), score: (score * 100.0).round() / 100.0, best,
            quest_steps: quests.len(), items_here: here, items_missing: missing,
            conditions: now.iter().map(|c| condition_note(c)).collect(),
        }
    }).collect();
    picks.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.map_name.cmp(&b.map_name)));
    picks
}

/// The auto plan for a raid: what it's for (with switches), and the best target areas.
pub fn auto_plan(ds: &Dataset, profile: &Profile, plan: &RaidPlan) -> RaidAuto {
    auto_plan_live(ds, profile, plan, &[])
}

/// The same, with the map's live conditions (event names from the schedule).
pub fn auto_plan_live(ds: &Dataset, profile: &Profile, plan: &RaidPlan, live: &[String]) -> RaidAuto {
    let off: BTreeSet<&str> = plan.off.iter().map(String::as_str).collect();
    let all = sources(ds, profile, &plan.extra_items);
    let source_on = |key: &str, kind: &str| !off.contains(key) && !off.contains(format!("cat:{kind}").as_str());
    let mut totals: BTreeMap<String, u32> = BTreeMap::new();
    for (key, kind, _, items) in &all {
        if source_on(key, kind) {
            for (id, qty) in items {
                *totals.entry(id.clone()).or_default() += qty;
            }
        }
    }
    let mut every: BTreeMap<String, u32> = BTreeMap::new();
    for (_, _, _, items) in &all {
        for (id, qty) in items {
            *every.entry(id.clone()).or_default() += qty;
        }
    }
    let wants_all = wanted(ds, profile, &every);
    let want_rows: Vec<Want> = wants_all.iter().map(|w| Want {
        item: item_link(ds, &w.item.id, Some(w.qty)),
        qty: w.qty,
        on: totals.contains_key(&w.item.id) && !off.contains(format!("item:{}", w.item.id).as_str()),
        placeable: placeable(ds, w),
    }).collect();
    let wants: Vec<Wanted> = wanted(ds, profile, &totals).into_iter()
        .filter(|w| !off.contains(format!("item:{}", w.item.id).as_str()))
        .collect();

    let boxes_off: BTreeSet<&str> = off.iter().filter_map(|k| k.strip_prefix("box:")).collect();
    let now = live_on(ds, &plan.map_id, live);
    let spots_here = spots(ds, &plan.map_id, &wants, &boxes_off, &now);
    let mut counts: BTreeMap<&str, (usize, &str)> = BTreeMap::new();
    for m in ds.files.markers.maps.get(&plan.map_id).map_or(&[][..], |m| &m[..]) {
        if let Some(c) = ds.files.markers.categories.get(usize::from(m.0)).filter(|c| c.id.starts_with("containers/")) {
            counts.entry(&c.id).or_insert((0, &c.name)).0 += 1;
        }
    }
    let boxes = counts.into_iter().map(|(id, (count, name))| BoxType {
        key: format!("box:{id}"), name: name.to_string(), count, on: !boxes_off.contains(id),
    }).collect();
    let quests = quest_stops(ds, profile, &plan.map_id, &off);
    let primary = primary_stop(plan);
    let areas = search(ds, &plan.map_id, &wants, &spots_here, &quests, &plan.settings, primary.as_ref());
    RaidAuto {
        sources: all.into_iter().map(|(key, kind, name, items)| Source {
            on: source_on(&key, kind),
            key,
            kind,
            name,
            items: items.iter().map(|(id, qty)| item_link(ds, id, Some(*qty))).collect(),
        }).collect(),
        wants: want_rows,
        boxes,
        areas,
        quest_stops: quests,
        primary,
        conditions: now.iter().map(|c| condition_note(c)).collect(),
    }
}

/// The best places across every map to farm these items, best first.
pub fn farm(ds: &Dataset, item_ids: &[String], settings: &RaidSettings) -> Vec<TargetArea> {
    let items: BTreeMap<String, u32> = item_ids.iter().map(|i| (i.clone(), 1)).collect();
    let wants: Vec<Wanted> = wanted(ds, &Profile::default(), &items);
    let per_map = RaidSettings { areas: 2, ..settings.clone() };
    let mut all: Vec<TargetArea> = ds.files.maps.iter().flat_map(|m| {
        let spots_here = spots(ds, &m.id, &wants, &BTreeSet::new(), &[]);
        search(ds, &m.id, &wants, &spots_here, &[], &per_map, None)
    }).collect();
    all.sort_by(|a, b| b.value.total_cmp(&a.value).then(b.depth.total_cmp(&a.depth)));
    all.truncate(8);
    for (i, a) in all.iter_mut().enumerate() {
        a.label = (i + 1).to_string();
    }
    all
}
