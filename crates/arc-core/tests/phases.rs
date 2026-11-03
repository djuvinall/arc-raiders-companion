//! Calculators for Phases 3 to 6, against the real dataset and pinned fixtures.

use arc_core::events::{events_now, parse_schedule, Event};
use arc_core::model::*;
use arc_core::patches::{bbcode_blocks, latest_version, parse_news, version_in_title};
use arc_core::routing::{needed_items, poi_cover};
use arc_core::tracking::*;
use arc_core::weapons::*;
use arc_core::Dataset;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn real() -> Dataset {
    Dataset::load(&root().join("data")).unwrap()
}

fn mods(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn row<'a>(sheet: &'a Sheet, key: &str) -> &'a StatRow {
    sheet.rows.iter().find(|r| r.key == key).unwrap()
}

#[test]
fn stat_dps_is_damage_times_fire_rate() {
    let ds = real();
    let sheet = apply_mods(&ds, "rattler-iv", &[], 100.0).unwrap();
    let dps = row(&sheet, "stat_dps").value.unwrap();
    assert!((dps - 9.0 * 33.3).abs() < 1e-9);
    assert!(sheet.effects.is_empty());
}

#[test]
fn levels_carry_their_upgrades_into_the_sheet() {
    let ds = real();
    let level = |id: &str| apply_mods(&ds, id, &[], 100.0).unwrap();
    assert!(level("anvil-i").upgrades.is_empty());
    let top = level("anvil-iv");
    let names: Vec<_> = top.upgrades.iter().map(|u| (u.label.as_str(), u.pct)).collect();
    assert_eq!(names, [("Increased Fire Rate", 75.0), ("Reduced Dispersion Recovery Time", 18.75)]);
    assert!((row(&top, "stat_dps").value.unwrap() - 40.0 * 10.3 * 1.75).abs() < 1e-9);
    let stiletto: Vec<_> = level("stiletto-iv").upgrades.iter().map(|u| u.pct).collect();
    assert_eq!(stiletto, [25.7, 31.6]);
}

#[test]
fn every_version_of_a_gun_opens_the_same_family() {
    let ds = real();
    let family = weapon_family_view(&ds, "anvil-iii").unwrap();
    assert_eq!(family.name, "Anvil");
    assert_eq!(family.levels.iter().map(|l| l.level).collect::<Vec<_>>(), [1, 2, 3, 4]);
    assert_eq!(family.amplified.len(), 3);
    assert_eq!(weapon_family_view(&ds, "amplified-anvil-x-rounds").unwrap(), family);
    assert!(weapon_family_view(&ds, "metal-parts").is_none());

    let aphelion = weapon_family_view(&ds, "amplified-aphelion-bigger-mag-charged-burst").unwrap();
    assert_eq!(aphelion.name, "Aphelion Rifle");
    assert_eq!(aphelion.levels.len(), 1);
    assert!(aphelion.amplified.iter().any(|v| v.amplifications == ["Bigger Mag", "Charged Burst"]));
}

#[test]
fn the_item_list_groups_weapons_into_guns() {
    let ds = real();
    let items = arc_core::views::item_list(&ds);
    let weapons: Vec<_> = items.iter().filter(|i| i.category == Category::Weapon).collect();
    let mut families: Vec<_> = weapons.iter().map(|i| i.family.clone().unwrap()).collect();
    families.sort();
    families.dedup();
    assert_eq!(families.len(), 26);
    assert!(items.iter().filter(|i| i.category != Category::Weapon).all(|i| i.family.is_none()));
    assert!(items.iter().all(|i| i.variant_of.is_none() || i.category == Category::Weapon));

    for id in families {
        let lead = weapons.iter().find(|i| i.family.as_deref() == Some(&id)).unwrap();
        let family = weapon_family_view(&ds, &lead.id).unwrap();
        assert!(!family.levels.is_empty(), "{id}");
        assert!(![" I", " II", " III", " IV"].iter().any(|s| family.name.ends_with(s)), "{id}");
        // The picker reaches a version by its amplifications, so each set must be unique.
        let mut sets: Vec<_> = family.amplified.iter().map(|v| {
            let mut set = v.amplifications.clone();
            set.sort();
            set
        }).collect();
        sets.sort();
        let total = sets.len();
        sets.dedup();
        assert_eq!(sets.len(), total, "{id}");
        assert!(total == 0 || sets.iter().any(|s| s.is_empty()), "{id} has no plain amplified version");
    }
}

