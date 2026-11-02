//! `arc-ingest probe`: snapshot every source into tests/fixtures and report, in
//! numbers, how much of the item list each one covers with recipes.

use crate::{fetch, http::Client};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn fixture_path(root: &Path, source: &str, date: &str) -> PathBuf {
    root.join("tests/fixtures").join(source).join(format!("{date}.json"))
}

fn save(root: &Path, source: &str, date: &str, doc: &Value) -> Result<()> {
    let path = fixture_path(root, source, date);
    fs::create_dir_all(path.parent().context("fixture parent")?)?;
    fs::write(&path, serde_json::to_vec_pretty(doc)?)?;
    eprintln!("wrote {}", path.display());
    Ok(())
}

fn load_or_fetch(
    root: &Path,
    source: &str,
    date: &str,
    fetcher: impl FnOnce() -> Result<Value>,
) -> Result<Value> {
    let path = fixture_path(root, source, date);
    if path.exists() {
        eprintln!("reusing {}", path.display());
        return Ok(serde_json::from_slice(&fs::read(path)?)?);
    }
    let doc = fetcher()?;
    save(root, source, date, &doc)?;
    Ok(doc)
}

pub fn run(root: &Path) -> Result<()> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let polite = Client::new(250);
    let quick = Client::new(25);

    let mf = load_or_fetch(root, "metaforge", &date, || fetch::metaforge(&polite))?;
    let wiki = load_or_fetch(root, "wiki", &date, || fetch::wiki(&polite))?;
    let rt = load_or_fetch(root, "raidtheory", &date, || fetch::raidtheory(&quick))?;
    load_or_fetch(root, "steam", &date, || fetch::steam(&polite))?;
    load_or_fetch(root, "wiki_skills", &date, || fetch::wiki_skills(&polite))?;

    report(&mf, &wiki, &rt);
    Ok(())
}

fn str_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or("")
}

fn report(mf: &Value, wiki: &Value, rt: &Value) {
    let items = mf["items"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let cosmetic = items.iter().filter(|i| !i["cosmetic"].is_null()).count();
    let game_items = items.len() - cosmetic;
    println!("MetaForge: {} items ({} cosmetic, {} gameplay)", items.len(), cosmetic, game_items);
    println!("  quests {}, arcs {}", mf["quests"].as_array().map_or(0, Vec::len),
        mf["arcs"].as_array().map_or(0, Vec::len));
    let mf_recipes = items.iter().filter(|i| {
        !i["workbench"].is_null() || !i["components"].is_null() || !i["recipe"].is_null()
    }).count();
    println!("  items with any recipe/workbench field: {mf_recipes}");

    let crafting = wiki["crafting"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let wiki_outputs: BTreeSet<&str> =
        crafting.iter().map(|r| str_field(r, "resultItem")).filter(|s| !s.is_empty()).collect();
    let recycling = wiki["recycling"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let wiki_recycle: BTreeSet<&str> =
        recycling.iter().map(|r| str_field(r, "inputItem")).filter(|s| !s.is_empty()).collect();
    println!("Wiki: {} crafting rows -> {} distinct outputs; {} recycling rows -> {} inputs; {} items",
        crafting.len(), wiki_outputs.len(), recycling.len(), wiki_recycle.len(),
        wiki["items"].as_array().map_or(0, Vec::len));

    let files = rt["files"].as_object();
    let rt_items: Vec<&Value> = files
        .map(|f| f.iter().filter(|(k, _)| k.starts_with("items/")).map(|(_, v)| v).collect())
        .unwrap_or_default();
    let has = |v: &Value, key: &str| v[key].as_object().is_some_and(|o| !o.is_empty());
    println!("Map markers kept: {}", mf["maps"].as_object().map_or(0, |m| {
        m.values().map(|v| v["markers"].as_array().map_or(0, Vec::len)).sum::<usize>()
    }));
    let rt_craft = rt_items.iter().filter(|i| has(i, "recipe")).count();
    let rt_recycle = rt_items.iter().filter(|i| has(i, "recyclesInto")).count();
    println!("RaidTheory @ {}: {} items, {} with recipe, {} with recyclesInto",
        str_field(rt, "commit_date"), rt_items.len(), rt_craft, rt_recycle);

    let mf_names: BTreeSet<String> = items.iter()
        .filter(|i| i["cosmetic"].is_null())
        .map(|i| str_field(i, "name").to_lowercase()).collect();
    let wiki_lower: BTreeSet<String> = wiki_outputs.iter().map(|s| s.to_lowercase()).collect();
    let matched = wiki_lower.intersection(&mf_names).count();
    println!("Name match: {matched}/{} wiki craft outputs match a MetaForge gameplay item by name",
        wiki_lower.len());
    let unmatched: Vec<_> = wiki_lower.difference(&mf_names).take(25).collect();
    println!("  sample unmatched: {unmatched:?}");
}
