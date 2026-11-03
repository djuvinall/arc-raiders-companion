//! Thin shell over arc-core: commands, profile.json, and the icon cache.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data_update;
mod feeds;
mod icons;
mod shots;
mod updates;

use anyhow::{Context, Result};
use arc_core::events::{self, MapEvents};
use arc_core::advice::{self, Home, Verdict};
use arc_core::model::{Arc, ChangelogEntry, GameMap, Poi, Profile, SkillPlan, SkillTree, WeaponBuild};
use arc_core::patches::{self, PatchNote};
use arc_core::routing::{self, MapCover, Needed};
use arc_core::skills::{self, PlanState};
use arc_core::tracking::{self, BenchView, Collection, PartyPlan, ProjectView, QuestRow};
use arc_core::views::{self, CraftPlan, EntityPage, ItemDetail, ItemSummary, Link, Overview};
use arc_core::weapons::{self, CompareTable, Sheet, SlotOptions};
use arc_core::Dataset;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

macro_rules! embedded {
    ($($name:literal),* $(,)?) => {
        fn embedded(name: &str) -> Result<String> {
            match name {
                $($name => Ok(include_str!(concat!("../../../data/", $name)).to_string()),)*
                other => anyhow::bail!("{other} is not embedded"),
            }
        }
    };
}

// The exe carries the dataset so it runs with no install step and no network.
embedded!(
    "items.json", "recipes.json", "workbenches.json", "weapon_mods.json", "maps.json",
    "pois.json", "quests.json", "projects.json", "skills.json", "arcs.json", "traders.json",
    "loot_areas.json", "map_markers.json", "version.json", "changelog.json",
);

// A pinned copy of the Steam feed, so patch notes read with a cold cache.
const NEWS_SNAPSHOT: &[u8] = include_bytes!("../../../tests/fixtures/steam/2026-11-02.json");
const NEWS_URL: &str = "https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=1808500&count=30";
const EVENTS_URL: &str = "https://metaforge.app/api/arc-raiders/events-schedule";
const EVENT_REGIONS: &[&str] = &["europe", "north-america", "oceania"];

struct AppState {
    dataset: Dataset,
    changelog: String,
    profile: Mutex<Profile>,
    profile_path: PathBuf,
}

type CommandResult<T> = Result<T, String>;

fn missing(what: &str, id: &str) -> String {
    format!("unknown {what}: {id}")
}

#[tauri::command]
fn overview(state: State<AppState>) -> Overview {
    views::overview(&state.dataset)
}

#[tauri::command]
fn list_items(state: State<AppState>) -> Vec<ItemSummary> {
    views::item_list(&state.dataset)
}

#[tauri::command]
fn get_item(state: State<AppState>, id: String) -> CommandResult<ItemDetail> {
    views::item_detail(&state.dataset, &id).ok_or_else(|| missing("item", &id))
}

#[tauri::command]
fn craft_plan(state: State<AppState>, id: String, qty: u32, owned: Option<String>) -> CommandResult<CraftPlan> {
    views::craft_plan(&state.dataset, &id, qty.max(1), owned.as_deref()).ok_or_else(|| missing("item", &id))
}

#[tauri::command]
fn set_tracked_from(state: State<AppState>, id: String, owned: Option<String>) -> CommandResult<()> {
    with_profile(&state, |p| match owned {
        Some(from) => { p.tracked_from.insert(id, from); }
        None => { p.tracked_from.remove(&id); }
    })
}

#[tauri::command]
fn item_verdict(state: State<AppState>, id: String) -> Option<Verdict> {
    advice::item_verdict(&state.dataset, &state.profile.lock().expect("profile lock"), &id)
}

#[tauri::command]
fn home_view(state: State<AppState>) -> Home {
    advice::home(&state.dataset, &state.profile.lock().expect("profile lock"))
}

/// Applies a reviewed screenshot import to the Codex.
#[tauri::command]
fn set_found(state: State<AppState>, add: Vec<String>, remove: Vec<String>) -> CommandResult<()> {
    with_profile(&state, |p| {
        p.blueprints_found.retain(|id| !remove.contains(id) && !add.contains(id));
        p.blueprints_found.extend(add);
    })
}