#[test]
fn every_stencil_reaches_its_guns() {
    let ds = real();
    let stencils: Vec<_> = ds.files.items.iter().filter(|i| i.category == Category::Stencil).collect();
    assert_eq!(stencils.len(), 14);
    for item in stencils {
        let fit = item.stencil.as_ref().unwrap();
        assert!(fit.swatch.starts_with("https://static.metaforge.app/"), "{}", item.id);
        assert_eq!(stencil_guns(&ds, fit).len(), fit.weapons.len(), "{}", item.id);
    }
    let guns = stencil_guns(&ds, ds.item("slipstream-stencil").unwrap().stencil.as_ref().unwrap());
    assert_eq!(guns.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), ["Anvil", "Il Toro", "Rattler"]);
    assert_eq!(guns[0].id, "anvil-i");

    let rows = gun_stencils(&ds, "amplified-anvil-x-rounds");
    assert!(rows.iter().any(|r| r.stencil.id == "fortuna-stencil"));
    assert!(!rows.iter().any(|r| r.stencil.id == "bulwark-stencil"));
    assert!(gun_stencils(&ds, "jupiter").is_empty());
    assert!(gun_stencils(&ds, "metal-parts").is_empty());
    let dragon = ds.item("dragons-breath-stencil").unwrap().stencil.as_ref().unwrap();
    assert_eq!((dragon.source.as_str(), dragon.parts), ("Collector Set DLC", 15));
}

#[test]
fn mods_list_only_what_fits_each_slot() {
    let ds = real();
    let slots = slot_options(&ds, "rattler-iv").unwrap();
    assert_eq!(slots.len(), 3);
    let underbarrel = slots.iter().find(|s| s.slot == "underbarrel").unwrap();
    assert!(underbarrel.mods.iter().any(|m| m.id == "angled-grip-i"));
    assert!(slots.iter().all(|s| s.mods.iter().all(|m| m.slot == s.slot && m.tier <= s.tier_max)));
    assert_eq!(weapon_family(&ds, "aphelion-rifle"), "aphelion");
    assert_eq!(weapon_family(&ds, "il-toro-iv"), "il-toro");
}

#[test]
fn percent_mods_become_effects_and_bad_mods_warn() {
    let ds = real();
    let sheet = apply_mods(&ds, "rattler-iv", &mods(&["angled-grip-i"]), 100.0).unwrap();
    let recoil = sheet.effects.iter().find(|e| e.stat == "horizontal_recoil").unwrap();
    assert!((recoil.pct + 20.0).abs() < 1e-9);
    let doubled = apply_mods(&ds, "rattler-iv", &mods(&["angled-grip-i", "angled-grip-ii"]), 100.0).unwrap();
    assert!(doubled.warnings.iter().any(|w| w.contains("slot")));
    let unknown = apply_mods(&ds, "rattler-iv", &mods(&["nope"]), 100.0).unwrap();
    assert_eq!(unknown.warnings.len(), 1);
}

#[test]
fn ttk_needs_measured_rpm() {
    let mut ds = real();
    assert!(apply_mods(&ds, "rattler-iv", &[], 100.0).unwrap().ttk.is_none());
    let t = ttk_flat(100.0, 9.0, 600.0).unwrap();
    assert_eq!(t.shots, 12);
    assert!((t.seconds - 1.2).abs() < 1e-9);
    let item = ds.files.items.iter_mut().find(|i| i.id == "rattler-iv").unwrap();
    let Some(Stats::Weapon(w)) = &mut item.stats else { panic!() };
    w.measured.rounds_per_minute = Some(600.0);
    let ds = Dataset::from_files(ds.files).unwrap();
    let sheet = apply_mods(&ds, "rattler-iv", &[], 100.0).unwrap();
    assert_eq!(sheet.ttk.unwrap().shots, 12);
    assert_eq!(sheet.measured.len(), 1);
}

#[test]
fn compare_marks_the_single_best_value() {
    let ds = real();
    let a = ("A".to_string(), apply_mods(&ds, "rattler-iv", &[], 100.0).unwrap());
    let b = ("B".to_string(), apply_mods(&ds, "hullcracker-i", &[], 100.0).unwrap());
    let same = compare(&[a.clone(), a.clone()]);
    assert!(same.rows.iter().all(|r| r.best.is_none()), "ties mark nothing");
    let table = compare(&[a, b]);
    assert_eq!(table.columns, ["A", "B"]);
    let damage = table.rows.iter().find(|r| r.label == "Stat DPS").unwrap();
    assert_eq!(damage.best, Some(1));
}

