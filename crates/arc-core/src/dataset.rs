//! `Dataset`: parse data/*.json, check referential integrity, build indices.

use crate::model::*;
use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// Every file `Dataset` reads, in load order.
pub const DATA_FILES: &[&str] = &[
    "items.json", "recipes.json", "workbenches.json", "weapon_mods.json", "maps.json",
    "pois.json", "quests.json", "projects.json", "skills.json", "arcs.json", "traders.json",
    "loot_areas.json", "version.json",
];

/// The serializable part of a dataset; `arc-ingest` writes exactly this.
#[derive(Debug, Clone, PartialEq)]
pub struct DatasetFiles {
    pub items: Vec<Item>,
    pub recipes: Vec<Recipe>,
    pub workbenches: Vec<Workbench>,
    pub weapon_mods: Vec<WeaponMod>,
    pub maps: Vec<GameMap>,
    pub pois: Vec<Poi>,
    pub quests: Vec<Quest>,
    pub projects: Vec<Project>,
    pub skills: SkillTree,
    pub arcs: Vec<Arc>,
    pub traders: Vec<Trader>,
    pub loot_areas: Vec<LootArea>,
    pub markers: MapMarkers,
    pub version: DatasetVersion,
}

#[derive(Debug, Default)]
pub struct Indices {
    pub item: HashMap<String, usize>,
    pub recipe: HashMap<String, usize>,
    pub by_category: HashMap<Category, Vec<usize>>,
    /// item id -> recipes that consume it as an input.
    pub used_in_recipes: HashMap<String, Vec<usize>>,
    pub used_in_projects: HashMap<String, Vec<(usize, u32)>>,
    pub used_in_quests: HashMap<String, Vec<usize>>,
    pub used_in_workbenches: HashMap<String, Vec<(usize, u8)>>,
    /// loot area id -> POIs that carry it.
    pub pois_by_loot_area: HashMap<String, Vec<usize>>,
}

#[derive(Debug)]
pub struct Dataset {
    pub files: DatasetFiles,
    pub index: Indices,
}

fn parse<T: DeserializeOwned>(read: &dyn Fn(&str) -> Result<String>, name: &str) -> Result<T> {
    let text = read(name)?;
    serde_json::from_str(&text).with_context(|| format!("schema error in {name}"))
}

impl Dataset {
    pub fn load(dir: &Path) -> Result<Self> {
        Self::load_with(&|name| {
            std::fs::read_to_string(dir.join(name)).with_context(|| format!("reading {name}"))
        })
    }

    /// Same as `load` with the bytes supplied by the caller (the exe embeds data/).
    pub fn load_with(read: &dyn Fn(&str) -> Result<String>) -> Result<Self> {
        let files = DatasetFiles {
            items: parse(read, "items.json")?,
            recipes: parse(read, "recipes.json")?,
            workbenches: parse(read, "workbenches.json")?,
            weapon_mods: parse(read, "weapon_mods.json")?,
            maps: parse(read, "maps.json")?,
            pois: parse(read, "pois.json")?,
            quests: parse(read, "quests.json")?,
            projects: parse(read, "projects.json")?,
            skills: parse(read, "skills.json")?,
            arcs: parse(read, "arcs.json")?,
            traders: parse(read, "traders.json")?,
            loot_areas: parse(read, "loot_areas.json")?,
            markers: parse(read, "map_markers.json")?,
            version: parse(read, "version.json")?,
        };
        Self::from_files(files)
    }

    pub fn from_files(files: DatasetFiles) -> Result<Self> {
        let errors = integrity_errors(&files);
        if !errors.is_empty() {
            let shown = errors.iter().take(20).cloned().collect::<Vec<_>>().join("\n  ");
            bail!("{} referential integrity errors:\n  {shown}", errors.len());
        }
        let index = build_indices(&files);
        Ok(Self { files, index })
    }

    pub fn item(&self, id: &str) -> Option<&Item> {
        self.index.item.get(id).map(|&i| &self.files.items[i])
    }

    pub fn recipe(&self, id: &str) -> Option<&Recipe> {
        self.index.recipe.get(id).map(|&i| &self.files.recipes[i])
    }

