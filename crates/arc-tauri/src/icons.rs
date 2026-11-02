//! `arcicon:` protocol: serves MetaForge icons from a local cache (D7, D12). Nothing
//! of Embark's ships in the repo. A cold cache offline answers 404 and the UI falls
//! back to a text tile.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tauri::http::{header, Response, StatusCode};

const ALLOWED_PREFIXES: &[&str] = &[
    "https://static.metaforge.app/",
    "https://arcraiders.wiki/wiki/Special:FilePath/",
];
const TTL: Duration = Duration::from_secs(30 * 24 * 3600);
const MAX_ICON_BYTES: u64 = 8 * 1024 * 1024;

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%' && i + 2 < bytes.len())
            .then(|| std::str::from_utf8(&bytes[i + 1..i + 3]).ok())
            .flatten()
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(b) => {
                out.push(b);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// FNV-1a: stable across runs, unlike the std hasher.
fn cache_name(url: &str) -> String {
    let hash = url.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    format!("{hash:016x}")
}

fn mime(url: &str) -> &'static str {
    if url.ends_with(".webp") {
        "image/webp"
    } else if url.ends_with(".jpg") || url.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "image/png"
    }
}

fn fresh(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < TTL)
}

/// `ARC_OFFLINE=1` forces the no-network path, which is how the offline test runs.
fn download(url: &str) -> Option<Vec<u8>> {
    if std::env::var_os("ARC_OFFLINE").is_some() {
        return None;
    }
    let resp = ureq::get(url).timeout(Duration::from_secs(15)).call().ok()?;
    let mut body = Vec::new();
    resp.into_reader().take(MAX_ICON_BYTES).read_to_end(&mut body).ok()?;
    (!body.is_empty()).then_some(body)
}

/// Fresh cache, else download, else stale cache.
fn load(cache_dir: Option<&Path>, url: &str) -> Option<Vec<u8>> {
    let path = cache_dir.map(|d| d.join("icons").join(cache_name(url)));
    if let Some(p) = path.as_deref().filter(|p| fresh(p)) {
        if let Ok(bytes) = std::fs::read(p) {
            return Some(bytes);
        }
    }
    if let Some(bytes) = download(url) {
        if let Some(p) = &path {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(p, &bytes);
        }
        return Some(bytes);
    }
    path.and_then(|p| std::fs::read(p).ok())
}

fn respond(status: StatusCode, mime_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(body)
        .expect("static response parts are valid")
}

pub fn serve(cache_dir: Option<PathBuf>, request_path: &str) -> Response<Vec<u8>> {
    let url = percent_decode(request_path.trim_start_matches('/'));
    if !ALLOWED_PREFIXES.iter().any(|p| url.starts_with(p)) {
        return respond(StatusCode::FORBIDDEN, "text/plain", Vec::new());
    }
    match load(cache_dir.as_deref(), &url) {
        Some(bytes) => respond(StatusCode::OK, mime(&url), bytes),
        None => respond(StatusCode::NOT_FOUND, "text/plain", Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_allowed_hosts_are_proxied() {
        let denied = serve(None, "/https%3A%2F%2Fexample.com%2Fx.png");
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn decodes_the_url_from_the_path() {
        assert_eq!(percent_decode("https%3A%2F%2Fa.b%2Fc.png"), "https://a.b/c.png");
        assert_eq!(cache_name("x"), cache_name("x"));
        assert_ne!(cache_name("x"), cache_name("y"));
    }
}
