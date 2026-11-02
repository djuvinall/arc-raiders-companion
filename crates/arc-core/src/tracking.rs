//! Phase 5 read models: quests, projects and collections joined with the profile.

use crate::dataset::Dataset;
use crate::model::*;
use crate::views::{item_link, lines, Link};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuestRow {
    pub quest: Quest,
    /// `done`, `active` (tracked), `available` or `locked`.
    pub status: &'static str,
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

pub fn quest_rows(ds: &Dataset, profile: &Profile) -> Vec<QuestRow> {
    let done = |id: &str| profile.quests.get(id).is_some_and(|p| p.done);
    ds.files.quests.iter().map(|quest| {
        let progress = profile.quests.get(&quest.id);
        let status = match progress {
            Some(p) if p.done => "done",
            Some(_) => "active",
            None if quest.prerequisites.iter().all(|p| done(&p.quest_id)) => "available",
            None => "locked",
        };
        let maps: BTreeSet<String> = quest.objectives.iter().flat_map(|o| o.maps.iter().cloned()).collect();
        QuestRow {
            status,
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
        .or_insert_with(|| ProjectProgress { stage: 1, committed: Default::default() });
    let Some(stage) = project.stages.iter().find(|s| s.n == progress.stage) else { return 0 };
    let Some(need) = stage.inputs.iter().find(|i| i.item_id == item_id) else { return 0 };
    let so_far = progress.committed.get(item_id).copied().unwrap_or(0);
    let taken = qty.min(need.qty.saturating_sub(so_far));
    progress.committed.insert(item_id.to_string(), so_far + taken);
    let complete = stage.inputs.iter()
        .all(|i| progress.committed.get(&i.item_id).copied().unwrap_or(0) >= i.qty);
    if complete {
        progress.stage += 1;
        progress.committed.clear();
    }
    if let Some(have) = profile.stash.get_mut(item_id) {
        *have = have.saturating_sub(taken);
        if *have == 0 {
            profile.stash.remove(item_id);
        }
    }
    taken
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
