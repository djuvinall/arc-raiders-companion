//! Data pipeline. No LLM anywhere in this path (D6): fetch, normalize, diff, and
//! a human commits the result.
//!
//!   arc-ingest probe            snapshot sources into tests/fixtures, print coverage
//!   arc-ingest diff             rebuild from the newest fixtures, compare with data/
//!   arc-ingest sync [--write]   same diff, and write data/ with --write

mod arcs;
mod fetch;
mod http;
mod normalize;
mod output;
mod precedence;
mod probe;
mod skills;

use anyhow::{bail, Context, Result};
use arc_core::model as m;
use arc_core::Dataset;
use normalize::Sources;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const PRECEDENCE: &str = include_str!("../precedence.toml");
const SKILLS_OVERLAY: &str = include_str!("../skills_overlay.json");
const MAPS_OVERLAY: &str = include_str!("../maps_overlay.json");
const BENCHES_OVERLAY: &str = include_str!("../benches_overlay.json");
const PROJECTS_OVERLAY: &str = include_str!("../projects_overlay.json");
const STENCILS_OVERLAY: &str = include_str!("../stencils_overlay.json");
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
fn apply_bench_fixes(benches: &mut [m::Workbench]) -> Result<()> {
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
    for bench in benches {
        for level in &mut bench.levels {
            level.requires = overlay.requires.get(&bench.id).and_then(|r| r.get(&level.level)).cloned();
        }
    }
    Ok(())
}

fn newest_fixture(root: &Path, source: &str) -> Result<(String, Value)> {
    let dir = root.join("tests/fixtures").join(source);
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
        ("version", schemars::schema_for!(m::DatasetVersion)),
        ("changelog", schemars::schema_for!(Vec<m::ChangelogEntry>)),
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
    apply_bench_fixes(&mut out.files.workbenches)?;
    let (_, arc_pages) = newest_fixture(root, "wiki_arcs")?;
    arcs::apply(&mut out.files.arcs, &arc_pages["pages"]);
    let (_, steam) = newest_fixture(root, "steam")?;
    let notes = arc_core::patches::parse_news(&serde_json::to_vec(&steam)?)?;
    out.files.version.game_patch = arc_core::patches::latest_version(&notes);
    let files = Dataset::from_files(out.files)?.files;
    let coverage = output::coverage(&files, &out.unresolved)?;
    let rendered = output::render(&files, &coverage)?;

    let data = root.join("data");
    let changes = output::diff(&data, &rendered);
    println!("{changes} difference(s); {} unresolved reference(s)", out.unresolved.len());
    if write {
        fs::create_dir_all(&data)?;
        output::update_changelog(&data, &rendered, &files)?;
        for (name, text) in &rendered {
            fs::write(data.join(name), text)?;
        }
        write_schemas(&data.join("schema"))?;
        println!("wrote {}", data.display());
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
                bail!("data/ does not match the fixtures");
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
