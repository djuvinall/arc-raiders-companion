//! Skill planner rules (plan §8 `skill_plan_validate`) and share codes. All pure.
//!
//! A plan is checked as a final state, not as a purchase order. For point gates that
//! means each choice pick must be affordable with the points spent before it, taking
//! picks in the order that needs the fewest points.

use crate::model::{SkillNode, SkillPlan, SkillTree};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Wiki: small nodes take "anywhere from 1 to 5". Used where a node's max is unpublished.
pub const UNKNOWN_RANK_CEILING: u32 = 5;
const CODE_PREFIX: &str = "ARC1-";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanReport {
    pub ok: bool,
    pub spent: u32,
    pub budget: u32,
    pub branch_points: BTreeMap<String, u32>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanState {
    pub report: PlanReport,
    /// Nodes that can take one more point / give one back and leave a valid plan.
    pub addable: Vec<String>,
    pub removable: Vec<String>,
}

fn rank_of(plan: &SkillPlan, id: &str) -> u32 {
    plan.ranks.get(id).copied().unwrap_or(0)
}

fn max_rank(node: &SkillNode) -> u32 {
    node.max_rank.unwrap_or(UNKNOWN_RANK_CEILING)
}

/// Choice nodes open from any one prior node (wiki: "at least one of the small nodes
/// prior to it"); other nodes need every prior node. Options sharing a slot count as
/// one prior: a skill after a large node opens from whichever option was picked.
fn prereqs_met(tree: &SkillTree, plan: &SkillPlan, node: &SkillNode) -> bool {
    if node.prereq_ids.is_empty() {
        return true;
    }
    let has = |id: &String| {
        let group = tree.nodes.iter().find(|n| n.id == *id).and_then(|n| n.choice_group.as_ref());
        match group {
            Some(g) => tree.nodes.iter()
                .any(|n| n.choice_group.as_ref() == Some(g) && rank_of(plan, &n.id) > 0),
            None => rank_of(plan, id) > 0,
        }
    };
    if node.kind == "choice" {
        node.prereq_ids.iter().any(has)
    } else {
        node.prereq_ids.iter().all(has)
    }
}

pub fn validate(tree: &SkillTree, plan: &SkillPlan) -> PlanReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let nodes: BTreeMap<&str, &SkillNode> = tree.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    if plan.tree_version != tree.tree_version {
        warnings.push(format!(
            "Plan was saved for tree {} but this is tree {}; check it before relying on it.",
            plan.tree_version, tree.tree_version
        ));
    }

    let mut branch_points: BTreeMap<String, u32> =
        tree.branches.iter().map(|b| (b.id.clone(), 0)).collect();
    let mut spent = 0;
    for (id, &rank) in plan.ranks.iter().filter(|(_, r)| **r > 0) {
        let Some(node) = nodes.get(id.as_str()) else {
            errors.push(format!("Unknown skill '{id}'."));
            continue;
        };
        spent += rank * node.cost_per_rank;
        *branch_points.entry(node.branch.clone()).or_default() += rank * node.cost_per_rank;
        if rank > max_rank(node) {
            errors.push(format!("{} is limited to {} points.", node.name, max_rank(node)));
        }
        if node.max_rank.is_none() {
            warnings.push(format!("{}: the maximum rank is unconfirmed.", node.name));
        }
        if !prereqs_met(tree, plan, node) {
            errors.push(format!("{} needs an earlier skill first.", node.name));
        }
        if node.choice_group.is_none() {
            let branch_total = branch_points_for(tree, plan, &node.branch);
            if branch_total - rank < node.branch_points_gate {
                errors.push(format!(
                    "{} needs {} points in {}.",
                    node.name, node.branch_points_gate, node.branch
                ));
            }
        }
    }

    for group in &tree.choice_groups {
        let picks = tree.nodes.iter()
            .filter(|n| n.choice_group.as_deref() == Some(&group.id) && rank_of(plan, &n.id) > 0)
            .count() as u32;
        if picks > group.max_picks {
            errors.push(format!(
                "Only {} of these can be taken in {}.",
                group.max_picks, group.branch
            ));
        }
        let total = branch_points_for(tree, plan, &group.branch);
        for pick in 0..picks.min(group.gates.len() as u32) {
            let before = total.saturating_sub(picks - pick);
            let gate = group.gates[pick as usize];
            if before < gate {
                errors.push(format!(
                    "Choice {} in {} needs {} points in the branch, has {}.",
                    pick + 1, group.branch, gate, before
                ));
            }
        }
    }

    let budget = tree.points.from_levels + plan.expedition_bonus.min(tree.points.expedition_bonus_max);
    if plan.expedition_bonus > tree.points.expedition_bonus_max {
        errors.push(format!(
            "Expedition bonus is limited to {} points.",
            tree.points.expedition_bonus_max
        ));
    }
    if spent > budget {
        errors.push(format!("{spent} points spent, {budget} available."));
    }

    warnings.sort();
    warnings.dedup();
    PlanReport { ok: errors.is_empty(), spent, budget, branch_points, errors, warnings }
}

