//! Skill tree: nodes from the wiki's Skills page (CC BY-SA), plus skills_overlay.json
//! for what the wiki table lacks. Prerequisites and in-game positions are hand-entered
//! from the game (plan §5); nothing here reads the wiki's SkillTree SVG, whose notice
//! does not allow reuse.

use crate::normalize::slug;
use anyhow::{bail, Context, Result};
use arc_core::model::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    #[serde(rename = "_note")]
    _note: String,
    max_rank: BTreeMap<String, u32>,
    positions: BTreeMap<String, [f64; 2]>,
    slots: BTreeMap<String, Vec<[f64; 2]>>,
    wires: Vec<(String, String, Vec<[f64; 2]>)>,
    per_rank: BTreeMap<String, Vec<String>>,
}

/// A wire into a skill makes its source a prerequisite; a wire into a group does so for
/// every option in it. Sources that are groups expand to their options.
fn prereqs_from_wires(overlay: &Overlay, nodes: &mut [SkillNode]) {
    let options = |group: &str| -> Vec<String> {
        nodes.iter().filter(|n| n.choice_group.as_deref() == Some(group)).map(|n| n.id.clone()).collect()
    };
    let mut prereqs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (from, to, _) in overlay.wires.iter().filter(|(from, to, _)| from != to) {
        let sources = Some(options(from)).filter(|o| !o.is_empty()).unwrap_or_else(|| vec![from.clone()]);
        let targets = Some(options(to)).filter(|o| !o.is_empty()).unwrap_or_else(|| vec![to.clone()]);
        for target in targets {
            prereqs.entry(target).or_default().extend(sources.iter().cloned());
        }
    }
    for node in nodes {
        node.prereq_ids = prereqs.remove(&node.id).unwrap_or_default();
    }
}

const BRANCHES: &[(&str, &str, f64)] = &[
    ("conditioning", "#2fe08a", 260.0),
    ("mobility", "#f5c400", 760.0),
    ("survival", "#ff2d2d", 1260.0),
];
const SPACING: f64 = 104.0;

pub(crate) fn clean(cell: &str) -> String {
    let mut s = cell.trim().to_string();
    while let (Some(a), Some(b)) = (s.find("<ref"), s.find('>')) {
        let end = if s[..=b].ends_with("/>") {
            b + 1
        } else {
            s.find("</ref>").map_or(b + 1, |e| e + 6)
        };
        s.replace_range(a..end, "");
    }
    while let Some(a) = s.find("[[") {
        let Some(len) = s[a..].find("]]") else { break };
        let inner = s[a + 2..a + len].to_string();
        let shown = inner.rsplit('|').next().unwrap_or("").to_string();
        s.replace_range(a..a + len + 2, &shown);
    }
    s.trim().to_string()
}

struct Row {
    branch: String,
    name: String,
    icon: Option<String>,
    description: String,
    details: Option<String>,
    max: Option<u32>,
    node_type: String,
}

/// The same art the game shows. A URL only: the app fetches and caches it at runtime.
fn icon_url(cell: &str) -> Option<String> {
    let (_, rest) = cell.split_once("[[File:")?;
    let file = rest.split(['|', ']']).next()?.trim().replace(' ', "_");
    Some(format!("https://arcraiders.wiki/wiki/Special:FilePath/{file}"))
}

fn parse_rows(wikitext: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    let start = wikitext.find("<tabber>").unwrap_or(0);
    let end = wikitext.find("</tabber>").unwrap_or(wikitext.len());
    for tab in wikitext[start..end].split("|-|").skip(1) {
        let branch = tab.split('=').next().unwrap_or("").trim().to_lowercase();
        for raw in tab.split("\n|-\n") {
            let row = raw.trim_start().trim_start_matches('|').replace("||", "\n|");
            let cells: Vec<&str> = row.split("\n|").collect();
            if cells.len() < 6 || !cells[0].contains("[[File:") {
                continue;
            }
            // `style="text-align:center; | 5` carries the value after its own bar.
            let max = cells[4].rsplit('|').next().unwrap_or("").trim();
            rows.push(Row {
                branch: branch.clone(),
                icon: icon_url(cells[0]),
                name: clean(cells[1]),
                description: clean(cells[2]),
                details: Some(clean(cells[3])).filter(|d| !d.is_empty()),
                max: if max.starts_with("Option") { Some(1) } else { max.parse().ok() },
                node_type: clean(cells[5]),
            });
        }
    }
    rows
}

