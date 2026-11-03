//! Runs against the real data/ directory so contributor edits fail in CI.

use arc_core::calc::{craft_tree, raw_totals, used_in};
use arc_core::dataset::integrity_errors;
use arc_core::model::*;
use arc_core::{Dataset, DatasetFiles};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/data")
}

fn dataset() -> Dataset {
    Dataset::load(&data_dir()).expect("dataset loads")
}

fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(data_dir().join(name)).unwrap()).unwrap()
}

/// serde fills a missing Option with None, so re-serializing exposes absent keys.
fn assert_round_trip<T: DeserializeOwned + Serialize>(name: &str) {
    let raw = read(name);
    let parsed: T = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), raw, "{name} has missing or extra keys");
}

#[test]
fn every_file_round_trips_exactly() {
    assert_round_trip::<Vec<Item>>("items.json");
    assert_round_trip::<Vec<Recipe>>("recipes.json");
    assert_round_trip::<Vec<Workbench>>("workbenches.json");
    assert_round_trip::<Vec<WeaponMod>>("weapon_mods.json");
    assert_round_trip::<Vec<GameMap>>("maps.json");
    assert_round_trip::<Vec<Poi>>("pois.json");
    assert_round_trip::<Vec<Quest>>("quests.json");
    assert_round_trip::<Vec<Project>>("projects.json");
    assert_round_trip::<SkillTree>("skills.json");
    assert_round_trip::<Vec<Arc>>("arcs.json");
    assert_round_trip::<Vec<Trader>>("traders.json");
    assert_round_trip::<Vec<LootArea>>("loot_areas.json");
    assert_round_trip::<DatasetVersion>("version.json");
}

#[test]
fn every_file_matches_its_json_schema() {
    let pairs = [
        ("item", "items"), ("recipe", "recipes"), ("workbench", "workbenches"),
        ("weapon_mod", "weapon_mods"), ("map", "maps"), ("poi", "pois"), ("quest", "quests"),
        ("project", "projects"), ("skill_tree", "skills"), ("arc", "arcs"),
        ("trader", "traders"), ("loot_area", "loot_areas"), ("version", "version"),
    ];
    for (schema, file) in pairs {
        let schema = read(&format!("schema/{schema}.schema.json"));
        let validator = jsonschema::validator_for(&schema).unwrap();
        let doc = read(&format!("{file}.json"));
        let first = validator.iter_errors(&doc).next().map(|e| e.to_string());
        assert_eq!(first, None, "{file}.json");
    }
}

#[test]
fn unknown_fields_are_rejected() {
    let mut rows = read("items.json");
    rows[0]["surprise"] = Value::Bool(true);
    assert!(serde_json::from_value::<Vec<Item>>(rows).is_err());

    let mut rows = read("items.json");
    let weapon = rows.as_array_mut().unwrap().iter_mut()
        .find(|i| i["category"] == "weapon").unwrap();
    weapon["stats"]["surprise"] = Value::Bool(true);
    assert!(serde_json::from_value::<Vec<Item>>(rows).is_err(), "nested stats block");
}

#[test]
fn broken_references_fail_validation() {
    let ds = dataset();
    let mut files: DatasetFiles = ds.files.clone();
    assert!(integrity_errors(&files).is_empty());
    files.recipes[0].inputs.push(ItemQty { item_id: "no-such-item".into(), qty: 1 });
    let errors = integrity_errors(&files);
    assert!(errors.iter().any(|e| e.contains("no-such-item")), "{errors:?}");
    assert!(Dataset::from_files(files).is_err());
}

#[test]
fn loads_within_budget() {
    let started = Instant::now();
    let ds = dataset();
    let elapsed = started.elapsed();
    assert!(ds.files.items.len() > 1000);
    // The plan's 100 ms budget assumes a release build; debug is ~10x slower.
    let limit = if cfg!(debug_assertions) { 1500 } else { 100 };
    assert!(elapsed.as_millis() < limit, "took {elapsed:?}");
}