    pub fn item_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.item(id).map_or(id, |i| i.name.as_str())
    }
}

fn build_indices(f: &DatasetFiles) -> Indices {
    let mut ix = Indices::default();
    for (i, item) in f.items.iter().enumerate() {
        ix.item.insert(item.id.clone(), i);
        ix.by_category.entry(item.category).or_default().push(i);
    }
    for (i, recipe) in f.recipes.iter().enumerate() {
        ix.recipe.insert(recipe.id.clone(), i);
        if matches!(recipe.kind, RecipeKind::Craft | RecipeKind::Trade) {
            for input in &recipe.inputs {
                ix.used_in_recipes.entry(input.item_id.clone()).or_default().push(i);
            }
        }
    }
    for (i, project) in f.projects.iter().enumerate() {
        for stage in &project.stages {
            for input in &stage.inputs {
                ix.used_in_projects.entry(input.item_id.clone()).or_default().push((i, stage.n));
            }
        }
    }
    for (i, quest) in f.quests.iter().enumerate() {
        for input in &quest.required_items {
            ix.used_in_quests.entry(input.item_id.clone()).or_default().push(i);
        }
    }
    for (i, bench) in f.workbenches.iter().enumerate() {
        for level in &bench.levels {
            for input in &level.upgrade_inputs {
                ix.used_in_workbenches
                    .entry(input.item_id.clone())
                    .or_default()
                    .push((i, level.level));
            }
        }
    }
    for (i, poi) in f.pois.iter().enumerate() {
        for area in &poi.loot_areas {
            ix.pois_by_loot_area.entry(area.clone()).or_default().push(i);
        }
    }
    ix
}

fn check_unique<'a>(errors: &mut Vec<String>, what: &str, ids: impl Iterator<Item = &'a str>) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(format!("duplicate {what} id: {id}"));
        }
    }
}

