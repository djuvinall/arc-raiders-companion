//! Player-reported corrections: one JSON file per accepted report in
//! `dataset/overlays/corrections/`, applied last so they win over every source.
//!
//! A file looks like:
//! `{"file": "items", "id": "anvil-i", "field": "/value", "value": 5200, "was": 5000, "report": 12}`
//! `field` is a JSON pointer into the row with that `id`, and the pointed-at field
//! must already exist. When the sources no longer give `was`, the sources changed
//! since the report, so the correction is still applied but flagged for review.

use anyhow::{bail, Context, Result};
use arc_core::dataset::DatasetFiles;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub file: String,
    pub id: String,
    pub field: String,
    pub value: Value,
    #[serde(default)]
    pub was: Option<Value>,
    /// Issue number on arc-raiders-data. Kept for the reviewer, not read here.
    #[serde(default)]
    #[allow(dead_code)]
    pub report: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub comment: Option<String>,
}

pub fn load(dir: &Path) -> Result<Vec<(String, Correction)>> {
    let Ok(entries) = fs::read_dir(dir) else { return Ok(Vec::new()) };
    let mut names: Vec<_> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let text = fs::read_to_string(&p)?;
            let c = serde_json::from_str(&text).with_context(|| format!("corrections/{name}"))?;
            Ok((name, c))
        })
        .collect()
}

/// Sets `field` on the row with `id`; returns the value it replaced.
fn patch<T: Serialize + DeserializeOwned>(rows: &mut [T], c: &Correction) -> Result<Value> {
    let mut found = None;
    for (i, row) in rows.iter().enumerate() {
        let v = serde_json::to_value(row)?;
        if v.get("id").and_then(Value::as_str) == Some(c.id.as_str()) {
            found = Some((i, v));
            break;
        }
    }
    let Some((i, mut row)) = found else { bail!("no {} row with id {}", c.file, c.id) };
    let slot = row.pointer_mut(&c.field).with_context(|| format!("{}/{} has no field {}", c.file, c.id, c.field))?;
    let old = std::mem::replace(slot, c.value.clone());
    rows[i] = serde_json::from_value(row).with_context(|| format!("{}/{}{}: value doesn't fit the field", c.file, c.id, c.field))?;
    Ok(old)
}

/// JSON equality where 1050 and 1050.0 are the same number.
fn same(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

pub fn apply(files: &mut DatasetFiles, list: &[(String, Correction)]) -> Result<()> {
    for (name, c) in list {
        if !c.field.starts_with('/') {
            bail!("corrections/{name}: field must be a JSON pointer starting with /");
        }
        let old = match c.file.as_str() {
            "items" => patch(&mut files.items, c),
            "recipes" => patch(&mut files.recipes, c),
            "workbenches" => patch(&mut files.workbenches, c),
            "weapon_mods" => patch(&mut files.weapon_mods, c),
            "maps" => patch(&mut files.maps, c),
            "pois" => patch(&mut files.pois, c),
            "quests" => patch(&mut files.quests, c),
            "projects" => patch(&mut files.projects, c),
            "skills" => patch(&mut files.skills.nodes, c),
            "arcs" => patch(&mut files.arcs, c),
            "traders" => patch(&mut files.traders, c),
            "loot_areas" => patch(&mut files.loot_areas, c),
            other => bail!("corrections/{name}: unknown file {other}"),
        }
        .with_context(|| format!("corrections/{name}"))?;
        if let Some(was) = &c.was {
            if !same(was, &old) && !same(&old, &c.value) {
                println!("note: corrections/{name}: sources now give {old}, not {was}; check it still applies");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Serialize, Deserialize)]
    struct Row {
        id: String,
        value: u32,
        stats: Option<Value>,
    }

    fn c(field: &str, value: Value) -> Correction {
        Correction { file: "items".into(), id: "b".into(), field: field.into(), value, was: None, report: None, comment: None }
    }

    fn rows() -> Vec<Row> {
        vec![
            Row { id: "a".into(), value: 1, stats: None },
            Row { id: "b".into(), value: 2, stats: Some(json!({"charge": 80.0})) },
        ]
    }

    #[test]
    fn sets_a_top_level_and_a_nested_field() {
        let mut r = rows();
        assert_eq!(patch(&mut r, &c("/value", json!(7))).unwrap(), json!(2));
        assert_eq!(r[1].value, 7);
        patch(&mut r, &c("/stats/charge", json!(90.0))).unwrap();
        assert_eq!(r[1].stats.as_ref().unwrap()["charge"], json!(90.0));
        assert_eq!(r[0].value, 1);
    }

    #[test]
    fn rejects_unknown_rows_fields_and_wrong_types() {
        let mut r = rows();
        let mut bad = c("/value", json!(3));
        bad.id = "zzz".into();
        assert!(patch(&mut r, &bad).is_err());
        assert!(patch(&mut r, &c("/nope", json!(3))).is_err());
        assert!(patch(&mut r, &c("/value", json!("lots"))).is_err());
        assert_eq!(r[1].value, 2);
    }
}