#[test]
fn craft_tree_reaches_raw_materials() {
    let ds = dataset();
    let tree = craft_tree(&ds, "rattler-ii", 1).unwrap();
    let ids: Vec<&str> = tree.children.iter().map(|c| c.item_id.as_str()).collect();
    assert!(ids.contains(&"rattler-i"), "{ids:?}");

    let totals = raw_totals(&tree);
    // Wiki: Rattler I = 16 metal + 12 rubber, Rattler II adds 10 metal + 10 rubber.
    assert_eq!(totals.get("metal-parts"), Some(&26));
    assert_eq!(totals.get("rubber-parts"), Some(&22));
    assert!(!totals.contains_key("rattler-i"), "only leaves are totalled");

    let double = raw_totals(&craft_tree(&ds, "rattler-ii", 2).unwrap());
    assert_eq!(double.get("metal-parts"), Some(&52));
}

#[test]
fn every_craftable_item_expands_without_a_cycle() {
    let ds = dataset();
    for item in ds.files.items.iter().filter(|i| !i.recipe_ids.is_empty()) {
        fn no_cycle(node: &arc_core::calc::TreeNode) -> bool {
            !node.cycle && node.children.iter().all(no_cycle)
        }
        assert!(no_cycle(&craft_tree(&ds, &item.id, 1).unwrap()), "{}", item.id);
    }
}

#[test]
fn craft_tree_stops_at_cycles_and_scales_by_batch() {
    let ds = dataset();
    let mut files = ds.files.clone();
    let craft = |id: &str, out: &str, per: u32, input: &str, qty: u32| {
        let mut r = files.recipes.iter().find(|r| r.kind == RecipeKind::Craft).unwrap().clone();
        r.id = id.to_string();
        r.output = Some(ItemQty { item_id: out.to_string(), qty: per });
        r.inputs = vec![ItemQty { item_id: input.to_string(), qty }];
        r.blueprint_item_id = None;
        r
    };
    let looped = [craft("loop-a", "metal-parts", 1, "rubber-parts", 2),
        craft("loop-b", "rubber-parts", 3, "metal-parts", 1)];
    files.recipes.extend(looped);
    for item in &mut files.items {
        match item.id.as_str() {
            "metal-parts" => item.recipe_ids = vec!["loop-a".into()],
            "rubber-parts" => item.recipe_ids = vec!["loop-b".into()],
            _ => {}
        }
    }
    let ds = Dataset::from_files(files).unwrap();
    let tree = craft_tree(&ds, "metal-parts", 2).unwrap();
    // 2 metal needs 4 rubber; rubber yields 3 per run, so 2 runs and 2 metal.
    assert_eq!(tree.children[0].qty, 4);
    assert_eq!(tree.children[0].children[0].qty, 2);
    assert!(tree.children[0].children[0].cycle);
}

#[test]
fn used_in_covers_every_consumer_kind() {
    let ds = dataset();
    let metal = used_in(&ds, "metal-parts");
    assert!(metal.recipes.iter().any(|r| r.output_item_id.as_deref() == Some("rattler-i")));
    assert!(!metal.workbenches.is_empty(), "workbench upgrades");

    let in_any_project = ds.files.projects.iter()
        .flat_map(|p| p.stages.iter().flat_map(|s| &s.inputs))
        .next().unwrap();
    assert!(!used_in(&ds, &in_any_project.item_id).projects.is_empty());

    let quest = ds.files.quests.iter().find(|q| !q.required_items.is_empty()).unwrap();
    let used = used_in(&ds, &quest.required_items[0].item_id);
    assert!(used.quests.iter().any(|q| q.quest_id == quest.id));
}

