//! Phase 5 read models: quests, projects and collections joined with the profile.

use crate::dataset::Dataset;
use crate::model::*;
use crate::skills::{base64_decode, base64_encode};
use crate::views::{item_link, lines, Link};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuestRow {
    pub quest: Quest,
    /// `done`, `active` (tracked), `needed` (a tracked quest depends on it),
    /// `available` or `locked`.
    pub status: &'static str,
    /// The tracked quests that depend on this one.
    pub needed_for: Vec<Link>,
    /// Every prerequisite is done, so it can be started now.
    pub open: bool,
    pub prerequisites: Vec<Link>,
    pub required: Vec<Link>,
    pub rewards: Vec<Link>,
    pub maps: Vec<String>,
    pub done_objectives: Vec<String>,
}

pub fn atomic_ids(quest: &Quest) -> Vec<&str> {
    quest.objectives.iter().filter(|o| o.kind == "atomic").map(|o| o.id.as_str()).collect()
}

fn quest_link(quest: &Quest) -> Link {
    Link { kind: "quest".into(), id: quest.id.clone(), name: quest.name.clone(), icon: None, qty: None, note: None }
}

/// Unfinished quests behind each tracked quest, walking back through its prerequisites.
fn needed_by_tracked<'a>(ds: &'a Dataset, profile: &Profile) -> BTreeMap<&'a str, Vec<&'a Quest>> {
    let done = |id: &str| profile.quests.get(id).is_some_and(|p| p.done);
    let mut needed: BTreeMap<&str, Vec<&Quest>> = BTreeMap::new();
    let tracked = ds.files.quests.iter()
        .filter(|q| profile.quests.get(&q.id).is_some_and(|p| !p.done));
    for quest in tracked {
        let mut seen = BTreeSet::new();
        let mut waiting: Vec<&str> = quest.prerequisites.iter().map(|p| p.quest_id.as_str()).collect();
        while let Some(id) = waiting.pop() {
            let Some(earlier) = ds.files.quests.iter().find(|q| q.id == id) else { continue };
            if done(id) || !seen.insert(id) {
                continue;
            }
            needed.entry(earlier.id.as_str()).or_default().push(quest);
            waiting.extend(earlier.prerequisites.iter().map(|p| p.quest_id.as_str()));
        }
    }
    needed
}

pub fn quest_rows(ds: &Dataset, profile: &Profile) -> Vec<QuestRow> {
    let done = |id: &str| profile.quests.get(id).is_some_and(|p| p.done);
    let needed = needed_by_tracked(ds, profile);
    ds.files.quests.iter().map(|quest| {
        let progress = profile.quests.get(&quest.id);
        let open = quest.prerequisites.iter().all(|p| done(&p.quest_id));
        let needed_for = needed.get(quest.id.as_str());
        let status = match progress {
            Some(p) if p.done => "done",
            Some(_) => "active",
            None if needed_for.is_some() => "needed",
            None if open => "available",
            None => "locked",
        };
        let maps: BTreeSet<String> = quest.objectives.iter().flat_map(|o| o.maps.iter().cloned()).collect();
        QuestRow {
            status,
            open,
            needed_for: needed_for.map_or(Vec::new(), |tracked| tracked.iter().map(|q| quest_link(q)).collect()),
            prerequisites: quest.prerequisites.iter()
                .filter_map(|p| ds.files.quests.iter().find(|q| q.id == p.quest_id))
                .map(quest_link)
                .collect(),
            required: lines(ds, &quest.required_items),
            rewards: lines(ds, &quest.rewards),
            maps: maps.into_iter().collect(),
            done_objectives: progress.map_or(Vec::new(), |p| p.objectives_done.clone()),
            quest: quest.clone(),
        }
    }).collect()
}

