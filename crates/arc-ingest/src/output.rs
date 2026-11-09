//! Serialize a dataset, report coverage, and diff it against data/.

use anyhow::Result;
use arc_core::model::{ChangelogEntry, FileChange};
use arc_core::DatasetFiles;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn pretty<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)? + "\n")
}

/// file name -> contents, exactly as written to data/.
pub fn render(files: &DatasetFiles, coverage: &Value) -> Result<BTreeMap<&'static str, String>> {
    Ok(BTreeMap::from([
        ("items.json", pretty(&files.items)?),
        ("recipes.json", pretty(&files.recipes)?),
        ("workbenches.json", pretty(&files.workbenches)?),
        ("weapon_mods.json", pretty(&files.weapon_mods)?),
        ("maps.json", pretty(&files.maps)?),
        ("pois.json", pretty(&files.pois)?),
        ("quests.json", pretty(&files.quests)?),
        ("projects.json", pretty(&files.projects)?),
        ("skills.json", pretty(&files.skills)?),
        ("arcs.json", pretty(&files.arcs)?),
        ("traders.json", pretty(&files.traders)?),
        ("loot_areas.json", pretty(&files.loot_areas)?),
        // One line: about 43,000 markers, so indenting would triple the file.
        ("map_markers.json", serde_json::to_string(&files.markers)? + "\n"),
        ("version.json", pretty(&files.version)?),
        ("notes.json", pretty(&files.notes)?),
        ("project_tasks.json", pretty(&files.project_tasks)?),
        ("containers.json", pretty(&files.containers)?),
        ("conditions.json", pretty(&files.conditions)?),
        ("coverage.json", pretty(coverage)?),
    ]))
}

fn populated(rows: &[Value], fields: &[&str]) -> Value {
    let total = rows.len().max(1) as f64;
    let mut out = Map::new();
    for field in fields {
        let filled = rows.iter().filter(|r| match &r[*field] {
            Value::Null => false,
            Value::Array(a) => !a.is_empty(),
            Value::String(s) => !s.is_empty(),
            _ => true,
        }).count();
        let pct = (filled as f64 / total * 1000.0).round() / 10.0;
        out.insert((*field).to_string(), json!(pct));
    }
    Value::Object(out)
}

fn by_key(rows: &[Value], key: &str) -> Value {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for r in rows {
        *counts.entry(r[key].as_str().unwrap_or("null").to_string()).or_default() += 1;
    }
    json!(counts)
}

pub fn coverage(files: &DatasetFiles, unresolved: &[String]) -> Result<Value> {
    let items: Vec<Value> = serde_json::from_value(serde_json::to_value(&files.items)?)?;
    let recipes: Vec<Value> = serde_json::from_value(serde_json::to_value(&files.recipes)?)?;
    let quests: Vec<Value> = serde_json::from_value(serde_json::to_value(&files.quests)?)?;
    let gameplay: Vec<Value> = items.iter()
        .filter(|i| !matches!(i["category"].as_str(), Some("cosmetic" | "furniture")))
        .cloned().collect();
    Ok(json!({
        "items": {
            "count": items.len(),
            "gameplay_count": gameplay.len(),
            "by_provenance": by_key(&items, "provenance"),
            "by_category": by_key(&items, "category"),
            "populated_pct_gameplay": populated(&gameplay, &[
                "description", "icon", "value", "weight_kg", "rarity", "tier", "stats",
                "loot_areas", "sold_by", "recipe_ids", "recycle_ids", "drops_from",
            ]),
        },
        "recipes": { "count": recipes.len(), "by_kind": by_key(&recipes, "kind"),
            "by_provenance": by_key(&recipes, "provenance"),
            "populated_pct": populated(&recipes, &["workbench_id", "workbench_level", "craft_time_s"]) },
        "quests": { "count": quests.len(),
            "with_map": files.quests.iter().filter(|q| q.objectives.iter().any(|o| !o.maps.is_empty())).count(),
            "with_prerequisites": files.quests.iter().filter(|q| !q.prerequisites.is_empty()).count() },
        "weapon_mods": files.weapon_mods.len(),
        "workbenches": files.workbenches.len(),
        "projects": files.projects.len(),
        "maps": files.maps.len(),
        "pois": files.pois.len(),
        "loot_areas": files.loot_areas.len(),
        "traders": files.traders.len(),
        "arcs": files.arcs.len(),
        "skill_nodes": files.skills.nodes.len(),
        "unresolved_count": unresolved.len(),
        "unresolved": unresolved,
    }))
}

fn keyed(text: &str) -> BTreeMap<String, Value> {
    let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(text) else {
        return BTreeMap::new();
    };
    rows.into_iter()
        .filter_map(|r| r["id"].as_str().map(|id| (id.to_string(), r.clone())))
        .collect()
}

/// Prints added/removed/changed per file. Returns the number of differences.
pub fn diff(data_dir: &Path, rendered: &BTreeMap<&'static str, String>) -> usize {
    let mut total = 0;
    for (name, new_text) in rendered {
        let old_text = fs::read_to_string(data_dir.join(name)).unwrap_or_default();
        if old_text == *new_text {
            continue;
        }
        let (old, new) = (keyed(&old_text), keyed(new_text));
        if old.is_empty() && new.is_empty() {
            println!("{name}: changed");
            total += 1;
            continue;
        }
        let added: Vec<_> = new.keys().filter(|k| !old.contains_key(*k)).collect();
        let removed: Vec<_> = old.keys().filter(|k| !new.contains_key(*k)).collect();
        let changed: Vec<_> = new.iter()
            .filter(|(k, v)| old.get(*k).is_some_and(|o| o != *v))
            .map(|(k, _)| k).collect();
        total += added.len() + removed.len() + changed.len();
        println!("{name}: +{} -{} ~{}", added.len(), removed.len(), changed.len());
        for (mark, list) in [("+", &added), ("-", &removed), ("~", &changed)] {
            for id in list.iter().take(10) {
                println!("  {mark} {id}");
            }
            if list.len() > 10 {
                println!("  {mark} ... {} more", list.len() - 10);
            }
        }
    }
    total
}

