//! One-line answers for mid-raid: keep it, sell it or recycle it. All pure.
//!
//! The order is what's needed first (tracked goals, the next workbench level, unfinished
//! quests), then what the item is worth. It's a suggestion: it can't know your plans.

use crate::dataset::Dataset;
use crate::model::*;
use crate::routing::needed_items;
use crate::views::{item_link, Link};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Verdict {
    /// `keep`, `recycle` or `sell`.
    pub action: &'static str,
    pub headline: String,
    pub reasons: Vec<String>,
    pub sell_value: Option<u32>,
    pub recycle_value: Option<u32>,
    pub gives: Vec<Link>,
}

/// Gear and collectibles are about your loadout, so only say something if a goal needs them.
fn is_loot(category: Category) -> bool {
    !matches!(
        category,
        Category::Weapon | Category::WeaponMod | Category::Cosmetic | Category::Furniture | Category::Design
            | Category::Stencil | Category::Blueprint | Category::Key | Category::QuestItem
            | Category::Augment | Category::Shield
    )
}

fn recycle_outputs<'a>(ds: &'a Dataset, item: &Item) -> &'a [ItemQty] {
    item.recycle_ids.iter()
        .filter_map(|id| ds.recipe(id))
        .find(|r| r.kind == RecipeKind::Recycle)
        .map_or(&[], |r| &r.outputs)
}

pub fn item_verdict(ds: &Dataset, profile: &Profile, id: &str) -> Option<Verdict> {
    let item = ds.item(id)?;
    let needed = needed_items(ds, profile);
    let have = profile.stash.get(id).copied().unwrap_or(0);
    let mut reasons = Vec::new();
    let mine = needed.iter().find(|n| n.item.id == id);
    if let Some(row) = mine {
        reasons.extend(row.reasons.iter().cloned());
    }
    for bench in &ds.files.workbenches {
        let next = profile.workbenches.get(&bench.id).copied().unwrap_or(0) + 1;
        let need = bench.levels.iter().find(|l| l.level == next)
            .and_then(|l| l.upgrade_inputs.iter().find(|q| q.item_id == id));
        if let Some(need) = need.filter(|n| n.qty > have) {
            reasons.push(format!("{} level {next} needs {}", bench.name, need.qty));
        }
    }
    for quest in ds.files.quests.iter().filter(|q| !profile.quests.get(&q.id).is_some_and(|p| p.done)) {
        let tracked = profile.quests.contains_key(&quest.id);
        if !tracked && quest.required_items.iter().any(|q| q.item_id == id && q.qty > have) {
            reasons.push(format!("Quest later: {}", quest.name));
        }
    }
    reasons.dedup();

    let outputs = recycle_outputs(ds, item);
    let recycle_value = (!outputs.is_empty()).then(|| {
        outputs.iter().map(|q| q.qty * ds.item(&q.item_id).and_then(|i| i.value).unwrap_or(0)).sum()
    });
    let gives: Vec<Link> = outputs.iter().map(|q| item_link(ds, &q.item_id, Some(q.qty))).collect();
    let verdict = |action, headline: String, reasons| Verdict {
        action, headline, reasons, sell_value: item.value, recycle_value, gives: gives.clone(),
    };

    if !reasons.is_empty() {
        let headline = mine.map_or("Keep it for later".to_string(), |row| format!("Keep: {} more needed", row.qty));
        return Some(verdict("keep", headline, reasons));
    }
    if !is_loot(item.category) {
        return None;
    }
    // Selling a staple only to buy it back later is the mistake worth preventing.
    if matches!(item.category, Category::MaterialBasic | Category::MaterialTopside | Category::MaterialRefined) {
        let recipes = crate::calc::used_in(ds, id).recipes.len();
        if recipes > 0 {
            return Some(verdict("keep", format!("Keep: crafting material, used in {recipes} recipes"), Vec::new()));
        }
    }
    if let Some(part) = gives.iter().find(|g| needed.iter().any(|n| n.item.id == g.id)) {
        return Some(verdict("recycle", format!("Recycle: gives {} you need", part.name), Vec::new()));
    }
    let sell = item.value?;
    match recycle_value.filter(|r| *r > sell) {
        Some(parts) => Some(verdict("recycle", format!("Recycle: parts are worth {parts}, selling gets {sell}"), Vec::new())),
        None => Some(verdict("sell", format!("Sell: worth {sell}"), Vec::new())),
    }
}