const SHARE_PREFIX: &str = "ARCQ1-";
const TAG_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Three characters per quest keep a code short enough to paste into a chat.
fn quest_tag(id: &str) -> String {
    let hash = id.bytes().fold(0x811c9dc5u32, |h, b| (h ^ u32::from(b)).wrapping_mul(0x01000193));
    (0..3).map(|i| TAG_ALPHABET[(hash >> (i * 6)) as usize & 63] as char).collect()
}

/// A code a friend can paste: this player's name and quest progress, nothing else.
pub fn encode_progress(ds: &Dataset, profile: &Profile, name: &str) -> String {
    let tags = |done: bool| -> String {
        ds.files.quests.iter()
            .filter(|q| profile.quests.get(&q.id).is_some_and(|p| p.done == done))
            .map(|q| quest_tag(&q.id))
            .collect()
    };
    format!("{SHARE_PREFIX}{}.{}.{}", base64_encode(name.trim().as_bytes()), tags(true), tags(false))
}

/// Tags for quests this dataset doesn't know are skipped, so codes survive data updates.
pub fn decode_progress(ds: &Dataset, code: &str) -> Result<Friend, String> {
    let bad = || "That isn't a quest share code.".to_string();
    let body = code.trim().strip_prefix(SHARE_PREFIX).ok_or_else(bad)?;
    let mut parts = body.split('.');
    let (name, done, active) = (parts.next().ok_or_else(bad)?, parts.next().ok_or_else(bad)?, parts.next().ok_or_else(bad)?);
    let name = base64_decode(name).and_then(|n| String::from_utf8(n).ok()).filter(|n| !n.is_empty()).ok_or_else(bad)?;
    let ids = |tags: &str| -> Vec<String> {
        let tags: BTreeSet<&[u8]> = tags.as_bytes().chunks(3).collect();
        ds.files.quests.iter().filter(|q| tags.contains(quest_tag(&q.id).as_bytes())).map(|q| q.id.clone()).collect()
    };
    Ok(Friend { name, done: ids(done), active: ids(active) })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PartyQuest {
    pub quest: Link,
    pub maps: Vec<String>,
    /// Quests before this one in its longest chain, so low numbers are older quests.
    pub depth: u32,
    /// Someone is already tracking it.
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PartyPlan {
    pub friend: String,
    /// Both can do these now.
    pub together: Vec<PartyQuest>,
    /// The friend has finished these, and this player can do them now.
    pub you_catch_up: Vec<PartyQuest>,
    /// This player has finished these, and the friend can do them now.
    pub friend_catch_up: Vec<PartyQuest>,
    /// Maps by how many of the together quests have objectives there.
    pub maps: Vec<(String, usize)>,
}

fn quest_depth(ds: &Dataset, id: &str, path: &mut Vec<String>) -> u32 {
    if path.iter().any(|p| p == id) {
        return 0;
    }
    let Some(quest) = ds.files.quests.iter().find(|q| q.id == id) else { return 0 };
    path.push(id.to_string());
    let deepest = quest.prerequisites.iter().map(|p| 1 + quest_depth(ds, &p.quest_id, path)).max().unwrap_or(0);
    path.pop();
    deepest
}

/// What two players should do next. Older quests come first, so whoever is behind
/// reaches the same point in the chain and the two can then play the rest together.
pub fn party_plan(ds: &Dataset, profile: &Profile, friend: &Friend) -> PartyPlan {
    let mine: BTreeSet<&str> = profile.quests.iter().filter(|(_, p)| p.done).map(|(id, _)| id.as_str()).collect();
    let theirs: BTreeSet<&str> = friend.done.iter().map(String::as_str).collect();
    let can_do = |quest: &Quest, done: &BTreeSet<&str>| {
        !done.contains(quest.id.as_str()) && quest.prerequisites.iter().all(|p| done.contains(p.quest_id.as_str()))
    };
    let (mut together, mut you_catch_up, mut friend_catch_up) = (Vec::new(), Vec::new(), Vec::new());
    for quest in &ds.files.quests {
        let (me, them) = (can_do(quest, &mine), can_do(quest, &theirs));
        let list = match (me, them) {
            (true, true) => &mut together,
            (true, false) if theirs.contains(quest.id.as_str()) => &mut you_catch_up,
            (false, true) if mine.contains(quest.id.as_str()) => &mut friend_catch_up,
            _ => continue,
        };
        let maps: BTreeSet<String> = quest.objectives.iter().flat_map(|o| o.maps.iter().cloned()).collect();
        list.push(PartyQuest {
            quest: quest_link(quest),
            maps: maps.into_iter().collect(),
            depth: quest_depth(ds, &quest.id, &mut Vec::new()),
            active: friend.active.contains(&quest.id) || profile.quests.get(&quest.id).is_some_and(|p| !p.done),
        });
    }
    for list in [&mut together, &mut you_catch_up, &mut friend_catch_up] {
        list.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.quest.name.cmp(&b.quest.name)));
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for map in together.iter().flat_map(|q| &q.maps) {
        *counts.entry(map.clone()).or_default() += 1;
    }
    let mut maps: Vec<(String, usize)> = counts.into_iter().collect();
    maps.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    PartyPlan { friend: friend.name.clone(), together, you_catch_up, friend_catch_up, maps }
}

/// Quests before any active one. The game only offers a quest once everything before it
/// is complete, so these must already be done.
pub fn implied_done(ds: &Dataset, profile: &Profile) -> Vec<String> {
    needed_by_tracked(ds, profile).keys()
        .filter(|id| !profile.quests.contains_key(**id))
        .map(|id| id.to_string())
        .collect()
}

/// Marks every quest the active ones imply. Returns how many changed.
pub fn complete_implied(ds: &Dataset, profile: &mut Profile) -> usize {
    let ids = implied_done(ds, profile);
    for id in &ids {
        profile.quests.entry(id.clone()).or_default().done = true;
    }
    ids.len()
}

/// Marks a quest and everything before it done, for catching up with where you are.
pub fn complete_chain(ds: &Dataset, profile: &mut Profile, quest_id: &str) {
    let mut waiting = vec![quest_id.to_string()];
    while let Some(id) = waiting.pop() {
        let Some(quest) = ds.files.quests.iter().find(|q| q.id == id) else { continue };
        if profile.quests.get(&id).is_some_and(|p| p.done) {
            continue;
        }
        profile.quests.entry(id).or_default().done = true;
        waiting.extend(quest.prerequisites.iter().map(|p| p.quest_id.clone()));
    }
}

/// A requirement reads "Outpost level 4"; it's met once the tracked level reaches that.
pub(crate) fn requirement_met(profile: &Profile, requirement: &str) -> bool {
    let needed: u8 = requirement.rsplit(' ').next().and_then(|n| n.parse().ok()).unwrap_or(u8::MAX);
    profile.workbenches.get("outpost").copied().unwrap_or(0) >= needed
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchView {
    pub id: String,
    pub name: String,
    pub level: u8,
    pub max_level: u8,
    pub tracked: bool,
    /// What the next level costs. Empty at the top level.
    pub next: Vec<StageInput>,
    pub requires: Option<String>,
}

pub fn workbench_views(ds: &Dataset, profile: &Profile) -> Vec<BenchView> {
    ds.files.workbenches.iter().filter(|b| b.max_level > 0).map(|bench| {
        let level = profile.workbenches.get(&bench.id).copied().unwrap_or(0);
        let upcoming = bench.levels.iter().find(|l| l.level == level + 1);
        let next = upcoming.map_or(Vec::new(), |l| {
            l.upgrade_inputs.iter().map(|q| StageInput {
                item: item_link(ds, &q.item_id, Some(q.qty)),
                need: q.qty,
                committed: 0,
                in_stash: profile.stash.get(&q.item_id).copied().unwrap_or(0),
            }).collect()
        });
        BenchView {
            id: bench.id.clone(),
            name: bench.name.clone(),
            level,
            max_level: bench.max_level,
            tracked: profile.tracked_benches.contains(&bench.id),
            next,
            requires: upcoming.and_then(|l| l.requires.clone()).filter(|r| !requirement_met(profile, r)),
        }
    }).collect()
}

/// Ticking the last objective completes the quest; unticking one reopens it.
pub fn set_objective(ds: &Dataset, profile: &mut Profile, quest_id: &str, objective_id: &str, done: bool) {
    let Some(quest) = ds.files.quests.iter().find(|q| q.id == quest_id) else { return };
    let progress = profile.quests.entry(quest_id.to_string()).or_default();
    progress.objectives_done.retain(|o| o != objective_id);
    if done {
        progress.objectives_done.push(objective_id.to_string());
    }
    let atomics = atomic_ids(quest);
    progress.done = !atomics.is_empty()
        && atomics.iter().all(|a| progress.objectives_done.iter().any(|d| d == a));
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StageInput {
    pub item: Link,
    pub need: u32,
    pub committed: u32,
    pub in_stash: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StageView {
    pub n: u32,
    pub title: String,
    pub inputs: Vec<StageInput>,
    pub rewards: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub expedition: bool,
    pub deadline_days: Option<u32>,
    pub starts: Option<String>,
    pub ends: Option<String>,
    pub permanent: bool,
    pub paused: bool,
    pub tracked: bool,
    /// Stage in progress; `stages.len() + 1` means complete.
    pub stage: u32,
    pub stages: Vec<StageView>,
}

pub fn project_views(ds: &Dataset, profile: &Profile) -> Vec<ProjectView> {
    ds.files.projects.iter().map(|project| {
        let progress = profile.projects.get(&project.id);
        let stage = progress.map_or(1, |p| p.stage);
        ProjectView {
            id: project.id.clone(),
            name: project.name.clone(),
            expedition: project.expedition,
            deadline_days: project.deadline_days,
            starts: project.starts.clone(),
            ends: project.ends.clone(),
            permanent: project.permanent,
            paused: project.paused,
            tracked: progress.is_some(),
            stage,
            stages: project.stages.iter().map(|s| StageView {
                n: s.n,
                title: s.title.clone(),
                rewards: lines(ds, &s.rewards),
                inputs: s.inputs.iter().map(|q| StageInput {
                    item: item_link(ds, &q.item_id, Some(q.qty)),
                    need: q.qty,
                    committed: match s.n.cmp(&stage) {
                        std::cmp::Ordering::Less => q.qty,
                        std::cmp::Ordering::Equal => progress
                            .and_then(|p| p.committed.get(&q.item_id).copied())
                            .unwrap_or(0),
                        std::cmp::Ordering::Greater => 0,
                    },
                    in_stash: profile.stash.get(&q.item_id).copied().unwrap_or(0),
                }).collect(),
            }).collect(),
        }
    }).collect()
}

/// Commits up to what the stage still needs, taking it from the stash like the game
/// does, and moves to the next stage once every input is met. Returns what was taken.
pub fn commit(ds: &Dataset, profile: &mut Profile, project_id: &str, item_id: &str, qty: u32) -> u32 {
    let Some(project) = ds.files.projects.iter().find(|p| p.id == project_id) else { return 0 };
    let progress = profile.projects.entry(project_id.to_string())
        .or_insert_with(|| ProjectProgress { stage: 1, ..Default::default() });
    let Some(stage) = project.stages.iter().find(|s| s.n == progress.stage) else { return 0 };
    let Some(need) = stage.inputs.iter().find(|i| i.item_id == item_id) else { return 0 };
    let so_far = progress.committed.get(item_id).copied().unwrap_or(0);
    let taken = qty.min(need.qty.saturating_sub(so_far));
    progress.committed.insert(item_id.to_string(), so_far + taken);
    if let Some(have) = profile.stash.get_mut(item_id) {
        let spent = taken.min(*have);
        *have -= spent;
        if *have == 0 {
            profile.stash.remove(item_id);
        }
        if spent > 0 {
            *progress.from_stash.entry(format!("{}/{item_id}", progress.stage)).or_insert(0) += spent;
        }
    }
    // A stage whose inputs are all in moves on, including one reopened with nothing taken back.
    let complete = stage.inputs.iter()
        .all(|i| progress.committed.get(&i.item_id).copied().unwrap_or(0) >= i.qty);
    if complete {
        progress.stage += 1;
        progress.committed.clear();
    }
    taken
}

/// Takes back up to `qty` of an item committed to the current stage. What came out of the
/// stash goes back into it. Returns how many were taken back.
pub fn uncommit(profile: &mut Profile, project_id: &str, item_id: &str, qty: u32) -> u32 {
    let Some(progress) = profile.projects.get_mut(project_id) else { return 0 };
    let so_far = progress.committed.get(item_id).copied().unwrap_or(0);
    let back = qty.min(so_far);
    if back == 0 {
        return 0;
    }
    if so_far == back {
        progress.committed.remove(item_id);
    } else {
        progress.committed.insert(item_id.to_string(), so_far - back);
    }
    let key = format!("{}/{item_id}", progress.stage);
    let from_stash = progress.from_stash.get(&key).copied().unwrap_or(0);
    let refund = back.min(from_stash);
    if refund > 0 {
        if from_stash == refund {
            progress.from_stash.remove(&key);
        } else {
            progress.from_stash.insert(key, from_stash - refund);
        }
        *profile.stash.entry(item_id.to_string()).or_insert(0) += refund;
    }
    back
}

/// Steps back to the previous stage, as it was when it finished. Anything committed to
/// the current stage is taken back first. Returns the stage now in progress.
pub fn reopen_stage(ds: &Dataset, profile: &mut Profile, project_id: &str) -> u32 {
    let Some(project) = ds.files.projects.iter().find(|p| p.id == project_id) else { return 0 };
    let Some(stage) = profile.projects.get(project_id).map(|p| p.stage) else { return 0 };
    if stage <= 1 {
        return stage;
    }
    let current: Vec<(String, u32)> = profile.projects[project_id].committed.clone().into_iter().collect();
    for (item, qty) in current {
        uncommit(profile, project_id, &item, qty);
    }
    let progress = profile.projects.get_mut(project_id).expect("checked above");
    progress.stage = stage - 1;
    progress.committed = project.stages.iter().find(|s| s.n == stage - 1)
        .map(|s| s.inputs.iter().map(|i| (i.item_id.clone(), i.qty)).collect())
        .unwrap_or_default();
    progress.stage
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CollectionEntry {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub rarity: Option<Rarity>,
    pub found: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Collection {
    pub id: &'static str,
    pub title: &'static str,
    pub found: usize,
    pub total: usize,
    pub entries: Vec<CollectionEntry>,
}

pub fn collections(ds: &Dataset, profile: &Profile) -> Vec<Collection> {
    [
        ("blueprints", "Blueprints", Category::Blueprint),
        ("stencils", "Stencils", Category::Stencil),
        ("designs", "Designs", Category::Design),
    ].into_iter().map(|(id, title, category)| {
        let entries: Vec<CollectionEntry> = ds.files.items.iter()
            .filter(|i| i.category == category)
            .map(|i| CollectionEntry {
                id: i.id.clone(),
                name: i.name.clone(),
                icon: i.icon.clone(),
                rarity: i.rarity,
                found: profile.blueprints_found.contains(&i.id),
            })
            .collect();
        Collection { id, title, found: entries.iter().filter(|e| e.found).count(), total: entries.len(), entries }
    }).collect()
}
