//! Player reports: the Report window sends one here, and this posts it to the
//! report relay, which files it on arc-raiders-data as the bot. The app holds
//! no GitHub credential.

use arc_core::DatasetFiles;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// The relay's address, from the first `wrangler deploy` (its workers.dev subdomain is
/// the Cloudflare account's). `ARC_REPORT_URL` overrides it for testing.
const REPORT_URL: &str = "https://arc-companion-reports.arc-companion-reports.workers.dev/report";

#[derive(Debug, Deserialize)]
pub struct ReportInput {
    /// "incorrect" or "suggestion".
    pub kind: String,
    pub data_ref: Option<String>,
    pub label: Option<String>,
    pub page: Option<String>,
    pub selected: Option<String>,
    pub suggestion: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSent {
    pub issue: u64,
    pub issue_url: String,
    pub pr_url: Option<String>,
}

fn rows(files: &DatasetFiles, file: &str) -> Option<Value> {
    let v = match file {
        "items" => serde_json::to_value(&files.items),
        "recipes" => serde_json::to_value(&files.recipes),
        "workbenches" => serde_json::to_value(&files.workbenches),
        "weapon_mods" => serde_json::to_value(&files.weapon_mods),
        "maps" => serde_json::to_value(&files.maps),
        "pois" => serde_json::to_value(&files.pois),
        "quests" => serde_json::to_value(&files.quests),
        "projects" => serde_json::to_value(&files.projects),
        "skills" => serde_json::to_value(&files.skills.nodes),
        "arcs" => serde_json::to_value(&files.arcs),
        "traders" => serde_json::to_value(&files.traders),
        "loot_areas" => serde_json::to_value(&files.loot_areas),
        _ => return None,
    };
    v.ok()
}

/// Whether `file/id[/field...]` names something in the dataset, and the field's
/// value when it names a field. Outer None: no such row or field.
pub fn resolve(files: &DatasetFiles, data_ref: &str) -> Option<Option<Value>> {
    let mut parts = data_ref.splitn(3, '/');
    let (file, id) = (parts.next()?, parts.next()?);
    let all = rows(files, file)?;
    let row = all.as_array()?.iter().find(|r| r.get("id").and_then(Value::as_str) == Some(id))?;
    match parts.next() {
        None => Some(None),
        Some(field) => row.pointer(&format!("/{field}")).map(|v| Some(v.clone())),
    }
}

pub fn send(files: &DatasetFiles, app_version: &str, input: ReportInput) -> Result<ReportSent, String> {
    if input.kind != "incorrect" && input.kind != "suggestion" {
        return Err("Pick a report type.".into());
    }
    // A tag that no longer resolves is the app's bug, not the player's; send it without one.
    let found = input.data_ref.as_deref().and_then(|r| resolve(files, r).map(|v| (r.to_string(), v)));
    let (data_ref, current) = match found {
        Some((r, v)) => (Some(r), v),
        None => (None, None),
    };
    let mut body = serde_json::json!({
        "type": input.kind,
        "ref": data_ref,
        "label": input.label,
        "page": input.page,
        "selected": input.selected,
        "suggestion": input.suggestion,
        "comment": input.comment,
        "app_version": app_version,
        "dataset_version": files.version.dataset_version,
        "game_patch": files.version.game_patch,
    });
    if let Some(v) = current {
        body["current"] = v;
    }
    let url = std::env::var("ARC_REPORT_URL").unwrap_or_else(|_| REPORT_URL.to_string());
    let resp = ureq::post(&url)
        .timeout(Duration::from_secs(20))
        .set("user-agent", &format!("arc-companion/{app_version}"))
        .set("content-type", "application/json")
        .send_string(&body.to_string());
    match resp {
        Ok(r) => r.into_string().ok().and_then(|t| serde_json::from_str(&t).ok())
            .ok_or_else(|| "The report service sent back something unexpected.".to_string()),
        Err(ureq::Error::Status(_, r)) => {
            let msg = r.into_string().ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                .and_then(|v| v["error"].as_str().map(str::to_string));
            Err(msg.unwrap_or_else(|| "The report service refused the report.".into()))
        }
        Err(_) => Err("Couldn't reach the report service. Check your connection and try again.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_rows_and_fields() {
        let ds = arc_core::Dataset::load(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/data")).unwrap();
        let f = &ds.files;
        assert_eq!(resolve(f, "items/anvil-i"), Some(None));
        assert_eq!(resolve(f, "items/anvil-i/rarity"), Some(Some(Value::from("rare"))));
        assert_eq!(resolve(f, "items/heavy-shield/stats/charge"), Some(Some(Value::from(90.0))));
        assert_eq!(resolve(f, "arcs/hydra/hp"), Some(Some(Value::from(1050.0))));
        assert_eq!(resolve(f, "items/no-such-item"), None);
        assert_eq!(resolve(f, "items/anvil-i/no_field"), None);
        assert_eq!(resolve(f, "secrets/x"), None);
    }
}