#[test]
fn needed_items_combine_sources_minus_stash() {
    let ds = real();
    let mut p = Profile::default();
    p.tracked_item_ids.push("rattler-i".into());
    let first = needed_items(&ds, &p);
    assert!(!first.is_empty());
    let id = first[0].item.id.clone();
    p.stash.insert(id.clone(), first[0].qty);
    assert!(needed_items(&ds, &p).iter().all(|n| n.item.id != id), "stash covers it");

    let project = &ds.files.projects[0];
    p.projects.insert(project.id.clone(), ProjectProgress { stage: 1, committed: Default::default() });
    let wanted = needed_items(&ds, &p);
    let input = &project.stages[0].inputs[0];
    let row = wanted.iter().find(|n| n.item.id == input.item_id).unwrap();
    assert!(row.reasons.iter().any(|r| r.contains(&project.name)));
}

#[test]
fn poi_cover_covers_and_ranks_maps() {
    let ds = real();
    let area = ds.files.loot_areas.iter().find(|a| a.id == "medical").unwrap();
    let wanted: Vec<String> = area.item_ids.iter().take(4).cloned().chain(["no-such-item".to_string()]).collect();
    let covers = poi_cover(&ds, &wanted);
    assert_eq!(covers.len(), ds.files.maps.len());
    let best = &covers[0];
    assert_eq!(best.covered_items.len(), 4);
    assert_eq!(best.uncovered, ["no-such-item"]);
    let reached: std::collections::BTreeSet<_> = best.pois.iter().flat_map(|p| p.covers.iter()).collect();
    assert_eq!(reached.len(), 4, "the chosen POIs reach everything reachable");
    assert!(best.pois.len() <= 4);
    assert!(covers.windows(2).all(|w| w[0].covered_items.len() >= w[1].covered_items.len()));
}

#[test]
fn events_split_into_current_and_next() {
    let ds = real();
    let ev = |name: &str, map: &str, s, e| Event { name: name.into(), map: map.into(), icon: None, start_ms: s, end_ms: e };
    let list = [ev("Past", "Dam", 0, 10), ev("Now", "Dam", 10, 30), ev("Later", "Dam", 40, 50), ev("Soon", "Dam", 30, 40)];
    let rows = events_now(&ds, &list, 20);
    let dam = rows.iter().find(|r| r.map_id == "dam").unwrap();
    assert_eq!(dam.current[0].name, "Now");
    assert_eq!(dam.next.as_ref().unwrap().name, "Soon");
    assert!(rows.iter().find(|r| r.map_id == "spaceport").unwrap().current.is_empty());
}

#[test]
fn the_pinned_schedule_reaches_every_map() {
    let ds = real();
    let raw: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root().join("tests/fixtures/metaforge/2026-11-02.json")).unwrap(),
    ).unwrap();
    let events = parse_schedule(&serde_json::to_vec(&raw["events"]).unwrap()).unwrap();
    assert_eq!(events.len(), 170);
    let start = events.iter().map(|e| e.start_ms).min().unwrap();
    assert!(events_now(&ds, &events, start).iter().all(|m| !m.current.is_empty() || m.next.is_some()));
}

#[test]
fn bbcode_turns_into_blocks_without_markup() {
    let blocks = bbcode_blocks(
        "[p]Raiders![/p][p][img src=\"x\"][/img][/p][h2]New [b]things[/b][/h2][list][*][p]One[/p][*]Two[/list][p]See [url=\"http://x\"]here[/url] [sic[/p]",
    );
    let got: Vec<(&str, &str)> = blocks.iter().map(|b| (b.kind.as_str(), b.text.as_str())).collect();
    assert_eq!(got, [
        ("text", "Raiders!"), ("heading", "New things"), ("item", "One"), ("item", "Two"),
        ("text", "See here [sic"),
    ]);
}

