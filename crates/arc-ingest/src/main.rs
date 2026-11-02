//! Data pipeline. No LLM anywhere in this path (D6): fetch, normalize, diff, and
//! a human commits the result.
//!
//!   arc-ingest probe            snapshot sources into tests/fixtures, print coverage
//!   arc-ingest diff             rebuild from the newest fixtures, compare with data/
//!   arc-ingest sync [--write]   same diff, and write data/ with --write

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