fn branch_points_for(tree: &SkillTree, plan: &SkillPlan, branch: &str) -> u32 {
    tree.nodes.iter()
        .filter(|n| n.branch == branch)
        .map(|n| rank_of(plan, &n.id) * n.cost_per_rank)
        .sum()
}

fn with_rank(plan: &SkillPlan, id: &str, rank: u32) -> SkillPlan {
    let mut next = plan.clone();
    if rank == 0 {
        next.ranks.remove(id);
    } else {
        next.ranks.insert(id.to_string(), rank);
    }
    next
}

pub fn plan_state(tree: &SkillTree, plan: &SkillPlan) -> PlanState {
    let valid = |p: &SkillPlan| validate(tree, p).ok;
    let (mut addable, mut removable) = (Vec::new(), Vec::new());
    for node in &tree.nodes {
        let rank = rank_of(plan, &node.id);
        if valid(&with_rank(plan, &node.id, rank + 1)) {
            addable.push(node.id.clone());
        }
        if rank > 0 && valid(&with_rank(plan, &node.id, rank - 1)) {
            removable.push(node.id.clone());
        }
    }
    PlanState { report: validate(tree, plan), addable, removable }
}

const BASE64URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(crate) fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().fold(0u32, |n, b| (n << 8) | u32::from(*b)) << (8 * (3 - chunk.len()));
        for i in 0..=chunk.len() {
            out.push(BASE64URL[(n >> (18 - 6 * i)) as usize & 63] as char);
        }
    }
    out
}

pub(crate) fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for chunk in text.as_bytes().chunks(4) {
        if chunk.len() < 2 {
            return None;
        }
        let mut n = 0u32;
        for c in chunk {
            n = (n << 6) | BASE64URL.iter().position(|b| b == c)? as u32;
        }
        n <<= 6 * (4 - chunk.len());
        for i in 0..chunk.len() - 1 {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    Some(out)
}

/// Ids rather than positions, so a code survives nodes being added or moved.
pub fn encode(plan: &SkillPlan) -> String {
    let ranks: Vec<String> = plan.ranks.iter()
        .filter(|(_, r)| **r > 0)
        .map(|(id, r)| format!("{id}={r}"))
        .collect();
    let body = format!("{}|{}|{}|{}", plan.tree_version, plan.expedition_bonus, plan.name, ranks.join(";"));
    format!("{CODE_PREFIX}{}", base64_encode(body.as_bytes()))
}

pub fn decode(code: &str) -> Result<SkillPlan, String> {
    let bad = || "That is not a valid plan code.".to_string();
    let body = code.trim().strip_prefix(CODE_PREFIX).ok_or_else(bad)?;
    let bytes = base64_decode(body).ok_or_else(bad)?;
    let text = String::from_utf8(bytes).map_err(|_| bad())?;
    let mut parts = text.splitn(3, '|');
    let tree_version = parts.next().ok_or_else(bad)?.to_string();
    let expedition_bonus = parts.next().and_then(|b| b.parse().ok()).ok_or_else(bad)?;
    // Names may contain '|', ranks cannot, so split the last field from the right.
    let (name, ranks) = parts.next().and_then(|r| r.rsplit_once('|')).ok_or_else(bad)?;
    let ranks = ranks.split(';').filter(|e| !e.is_empty()).map(|entry| {
        let (id, rank) = entry.split_once('=').ok_or_else(bad)?;
        Ok((id.to_string(), rank.parse().map_err(|_| bad())?))
    }).collect::<Result<BTreeMap<String, u32>, String>>()?;
    Ok(SkillPlan { name: name.to_string(), tree_version, expedition_bonus, ranks })
}

/// Every id a plan references, for callers that report them.
pub fn plan_ids(plan: &SkillPlan) -> BTreeSet<&str> {
    plan.ranks.keys().map(String::as_str).collect()
}