#[test]
fn steam_feed_keeps_official_notes_and_finds_the_patch() {
    let notes = parse_news(&std::fs::read(root().join("tests/fixtures/steam/2026-11-02.json")).unwrap()).unwrap();
    assert!(notes.len() >= 5);
    assert!(notes.iter().all(|n| n.title.is_ascii()), "press items dropped");
    assert_eq!(latest_version(&notes).as_deref(), Some("2.3.0"));
    assert_eq!(version_in_title("FROZEN TRAIL - 2.0 Update").as_deref(), Some("2.0"));
    assert_eq!(version_in_title("PvE Toggle Beta Test: October 13-20"), None);
    assert!(notes[0].blocks.iter().all(|b| !b.text.contains("[/")));
}

#[test]
fn two_floor_maps_list_both_floors() {
    let ds = real();
    let floors = |id: &str| ds.files.maps.iter().find(|m| m.id == id).unwrap().image.layers.clone();
    assert_eq!(floors("spaceport").iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), ["Surface", "Tunnels"]);
    assert!(!floors("spaceport")[0].url.contains("underground"));
    assert_eq!(floors("blue-gate").len(), 2);
    assert_eq!(floors("stella-montis").len(), 2);
    assert!(floors("dam").is_empty());
    assert!(ds.files.pois.iter().all(|p| p.zlayers.is_some()));
}

#[test]
fn dataset_records_the_patch_and_map_tiles() {
    let ds = real();
    assert_eq!(ds.files.version.game_patch.as_deref(), Some("2.3.0"));
    assert!(ds.files.maps.iter().all(|m| m.image.url.is_some() && m.image.world_extent.is_some()));
}

#[test]
fn quest_status_follows_prerequisites_and_objectives() {
    let ds = real();
    let quest = ds.files.quests.iter().find(|q| q.id == "a-balanced-harvest").unwrap();
    let mut p = Profile::default();
    let status = |p: &Profile| quest_rows(&ds, p).into_iter().find(|r| r.quest.id == quest.id).unwrap().status;
    assert_eq!(status(&p), "locked");
    p.quests.insert("eyes-in-the-sky".into(), QuestProgress { objectives_done: vec![], done: true });
    assert_eq!(status(&p), "available");
    let atomics: Vec<String> = atomic_ids(quest).iter().map(|s| s.to_string()).collect();
    set_objective(&ds, &mut p, &quest.id, &atomics[0], true);
    assert_eq!(status(&p), "active");
    for id in &atomics {
        set_objective(&ds, &mut p, &quest.id, id, true);
    }
    assert_eq!(status(&p), "done");
    set_objective(&ds, &mut p, &quest.id, &atomics[0], false);
    assert_eq!(status(&p), "active");
}

fn chain(ds: &Dataset) -> (String, String, String) {
    ds.files.quests.iter().find_map(|last| {
        let before = |quest: &Quest| {
            let id = &quest.prerequisites.first()?.quest_id;
            ds.files.quests.iter().find(|q| q.id == *id)
        };
        let middle = before(last)?;
        let first = before(middle)?;
        Some((first.id.clone(), middle.id.clone(), last.id.clone()))
    }).unwrap()
}

fn status<'a>(rows: &'a [QuestRow], id: &str) -> &'a QuestRow {
    rows.iter().find(|r| r.quest.id == id).unwrap()
}

#[test]
fn quests_behind_a_tracked_quest_are_needed() {
    let ds = real();
    let (first, middle, last) = chain(&ds);
    let mut profile = Profile::default();
    assert_ne!(status(&quest_rows(&ds, &profile), &first).status, "needed");

    profile.quests.insert(last.clone(), QuestProgress::default());
    let rows = quest_rows(&ds, &profile);
    assert_eq!(status(&rows, &first).status, "needed");
    assert_eq!(status(&rows, &middle).status, "needed");
    assert_eq!(status(&rows, &middle).needed_for[0].id, last);
    assert_eq!(status(&rows, &last).status, "active");

    profile.quests.insert(first.clone(), QuestProgress { done: true, ..Default::default() });
    let rows = quest_rows(&ds, &profile);
    assert_eq!(status(&rows, &first).status, "done");
    assert_eq!(status(&rows, &middle).status, "needed");
}

