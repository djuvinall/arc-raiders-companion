//! The dataset version follows the data, not the source snapshot date. `dataset/stamp.json`
//! holds a hash of the rebuilt data and the version it was given; when a rebuild gives
//! different data (new snapshot, overlay, correction or ingest change), `sync --write` moves
//! the version to today, so apps download it. A second change on the same day is
//! `2026-11-08.02`, which still sorts after `2026-11-08` as the apps compare versions as text.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stamp {
    pub date: String,
    /// Changes so far on `date`, from 1.
    pub n: u32,
    pub hash: String,
}

impl Stamp {
    pub fn version(&self) -> String {
        if self.n <= 1 { self.date.clone() } else { format!("{}.{:02}", self.date, self.n) }
    }
}

/// FNV-1a over every rendered file except version.json, which holds the version itself.
pub fn hash(rendered: &BTreeMap<&str, String>) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (name, text) in rendered.iter().filter(|(n, _)| **n != "version.json") {
        for b in name.bytes().chain([0]).chain(text.bytes()).chain([0]) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

pub fn read(path: &Path) -> Result<Option<Stamp>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(serde_json::from_str(&text).with_context(|| format!("reading {}", path.display()))?)),
        Err(_) => Ok(None),
    }
}

pub fn write(path: &Path, stamp: &Stamp) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(stamp)? + "\n")?;
    Ok(())
}

/// The stamp for data with this hash: the old one when nothing changed, else a new one
/// dated `today`.
pub fn next(old: Option<&Stamp>, hash: &str, today: &str) -> Stamp {
    match old {
        Some(s) if s.hash == hash => s.clone(),
        Some(s) if s.date == today => Stamp { date: today.into(), n: s.n + 1, hash: hash.into() },
        _ => Stamp { date: today.into(), n: 1, hash: hash.into() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_move_with_the_data_and_sort_as_text() {
        let first = next(None, "a", "2026-11-08");
        assert_eq!(first.version(), "2026-11-08");
        assert_eq!(next(Some(&first), "a", "2026-11-09"), first, "same data, same version");
        let second = next(Some(&first), "b", "2026-11-08");
        assert_eq!(second.version(), "2026-11-08.02");
        assert!(second.version().as_str() > first.version().as_str());
        let third = next(Some(&second), "c", "2026-11-09");
        assert_eq!(third.version(), "2026-11-09");
        assert!(third.version().as_str() > second.version().as_str());
    }

    #[test]
    fn the_hash_skips_the_version_file() {
        let mut a = BTreeMap::from([("items.json", "x".to_string()), ("version.json", "1".to_string())]);
        let h = hash(&a);
        a.insert("version.json", "2".to_string());
        assert_eq!(hash(&a), h);
        a.insert("items.json", "y".to_string());
        assert_ne!(hash(&a), h);
    }
}
