use arc_core::model::*;
use arc_core::skills::{decode, encode, plan_state, validate};
use arc_core::Dataset;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn node(id: &str, kind: &str, max: Option<u32>, prereqs: &[&str], gate: u32, group: Option<&str>) -> SkillNode {
    SkillNode {
        id: id.into(),
        name: id.into(),
        branch: "a".into(),
        kind: kind.into(),
        description: String::new(),
        details: None,
        max_rank: max,
        per_rank: Vec::new(),
        cost_per_rank: 1,
        prereq_ids: prereqs.iter().map(|p| p.to_string()).collect(),
        branch_points_gate: gate,
        choice_group: group.map(str::to_string),
        x: 0.0,
        y: 0.0,
        icon: None,
    }
}

/// root -> filler(5) x 8; mid picks one of two at 10; end picks two of three at 30/40.
fn tree() -> SkillTree {
    let mut nodes = vec![node("root", "keystone", Some(1), &[], 0, None)];
    for i in 0..9 {
        nodes.push(node(&format!("f{i}"), "rank", Some(5), &["root"], 0, None));
    }
    nodes.push(node("gated", "rank", Some(1), &["root"], 5, None));
    nodes.push(node("all", "rank", Some(1), &["f0", "f1"], 0, None));
    nodes.push(node("open", "rank", None, &[], 0, None));
    nodes.push(node("mid1", "choice", Some(1), &["f0", "f1"], 10, Some("mid")));
    nodes.push(node("mid2", "choice", Some(1), &["f0", "f1"], 10, Some("mid")));
    for id in ["end1", "end2", "end3"] {
        nodes.push(node(id, "choice", Some(1), &[], 30, Some("end")));
    }
    SkillTree {
        tree_version: "2.0".into(),
        branches: vec![SkillBranch { id: "a".into(), color: "#fff".into(), points_label: "A".into() }],
        choice_groups: vec![
            ChoiceGroup { id: "mid".into(), branch: "a".into(), max_picks: 1, gates: vec![10], slots: vec![[0.0, 0.0]] },
            ChoiceGroup {
                id: "end".into(), branch: "a".into(), max_picks: 2, gates: vec![30, 40],
                slots: vec![[0.0, 0.0], [0.0, 1.0]],
            },
        ],
        wires: Vec::new(),
        nodes,
        points: SkillPoints { from_levels: 76, max_level: 75, expedition_bonus_max: 5 },
        prereqs_entered: true,
        layout_in_game: false,
    }
}

fn plan(ranks: &[(&str, u32)]) -> SkillPlan {
    SkillPlan {
        name: "Test".into(),
        tree_version: "2.0".into(),
        expedition_bonus: 0,
        ranks: ranks.iter().map(|(id, r)| (id.to_string(), *r)).collect(),
    }
}

fn filled(points: u32) -> Vec<(String, u32)> {
    let mut left = points - 1;
    let mut ranks = vec![("root".to_string(), 1)];
    for i in 0..9 {
        let take = left.min(5);
        if take > 0 {
            ranks.push((format!("f{i}"), take));
        }
        left -= take;
    }
    ranks
}

fn with(mut ranks: Vec<(String, u32)>, extra: &[&str]) -> SkillPlan {
    ranks.extend(extra.iter().map(|id| (id.to_string(), 1)));
    let refs: Vec<(&str, u32)> = ranks.iter().map(|(i, r)| (i.as_str(), *r)).collect();
    plan(&refs)
}

#[test]
fn prerequisites_are_enforced() {
    let t = tree();
    assert!(!validate(&t, &plan(&[("f0", 1)])).ok, "needs root");
    assert!(validate(&t, &plan(&[("root", 1), ("f0", 1)])).ok);
    assert!(!validate(&t, &plan(&[("root", 1), ("f0", 1), ("all", 1)])).ok, "rank nodes need all");
    assert!(validate(&t, &plan(&[("root", 1), ("f0", 1), ("f1", 1), ("all", 1)])).ok);
}

#[test]
fn choice_nodes_open_from_any_prior_node() {
    let t = tree();
    assert!(validate(&t, &with(filled(10), &["mid1"])).ok, "f0 alone is enough");
}

#[test]
fn branch_gates_count_other_points_only() {
    let t = tree();
    assert!(!validate(&t, &plan(&[("root", 1), ("f0", 3), ("gated", 1)])).ok);
    assert!(validate(&t, &plan(&[("root", 1), ("f0", 4), ("gated", 1)])).ok);
}