#[tauri::command]
fn arc_list(state: State<AppState>) -> Vec<ArcView> {
    state.dataset.files.arcs.iter().map(|arc| ArcView {
        drops: arc.drops.iter().map(|d| views::item_link(&state.dataset, &d.item_id, None)).collect(),
        arc: arc.clone(),
    }).collect()
}

#[tauri::command]
fn workbench_list(state: State<AppState>) -> Vec<BenchView> {
    tracking::workbench_views(&state.dataset, &state.profile.lock().expect("profile lock"))
}

#[tauri::command]
fn set_workbench_level(state: State<AppState>, id: String, level: u8) -> CommandResult<()> {
    with_profile(&state, |p| { p.workbenches.insert(id, level); })
}

#[tauri::command]
fn toggle_tracked_bench(state: State<AppState>, id: String) -> CommandResult<()> {
    with_profile(&state, |p| {
        if p.tracked_benches.contains(&id) {
            p.tracked_benches.retain(|b| *b != id);
        } else {
            p.tracked_benches.push(id);
        }
    })
}

#[tauri::command]
fn complete_quest_chain(state: State<AppState>, quest_id: String) -> CommandResult<()> {
    with_profile(&state, |p| tracking::complete_chain(&state.dataset, p, &quest_id))
}

/// The always-on-top window. `async` because Windows deadlocks creating one on the main thread.
#[tauri::command(async)]
fn open_mini(app: AppHandle) -> CommandResult<()> {
    if let Some(window) = app.get_webview_window("mini") {
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(&app, "mini", WebviewUrl::App("index.html#/mini".into()))
        .title("ARC Companion")
        .inner_size(340.0, 600.0)
        .min_inner_size(280.0, 320.0)
        .always_on_top(true)
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_entity(state: State<AppState>, kind: String, id: String) -> CommandResult<EntityPage> {
    views::entity(&state.dataset, &kind, &id).ok_or_else(|| missing(&kind, &id))
}

#[tauri::command]
fn get_profile(state: State<AppState>) -> Profile {
    state.profile.lock().expect("profile lock").clone()
}

#[tauri::command]
fn set_stash(state: State<AppState>, item_id: String, qty: u32) -> CommandResult<()> {
    let mut profile = state.profile.lock().expect("profile lock");
    if qty == 0 {
        profile.stash.remove(&item_id);
    } else {
        profile.stash.insert(item_id, qty);
    }
    save_profile(&state.profile_path, &profile).map_err(|e| e.to_string())
}

/// Applies a change to the profile and saves it.
fn with_profile<T>(state: &AppState, change: impl FnOnce(&mut Profile) -> T) -> CommandResult<T> {
    let mut profile = state.profile.lock().expect("profile lock");
    let result = change(&mut profile);
    save_profile(&state.profile_path, &profile).map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
fn weapon_slots(state: State<AppState>, id: String) -> CommandResult<Vec<SlotOptions>> {
    weapons::slot_options(&state.dataset, &id).ok_or_else(|| missing("weapon", &id))
}

#[tauri::command]
fn weapon_sheet(state: State<AppState>, build: WeaponBuild, target_hp: f64) -> CommandResult<Sheet> {
    weapons::apply_mods(&state.dataset, &build.item_id, &build.mod_ids, target_hp)
        .ok_or_else(|| missing("weapon", &build.item_id))
}

#[tauri::command]
fn weapon_compare(state: State<AppState>, builds: Vec<WeaponBuild>) -> CompareTable {
    let sheets: Vec<(String, Sheet)> = builds.iter().filter_map(|b| {
        weapons::apply_mods(&state.dataset, &b.item_id, &b.mod_ids, 0.0).map(|s| (b.name.clone(), s))
    }).collect();
    weapons::compare(&sheets)
}

#[tauri::command]
fn save_weapon_build(state: State<AppState>, build: WeaponBuild) -> CommandResult<()> {
    with_profile(&state, |p| match p.weapon_builds.iter_mut().find(|b| b.name == build.name) {
        Some(existing) => *existing = build,
        None => p.weapon_builds.push(build),
    })
}

#[tauri::command]
fn delete_weapon_build(state: State<AppState>, name: String) -> CommandResult<()> {
    with_profile(&state, |p| p.weapon_builds.retain(|b| b.name != name))
}

#[tauri::command]
fn map_list(state: State<AppState>) -> Vec<GameMap> {
    state.dataset.files.maps.clone()
}

#[derive(Serialize)]
struct MarkerView {
    categories: Vec<arc_core::model::MarkerCategory>,
    markers: Vec<arc_core::model::MapMarker>,
}

#[tauri::command]
fn map_markers(state: State<AppState>, id: String) -> MarkerView {
    let markers = &state.dataset.files.markers;
    MarkerView { categories: markers.categories.clone(), markers: markers.maps.get(&id).cloned().unwrap_or_default() }
}

#[tauri::command]
fn map_pois(state: State<AppState>, id: String) -> Vec<Poi> {
    state.dataset.files.pois.iter().filter(|p| p.map_id == id).cloned().collect()
}

#[derive(Serialize)]
struct ArcView {
    arc: Arc,
    drops: Vec<Link>,
}

#[derive(Serialize)]
struct EventsView {
    maps: Vec<MapEvents>,
    region: String,
    fetched_ms: Option<i64>,
    stale: bool,
}

// `async` keeps the network call off the main thread, so the window stays responsive.
#[tauri::command(async)]
fn live_events(app: AppHandle, state: State<AppState>, region: String) -> EventsView {
    let region = if EVENT_REGIONS.contains(&region.as_str()) { region } else { EVENT_REGIONS[0].to_string() };
    let fetched = feeds::get(
        app.path().app_cache_dir().ok(),
        &format!("events-{region}.json"),
        &format!("{EVENTS_URL}?region={region}"),
        feeds::EVENTS_TTL,
        |b| events::parse_schedule(b).is_ok(),
    );
    let schedule = fetched.as_ref().and_then(|f| events::parse_schedule(&f.bytes).ok()).unwrap_or_default();
    EventsView {
        maps: events::events_now(&state.dataset, &schedule, feeds::now_ms()),
        region,
        fetched_ms: fetched.as_ref().map(|f| f.fetched_ms),
        stale: fetched.is_some_and(|f| f.stale),
    }
}

#[derive(Serialize)]
struct RoutePlan {
    needed: Vec<Needed>,
    covers: Vec<MapCover>,
}

#[tauri::command]
fn route_plan(state: State<AppState>) -> RoutePlan {
    let profile = state.profile.lock().expect("profile lock");
    let needed = routing::needed_items(&state.dataset, &profile);
    let ids: Vec<String> = needed.iter().map(|n| n.item.id.clone()).collect();
    RoutePlan { covers: routing::poi_cover(&state.dataset, &ids), needed }
}

#[tauri::command]
fn toggle_tracked_item(state: State<AppState>, id: String) -> CommandResult<bool> {
    with_profile(&state, |p| {
        let tracked = p.tracked_item_ids.contains(&id);
        p.tracked_item_ids.retain(|i| *i != id);
        if !tracked {
            p.tracked_item_ids.push(id);
        }
        !tracked
    })
}

#[tauri::command]
fn quest_list(state: State<AppState>) -> Vec<QuestRow> {
    tracking::quest_rows(&state.dataset, &state.profile.lock().expect("profile lock"))
}

#[tauri::command]
fn set_quest_objective(state: State<AppState>, quest_id: String, objective_id: String, done: bool) -> CommandResult<()> {
    with_profile(&state, |p| tracking::set_objective(&state.dataset, p, &quest_id, &objective_id, done))
}

/// `tracked` false forgets the quest's progress; `done` marks it finished outright.
#[tauri::command]
fn set_quest_state(state: State<AppState>, quest_id: String, tracked: bool, done: bool) -> CommandResult<()> {
    with_profile(&state, |p| {
        if tracked {
            p.quests.entry(quest_id).or_default().done = done;
        } else {
            p.quests.remove(&quest_id);
        }
    })
}

/// Saves the name so the next code uses it too.
#[tauri::command]
fn quest_share_code(state: State<AppState>, name: String) -> CommandResult<String> {
    with_profile(&state, |p| {
        p.player_name = Some(name.trim().to_string());
        tracking::encode_progress(&state.dataset, p, &name)
    })
}

/// Pasting a newer code from the same friend replaces their progress.
#[tauri::command]
fn add_friend(state: State<AppState>, code: String) -> CommandResult<String> {
    let friend = tracking::decode_progress(&state.dataset, &code)?;
    let name = friend.name.clone();
    with_profile(&state, |p| {
        p.friends.retain(|f| f.name != friend.name);
        p.friends.push(friend);
    })?;
    Ok(name)
}

#[tauri::command]
fn remove_friend(state: State<AppState>, name: String) -> CommandResult<()> {
    with_profile(&state, |p| p.friends.retain(|f| f.name != name))
}

#[tauri::command]
fn party_plan(state: State<AppState>, name: String) -> CommandResult<PartyPlan> {
    let profile = state.profile.lock().expect("profile lock");
    let friend = profile.friends.iter().find(|f| f.name == name).ok_or_else(|| missing("friend", &name))?;
    Ok(tracking::party_plan(&state.dataset, &profile, friend))
}

#[tauri::command]
fn project_list(state: State<AppState>) -> Vec<ProjectView> {
    tracking::project_views(&state.dataset, &state.profile.lock().expect("profile lock"))
}

#[tauri::command]
fn commit_resource(state: State<AppState>, project_id: String, item_id: String, qty: u32) -> CommandResult<u32> {
    with_profile(&state, |p| tracking::commit(&state.dataset, p, &project_id, &item_id, qty))
}

#[tauri::command]
fn set_project_tracked(state: State<AppState>, project_id: String, tracked: bool) -> CommandResult<()> {
    with_profile(&state, |p| {
        if tracked {
            p.projects.entry(project_id).or_insert_with(|| arc_core::model::ProjectProgress {
                stage: 1,
                committed: Default::default(),
            });
        } else {
            p.projects.remove(&project_id);
        }
    })
}

#[tauri::command]
fn collection_list(state: State<AppState>) -> Vec<Collection> {
    tracking::collections(&state.dataset, &state.profile.lock().expect("profile lock"))
}

#[tauri::command]
fn toggle_found(state: State<AppState>, id: String) -> CommandResult<bool> {
    with_profile(&state, |p| {
        let found = p.blueprints_found.contains(&id);
        p.blueprints_found.retain(|i| *i != id);
        if !found {
            p.blueprints_found.push(id);
        }
        !found
    })
}

#[derive(Serialize)]
struct NewsView {
    notes: Vec<PatchNote>,
    /// Null means the notes are the snapshot shipped in the exe.
    fetched_ms: Option<i64>,
}

#[tauri::command(async)]
fn patch_notes(app: AppHandle) -> NewsView {
    let fetched = feeds::get(app.path().app_cache_dir().ok(), "steam-news.json", NEWS_URL, feeds::NEWS_TTL, |b| {
        patches::parse_news(b).is_ok()
    });
    if let Some(notes) = fetched.as_ref().and_then(|f| patches::parse_news(&f.bytes).ok()) {
        return NewsView { notes, fetched_ms: fetched.map(|f| f.fetched_ms) };
    }
    NewsView { notes: patches::parse_news(NEWS_SNAPSHOT).unwrap_or_default(), fetched_ms: None }
}

#[tauri::command]
fn dataset_changelog(state: State<AppState>) -> Vec<ChangelogEntry> {
    serde_json::from_str(&state.changelog).unwrap_or_default()
}

#[tauri::command]
fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command(async)]
fn check_data_update(app: AppHandle, state: State<AppState>) -> CommandResult<Option<String>> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("data-update");
    data_update::fetch(&dir, &state.dataset.files.version.dataset_version).map_err(|e| e.to_string())
}

#[tauri::command]
fn restart_app(app: AppHandle) {
    app.restart()
}

/// Checks the stencil is real, which also keeps the id safe to use as a folder name.
fn shot_dir(app: &AppHandle, state: &AppState, stencil_id: &str) -> CommandResult<PathBuf> {
    if state.dataset.item(stencil_id).and_then(|i| i.stencil.as_ref()).is_none() {
        return Err(missing("stencil", stencil_id));
    }
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("stencil-shots").join(stencil_id))
}