#[test]
fn share_codes_round_trip_and_stay_short() {
    let ds = real();
    let mut profile = Profile::default();
    for quest in &ds.files.quests {
        profile.quests.insert(quest.id.clone(), QuestProgress { done: true, ..Default::default() });
    }
    let (_, _, last) = chain(&ds);
    profile.quests.insert(last.clone(), QuestProgress::default());
    let code = encode_progress(&ds, &profile, " Devon ");
    assert!(code.len() < 500, "{}", code.len());
    let friend = decode_progress(&ds, &code).unwrap();
    assert_eq!(friend.name, "Devon");
    assert_eq!(friend.active, [last]);
    assert_eq!(friend.done.len(), ds.files.quests.len() - 1);
    assert!(decode_progress(&ds, "hello").is_err());
    assert!(decode_progress(&ds, "ARCQ1-").is_err());
}

#[test]
fn the_party_plan_starts_with_the_older_quest() {
    let ds = real();
    let (first, middle, _) = chain(&ds);
    let done = |ids: &[&String]| ids.iter().map(|i| (i.to_string(), QuestProgress { done: true, ..Default::default() })).collect();
    let behind = |d: &[&String]| Profile { quests: done(d), ..Default::default() };

    // Everything before the first quest is finished by both, so only the chain differs.
    let early: Vec<String> = ds.files.quests.iter().filter(|q| q.prerequisites.is_empty()).map(|q| q.id.clone()).collect();
    let mut me = behind(&early.iter().collect::<Vec<_>>());
    me.quests.insert(first.clone(), QuestProgress { done: true, ..Default::default() });
    let them = Friend { name: "Sam".into(), done: early.into_iter().filter(|q| *q != first).collect(), active: vec![] };

    let plan = party_plan(&ds, &me, &them);
    assert_eq!(plan.friend, "Sam");
    assert!(plan.friend_catch_up.iter().any(|q| q.quest.id == first));
    assert!(!plan.together.iter().any(|q| q.quest.id == middle));
    assert!(plan.together.windows(2).all(|w| w[0].depth <= w[1].depth));
    assert!(plan.maps.windows(2).all(|w| w[0].1 >= w[1].1));
}

#[test]
fn projects_carry_what_the_page_sections_need() {
    let ds = real();
    let views = project_views(&ds, &Profile::default());
    let view = |id: &str| views.iter().find(|p| p.id == id).unwrap();
    assert_eq!(view("souls-offering").ends.as_deref(), Some("2026-11-16"));
    assert!(view("trophy-display").permanent && view("trophy-display").ends.is_none());
    assert!(view("expedition").expedition && view("expedition").paused);
    assert_eq!(views.iter().filter(|p| p.permanent || p.paused).count(), 2);
}

#[test]
fn no_blueprint_is_listed_twice() {
    let ds = real();
    let blueprints: Vec<_> = ds.files.items.iter().filter(|i| i.category == Category::Blueprint).collect();
    assert_eq!(blueprints.len(), 91, "the game's Codex count");
    let mut names: Vec<String> = blueprints.iter().map(|b| b.name.replace("'", "").to_lowercase()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), blueprints.len());
    assert!(ds.item("trigger-nade-blueprint").is_some() && ds.item("trigger-nade-blueprint-2").is_none());
}

#[test]
fn verdicts_put_goals_before_value() {
    let ds = real();
    let mut profile = Profile::default();
    profile.tracked_item_ids.push("rattler-ii".into());
    let keep = arc_core::advice::item_verdict(&ds, &profile, "metal-parts").unwrap();
    assert_eq!(keep.action, "keep");
    assert!(keep.headline.contains("more needed") && !keep.reasons.is_empty());

    let mut stocked = Profile::default();
    stocked.stash.insert("metal-parts".into(), 5000);
    let material = arc_core::advice::item_verdict(&ds, &stocked, "metal-parts").unwrap();
    assert_eq!(material.action, "keep");
    assert!(material.headline.contains("recipes"));

    let plain = Profile::default();
    let sold: Vec<_> = ds.files.items.iter().filter(|i| i.category == Category::Recyclable).take(40)
        .filter_map(|i| arc_core::advice::item_verdict(&ds, &plain, &i.id).map(|v| (i, v))).collect();
    assert!(!sold.is_empty());
    for (item, verdict) in sold {
        if verdict.action == "sell" {
            assert!(verdict.recycle_value.unwrap_or(0) <= item.value.unwrap());
        }
    }
    assert!(arc_core::advice::item_verdict(&ds, &plain, "jupiter").is_none());
}

