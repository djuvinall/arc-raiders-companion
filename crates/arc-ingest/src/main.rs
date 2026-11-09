//! Data pipeline. No LLM anywhere in this path (D6): fetch, normalize, diff, and
//! a human commits the result.
//!
//!   arc-ingest probe            snapshot sources into dataset/fixtures, print coverage
//!   arc-ingest diff             rebuild from the newest fixtures, compare with dataset/data
//!   arc-ingest sync [--write]   same diff, and write dataset/data with --write

mod arcs;
mod arc_loot;
mod containers;
mod corrections;
mod fetch;
mod guns;
mod http;
mod normalize;
mod output;
mod precedence;
mod probe;
mod stamp;
mod skills;

use anyhow::{bail, Context, Result};
use arc_core::model as m;
use arc_core::Dataset;
use normalize::Sources;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const PRECEDENCE: &str = include_str!("../../../dataset/overlays/precedence.toml");
const SKILLS_OVERLAY: &str = include_str!("../../../dataset/overlays/skills_overlay.json");
const MAPS_OVERLAY: &str = include_str!("../../../dataset/overlays/maps_overlay.json");
const BENCHES_OVERLAY: &str = include_str!("../../../dataset/overlays/benches_overlay.json");
const PROJECTS_OVERLAY: &str = include_str!("../../../dataset/overlays/projects_overlay.json");
const PROJECT_TASKS_OVERLAY: &str = include_str!("../../../dataset/overlays/project_tasks_overlay.json");
const NOTES_OVERLAY: &str = include_str!("../../../dataset/overlays/notes_overlay.json");
const QUESTS_OVERLAY: &str = include_str!("../../../dataset/overlays/quests_overlay.json");
const CONTAINERS_OVERLAY: &str = include_str!("../../../dataset/overlays/containers_overlay.json");
const CONDITIONS_OVERLAY: &str = include_str!("../../../dataset/overlays/conditions_overlay.json");
const ARC_LOOT: &str = include_str!("../../../dataset/overlays/arc_loot.json");
const TIP_FIXES: &str = include_str!("../../../dataset/overlays/tip_fixes.json");
const STENCILS_OVERLAY: &str = include_str!("../../../dataset/overlays/stencils_overlay.json");
const STENCIL_SWATCHES: &str = "https://static.metaforge.app/arc-raiders/ui/stencils";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MapsOverlay {
    #[serde(rename = "_note")]
    _note: String,
    maps: std::collections::BTreeMap<String, MapTiles>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MapTiles {
    url: String,
    tile_size: u32,
    max_native_zoom: u8,
    world_extent: [f64; 4],
    tile_extent: [f64; 4],
    #[serde(default)]
    layers: Vec<m::MapLayer>,
}

fn apply_map_tiles(maps: &mut [m::GameMap]) -> Result<()> {
    let overlay: MapsOverlay = serde_json::from_str(MAPS_OVERLAY).context("maps_overlay.json")?;
    for map in maps {
        let tiles = overlay.maps.get(&map.id).with_context(|| format!("no tiles for {}", map.id))?;
        map.image.url = Some(tiles.url.clone());
        map.image.tile_size = Some(tiles.tile_size);
        map.image.max_native_zoom = Some(tiles.max_native_zoom);
        map.image.world_extent = Some(tiles.world_extent);
        map.image.tile_extent = Some(tiles.tile_extent);
        map.image.layers = tiles.layers.clone();
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StencilsOverlay {
    #[serde(rename = "_note")]
    _note: String,
    stencils: std::collections::BTreeMap<String, StencilEntry>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StencilEntry {
    weapons: Vec<String>,
    source: String,
    parts: u32,
}

fn apply_stencils(items: &mut [m::Item]) -> Result<()> {
    let overlay: StencilsOverlay = serde_json::from_str(STENCILS_OVERLAY).context("stencils_overlay.json")?;
    for item in items.iter_mut().filter(|i| i.category == m::Category::Stencil) {
        let entry = overlay.stencils.get(&item.id).with_context(|| format!("no overlay for {}", item.id))?;
        item.stencil = Some(m::StencilFit {
            swatch: format!("{STENCIL_SWATCHES}/{}.webp", item.id),
            weapons: entry.weapons.clone(),
            source: entry.source.clone(),
            parts: entry.parts,
        });
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectsOverlay {
    #[serde(rename = "_note")]
    _note: String,
    permanent: Vec<String>,
    paused: Vec<String>,
}

#[derive(serde::Deserialize)]
struct QuestsOverlay {
    #[serde(rename = "_note")]
    _note: String,
    quests: std::collections::BTreeMap<String, QuestNesting>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct QuestNesting {
    #[serde(rename = "_seen")]
    _seen: String,
    #[serde(default)]
    optional: Vec<String>,
    /// See the overlay's note for the shape.
    tree: Option<Value>,
}

const OPTIONAL_PREFIX: &str = "(Optional)";

fn group_node(id: String, kind: &str, children: Vec<String>, title: Option<String>) -> m::Objective {
    m::Objective {
        id,
        kind: kind.into(),
        children,
        round_constraint: None,
        action: None,
        amount: None,
        description: title,
        maps: Vec::new(),
        poi_ids: Vec::new(),
        item_ids: Vec::new(),
        maps_provenance: None,
    }
}

/// Turns one tree entry into a node id, adding group nodes as it goes.
fn tree_node(quest: &str, value: &Value, steps: &mut Vec<String>, groups: &mut Vec<m::Objective>) -> Result<String> {
    match value {
        Value::String(id) => {
            steps.push(id.clone());
            Ok(id.clone())
        }
        Value::Array(list) => {
            let children = list.iter().map(|v| tree_node(quest, v, steps, groups)).collect::<Result<Vec<_>>>()?;
            let id = format!("g{}", groups.len() + 1);
            groups.push(group_node(id.clone(), "sequence", children, None));
            Ok(id)
        }
        Value::Object(map) => {
            let title = map.get("title").and_then(Value::as_str).map(str::to_string);
            let (kind, list) = match (map.get("all"), map.get("any")) {
                (Some(Value::Array(l)), None) => ("all", l),
                (None, Some(Value::Array(l))) => ("any", l),
                _ => bail!("quests_overlay.json: {quest} has a group without exactly one of all/any"),
            };
            if map.keys().any(|k| !matches!(k.as_str(), "all" | "any" | "title")) {
                bail!("quests_overlay.json: {quest} has a group with an unknown key");
            }
            let children = list.iter().map(|v| tree_node(quest, v, steps, groups)).collect::<Result<Vec<_>>>()?;
            let id = format!("g{}", groups.len() + 1);
            groups.push(group_node(id.clone(), kind, children, title));
            Ok(id)
        }
        _ => bail!("quests_overlay.json: {quest} has a step that isn't an id, list or group"),
    }
}

/// Nests steps the way the in-game quest log does, and marks optional steps. Steps keep
/// their ids; the root sequence's children become the tree's top level.
fn apply_quest_nesting(quests: &mut [m::Quest]) -> Result<()> {
    for quest in quests.iter_mut() {
        for o in quest.objectives.iter_mut().filter(|o| o.kind == "atomic") {
            let text = o.description.as_deref().unwrap_or("");
            if let Some(rest) = text.strip_prefix(OPTIONAL_PREFIX) {
                o.description = Some(rest.trim().to_string());
                o.kind = "optional".into();
            }
        }
    }
    let overlay: QuestsOverlay = serde_json::from_str(QUESTS_OVERLAY).context("quests_overlay.json")?;
    for (id, nesting) in &overlay.quests {
        let quest = quests.iter_mut().find(|q| &q.id == id)
            .with_context(|| format!("quests_overlay.json names unknown quest {id}"))?;
        let leaves: Vec<String> = quest.objectives.iter()
            .filter(|o| o.kind == "atomic" || o.kind == "optional").map(|o| o.id.clone()).collect();
        for o in &nesting.optional {
            if !leaves.contains(o) {
                bail!("quests_overlay.json: {id} has no step {o}");
            }
        }
        for o in quest.objectives.iter_mut().filter(|o| nesting.optional.contains(&o.id)) {
            o.kind = "optional".into();
        }
        let Some(tree) = &nesting.tree else { continue };
        let Value::Array(top) = tree else { bail!("quests_overlay.json: {id}'s tree must be a list") };
        let (mut steps, mut groups) = (Vec::new(), Vec::new());
        let children = top.iter().map(|v| tree_node(id, v, &mut steps, &mut groups)).collect::<Result<Vec<_>>>()?;
        let mut sorted = steps.clone();
        sorted.sort();
        sorted.dedup();
        let mut want = leaves.clone();
        want.sort();
        if sorted.len() != steps.len() || sorted != want {
            bail!("quests_overlay.json: {id}'s tree must list each of its steps once");
        }
        let root_id = quest.objective_root.clone();
        let root = quest.objectives.iter_mut().find(|o| o.id == root_id).context("no root")?;
        root.children = children;
        quest.objectives.extend(groups);
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotesOverlay {
    #[serde(rename = "_note")]
    _note: String,
    doors: Vec<HandDoor>,
    items: std::collections::BTreeMap<String, m::ItemNote>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct HandDoor {
    map: String,
    x: f64,
    y: f64,
    floors: Option<u32>,
    label: String,
    #[allow(dead_code)]
    source: String,
}

/// Hand-entered doors become locked-room markers; item notes go to notes.json.
fn apply_notes(files: &mut arc_core::dataset::DatasetFiles) -> Result<()> {
    let overlay: NotesOverlay = serde_json::from_str(NOTES_OVERLAY).context("notes_overlay.json")?;
    let room = files.markers.categories.iter().position(|c| c.id == "locations/locked_room")
        .context("no locked-room marker type")? as u16;
    for door in overlay.doors {
        let list = files.markers.maps.get_mut(&door.map)
            .with_context(|| format!("notes_overlay.json: unknown map {}", door.map))?;
        list.push(m::MapMarker(room, door.x, door.y, door.floors, Some(door.label)));
    }
    files.notes.items = overlay.items;
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectTasksOverlay {
    #[serde(rename = "_note")]
    _note: String,
    stages: Vec<HandTask>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct HandTask {
    project: String,
    stage: u32,
    title: String,
    map: Option<String>,
    description: Option<String>,
    steps: Vec<HandStep>,
    #[allow(dead_code)]
    source: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct HandStep {
    text: String,
    amount: Option<u32>,
}

/// Mission stages from project_tasks_overlay.json, as a sequence of steps per stage.
fn apply_project_tasks(files: &mut arc_core::dataset::DatasetFiles) -> Result<()> {
    let overlay: ProjectTasksOverlay = serde_json::from_str(PROJECT_TASKS_OVERLAY).context("project_tasks_overlay.json")?;
    files.project_tasks.stages = overlay.stages.into_iter().map(|t| {
        let base = format!("{}-s{}", t.project, t.stage);
        let maps: Vec<String> = t.map.iter().cloned().collect();
        let step = |id: String, kind: &str, children: Vec<String>, amount: Option<u32>, text: Option<String>| m::Objective {
            id, kind: kind.into(), children, round_constraint: None, action: None, amount, description: text,
            maps: maps.clone(), poi_ids: Vec::new(), item_ids: Vec::new(), maps_provenance: None,
        };
        let mut objectives: Vec<m::Objective> = t.steps.into_iter().enumerate()
            .map(|(i, s)| step(format!("{base}-o{}", i + 1), "atomic", Vec::new(), s.amount, Some(s.text)))
            .collect();
        let children = objectives.iter().map(|o| o.id.clone()).collect();
        objectives.insert(0, step(format!("{base}-root"), "sequence", children, None, None));
        m::ProjectTask {
            project_id: t.project, stage: t.stage, title: t.title, map: t.map, description: t.description,
            objective_root: format!("{base}-root"), objectives,
        }
    }).collect();
    Ok(())
}

fn apply_project_status(projects: &mut [m::Project]) -> Result<()> {
    let overlay: ProjectsOverlay = serde_json::from_str(PROJECTS_OVERLAY).context("projects_overlay.json")?;
    for project in projects {
        project.permanent = overlay.permanent.contains(&project.id);
        project.paused = overlay.paused.contains(&project.id);
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchesOverlay {
    #[serde(rename = "_note")]
    _note: String,
    fixes: Vec<BenchFix>,
    requires: std::collections::BTreeMap<String, std::collections::BTreeMap<u8, String>>,
    extra: Vec<ExtraBench>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtraBench {
    id: String,
    name: String,
    levels: Vec<ExtraLevel>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtraLevel {
    level: u8,
    inputs: Vec<m::ItemQty>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchFix {
    station: String,
    item: String,
    level: u8,
    qty: u32,
}

/// Moves each listed item to its true level and quantity, adding the level if it's new.
fn apply_bench_fixes(benches: &mut Vec<m::Workbench>) -> Result<()> {
    let overlay: BenchesOverlay = serde_json::from_str(BENCHES_OVERLAY).context("benches_overlay.json")?;
    for fix in &overlay.fixes {
        let bench = benches.iter_mut().find(|b| b.id == fix.station).with_context(|| format!("no bench {}", fix.station))?;
        bench.levels.iter_mut().for_each(|l| l.upgrade_inputs.retain(|q| q.item_id != fix.item));
        if !bench.levels.iter().any(|l| l.level == fix.level) {
            bench.levels.push(m::WorkbenchLevel {
                level: fix.level,
                upgrade_inputs: Vec::new(),
                requires: None,
                unlocks_recipe_ids: Vec::new(),
            });
            bench.max_level = bench.max_level.max(fix.level);
        }
        let level = bench.levels.iter_mut().find(|l| l.level == fix.level).context("level just added")?;
        level.upgrade_inputs.push(m::ItemQty { item_id: fix.item.clone(), qty: fix.qty });
    }
    for extra in overlay.extra {
        benches.push(m::Workbench {
            max_level: extra.levels.iter().map(|l| l.level).max().unwrap_or(0),
            id: extra.id,
            name: extra.name,
            levels: extra.levels.into_iter().map(|l| m::WorkbenchLevel {
                level: l.level,
                upgrade_inputs: l.inputs,
                requires: None,
                unlocks_recipe_ids: Vec::new(),
            }).collect(),
        });
    }
    for bench in benches {
        for level in &mut bench.levels {
            level.requires = overlay.requires.get(&bench.id).and_then(|r| r.get(&level.level)).cloned();
        }
    }
    Ok(())
}

const MARKER_GROUPS: &[&str] = &["locations", "quests", "containers", "arc", "events", "nature"];
/// Labels only where a name says something: every locker is "Locker".
const LABELLED: &[&str] = &["locations", "events", "quests"];

fn title(sub: &str) -> String {
    sub.split(['_', '-', ' ']).filter(|w| !w.is_empty())
        .map(|w| w[..1].to_uppercase() + &w[1..]).collect::<Vec<_>>().join(" ")
}

fn build_markers(fixture: &Value, arcs: &[m::Arc]) -> Result<m::MapMarkers> {
    let mut categories: Vec<m::MarkerCategory> = Vec::new();
    let mut seen: std::collections::BTreeMap<String, ()> = std::collections::BTreeMap::new();
    let maps = fixture["maps"].as_object().context("markers fixture: no maps")?;
    let key = |row: &[Value]| -> Option<(String, String)> {
        let group = row.first()?.as_str()?.to_string();
        let sub = row.get(1)?.as_str()?.to_string();
        Some((group, sub))
    };
    for rows in maps.values() {
        for row in rows.as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let Some((group, sub)) = row.as_array().and_then(|r| key(r)) else { continue };
            if !MARKER_GROUPS.contains(&group.as_str()) {
                continue;
            }
            let id = if group == "quests" { "quests".to_string() } else { format!("{group}/{sub}") };
            if seen.insert(id.clone(), ()).is_some() {
                continue;
            }
            let name = match group.as_str() {
                "quests" => "Quest objective".to_string(),
                "arc" => arcs.iter().find(|a| a.id == sub).map_or_else(|| title(&sub), |a| a.name.clone()),
                _ => title(&sub),
            };
            categories.push(m::MarkerCategory { id, name, group });
        }
    }
    let rank = |g: &str| MARKER_GROUPS.iter().position(|x| *x == g).unwrap_or(99);
    categories.sort_by(|a, b| rank(&a.group).cmp(&rank(&b.group)).then_with(|| a.name.cmp(&b.name)));
    let index: std::collections::BTreeMap<&str, u16> = categories.iter().enumerate().map(|(i, c)| (c.id.as_str(), i as u16)).collect();
    let mut out = std::collections::BTreeMap::new();
    for (map, rows) in maps {
        let markers = rows.as_array().map(Vec::as_slice).unwrap_or(&[]).iter().filter_map(|row| {
            let row = row.as_array()?;
            let (group, sub) = key(row)?;
            let id = if group == "quests" { "quests".to_string() } else { format!("{group}/{sub}") };
            let label = if group == "quests" {
                Some(sub)
            } else if LABELLED.contains(&group.as_str()) {
                row.get(5)?.as_str().map(str::to_string)
            } else {
                None
            };
            Some(m::MapMarker(*index.get(id.as_str())?, row.get(2)?.as_f64()?, row.get(3)?.as_f64()?,
                row.get(4)?.as_u64().and_then(|z| u32::try_from(z).ok()), label))
        }).collect();
        out.insert(map.clone(), markers);
    }
    Ok(m::MapMarkers { categories, maps: out })
}

/// Names the sources spell differently from the game, and placeholder duplicates.
fn tidy(files: &mut arc_core::dataset::DatasetFiles) {
    for trader in &mut files.traders {
        if trader.id == "tianwen" {
            trader.id = "tian-wen".into();
            trader.name = "Tian Wen".into();
        }
    }
    for quest in &mut files.quests {
        if quest.trader.as_deref() == Some("tianwen") {
            quest.trader = Some("tian-wen".into());
        }
    }
    for item in &mut files.items {
        for s in &mut item.sold_by {
            if s.trader == "tianwen" {
                s.trader = "tian-wen".into();
            }
        }
    }
    // The wiki's "Crane Secure Storage Keycard" is a naming slip for the Secure Storage
    // Keycard: MetaForge and RaidTheory list only that card.
    if files.items.iter().any(|i| i.id == "riven-tides-secure-storage-keycard") {
        files.items.retain(|i| i.id != "riven-tides-crane-secure-storage-keycard");
    }
    // MetaForge lists a few cosmetics twice, once under a placeholder id.
    let named: std::collections::BTreeSet<String> = files.items.iter()
        .filter(|i| !i.id.starts_with("game-cosmetic-"))
        .map(|i| i.name.to_lowercase())
        .collect();
    files.items.retain(|i| !(i.id.starts_with("game-cosmetic-") && named.contains(&i.name.to_lowercase())));
}

fn newest_fixture(root: &Path, source: &str) -> Result<(String, Value)> {
    let dir = root.join("dataset/fixtures").join(source);
    let newest = fs::read_dir(&dir)
        .with_context(|| format!("no fixtures in {}; run probe", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .max()
        .with_context(|| format!("no fixtures in {}", dir.display()))?;
    let date = newest.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    Ok((date, serde_json::from_slice(&fs::read(&newest)?)?))
}

fn write_schemas(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    let schemas = [
        ("item", schemars::schema_for!(Vec<m::Item>)),
        ("recipe", schemars::schema_for!(Vec<m::Recipe>)),
        ("workbench", schemars::schema_for!(Vec<m::Workbench>)),
        ("weapon_mod", schemars::schema_for!(Vec<m::WeaponMod>)),
        ("map", schemars::schema_for!(Vec<m::GameMap>)),
        ("poi", schemars::schema_for!(Vec<m::Poi>)),
        ("quest", schemars::schema_for!(Vec<m::Quest>)),
        ("project", schemars::schema_for!(Vec<m::Project>)),
        ("skill_tree", schemars::schema_for!(m::SkillTree)),
        ("arc", schemars::schema_for!(Vec<m::Arc>)),
        ("trader", schemars::schema_for!(Vec<m::Trader>)),
        ("loot_area", schemars::schema_for!(Vec<m::LootArea>)),
        ("map_markers", schemars::schema_for!(m::MapMarkers)),
        ("version", schemars::schema_for!(m::DatasetVersion)),
        ("changelog", schemars::schema_for!(Vec<m::ChangelogEntry>)),
        ("notes", schemars::schema_for!(m::Notes)),
        ("container", schemars::schema_for!(Vec<m::Container>)),
        ("condition", schemars::schema_for!(Vec<m::MapCondition>)),
    ];
    for (name, schema) in schemas {
        let text = serde_json::to_string_pretty(&schema)? + "\n";
        fs::write(dir.join(format!("{name}.schema.json")), text)?;
    }
    Ok(())
}

fn rebuild(root: &Path, write: bool) -> Result<usize> {
    let (date, metaforge) = newest_fixture(root, "metaforge")?;
    let (_, wiki) = newest_fixture(root, "wiki")?;
    let (_, raidtheory) = newest_fixture(root, "raidtheory")?;
    let (_, skills_page) = newest_fixture(root, "wiki_skills")?;
    let skills_wikitext = skills_page["wikitext"].as_str().context("no wikitext")?.to_string();
    let prec = precedence::Precedence::parse(PRECEDENCE)?;
    let overlay = skills::load_overlay(SKILLS_OVERLAY)?;

    let sources = Sources { date, metaforge, wiki, raidtheory, skills_wikitext };
    let mut out = normalize::normalize(&sources, &prec, &overlay)?;
    apply_map_tiles(&mut out.files.maps)?;
    apply_stencils(&mut out.files.items)?;
    apply_project_status(&mut out.files.projects)?;
    apply_quest_nesting(&mut out.files.quests)?;
    apply_bench_fixes(&mut out.files.workbenches)?;
    tidy(&mut out.files);
    let (_, arc_pages) = newest_fixture(root, "wiki_arcs")?;
    arcs::apply(&mut out.files.arcs, &arc_pages["pages"]);
    arcs::fix_tips(&mut out.files.arcs, TIP_FIXES)?;
    let arc_loot = arc_loot::load(ARC_LOOT)?;
    arc_loot::apply(&mut out.files.arcs, &mut out.files.items, &arc_loot)?;
    // Older snapshots predate these pages, so a missing one leaves the API's values.
    if let Ok((_, gun_pages)) = newest_fixture(root, "wiki_gear") {
        guns::apply(&mut out.files.items, &gun_pages["pages"]);
    }
    let (_, marker_fixture) = newest_fixture(root, "metaforge_markers")?;
    out.files.markers = build_markers(&marker_fixture, &out.files.arcs)?;
    out.files.containers = containers::build(CONTAINERS_OVERLAY, &arc_loot, &out.files.items, &mut out.files.markers)?;
    out.files.conditions = containers::conditions(CONDITIONS_OVERLAY, &out.files.maps, &out.files.markers)?;
    apply_notes(&mut out.files)?;
    apply_project_tasks(&mut out.files)?;
    let fixes = corrections::load(&root.join("dataset/overlays/corrections"))?;
    corrections::apply(&mut out.files, &fixes)?;
    let (_, steam) = newest_fixture(root, "steam")?;
    let notes = arc_core::patches::parse_news(&serde_json::to_vec(&steam)?)?;
    out.files.version.game_patch = arc_core::patches::latest_version(&notes);
    let mut files = Dataset::from_files(out.files)?.files;
    let coverage = output::coverage(&files, &out.unresolved)?;
    // The version follows the data: see stamp.rs.
    let stamp_path = root.join("dataset/stamp.json");
    let old_stamp = stamp::read(&stamp_path)?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let new_stamp = stamp::next(old_stamp.as_ref(), &stamp::hash(&output::render(&files, &coverage)?), &today);
    if old_stamp.as_ref() != Some(&new_stamp) && !write {
        println!("note: the data changed since dataset/stamp.json; sync --write gives it version {}", new_stamp.version());
    }
    files.version.dataset_version = new_stamp.version();
    let rendered = output::render(&files, &coverage)?;

    let data = root.join("dataset/data");
    let changes = output::diff(&data, &rendered);
    println!("{changes} difference(s); {} unresolved reference(s)", out.unresolved.len());
    if write {
        fs::create_dir_all(&data)?;
        output::update_changelog(&data, &rendered, &files)?;
        for (name, text) in &rendered {
            fs::write(data.join(name), text)?;
        }
        write_schemas(&data.join("schema"))?;
        stamp::write(&stamp_path, &new_stamp)?;
        println!("wrote {} as version {}", data.display(), new_stamp.version());
    }
    Ok(changes)
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = args.iter().any(|a| a == "--write");
    match args.first().map(String::as_str) {
        Some("probe") => probe::run(&root),
        Some("sync") => rebuild(&root, flag).map(|_| ()),
        Some("diff") => {
            if rebuild(&root, false)? > 0 {
                bail!("dataset/data does not match the fixtures");
            }
            Ok(())
        }
        _ => bail!("usage: arc-ingest <probe | diff | sync [--write]>"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_matches_the_pinned_fixtures() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert_eq!(rebuild(&root, false).unwrap(), 0);
    }
}