#[tauri::command]
fn stencil_shots(app: AppHandle, state: State<AppState>, stencil_id: String) -> CommandResult<BTreeMap<String, String>> {
    Ok(shots::list(&shot_dir(&app, &state, &stencil_id)?))
}

#[tauri::command]
fn save_stencil_shot(app: AppHandle, state: State<AppState>, stencil_id: String, gun_id: String, image: String) -> CommandResult<()> {
    let dir = shot_dir(&app, &state, &stencil_id)?;
    state.dataset.item(&gun_id).ok_or_else(|| missing("weapon", &gun_id))?;
    shots::save(&dir, &gun_id, &image).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_stencil_shot(app: AppHandle, state: State<AppState>, stencil_id: String, gun_id: String) -> CommandResult<()> {
    let dir = shot_dir(&app, &state, &stencil_id)?;
    state.dataset.item(&gun_id).ok_or_else(|| missing("weapon", &gun_id))?;
    shots::remove(&dir, &gun_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_profile(state: State<AppState>) -> CommandResult<String> {
    serde_json::to_string_pretty(&*state.profile.lock().expect("profile lock")).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_profile(state: State<AppState>, text: String) -> CommandResult<()> {
    let imported: Profile = serde_json::from_str(&text).map_err(|e| format!("That isn't a profile: {e}"))?;
    with_profile(&state, |p| *p = imported)
}

#[tauri::command]
fn skill_tree(state: State<AppState>) -> SkillTree {
    state.dataset.files.skills.clone()
}

#[tauri::command]
fn skill_plan_state(state: State<AppState>, plan: SkillPlan) -> PlanState {
    skills::plan_state(&state.dataset.files.skills, &plan)
}

#[tauri::command]
fn save_skill_plan(state: State<AppState>, plan: SkillPlan) -> CommandResult<()> {
    let mut profile = state.profile.lock().expect("profile lock");
    match profile.skill_plans.iter_mut().find(|p| p.name == plan.name) {
        Some(existing) => *existing = plan,
        None => profile.skill_plans.push(plan),
    }
    profile.skill_tree_version = Some(state.dataset.files.skills.tree_version.clone());
    save_profile(&state.profile_path, &profile).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_skill_plan(state: State<AppState>, name: String) -> CommandResult<()> {
    let mut profile = state.profile.lock().expect("profile lock");
    profile.skill_plans.retain(|p| p.name != name);
    save_profile(&state.profile_path, &profile).map_err(|e| e.to_string())
}

#[tauri::command]
fn encode_skill_plan(plan: SkillPlan) -> String {
    skills::encode(&plan)
}

#[tauri::command]
fn decode_skill_plan(code: String) -> CommandResult<SkillPlan> {
    skills::decode(&code)
}

fn load_profile(path: &PathBuf) -> Profile {
    let mut profile: Profile = std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    // The Trigger Nade blueprint was once listed twice; keep a mark made on the duplicate.
    let found = &mut profile.blueprints_found;
    if found.iter().any(|id| id == "trigger-nade-blueprint-2") {
        found.retain(|id| id != "trigger-nade-blueprint-2" && id != "trigger-nade-blueprint");
        found.push("trigger-nade-blueprint".to_string());
    }
    profile
}

fn save_profile(path: &PathBuf, profile: &Profile) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(profile)?)?;
    std::fs::rename(&tmp, path).context("replacing profile.json")
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updates::Pending::default())
        .register_asynchronous_uri_scheme_protocol("arcicon", |ctx, request, responder| {
            let cache = ctx.app_handle().path().app_cache_dir().ok();
            let path = request.uri().path().to_string();
            std::thread::spawn(move || responder.respond(icons::serve(cache, &path)));
        })
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let profile_path = data_dir.join("profile.json");
            let (dataset, changelog) = data_update::load(&data_dir.join("data-update"), &embedded)?;
            app.manage(AppState {
                dataset,
                changelog,
                profile: Mutex::new(load_profile(&profile_path)),
                profile_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            overview, list_items, get_item, craft_plan, get_entity, get_profile, set_stash,
            skill_tree, skill_plan_state, save_skill_plan, delete_skill_plan,
            encode_skill_plan, decode_skill_plan, weapon_slots, weapon_sheet, weapon_compare,
            save_weapon_build, delete_weapon_build, map_list, map_pois, map_markers, live_events, route_plan,
            toggle_tracked_item, quest_list, set_quest_objective, set_quest_state, project_list,
            commit_resource, set_project_tracked, collection_list, toggle_found, patch_notes,
            dataset_changelog, export_profile, import_profile, app_version, check_data_update,
            home_view, set_found, set_tracked_from, item_verdict, arc_list, workbench_list, set_workbench_level,
            toggle_tracked_bench, complete_quest_chain, open_mini,
            restart_app, quest_share_code, add_friend, remove_friend, party_plan, stencil_shots, save_stencil_shot, delete_stencil_shot,
            updates::check_app_update, updates::install_app_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running the app");
}