/// Every cross-file reference must resolve. Contributor PRs fail here in CI.
pub fn integrity_errors(f: &DatasetFiles) -> Vec<String> {
    let mut errors = Vec::new();
    check_unique(&mut errors, "item", f.items.iter().map(|x| x.id.as_str()));
    check_unique(&mut errors, "recipe", f.recipes.iter().map(|x| x.id.as_str()));
    check_unique(&mut errors, "poi", f.pois.iter().map(|x| x.id.as_str()));
    check_unique(&mut errors, "quest", f.quests.iter().map(|x| x.id.as_str()));
    check_unique(&mut errors, "project", f.projects.iter().map(|x| x.id.as_str()));
    check_unique(&mut errors, "skill", f.skills.nodes.iter().map(|x| x.id.as_str()));

    let items: BTreeSet<&str> = f.items.iter().map(|x| x.id.as_str()).collect();
    let recipes: BTreeSet<&str> = f.recipes.iter().map(|x| x.id.as_str()).collect();
    let benches: BTreeSet<&str> = f.workbenches.iter().map(|x| x.id.as_str()).collect();
    let maps: BTreeSet<&str> = f.maps.iter().map(|x| x.id.as_str()).collect();
    let pois: BTreeSet<&str> = f.pois.iter().map(|x| x.id.as_str()).collect();
    let quests: BTreeSet<&str> = f.quests.iter().map(|x| x.id.as_str()).collect();
    let areas: BTreeSet<&str> = f.loot_areas.iter().map(|x| x.id.as_str()).collect();
    let arcs: BTreeSet<&str> = f.arcs.iter().map(|x| x.id.as_str()).collect();
    let traders: BTreeSet<&str> = f.traders.iter().map(|x| x.id.as_str()).collect();

    let mut need = |ok: bool, owner: &str, kind: &str, id: &str| {
        if !ok {
            errors.push(format!("{owner}: unknown {kind} '{id}'"));
        }
    };

    for x in &f.items {
        for id in x.recipe_ids.iter().chain(&x.recycle_ids) {
            need(recipes.contains(id.as_str()), &x.id, "recipe", id);
        }
        for id in &x.loot_areas {
            need(areas.contains(id.as_str()), &x.id, "loot area", id);
        }
        for id in &x.drops_from {
            need(arcs.contains(id.as_str()), &x.id, "arc", id);
        }
        for s in &x.sold_by {
            need(traders.contains(s.trader.as_str()), &x.id, "trader", &s.trader);
        }
        if let Some(id) = &x.variant_of {
            need(items.contains(id.as_str()), &x.id, "item", id);
        }
    }
    for r in &f.recipes {
        let refs = r.inputs.iter().chain(&r.outputs).chain(r.output.as_ref());
        for q in refs {
            need(items.contains(q.item_id.as_str()), &r.id, "item", &q.item_id);
        }
        if let Some(id) = &r.blueprint_item_id {
            need(items.contains(id.as_str()), &r.id, "item", id);
        }
        let stations = r.workbench_id.iter().chain(r.alt_stations.iter().map(|s| &s.workbench_id));
        for id in stations {
            need(benches.contains(id.as_str()), &r.id, "workbench", id);
        }
    }
    for b in &f.workbenches {
        for l in &b.levels {
            for q in &l.upgrade_inputs {
                need(items.contains(q.item_id.as_str()), &b.id, "item", &q.item_id);
            }
            for id in &l.unlocks_recipe_ids {
                need(recipes.contains(id.as_str()), &b.id, "recipe", id);
            }
        }
    }
    for m in &f.weapon_mods {
        need(items.contains(m.id.as_str()), &m.id, "item", &m.id);
        if let Some(id) = &m.recipe_id {
            need(recipes.contains(id.as_str()), &m.id, "recipe", id);
        }
    }
    for m in &f.maps {
        for id in &m.poi_ids {
            need(pois.contains(id.as_str()), &m.id, "poi", id);
        }
    }
    for p in &f.pois {
        need(maps.contains(p.map_id.as_str()), &p.id, "map", &p.map_id);
        for id in &p.loot_areas {
            need(areas.contains(id.as_str()), &p.id, "loot area", id);
        }
    }
    for a in &f.loot_areas {
        for id in &a.item_ids {
            need(items.contains(id.as_str()), &a.id, "item", id);
        }
    }
    for q in &f.quests {
        for p in &q.prerequisites {
            need(quests.contains(p.quest_id.as_str()), &q.id, "quest", &p.quest_id);
        }
        for r in q.required_items.iter().chain(&q.rewards) {
            need(items.contains(r.item_id.as_str()), &q.id, "item", &r.item_id);
        }
        for o in &q.objectives {
            for id in &o.maps {
                need(maps.contains(id.as_str()), &q.id, "map", id);
            }
            for id in &o.poi_ids {
                need(pois.contains(id.as_str()), &q.id, "poi", id);
            }
            for id in &o.item_ids {
                need(items.contains(id.as_str()), &q.id, "item", id);
            }
        }
    }
    for p in &f.projects {
        for s in &p.stages {
            for q in s.inputs.iter().chain(&s.rewards) {
                need(items.contains(q.item_id.as_str()), &p.id, "item", &q.item_id);
            }
        }
    }
    let skills: BTreeSet<&str> = f.skills.nodes.iter().map(|x| x.id.as_str()).collect();
    let branches: BTreeSet<&str> = f.skills.branches.iter().map(|x| x.id.as_str()).collect();
    let groups: BTreeSet<&str> = f.skills.choice_groups.iter().map(|x| x.id.as_str()).collect();
    for n in &f.skills.nodes {
        need(branches.contains(n.branch.as_str()), &n.id, "branch", &n.branch);
        for id in &n.prereq_ids {
            need(skills.contains(id.as_str()), &n.id, "skill", id);
        }
        if let Some(id) = &n.choice_group {
            need(groups.contains(id.as_str()), &n.id, "choice group", id);
        }
    }
    for w in &f.skills.wires {
        for id in [&w.from, &w.to] {
            need(skills.contains(id.as_str()) || groups.contains(id.as_str()), &w.from, "wire end", id);
        }
    }
    for a in &f.arcs {
        for d in &a.drops {
            need(items.contains(d.item_id.as_str()), &a.id, "item", &d.item_id);
        }
    }
    for t in &f.traders {
        for s in &t.inventory {
            need(items.contains(s.item_id.as_str()), &t.id, "item", &s.item_id);
        }
    }
    errors
}
