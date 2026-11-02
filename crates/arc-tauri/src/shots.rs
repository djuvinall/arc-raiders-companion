//! Screenshots people add of a stencil on a gun. They stay in the app data folder as
//! data URLs, so nothing of Embark's reaches the repo.

use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::path::Path;

const KINDS: &[&str] = &["data:image/png;base64,", "data:image/jpeg;base64,", "data:image/webp;base64,"];
const MAX_BYTES: usize = 24 * 1024 * 1024;

/// Screenshots by gun. `dir` is the stencil's own folder.
pub fn list(dir: &Path) -> BTreeMap<String, String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return BTreeMap::new() };
    entries.filter_map(|e| {
        let path = e.ok()?.path();
        Some((path.file_stem()?.to_string_lossy().into_owned(), std::fs::read_to_string(&path).ok()?))
    }).collect()
}

pub fn save(dir: &Path, gun: &str, image: &str) -> Result<()> {
    if !KINDS.iter().any(|k| image.starts_with(k)) {
        bail!("Use a PNG, JPEG or WebP image.");
    }
    if image.len() > MAX_BYTES {
        bail!("That image is too large.");
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(format!("{gun}.txt")), image)?;
    Ok(())
}

pub fn remove(dir: &Path, gun: &str) -> Result<()> {
    std::fs::remove_file(dir.join(format!("{gun}.txt")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshots_round_trip_and_only_images_are_kept() {
        let dir = std::env::temp_dir().join(format!("arc-shots-{}", std::process::id()));
        let image = "data:image/png;base64,AAAA";
        save(&dir, "anvil-i", image).unwrap();
        assert!(save(&dir, "anvil-ii", "data:text/html;base64,AAAA").is_err());
        assert_eq!(list(&dir).get("anvil-i").map(String::as_str), Some(image));
        assert_eq!(list(&dir).len(), 1);
        remove(&dir, "anvil-i").unwrap();
        assert!(list(&dir).is_empty());
    }
}
