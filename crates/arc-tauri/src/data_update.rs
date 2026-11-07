//! Item data newer than the exe. A download goes through the same validation as the
//! embedded copy and lands in the app data folder, and the next launch uses whichever
//! copy is newer. A file this exe can't read is refused, so old builds stay on their data.

use crate::feeds;
use anyhow::{Context, Result};
use arc_core::Dataset;
use serde_json::Value;
use std::path::Path;

const BASE_URL: &str = "https://raw.githubusercontent.com/djuvinall/arc-raiders-data/main/data";
pub const FILES: &[&str] = &[
    "items.json", "recipes.json", "workbenches.json", "weapon_mods.json", "maps.json",
    "pois.json", "quests.json", "projects.json", "skills.json", "arcs.json", "traders.json",
    "loot_areas.json", "map_markers.json", "version.json", "changelog.json", "notes.json",
];

fn dataset_version(text: &str) -> Option<String> {
    let version: Value = serde_json::from_str(text).ok()?;
    version["dataset_version"].as_str().map(str::to_string)
}

fn read_from(dir: &Path) -> impl Fn(&str) -> Result<String> + '_ {
    move |name| std::fs::read_to_string(dir.join(name)).with_context(|| format!("reading {name}"))
}

/// The dataset to run on and its changelog: the download when it's newer and valid.
pub fn load(dir: &Path, embedded: &dyn Fn(&str) -> Result<String>) -> Result<(Dataset, String)> {
    let read = read_from(dir);
    let shipped = embedded("version.json").ok().and_then(|t| dataset_version(&t));
    let downloaded = read("version.json").ok().and_then(|t| dataset_version(&t));
    if downloaded > shipped {
        if let (Ok(dataset), Ok(changelog)) = (Dataset::load_with(&read), read("changelog.json")) {
            return Ok((dataset, changelog));
        }
    }
    Ok((Dataset::load_with(embedded)?, embedded("changelog.json")?))
}

/// Downloads newer data if there is any. Returns its version, ready for the next launch.
pub fn fetch(dir: &Path, current: &str) -> Result<Option<String>> {
    let text = |name: &str| -> Result<Vec<u8>> {
        feeds::download(&format!("{BASE_URL}/{name}")).with_context(|| format!("downloading {name}"))
    };
    let remote = dataset_version(&String::from_utf8_lossy(&text("version.json")?)).context("no dataset version")?;
    if remote.as_str() <= current {
        return Ok(None);
    }
    let waiting = read_from(dir)("version.json").ok().and_then(|t| dataset_version(&t));
    if waiting.as_deref() == Some(remote.as_str()) {
        return Ok(Some(remote));
    }
    let staging = dir.with_extension("tmp");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging)?;
    for name in FILES {
        std::fs::write(staging.join(name), text(name)?)?;
    }
    Dataset::load_with(&read_from(&staging)).context("this version can't read the new data")?;
    let _ = std::fs::remove_dir_all(dir);
    std::fs::rename(&staging, dir)?;
    Ok(Some(remote))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/data")
    }

    fn copy_with_version(version: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("arc-data-{version}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in FILES {
            std::fs::copy(data().join(name), dir.join(name)).unwrap();
        }
        let path = dir.join("version.json");
        let mut parsed: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        parsed["dataset_version"] = version.into();
        std::fs::write(path, parsed.to_string()).unwrap();
        dir
    }

    fn loaded_version(dir: &Path) -> String {
        let shipped = data();
        let (dataset, _) = load(dir, &read_from(&shipped)).unwrap();
        dataset.files.version.dataset_version
    }

    #[test]
    fn a_newer_download_wins_and_an_older_one_does_not() {
        let newer = copy_with_version("2999-01-01");
        assert_eq!(loaded_version(&newer), "2999-01-01");
        let older = copy_with_version("2000-01-01");
        assert_ne!(loaded_version(&older), "2000-01-01");
    }

    #[test]
    fn a_broken_download_falls_back_to_the_shipped_data() {
        let broken = copy_with_version("2998-01-01");
        std::fs::write(broken.join("items.json"), "[{\"unknown\": true}]").unwrap();
        assert_ne!(loaded_version(&broken), "2998-01-01");
    }
}
