//! Phase 1 calculators: `craft_tree`, `raw_totals`, `used_in`. All pure.

use crate::dataset::Dataset;
use crate::model::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TreeNode {
    pub item_id: String,
    pub name: String,
    pub qty: u32,
    pub recipe_id: Option<String>,
    pub workbench_id: Option<String>,
    pub workbench_level: Option<u8>,
    /// True when this node stopped expanding because it appears among its own ancestors.
    pub cycle: bool,
    pub children: Vec<TreeNode>,
}

/// The recipe `craft_tree` follows for an item: its first craft recipe.
fn craft_recipe<'a>(ds: &'a Dataset, item: &Item) -> Option<&'a Recipe> {
    item.recipe_ids
        .iter()
        .filter_map(|id| ds.recipe(id))
        .find(|r| r.kind == RecipeKind::Craft)
}

/// Runs of output: crafting `qty` from a recipe yielding `per` needs ceil(qty/per) runs.
fn runs(qty: u32, per: u32) -> u32 {
    qty.div_ceil(per.max(1))
}

/// Expands recursively to items with no craft recipe. `None` for an unknown id.
pub fn craft_tree(ds: &Dataset, item_id: &str, qty: u32) -> Option<TreeNode> {
    ds.item(item_id)?;
    let mut path = Vec::new();
    Some(expand(ds, item_id, qty, &mut path))
}

fn expand(ds: &Dataset, item_id: &str, qty: u32, path: &mut Vec<String>) -> TreeNode {
    let mut node = TreeNode {
        item_id: item_id.to_string(),
        name: ds.item_name(item_id).to_string(),
        qty,
        recipe_id: None,
        workbench_id: None,
        workbench_level: None,
        cycle: path.iter().any(|p| p == item_id),
        children: Vec::new(),
    };
    if node.cycle {
        return node;
    }
    let Some(recipe) = ds.item(item_id).and_then(|i| craft_recipe(ds, i)) else {
        return node;
    };
    node.recipe_id = Some(recipe.id.clone());
    node.workbench_id = recipe.workbench_id.clone();
    node.workbench_level = recipe.workbench_level;
    let per = recipe.output.as_ref().map_or(1, |o| o.qty);
    let batches = runs(qty, per);
    path.push(item_id.to_string());
    node.children = recipe
        .inputs
        .iter()
        .map(|input| expand(ds, &input.item_id, input.qty * batches, path))
        .collect();
    path.pop();
    node
}

/// Sums leaves: the raw materials to gather for the whole tree.
pub fn raw_totals(tree: &TreeNode) -> BTreeMap<String, u32> {
    let mut totals = BTreeMap::new();
    add_leaves(tree, &mut totals);
    totals
}

fn add_leaves(node: &TreeNode, totals: &mut BTreeMap<String, u32>) {
    if node.children.is_empty() {
        *totals.entry(node.item_id.clone()).or_insert(0) += node.qty;
    }
    node.children.iter().for_each(|child| add_leaves(child, totals));
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecipeUse {
    pub recipe_id: String,
    pub output_item_id: Option<String>,
    pub output_name: Option<String>,
    pub output_icon: Option<String>,
    pub qty: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectUse {
    pub project_id: String,
    pub project_name: String,
    pub stage: u32,
    pub qty: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuestUse {
    pub quest_id: String,
    pub quest_name: String,
    pub qty: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorkbenchUse {
    pub workbench_id: String,
    pub workbench_name: String,
    pub level: u8,
    pub qty: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct UsedIn {
    pub recipes: Vec<RecipeUse>,
    pub projects: Vec<ProjectUse>,
    pub quests: Vec<QuestUse>,
    pub workbenches: Vec<WorkbenchUse>,
}

fn qty_of(inputs: &[ItemQty], item_id: &str) -> u32 {
    inputs.iter().filter(|q| q.item_id == item_id).map(|q| q.qty).sum()
}

pub fn used_in(ds: &Dataset, item_id: &str) -> UsedIn {
    let f = &ds.files;
    let ix = &ds.index;
    let empty_r = Vec::new();
    let empty_p = Vec::new();
    let empty_q = Vec::new();
    let empty_w = Vec::new();

    let recipes = ix.used_in_recipes.get(item_id).unwrap_or(&empty_r).iter().map(|&i| {
        let r = &f.recipes[i];
        let output = r.output.as_ref().map(|o| o.item_id.clone());
        RecipeUse {
            recipe_id: r.id.clone(),
            output_name: output.as_deref().map(|id| ds.item_name(id).to_string()),
            output_icon: output.as_deref().and_then(|id| ds.item(id)).and_then(|i| i.icon.clone()),
            output_item_id: output,
            qty: qty_of(&r.inputs, item_id),
        }
    }).collect();

    let projects = ix.used_in_projects.get(item_id).unwrap_or(&empty_p).iter().map(|&(i, n)| {
        let p = &f.projects[i];
        let inputs = p.stages.iter().find(|s| s.n == n).map_or(&[][..], |s| &s.inputs);
        ProjectUse {
            project_id: p.id.clone(),
            project_name: p.name.clone(),
            stage: n,
            qty: qty_of(inputs, item_id),
        }
    }).collect();

    let quests = ix.used_in_quests.get(item_id).unwrap_or(&empty_q).iter().map(|&i| {
        let q = &f.quests[i];
        QuestUse {
            quest_id: q.id.clone(),
            quest_name: q.name.clone(),
            qty: qty_of(&q.required_items, item_id),
        }
    }).collect();

    let workbenches = ix.used_in_workbenches.get(item_id).unwrap_or(&empty_w).iter().map(|&(i, l)| {
        let b = &f.workbenches[i];
        let inputs = b.levels.iter().find(|x| x.level == l).map_or(&[][..], |x| &x.upgrade_inputs);
        WorkbenchUse {
            workbench_id: b.id.clone(),
            workbench_name: b.name.clone(),
            level: l,
            qty: qty_of(inputs, item_id),
        }
    }).collect();

    UsedIn { recipes, projects, quests, workbenches }
}
