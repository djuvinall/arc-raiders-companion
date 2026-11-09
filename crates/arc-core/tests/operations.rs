//! ARC Operations: the guide data, the gear check, and how an operation shapes a raid plan.

use arc_core::model::*;
use arc_core::operations;
use arc_core::raid::*;
use arc_core::Dataset;
use std::path::PathBuf;

fn real() -> Dataset {
    Dataset::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/data")).unwrap()
}

fn plan(map: &str) -> RaidPlan {
    RaidPlan { id: "t".into(), name: "t".into(), map_id: map.into(), extra_items: vec!["fabric".into()], ..Default::default() }
}

#[test]
fn every_operation_is_a_complete_guide() {
    let ds = real();
    assert!(ds.files.operations.len() >= 5);
    for op in &ds.files.operations {
        assert!(!op.steps.is_empty() && !op.sources.is_empty() && !op.maps.is_empty(), "{}", op.id);
        assert!(ds.files.conditions.iter().any(|c| c.name == op.condition), "{} has no condition", op.id);
        for r in &op.rewards {
            assert!(ds.item(&r.item_id).is_some(), "{} reward {}", op.id, r.item_id);
        }
        for e in &op.enemies {
            assert!(ds.files.arcs.iter().any(|a| &a.id == e), "{} enemy {e}", op.id);
        }
    }
    let frigate = operations::for_condition(&ds, "ARC Frigate").unwrap();
    assert_eq!(frigate.route.raid_minutes, Some(35));
    assert!(frigate.route.no_hatches);
}

#[test]
fn the_gear_check_reads_the_stash() {
    let ds = real();
    let frigate = ds.files.operations.iter().find(|o| o.id == "arc-frigate").unwrap();
    let empty = operations::view(&ds, &Profile::default(), frigate);
    assert!(!empty.ready);
    assert!(empty.gear.iter().all(|g| g.have == 0));

    let mut profile = Profile::default();
    for (id, n) in [("anvil-ii", 1), ("medium-shield", 1), ("shield-recharger", 3), ("bandage", 5)] {
        profile.stash.insert(id.into(), n);
    }
    let kitted = operations::view(&ds, &profile, frigate);
    let minimum: Vec<&operations::GearCheck> = kitted.gear.iter().filter(|g| g.need == "minimum").collect();
    assert!(minimum.iter().all(|g| g.met), "{:?}", minimum.iter().filter(|g| !g.met).collect::<Vec<_>>());
    assert!(kitted.ready);
    // The Anvil counts by its armour penetration, and weapon tiers fold into one name.
    let anti_armour = kitted.gear.iter().find(|g| g.need == "recommended" && g.slot == "weapon").unwrap();
    assert!(anti_armour.names.iter().any(|n| n == "Hullcracker"));
    assert!(!anti_armour.names.iter().any(|n| n.contains("Amplified") || n.ends_with(" I")));
}

#[test]
fn an_operation_primary_shapes_the_plan() {
    let ds = real();
    let quiet = auto_plan(&ds, &Profile::default(), &plan("dam"));
    assert_eq!(quiet.raid_minutes, 30);
    assert!(quiet.operations.is_empty());
    let cargo = quiet.op_spots.iter().find(|s| s.op_id == "arc-frigate").expect("Frigate Cargo spots on Dam");
    assert!(quiet.op_spots.iter().any(|s| s.op_id == "harvester"));
    assert!(!quiet.op_spots.iter().any(|s| s.op_id == "redirection"), "Redirection isn't on Dam");

    let mut p = plan("dam");
    p.primary = Some(Primary {
        kind: "operation".into(), id: format!("arc-frigate@{}", cargo.stop.key), name: cargo.stop.name.clone(),
        x: cargo.stop.x, y: cargo.stop.y, floors: cargo.stop.floors,
    });
    let auto = auto_plan(&ds, &Profile::default(), &p);
    assert_eq!(auto.raid_minutes, 35);
    let note = auto.operations.iter().find(|o| o.id == "arc-frigate").unwrap();
    assert!(note.primary && !note.live && !note.ready && !note.short.is_empty());
    assert!(auto.areas.iter().all(|a| a.exits.iter().all(|e| !e.hatch)), "no Raider Hatches during the Frigate");
    assert!(auto.areas.iter().all(|a| a.minutes > 0.0));
}

#[test]
fn a_live_operation_shows_on_the_plan() {
    let ds = real();
    let auto = auto_plan_live(&ds, &Profile::default(), &plan("dam"), &["Harvester".to_string()]);
    let note = auto.operations.iter().find(|o| o.id == "harvester").unwrap();
    assert!(note.live && !note.primary);
    assert_eq!(auto.raid_minutes, 30);
    // Hatches stay open, and every area offers at least one exit that isn't a hatch.
    assert!(auto.areas.iter().all(|a| a.exits.iter().any(|e| !e.hatch)));
}
