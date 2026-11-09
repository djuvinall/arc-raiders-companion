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
    let med = kinds.iter().filter(|k| **k == "Med Crate").count();
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
    let ds = real();
    let sources = item_sources(&ds, "turbine-compressor");
    for s in sources.iter().take(8) {
        println!("{} {} {:.2} {:?}", s.name, s.chance, s.score, s.maps);
    }
    assert!(!sources.is_empty());
    let ids: std::collections::BTreeSet<&str> = sources.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.len(), sources.len(), "each source once");
    assert!(sources[0].group == "arc" || sources[0].id.contains("arc") || sources[0].id.contains("husk"), "top source {}", sources[0].id);
    assert!(sources.windows(2).all(|w| w[0].score >= w[1].score));
}
