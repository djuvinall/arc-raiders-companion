//! Live feeds (D12): every remote call is cached with a TTL, stale data beats no data,
//! and a cold cache offline just reports nothing. `ARC_OFFLINE=1` forces that path.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const EVENTS_TTL: Duration = Duration::from_secs(15 * 60);
pub const NEWS_TTL: Duration = Duration::from_secs(6 * 3600);
const MAX_FEED_BYTES: u64 = 16 * 1024 * 1024;

pub struct Fetched {
    pub bytes: Vec<u8>,
    pub fetched_ms: i64,
    /// True when the copy is older than the TTL because the network failed.
    pub stale: bool,
}

fn age(path: &Path) -> Option<(Duration, i64)> {
    let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok()?;
    let at = modified.duration_since(UNIX_EPOCH).ok()?.as_millis() as i64;
    Some((SystemTime::now().duration_since(modified).unwrap_or_default(), at))
}

pub fn download(url: &str) -> Option<Vec<u8>> {
    if std::env::var_os("ARC_OFFLINE").is_some() {
        return None;
    }
    let resp = ureq::get(url).timeout(Duration::from_secs(15)).call().ok()?;
    let mut body = Vec::new();
    resp.into_reader().take(MAX_FEED_BYTES).read_to_end(&mut body).ok()?;
    (!body.is_empty()).then_some(body)
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// Fresh cache, else download if `valid` accepts it, else stale cache.
pub fn get(dir: Option<PathBuf>, name: &str, url: &str, ttl: Duration, valid: impl Fn(&[u8]) -> bool) -> Option<Fetched> {
    let path = dir.map(|d| d.join("feeds").join(name));
    if let Some(p) = &path {
        if let (Some((old, at)), Ok(bytes)) = (age(p), std::fs::read(p)) {
            if old < ttl {
                return Some(Fetched { bytes, fetched_ms: at, stale: false });
            }
        }
    }
    if let Some(bytes) = download(url).filter(|b| valid(b)) {
        if let Some(p) = &path {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(p, &bytes);
        }
        return Some(Fetched { bytes, fetched_ms: now_ms(), stale: false });
    }
    let p = path?;
    let (_, at) = age(&p)?;
    Some(Fetched { bytes: std::fs::read(&p).ok()?, fetched_ms: at, stale: true })
}
