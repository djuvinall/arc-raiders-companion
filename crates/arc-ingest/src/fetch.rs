//! Raw source fetchers. Each returns one JSON document that is saved verbatim as
//! a fixture, so normalize/merge can run offline against pinned snapshots.

use crate::http::Client;
use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

const METAFORGE: &str = "https://metaforge.app/api/arc-raiders";
const MAP_DATA: &str = "https://metaforge.app/api/game-map-data";
const MAP_IDS: &[&str] =
    &["dam", "spaceport", "buried-city", "blue-gate", "stella-montis", "riven-tides", "pendola-pass"];
const WIKI_API: &str = "https://arcraiders.wiki/w/api.php";
const RAIDTHEORY: &str = "RaidTheory/arcraiders-data";
const STEAM_NEWS: &str =
    "https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=1808500&count=20";

const CARGO_TABLES: &[&str] = &[
    "augments", "crafting", "items", "projectItems", "projects", "questItems", "quests",
    "recycling", "trials", "weaponMods", "weeklyTrials", "workshopUpgrades",
];
const CARGO_PAGE: usize = 500;

fn metaforge_paged(client: &Client, endpoint: &str) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    let mut page = 1;
    loop {
        let doc = client.get_json(&format!("{METAFORGE}/{endpoint}?page={page}&limit=100"))?;
        let rows = doc["data"].as_array().context("metaforge: no data array")?;
        out.extend(rows.iter().cloned());
        if !doc["pagination"]["hasNextPage"].as_bool().unwrap_or(false) {
            return Ok(out);
        }
        page += 1;
    }
}

/// `tableID` is not documented; `arc_map_data` was found by probing (see docs/DATA_SOURCES.md).
fn metaforge_maps(client: &Client) -> Result<Value> {
    let mut doc = Map::new();
    for map in MAP_IDS {
        let url = format!("{MAP_DATA}?mapID={map}&tableID=arc_map_data");
        match client.get_json(&url) {
            Ok(body) => { doc.insert((*map).to_string(), prune_markers(&body)); }
            Err(e) => eprintln!("  maps: {map} unavailable: {e}"),
        }
    }
    Ok(Value::Object(doc))
}

/// The raw payload is ~30 MB of per-container markers. Fixtures keep what ingest
/// reads: markers that carry loot areas, plus named locations.
fn prune_markers(body: &Value) -> Value {
    let kept: Vec<Value> = body["allData"].as_array().map(Vec::as_slice).unwrap_or(&[]).iter()
        .filter(|m| {
            m["category"] == "locations"
                || m["lootAreas"].as_str().is_some_and(|l| !l.is_empty())
        })
        .map(|m| json!({
            "id": m["id"], "lat": m["lat"], "lng": m["lng"], "zlayers": m["zlayers"], "category": m["category"],
            "subcategory": m["subcategory"], "instanceName": m["instanceName"],
            "lootAreas": m["lootAreas"],
        }))
        .collect();
    json!({ "marker_total": body["allData"].as_array().map_or(0, Vec::len), "markers": kept })
}

/// RaidTheory strings are `{lang: text}` maps; only English is used.
fn english_only(value: &mut Value) {
    match value {
        Value::Object(map) if map.contains_key("en") && map.contains_key("de") => {
            *value = map["en"].clone();
        }
        Value::Object(map) => map.values_mut().for_each(english_only),
        Value::Array(rows) => rows.iter_mut().for_each(english_only),
        _ => {}
    }
}

pub fn metaforge(client: &Client) -> Result<Value> {
    Ok(json!({
        "maps": metaforge_maps(client)?,
        "items": metaforge_paged(client, "items")?,
        "quests": metaforge_paged(client, "quests")?,
        "arcs": metaforge_paged(client, "arcs")?,
        "traders": client.get_json(&format!("{METAFORGE}/traders"))?["data"],
        "events": client.get_json(&format!("{METAFORGE}/events-schedule"))?,
    }))
}

fn cargo_fields(client: &Client, table: &str) -> Result<Vec<String>> {
    let url = format!("{WIKI_API}?action=cargofields&table={table}&format=json");
    let doc = client.get_json(&url)?;
    let fields = doc["cargofields"].as_object().context("no cargofields")?;
    Ok(fields.keys().cloned().collect())
}

fn cargo_table(client: &Client, table: &str) -> Result<Vec<Value>> {
    let mut fields = vec!["_pageName=page".to_string(), "_ID=row_id".to_string()];
    fields.extend(cargo_fields(client, table)?.into_iter().map(|f| format!("{f}={f}")));
    let fields = urlencoding::encode(&fields.join(",")).into_owned();

    let mut out = Vec::new();
    loop {
        let url = format!(
            "{WIKI_API}?action=cargoquery&format=json&tables={table}&fields={fields}\
             &order_by=_ID&limit={CARGO_PAGE}&offset={}",
            out.len()
        );
        let doc = client.get_json(&url)?;
        if let Some(err) = doc.get("error") {
            anyhow::bail!("cargoquery {table}: {err}");
        }
        let rows = doc["cargoquery"].as_array().context("no cargoquery array")?;
        let got = rows.len();
        out.extend(rows.iter().map(|r| r["title"].clone()));
        if got < CARGO_PAGE {
            return Ok(out);
        }
    }
}

pub fn wiki(client: &Client) -> Result<Value> {
    let mut doc = Map::new();
    for table in CARGO_TABLES {
        eprintln!("  wiki: {table}");
        doc.insert((*table).to_string(), Value::Array(cargo_table(client, table)?));
    }
    Ok(Value::Object(doc))
}

/// Fetches only the repo's JSON (the repo is ~228 MB of images otherwise).
pub fn raidtheory(client: &Client) -> Result<Value> {
    let commit = client.get_json(&format!(
        "https://api.github.com/repos/{RAIDTHEORY}/commits/main"
    ))?;
    let sha = commit["sha"].as_str().context("no commit sha")?.to_string();
    let date = commit["commit"]["author"]["date"].clone();
    let tree = client.get_json(&format!(
        "https://api.github.com/repos/{RAIDTHEORY}/git/trees/{sha}?recursive=1"
    ))?;

    let mut files = Map::new();
    for entry in tree["tree"].as_array().context("no tree")? {
        let path = entry["path"].as_str().unwrap_or("");
        let wanted = path.ends_with(".json")
            && !path.starts_with("arctracker-ui/")
            && !path.starts_with("images/")
            && !matches!(path, "package.json" | "package-lock.json");
        if wanted {
            let url = format!("https://raw.githubusercontent.com/{RAIDTHEORY}/{sha}/{path}");
            let mut doc = client.get_json(&url)?;
            english_only(&mut doc);
            files.insert(path.to_string(), doc);
        }
    }
    eprintln!("  raidtheory: {} files at {sha}", files.len());
    Ok(json!({ "commit": sha, "commit_date": date, "files": files }))
}

/// The Skills article only. The page's SkillTree widget is deliberately not fetched.
pub fn wiki_skills(client: &Client) -> Result<Value> {
    let url = format!("{WIKI_API}?action=parse&page=Skills&prop=wikitext&format=json");
    let doc = client.get_json(&url)?;
    Ok(json!({ "wikitext": doc["parse"]["wikitext"]["*"] }))
}

pub fn steam(client: &Client) -> Result<Value> {
    client.get_json(STEAM_NEWS)
}