#[test]
fn amplified_weapons_fold_under_their_base() {
    let ds = dataset();
    let item = ds.item("amplified-rattler").unwrap();
    assert_eq!(item.variant_of.as_deref(), Some("rattler-iv"));
    assert_eq!(item.tier, Some(4));
    assert_eq!(item.rarity, ds.item("rattler-iv").unwrap().rarity);
}

#[test]
fn every_amplified_weapon_has_a_base() {
    let ds = dataset();
    let orphans: Vec<&str> = ds.files.items.iter()
        .filter(|i| i.category == Category::Weapon && i.name.starts_with("Amplified "))
        .filter(|i| i.variant_of.is_none())
        .map(|i| i.name.as_str())
        .collect();
    assert!(orphans.is_empty(), "{orphans:?}");
    let aphelion = ds.item("aphelion-rifle").unwrap();
    assert!(matches!(&aphelion.stats, Some(Stats::Weapon(w)) if !w.mod_slots.is_empty()));
}

#[test]
fn trader_prices_name_their_currency() {
    let ds = dataset();
    let sold = &ds.item("voltage-converter").unwrap().sold_by[0];
    assert_eq!((sold.trader.as_str(), sold.currency.as_deref()), ("celeste", Some("Assorted Seeds")));
    let coins = ds.files.traders.iter().find(|t| t.id == "tian-wen").unwrap();
    assert!(coins.inventory.iter().all(|s| s.currency.as_deref() == Some("Coins")));
}

#[test]
fn every_item_and_entity_has_a_page() {
    use arc_core::views::{craft_plan, entity, item_detail};
    let ds = dataset();
    for item in &ds.files.items {
        let detail = item_detail(&ds, &item.id).unwrap();
        for link in detail.sold_by.iter().chain(&detail.dropped_by)
            .chain(detail.found_in.iter().map(|f| &f.area)) {
            assert!(entity(&ds, link.kind, &link.id).is_some(), "{} {}", link.kind, link.id);
        }
        assert!(craft_plan(&ds, &item.id, 1, None).is_some());
    }
    for w in &ds.files.workbenches { assert!(entity(&ds, "workbench", &w.id).is_some()); }
    for q in &ds.files.quests { assert!(entity(&ds, "quest", &q.id).is_some()); }
    for p in &ds.files.projects { assert!(entity(&ds, "project", &p.id).is_some()); }
    for m in &ds.files.maps { assert!(entity(&ds, "map", &m.id).is_some()); }
    assert!(entity(&ds, "nonsense", "x").is_none());
}

#[test]
fn zeroes_outside_a_category_do_not_become_stats() {
    let ds = dataset();
    match ds.item("rattler-i").unwrap().stats.as_ref().unwrap() {
        Stats::Weapon(w) => {
            assert_eq!(w.magazine_size, Some(12));
            assert!(w.mod_slots.iter().any(|s| s.slot == "muzzle"));
            assert_eq!(w.measured, MeasuredStats::default());
        }
        other => panic!("{other:?}"),
    }
    assert!(ds.item("metal-parts").unwrap().stats.is_none());
}

#[test]
fn mod_modifiers_keep_their_sign() {
    let ds = dataset();
    let barrel = ds.files.weapon_mods.iter().find(|m| m.id == "extended-barrel-iii").unwrap();
    let got: BTreeMap<String, f64> = barrel.modifiers.iter()
        .map(|m| (format!("{:?}", m.stat), m.pct)).collect();
    assert_eq!(got["BulletVelocity"], 30.0);
    assert_eq!(got["VerticalRecoil"], 15.0);
    let grip = ds.files.weapon_mods.iter().find(|m| m.id == "angled-grip-i").unwrap();
    assert_eq!(grip.modifiers[0].pct, -20.0);
}

#[test]
fn profile_defaults_for_missing_keys() {
    let p: Profile = serde_json::from_str(r#"{"stash":{"metal-parts":3}}"#).unwrap();
    assert_eq!(p.schema_version, 1);
    assert_eq!(p.stash["metal-parts"], 3);
}