const NAMES_KEPT: usize = 200;

fn rows(text: &str) -> BTreeMap<String, (String, Value)> {
    let parsed: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let list = parsed.get("nodes").unwrap_or(&parsed);
    list.as_array().into_iter().flatten().filter_map(|row| {
        let id = row["id"].as_str()?.to_string();
        let name = row["name"].as_str().unwrap_or(&id).to_string();
        Some((id, (name, row.clone())))
    }).collect()
}

fn file_change(file: &str, old: &str, new: &str) -> Option<FileChange> {
    let (before, after) = (rows(old), rows(new));
    let names = |ids: Vec<&String>, side: &BTreeMap<String, (String, Value)>| -> Vec<String> {
        ids.into_iter().take(NAMES_KEPT).map(|id| side[id].0.clone()).collect()
    };
    let added = names(after.keys().filter(|id| !before.contains_key(*id)).collect(), &after);
    let removed = names(before.keys().filter(|id| !after.contains_key(*id)).collect(), &before);
    let changed = names(after.iter().filter(|(id, row)| before.get(*id).is_some_and(|b| b.1 != row.1))
        .map(|(id, _)| id).collect(), &after);
    (!added.is_empty() || !removed.is_empty() || !changed.is_empty())
        .then(|| FileChange { file: file.to_string(), added, removed, changed })
}

/// Records what this write changes, before data/ is overwritten. Writes on one day merge
/// into one entry; the first ever write records totals rather than every row as "added".
pub fn update_changelog(data: &Path, rendered: &BTreeMap<&str, String>, files: &DatasetFiles) -> Result<()> {
    let path = data.join("changelog.json");
    let existing = fs::read_to_string(&path).ok();
    let mut log: Vec<ChangelogEntry> = match &existing {
        Some(text) => serde_json::from_str(text)?,
        None => Vec::new(),
    };
    let version = files.version.dataset_version.clone();
    let patch = files.version.game_patch.clone();
    let mut changes = Vec::new();
    if existing.is_some() {
        for (name, text) in rendered {
            if matches!(*name, "coverage.json" | "version.json") {
                continue;
            }
            let old = fs::read_to_string(data.join(name)).unwrap_or_default();
            changes.extend(file_change(name, &old, text));
        }
        if changes.is_empty() && log.first().is_some_and(|e| e.game_patch == patch) {
            return Ok(());
        }
    }
    let position = log.iter().position(|e| e.dataset_version == version);
    let entry = match position {
        Some(i) => &mut log[i],
        None => {
            log.insert(0, ChangelogEntry {
                dataset_version: version, game_patch: None, summary: Vec::new(), files: Vec::new(),
            });
            &mut log[0]
        }
    };
    entry.game_patch = patch;
    if existing.is_none() {
        entry.summary.push(format!(
            "First dataset: {} items, {} recipes, {} quests, {} projects, {} skills.",
            files.items.len(), files.recipes.len(), files.quests.len(), files.projects.len(), files.skills.nodes.len(),
        ));
    }
    entry.files.extend(changes);
    // Entries are newest first, so the one after each says which patch came before it.
    for i in 0..log.len() {
        let earlier = log.get(i + 1).and_then(|e| e.game_patch.clone());
        tidy_entry(&mut log[i], earlier.as_deref());
    }
    fs::write(path, serde_json::to_string_pretty(&log)? + "\n")?;
    Ok(())
}

/// Folds a later write for the same file into one. Names repeat (many pois are called
/// "Locker"), so each name is kept as many times as either write listed it.
fn merge_change(files: &mut Vec<FileChange>, change: FileChange) {
    let Some(old) = files.iter_mut().find(|f| f.file == change.file) else {
        files.push(change);
        return;
    };
    for (into, from) in [(&mut old.added, change.added), (&mut old.removed, change.removed), (&mut old.changed, change.changed)] {
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for name in from {
            let n = seen.entry(name.clone()).or_default();
            *n += 1;
            if into.iter().filter(|x| **x == name).count() < *n {
                into.push(name);
            }
        }
    }
}

/// Rebuilds an entry's lines from its files, so a re-sync on the same dataset version
/// leaves one line per file rather than stacking duplicates. A count at the name cap
/// gets "+", since names past it aren't kept.
fn tidy_entry(entry: &mut ChangelogEntry, earlier_patch: Option<&str>) {
    let mut merged: Vec<FileChange> = Vec::new();
    for change in std::mem::take(&mut entry.files) {
        merge_change(&mut merged, change);
    }
    entry.files = merged;
    let count = |v: &[String]| if v.len() >= NAMES_KEPT { format!("{}+", v.len()) } else { v.len().to_string() };
    let mut summary: Vec<String> = entry.summary.iter().filter(|l| l.starts_with("First dataset")).cloned().collect();
    if let (Some(was), Some(now)) = (earlier_patch, entry.game_patch.as_deref()) {
        if was != now {
            summary.push(format!("Checked against game patch {now} (was {was})."));
        }
    }
    for f in &entry.files {
        summary.push(format!(
            "{}: {} added, {} removed, {} changed.",
            f.file.trim_end_matches(".json"), count(&f.added), count(&f.removed), count(&f.changed),
        ));
    }
    if entry.files.is_empty() {
        summary.push("No data changes.".into());
    }
    entry.summary = summary;
}