#[test]
fn choice_groups_limit_picks_and_step_their_gates() {
    let t = tree();
    assert!(!validate(&t, &with(filled(9), &["mid1"])).ok, "needs 10");
    assert!(!validate(&t, &with(filled(10), &["mid1", "mid2"])).ok, "one of two");
    assert!(!validate(&t, &with(filled(29), &["end1"])).ok, "first pick needs 30");
    assert!(validate(&t, &with(filled(30), &["end1"])).ok);
    // The first pick counts towards the second pick's 40.
    assert!(!validate(&t, &with(filled(38), &["end1", "end2"])).ok, "only 39 before the second");
    assert!(validate(&t, &with(filled(39), &["end1", "end2"])).ok);
    assert!(!validate(&t, &with(filled(46), &["end1", "end2", "end3"])).ok, "two of three");
}

#[test]
fn budget_is_levels_plus_bonus() {
    let mut t = tree();
    for i in 0..12 {
        t.nodes.push(node(&format!("g{i}"), "rank", Some(5), &[], 0, None));
    }
    let mut p = plan(&[]);
    for i in 0..12 {
        p.ranks.insert(format!("g{i}"), 5);
    }
    for i in 0..4 {
        p.ranks.insert(format!("f{i}"), 4);
    }
    p.ranks.insert("root".into(), 1);
    assert_eq!(validate(&t, &p).spent, 77);
    assert!(!validate(&t, &p).ok, "77 > 76");
    p.expedition_bonus = 1;
    assert!(validate(&t, &p).ok);
    assert_eq!(validate(&t, &p).budget, 77);
    p.expedition_bonus = 6;
    assert!(!validate(&t, &p).ok, "bonus caps at 5");
}

#[test]
fn ranks_and_unknown_ids_are_rejected() {
    let t = tree();
    assert!(!validate(&t, &plan(&[("root", 2)])).ok);
    assert!(!validate(&t, &plan(&[("nope", 1)])).ok);
    let unknown_max = validate(&t, &plan(&[("open", 5)]));
    assert!(unknown_max.ok && unknown_max.warnings.iter().any(|w| w.contains("unconfirmed")));
    assert!(!validate(&t, &plan(&[("open", 6)])).ok, "ceiling applies");
}

#[test]
fn a_different_tree_version_warns_without_failing() {
    let mut p = plan(&[("root", 1)]);
    p.tree_version = "1.9".into();
    let report = validate(&tree(), &p);
    assert!(report.ok);
    assert!(report.warnings.iter().any(|w| w.contains("1.9")));
}

#[test]
fn state_lists_only_moves_that_stay_valid() {
    let t = tree();
    let state = plan_state(&t, &plan(&[("root", 1), ("f0", 1)]));
    assert!(state.addable.contains(&"f1".to_string()));
    assert!(!state.addable.contains(&"mid1".to_string()));
    assert!(!state.removable.contains(&"root".to_string()), "f0 depends on it");
    assert!(state.removable.contains(&"f0".to_string()));
}

