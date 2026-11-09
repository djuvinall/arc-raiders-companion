//! The raid planner's spawn model and target areas, against the real dataset.

use arc_core::model::*;
use arc_core::raid::*;
use arc_core::Dataset;
use std::path::PathBuf;

fn real() -> Dataset {
    Dataset::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/data")).unwrap()
}

fn plan(map: &str, extra: &[&str]) -> RaidPlan {
    RaidPlan { id: "t".into(), map_id: map.into(), extra_items: extra.iter().map(|s| s.to_string()).collect(), ..Default::default() }
}

/// The best stops for a medical item lean on med crates, not lockers or cars.
#[test]
fn medical_items_lean_on_med_crates() {
    let ds = real();
    let auto = auto_plan(&ds, &Profile::default(), &plan("buried-city", &["antiseptic"]));
    assert!(!auto.areas.is_empty(), "an area for antiseptic");
    let kinds: Vec<&str> = auto.areas.iter().flat_map(|a| &a.stops).filter(|s| !s.items.is_empty()).map(|s| s.name.as_str()).collect();
    let med = kinds.iter().filter(|k| **k == "Medical Bag").count();
    let cars = kinds.iter().filter(|k| **k == "Car").count();
    assert!(med > cars, "med crates {med} vs cars {cars}: {kinds:?}");
}

#[test]
fn switches_turn_things_off() {
    let ds = real();
    let mut p = plan("dam", &["antiseptic", "mechanical-components"]);
    let on = auto_plan(&ds, &Profile::default(), &p);
    assert!(on.wants.iter().all(|w| w.on && w.placeable));
    p.off.push("item:antiseptic".into());
    p.off.push("box:containers/med_crate".into());
    let off = auto_plan(&ds, &Profile::default(), &p);
    assert!(!off.wants.iter().find(|w| w.item.id == "antiseptic").unwrap().on, "switched-off items stay listed");
    assert!(off.areas.iter().flat_map(|a| &a.covers).all(|c| c.item_id != "antiseptic"));
    assert!(off.areas.iter().flat_map(|a| &a.stops).all(|s| s.name != "Med Crate"));
    assert!(!off.boxes.iter().find(|b| b.key == "box:containers/med_crate").unwrap().on);
}

#[test]
fn areas_are_apart_and_loops_end_near_an_exit() {
    let ds = real();
    let p = plan("dam", &["fabric", "rubber-parts", "plastic-parts"]);
    let auto = auto_plan(&ds, &Profile::default(), &p);
    assert_eq!(auto.areas.len(), 3);
    for (i, a) in auto.areas.iter().enumerate() {
        assert!(a.stops.len() <= 8 && !a.stops.is_empty());
        assert!(!a.exits.is_empty(), "area {} has exits", a.label);
        for b in &auto.areas[i + 1..] {
            assert!((a.x - b.x).hypot(a.y - b.y) > a.radius, "{} and {} overlap", a.label, b.label);
        }
    }
    assert!(auto.areas[0].value >= auto.areas[2].value, "best area first");
}

#[test]
fn tracked_quests_add_their_stops() {
    let ds = real();
    let quest = ds.files.pois.iter().find(|p| p.quest_id.is_some() && p.map_id == "dam").unwrap();
    let mut profile = Profile::default();
    profile.quests.insert(quest.quest_id.clone().unwrap(), QuestProgress::default());
    let auto = auto_plan(&ds, &profile, &plan("dam", &[]));
    assert!(auto.quest_stops.iter().any(|s| s.key == quest.id));
    assert!(auto.areas.iter().flat_map(|a| &a.stops).any(|s| s.kind == "quest"), "an area forms around quest steps alone");
    let mut off = plan("dam", &[]);
    off.off.push("cat:quest".into());
    assert!(auto_plan(&ds, &profile, &off).quest_stops.is_empty());
}

#[test]
fn ordering_takes_the_short_way_round() {
    // A square visited corner to corner: 2-opt removes the crossing.
    let pts = [(0.0, 0.0), (10.0, 10.0), (10.0, 0.0), (0.0, 10.0)];
    let path = order(&pts, 0);
    let length: f64 = path.windows(2).map(|w| (pts[w[0]].0 - pts[w[1]].0).hypot(pts[w[0]].1 - pts[w[1]].1)).sum();
    assert_eq!(path[0], 0);
    assert!((length - 30.0).abs() < 1e-6, "{path:?} is {length}");
}

#[test]
fn farm_finder_ranks_across_maps() {
    let ds = real();
    let spots = farm(&ds, &["antiseptic".into()], &RaidSettings::default());
    assert!(spots.len() >= 3);
    let maps: std::collections::BTreeSet<&str> = spots.iter().map(|s| s.map_id.as_str()).collect();
    assert!(maps.len() >= 2, "more than one map: {maps:?}");
    assert!(spots.windows(2).all(|w| w[0].value >= w[1].value));
}

/// The item page's sources: ARC parts lean on ARC containers, and each source is listed once.
#[test]
fn item_sources_rank_containers_once() {
    // The Turbine Compressor comes from the ARC Turbine, not from husks.
    let ds = real();
    let sources = item_sources(&ds, "turbine-compressor");
    for s in sources.iter().take(8) {
        println!("{} {} {:.2} {:?}", s.name, s.chance, s.score, s.maps);
    }
    assert!(!sources.is_empty());
    let ids: std::collections::BTreeSet<&str> = sources.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.len(), sources.len(), "each source once");
    assert_eq!(sources[0].id, "arc/turbine", "top source");
    assert!(!sources.iter().any(|s| s.id.contains("husk") && s.id != "containers/arc_husk"), "no specific husk drops it");
    assert!(sources.windows(2).all(|w| w[0].score >= w[1].score));
}

