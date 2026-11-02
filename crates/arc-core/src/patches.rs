//! Phase 6: Steam news to readable notes. BBCode becomes plain blocks so the UI never
//! renders remote markup. The feed mixes in press pieces; only the developer's own
//! announcements count as patch notes.

use serde::{Deserialize, Serialize};

const OFFICIAL_FEED: &str = "steam_community_announcements";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    /// `heading`, `item` or `text`.
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PatchNote {
    pub gid: String,
    pub title: String,
    pub url: String,
    pub date: i64,
    pub version: Option<String>,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize)]
struct Feed {
    appnews: AppNews,
}

#[derive(Deserialize)]
struct AppNews {
    newsitems: Vec<NewsItem>,
}

#[derive(Deserialize)]
struct NewsItem {
    gid: String,
    title: String,
    url: String,
    date: i64,
    feedname: String,
    contents: String,
}

/// First `2.3` or `2.3.0` shaped word in a title.
pub fn version_in_title(title: &str) -> Option<String> {
    title.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric()))
        .find(|w| {
            let parts: Vec<&str> = w.split('.').collect();
            (2..=3).contains(&parts.len())
                && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        })
        .map(str::to_string)
}

struct Builder {
    blocks: Vec<Block>,
    text: String,
    heading: bool,
    lists: u32,
}

impl Builder {
    fn flush(&mut self) {
        for line in std::mem::take(&mut self.text).lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let kind = if self.heading { "heading" } else if self.lists > 0 { "item" } else { "text" };
            self.blocks.push(Block { kind: kind.into(), text: line.to_string() });
        }
    }
}

const BLOCK_TAGS: &[&str] = &["p", "h1", "h2", "h3", "h4", "*", "li", "list", "olist", "table", "tr", "td"];
const DROPPED_BODY: &[&str] = &["img", "previewyoutube", "video"];

fn tag_name(body: &str) -> Option<String> {
    let name: String = body.trim_start_matches('/')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '*')
        .collect();
    (!name.is_empty()).then(|| name.to_lowercase())
}

/// Unknown tags are removed but their text stays, so `[url=x]here[/url]` reads "here".
pub fn bbcode_blocks(source: &str) -> Vec<Block> {
    let mut out = Builder { blocks: Vec::new(), text: String::new(), heading: false, lists: 0 };
    let mut rest = source;
    while let Some(open) = rest.find('[') {
        out.text.push_str(&rest[..open]);
        rest = &rest[open..];
        let Some(close) = rest.find(']') else { break };
        let body = &rest[1..close];
        let Some(name) = tag_name(body).filter(|_| !body.contains('[')) else {
            out.text.push('[');
            rest = &rest[1..];
            continue;
        };
        rest = &rest[close + 1..];
        let closing = body.starts_with('/');
        if !closing && DROPPED_BODY.contains(&name.as_str()) {
            let end = format!("[/{name}]");
            if let Some(i) = rest.find(&end) {
                rest = &rest[i + end.len()..];
            }
            continue;
        }
        if BLOCK_TAGS.contains(&name.as_str()) {
            out.flush();
            match (name.as_str(), closing) {
                ("list" | "olist", false) => out.lists += 1,
                ("list" | "olist", true) => out.lists = out.lists.saturating_sub(1),
                ("h1" | "h2" | "h3" | "h4", open_tag) => out.heading = !open_tag,
                _ => {}
            }
        }
    }
    out.text.push_str(rest);
    out.flush();
    out.blocks
}

pub fn parse_news(json: &[u8]) -> Result<Vec<PatchNote>, serde_json::Error> {
    let feed: Feed = serde_json::from_slice(json)?;
    let mut notes: Vec<PatchNote> = feed.appnews.newsitems.into_iter()
        .filter(|n| n.feedname == OFFICIAL_FEED)
        .map(|n| PatchNote {
            version: version_in_title(&n.title),
            blocks: bbcode_blocks(&n.contents),
            gid: n.gid,
            title: n.title,
            url: n.url,
            date: n.date,
        })
        .collect();
    notes.sort_by_key(|n| std::cmp::Reverse(n.date));
    Ok(notes)
}

/// The patch the newest official versioned note names.
pub fn latest_version(notes: &[PatchNote]) -> Option<String> {
    notes.iter().find_map(|n| n.version.clone())
}