#[test]
fn an_owned_level_comes_off_the_upgrade_cost() {
    let ds = real();
    let full = arc_core::calc::raw_totals(&arc_core::calc::craft_tree(&ds, "anvil-iv", 1).unwrap());
    let from = arc_core::calc::raw_totals(&arc_core::calc::craft_tree_from(&ds, "anvil-iv", 1, Some("anvil-iii")).unwrap());
    assert!(from.values().sum::<u32>() < full.values().sum::<u32>());

    let mut profile = Profile::default();
    profile.tracked_item_ids.push("anvil-iv".into());
    let before: u32 = needed_items(&ds, &profile).iter().map(|n| n.qty).sum();
    profile.tracked_from.insert("anvil-iv".into(), "anvil-iii".into());
    let after: u32 = needed_items(&ds, &profile).iter().map(|n| n.qty).sum();
    assert!(after < before);
}

#[test]
fn a_tracked_bench_adds_its_next_level() {
    let ds = real();
    let bench = ds.files.workbenches.iter().find(|b| b.levels.iter().any(|l| !l.upgrade_inputs.is_empty())).unwrap();
    let level = bench.levels.iter().find(|l| !l.upgrade_inputs.is_empty()).unwrap();
    let mut profile = Profile::default();
    profile.workbenches.insert(bench.id.clone(), level.level - 1);
    profile.tracked_benches.push(bench.id.clone());
    let needed = needed_items(&ds, &profile);
    assert!(needed.iter().any(|n| n.item.id == level.upgrade_inputs[0].item_id));
    let view = workbench_views(&ds, &profile).into_iter().find(|b| b.id == bench.id).unwrap();
    assert_eq!(view.next.len(), level.upgrade_inputs.len());
}

#[test]
fn completing_a_chain_marks_everything_before() {
    let ds = real();
    let (first, middle, last) = chain(&ds);
    let mut profile = Profile::default();
    complete_chain(&ds, &mut profile, &last);
    for id in [&first, &middle, &last] {
        assert!(profile.quests[id].done);
    }
}

#[test]
fn arcs_have_guide_details_and_quests_have_pins() {
    let ds = real();
    let bastion = ds.files.arcs.iter().find(|a| a.id == "bastion").unwrap();
    assert_eq!(bastion.hp, Some(2050.0));
    assert_eq!(bastion.weak_points, ["Rear plate", "Leg joints"]);
    assert!(bastion.tips.is_some());
    assert!(ds.files.arcs.iter().filter(|a| a.hp.is_some()).count() >= 15);
    let pins = ds.files.pois.iter().filter(|p| p.quest_id.is_some()).count();
    assert!(pins > 800, "{pins}");
}

#[test]
fn committing_spends_stash_and_advances_stages() {
    let ds = real();
    let project = &ds.files.projects[0];
    let inputs = &project.stages[0].inputs;
    let mut p = Profile::default();
    p.stash.insert(inputs[0].item_id.clone(), 1000);
    let taken = commit(&ds, &mut p, &project.id, &inputs[0].item_id, 9999);
    assert_eq!(taken, inputs[0].qty, "never more than the stage needs");
    assert_eq!(p.stash[&inputs[0].item_id], 1000 - inputs[0].qty);
    assert_eq!(commit(&ds, &mut p, &project.id, "no-such-item", 1), 0);
    for input in &inputs[1..] {
        commit(&ds, &mut p, &project.id, &input.item_id, input.qty);
    }
    assert_eq!(p.projects[&project.id].stage, 2);
    let view = project_views(&ds, &p).remove(0);
    assert_eq!(view.stages[0].inputs[0].committed, inputs[0].qty, "finished stages read as full");
}

#[test]
fn collections_count_found_entries() {
    let ds = real();
    let mut p = Profile::default();
    let first = collections(&ds, &p).remove(0);
    assert_eq!((first.title, first.found, first.total), ("Blueprints", 0, 91));
    p.blueprints_found.push(first.entries[0].id.clone());
    assert_eq!(collections(&ds, &p)[0].found, 1);
}

#[test]
fn profile_round_trips_through_json() {
    let mut p = Profile::default();
    p.weapon_builds.push(WeaponBuild { name: "x".into(), item_id: "rattler-iv".into(), mod_ids: vec![] });
    p.quests.insert("q".into(), QuestProgress::default());
    let text = serde_json::to_string(&p).unwrap();
    assert_eq!(serde_json::from_str::<Profile>(&text).unwrap(), p);
}