/// Rows by stage until the real layout is hand-entered. The wiki lists each branch
/// start, early, choice, late, end, which is all this reflects.
fn provisional_position(center: f64, index: usize, total_small: &mut usize, row: &Row) -> (f64, f64) {
    let spread = |i: usize, n: usize| center + (i as f64 - (n as f64 - 1.0) / 2.0) * SPACING;
    if row.node_type.contains("Initial") {
        return (center, 660.0);
    }
    if row.node_type.contains("(10 Points)") {
        return (spread(index - 5, 2), 420.0);
    }
    if row.node_type.contains("30/40") {
        return (spread(index - 13, 3), 60.0);
    }
    let small = *total_small;
    *total_small += 1;
    match small {
        0..=3 => (spread(small, 4), 540.0),
        4..=6 => (spread(small - 4, 3), 300.0),
        _ => (spread(small - 7, 3), 180.0),
    }
}

pub fn build(wikitext: &str, overlay: &Overlay) -> Result<SkillTree> {
    let rows = parse_rows(wikitext);
    if rows.len() != 48 {
        bail!("expected 48 skills on the wiki page, parsed {}", rows.len());
    }
    let mut nodes = Vec::new();
    let mut choice_groups = Vec::new();
    for (branch, _, center) in BRANCHES {
        let mut small = 0;
        let branch_rows: Vec<&Row> = rows.iter().filter(|r| r.branch == *branch).collect();
        for (index, row) in branch_rows.iter().enumerate() {
            let id = slug(&row.name);
            let (kind, group, gate) = if row.node_type.contains("Initial") {
                ("keystone", None, 0)
            } else if row.node_type.contains("(10 Points)") {
                ("choice", Some(format!("{branch}-mid")), 10)
            } else if row.node_type.contains("30/40") {
                ("choice", Some(format!("{branch}-end")), 30)
            } else if row.max.or_else(|| overlay.max_rank.get(&id).copied()) == Some(1) {
                ("toggle", None, 0)
            } else {
                ("rank", None, 0)
            };
            let (mut x, mut y) = provisional_position(*center, index, &mut small, row);
            if let Some([px, py]) = overlay.positions.get(&id) {
                (x, y) = (*px, *py);
            }
            nodes.push(SkillNode {
                name: row.name.clone(),
                branch: (*branch).to_string(),
                kind: kind.to_string(),
                description: row.description.clone(),
                details: row.details.clone(),
                max_rank: row.max.or_else(|| overlay.max_rank.get(&id).copied()),
                per_rank: overlay.per_rank.get(&id).cloned().unwrap_or_default(),
                cost_per_rank: 1,
                prereq_ids: Vec::new(),
                branch_points_gate: gate,
                choice_group: group,
                x,
                y,
                icon: row.icon.clone(),
                id,
            });
        }
        let slots = |id: &str| overlay.slots.get(id).cloned().unwrap_or_default();
        let mid = format!("{branch}-mid");
        let end = format!("{branch}-end");
        choice_groups.push(ChoiceGroup {
            slots: slots(&mid), id: mid, branch: (*branch).to_string(), max_picks: 1, gates: vec![10],
        });
        choice_groups.push(ChoiceGroup {
            slots: slots(&end), id: end, branch: (*branch).to_string(), max_picks: 2, gates: vec![30, 40],
        });
    }
    for node in &mut nodes {
        let slot = choice_groups.iter()
            .find(|g| Some(&g.id) == node.choice_group.as_ref())
            .and_then(|g| g.slots.first());
        if let Some([x, y]) = slot {
            (node.x, node.y) = (*x, *y);
        }
    }
    prereqs_from_wires(overlay, &mut nodes);
    let keystones = nodes.iter().filter(|n| n.kind == "keystone").count();
    let wired = nodes.iter().filter(|n| !n.prereq_ids.is_empty()).count();
    let placed = nodes.iter().filter(|n| overlay.positions.contains_key(&n.id)).count();
    let options = nodes.iter().filter(|n| n.choice_group.is_some()).count();
    let slotted = choice_groups.iter().all(|g| g.slots.len() == g.max_picks as usize);
    Ok(SkillTree {
        tree_version: "2.0".into(),
        branches: BRANCHES.iter().map(|(id, color, _)| SkillBranch {
            id: (*id).to_string(),
            color: (*color).to_string(),
            points_label: id.to_uppercase(),
        }).collect(),
        choice_groups,
        wires: overlay.wires.iter().map(|(from, to, points)| SkillWire {
            from: from.clone(), to: to.clone(), points: points.clone(),
        }).collect(),
        prereqs_entered: wired == nodes.len() - keystones,
        layout_in_game: placed == nodes.len() - options && slotted,
        nodes,
        // Up to 5 per Expedition, 15 in all (Embark, "The 5th Expedition", 2026-08-21).
        points: SkillPoints { from_levels: 76, max_level: 75, expedition_bonus_max: 15 },
    })
}

pub fn load_overlay(text: &str) -> Result<Overlay> {
    serde_json::from_str(text).context("skills_overlay.json")
}