/// Every map is ranked, and a map with nothing for the item ranks below one that has it.
#[test]
fn suggests_maps_by_what_they_hold() {
    let ds = real();
    let mut p = plan("", &["antiseptic"]);
    p.off = vec!["cat:quest".into(), "cat:craft".into(), "cat:project".into(), "cat:bench".into()];
    let picks = suggest_maps(&ds, &Profile::default(), &p);
    assert_eq!(picks.len(), ds.files.maps.len());
    for pick in &picks {
        println!("{} {} best={:?} here={:?} missing={:?}", pick.map_name, pick.score, pick.best.as_ref().map(|b| &b.name), pick.items_here, pick.items_missing);
    }
    assert!(picks[0].best.is_some(), "the top map has an area");
    assert!(picks.windows(2).all(|w| w[0].score >= w[1].score));
}

/// With a primary objective, every area's loop goes through it and nearer areas rank first.
#[test]
fn side_areas_hug_the_primary_objective() {
    let ds = real();
    let base = plan("buried-city", &["antiseptic", "fabric", "rubber-parts"]);
    let free = auto_plan(&ds, &Profile::default(), &base);
    assert!(free.areas.len() >= 2);
    // Put the objective on the last free area, so the ranking has to change.
    let far = free.areas.last().unwrap();
    let mut with = base.clone();
    with.primary = Some(Primary { kind: "marker".into(), id: "x".into(), name: "Objective".into(), x: far.x, y: far.y, floors: None });
    let auto = auto_plan(&ds, &Profile::default(), &with);
    assert!(auto.primary.is_some());
    assert!(auto.areas.iter().all(|a| a.stops.iter().any(|s| s.kind == "primary")), "every loop passes the objective");
    assert!(auto.areas.iter().all(|a| a.detour.is_some()));
    let first = &auto.areas[0];
    let d = (first.x - far.x).hypot(first.y - far.y);
    assert!(d < (free.areas[0].x - far.x).hypot(free.areas[0].y - far.y) + 1.0, "top area moved toward the objective");
    // The detour only ranks areas; what an area covers is shown as is.
    for a in &auto.areas {
        if let Some(same) = free.areas.iter().find(|f| (f.x - a.x).abs() < 1.0 && (f.y - a.y).abs() < 1.0) {
            assert!((same.value - a.value).abs() < 0.02, "value {} vs {}", a.value, same.value);
        }
    }
}

/// A quest step picked as the primary is one stop, not that step twice.
#[test]
fn a_quest_step_primary_is_one_stop() {
    let ds = real();
    let quest = ds.files.pois.iter().find(|p| p.quest_id.is_some() && p.map_id == "dam").unwrap();
    let mut profile = Profile::default();
    profile.quests.insert(quest.quest_id.clone().unwrap(), QuestProgress::default());
    let base = plan("dam", &["fabric"]);
    let free = auto_plan(&ds, &profile, &base);
    let q = free.quest_stops.iter().find(|s| s.key == quest.id).cloned().unwrap();
    let mut with = base.clone();
    with.primary = Some(Primary { kind: "quest".into(), id: q.key.clone(), name: q.name.clone(), x: q.x, y: q.y, floors: q.floors });
    let auto = auto_plan(&ds, &profile, &with);
    for a in &auto.areas {
        let here = a.stops.iter().filter(|s| (s.x - q.x).abs() < 1.0 && (s.y - q.y).abs() < 1.0).count();
        assert!(here <= 1, "area {} visits the step {here} times", a.label);
    }
}

/// An unset primary isn't written, so a profile saved here still opens in v0.6.1.
#[test]
fn an_unset_primary_is_not_saved() {
    let p = plan("dam", &[]);
    assert!(!serde_json::to_string(&p).unwrap().contains("primary"));
}

/// A live condition changes the plan: Uncovered Caches makes Raider Caches count for more,
/// and the plan says which conditions it counted.
#[test]
fn live_conditions_change_the_plan() {
    let ds = real();
    let mut p = plan("dam", &["light-ammo"]);
    p.off = vec!["cat:quest".into(), "cat:craft".into(), "cat:project".into(), "cat:bench".into()];
    let quiet = auto_plan(&ds, &Profile::default(), &p);
    let busy = auto_plan_live(&ds, &Profile::default(), &p, &["Uncovered Caches".to_string()]);
    assert!(quiet.conditions.is_empty());
    assert_eq!(busy.conditions.first().map(|c| c.name.as_str()), Some("Uncovered Caches"));
    let caches = |a: &RaidAuto| a.areas.iter().flat_map(|x| &x.stops).filter(|s| s.name == "Raider Cache").count();
    assert!(caches(&busy) >= caches(&quiet), "caches {} vs {}", caches(&busy), caches(&quiet));
    assert_ne!(quiet.areas, busy.areas, "the boost moves something");
}

#[test]
fn arc_parts_show_only_their_arcs_spawns() {
    let ds = real();
    let detail = arc_core::views::item_detail(&ds, "turbine-compressor").unwrap();
    let places: Vec<&str> = detail.found_maps.iter().flat_map(|m| m.places.iter().map(String::as_str)).collect();
    assert!(!places.is_empty(), "the Turbine spawns should be listed");
    assert!(places.iter().all(|p| *p == "Turbine"), "only ARC Turbine spawns, got {places:?}");
}