#[test]
fn share_codes_round_trip() {
    let mut p = plan(&[("root", 1), ("f0", 3)]);
    p.name = "Looter | v2 ✓".into();
    p.expedition_bonus = 4;
    let code = encode(&p);
    assert!(code.starts_with("ARC1-"));
    assert!(code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    assert_eq!(decode(&code).unwrap(), p);
    assert_eq!(decode(&format!("  {code}\n")).unwrap(), p);
    for length_case in ["a", "ab", "abc"] {
        let mut q = plan(&[]);
        q.name = length_case.into();
        assert_eq!(decode(&encode(&q)).unwrap(), q);
    }
    for bad in ["", "ARC1-", "ARC1-!!!!", "hello", "ARC1-aGVsbG8"] {
        assert!(decode(bad).is_err(), "{bad}");
    }
}

fn real() -> Dataset {
    Dataset::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/data")).unwrap()
}

#[test]
fn real_tree_has_all_48_skills_and_the_published_gates() {
    let t = real().files.skills;
    assert_eq!(t.nodes.len(), 48);
    for branch in ["conditioning", "mobility", "survival"] {
        assert_eq!(t.nodes.iter().filter(|n| n.branch == branch).count(), 16, "{branch}");
        let mid = t.choice_groups.iter().find(|g| g.id == format!("{branch}-mid")).unwrap();
        assert_eq!((mid.max_picks, mid.gates.clone()), (1, vec![10]));
        let end = t.choice_groups.iter().find(|g| g.id == format!("{branch}-end")).unwrap();
        assert_eq!((end.max_picks, end.gates.clone()), (2, vec![30, 40]));
    }
    let broad = t.nodes.iter().find(|n| n.id == "broad-shoulders").unwrap();
    assert_eq!((broad.max_rank, broad.branch.as_str()), (Some(5), "conditioning"));
    assert!(broad.details.as_deref().unwrap().contains("2 points"));
    let unknown: Vec<_> = t.nodes.iter().filter(|n| n.max_rank.is_none()).map(|n| n.id.as_str()).collect();
    assert!(unknown.is_empty(), "{unknown:?}");
    let right_back = t.nodes.iter().find(|n| n.id == "right-back-at-it").unwrap();
    assert_eq!((right_back.max_rank, right_back.kind.as_str()), (Some(1), "toggle"));
}

#[test]
fn real_tree_accepts_a_legal_build() {
    let t = real().files.skills;
    let mut ranks = BTreeMap::new();
    for (id, rank) in [
        ("used-to-the-weight", 1), ("three-deep-breaths", 5), ("vigorous-vaulter", 1),
        ("stubborn-mule", 3), ("mules-gallop", 1), ("broad-shoulders", 5),
    ] {
        ranks.insert(id.to_string(), rank);
    }
    let p = SkillPlan { name: "x".into(), tree_version: "2.0".into(), expedition_bonus: 0, ranks };
    let report = validate(&t, &p);
    assert!(report.ok, "{:?}", report.errors);
    assert_eq!(report.branch_points["conditioning"], 16);
}

#[test]
fn real_tree_wiring_matches_the_game() {
    let t = real().files.skills;
    assert!(t.prereqs_entered && t.layout_in_game);
    let prior = |id: &str| t.nodes.iter().find(|n| n.id == id).unwrap().prereq_ids.clone();
    assert_eq!(prior("vigorous-vaulter"), ["three-deep-breaths"]);
    assert_eq!(prior("broad-shoulders"), ["mules-gallop", "back-on-your-feet"]);
    assert_eq!(prior("mules-gallop"), ["vigorous-vaulter", "vault-n-carry"]);
    assert_eq!(prior("wall-run"), ["slip-and-slide", "off-the-wall", "swift-start"]);
    for group in &t.choice_groups {
        assert_eq!(group.slots.len() as u32, group.max_picks, "{}", group.id);
    }
}

#[test]
fn real_tree_reproduces_the_screenshot_build() {
    // The reference screenshot: Mobility 41 with both end slots, Conditioning 10 with its
    // large node open, Survival 9 with its large node locked. Unbadged filled nodes are maxed.
    let t = real().files.skills;
    let mut ranks = BTreeMap::new();
    for (id, rank) in [
        ("used-to-the-weight", 1), ("three-deep-breaths", 3), ("vigorous-vaulter", 1),
        ("stubborn-mule", 4), ("vault-n-carry", 1),
        ("youthful-lungs", 1), ("sure-footed-scrambler", 5), ("nimble-climber", 5),
        ("fight-or-flight", 1), ("vaults-on-vaults-on-vaults", 1), ("effortless-roll", 5),
        ("slip-and-slide", 5), ("right-back-at-it", 1), ("off-the-wall", 5),
        ("marathon-runner", 5), ("swift-start", 5), ("sprinting-on-fumes", 1), ("wall-run", 1),
        ("in-round-crafting", 1), ("looters-instincts", 4), ("silent-scavenger", 2),
        ("sturdy-ankles", 1), ("agile-croucher", 1),
    ] {
        ranks.insert(id.to_string(), rank);
    }
    let p = SkillPlan { name: "x".into(), tree_version: "2.0".into(), expedition_bonus: 0, ranks };
    let report = validate(&t, &p);
    assert!(report.ok, "{:?}", report.errors);
    assert_eq!(report.branch_points["mobility"], 41);
    assert_eq!(report.branch_points["conditioning"], 10);
    assert_eq!(report.branch_points["survival"], 9);
    let state = plan_state(&t, &p);
    assert!(state.addable.contains(&"mules-gallop".to_string()), "10 points opens it");
    assert!(!state.addable.contains(&"quick-roll".to_string()), "9 points does not");
    assert!(!state.addable.contains(&"broad-shoulders".to_string()), "needs the large node");
    assert!(!state.addable.contains(&"in-the-flow".to_string()), "both end slots are taken");
}
