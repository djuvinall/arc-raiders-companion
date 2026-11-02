//! Canonical dataset schema (plan §4). `null` means unknown; a missing key is a
//! schema error. Fields beyond the plan are noted where the sources required them.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Api,
    Wiki,
    /// RaidTheory community JSON; added to the plan's list so fallback rows stay visible.
    Community,
    CommunityMeasured,
    HandEntered,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRefs {
    pub metaforge: Option<String>,
    pub wiki: Option<String>,
    pub raidtheory: Option<String>,
}

/// The plan's enum plus the categories MetaForge has and the plan lacked
/// (`weapon-mod`, `recyclable`, `nature`, `research`, `furniture`, `quest-item`, `misc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Weapon,
    WeaponMod,
    Shield,
    Augment,
    Ammo,
    Healing,
    Gadget,
    Grenade,
    Tool,
    Key,
    MaterialBasic,
    MaterialTopside,
    MaterialRefined,
    Recyclable,
    Nature,
    Trinket,
    Valuable,
    Food,
    Research,
    QuestItem,
    Blueprint,
    Stencil,
    Design,
    Furniture,
    Cosmetic,
    Misc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SoldBy {
    pub trader: String,
    pub price: u32,
    /// Not always coins: Celeste trades in Assorted Seeds. Null when no source says.
    pub currency: Option<String>,
    pub daily_limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub subcategory: Option<String>,
    pub rarity: Option<Rarity>,
    pub tier: Option<u8>,
    pub description: String,
    pub flavor_text: String,
    pub icon: Option<String>,
    pub value: Option<u32>,
    pub weight_kg: Option<f64>,
    pub stack_size: Option<u32>,
    pub stats: Option<Stats>,
    pub ammo_type: Option<String>,
    pub loadout_slots: Vec<String>,
    pub loot_areas: Vec<String>,
    pub drops_from: Vec<String>,
    pub sold_by: Vec<SoldBy>,
    pub recipe_ids: Vec<String>,
    pub recycle_ids: Vec<String>,
    pub blueprint_required: bool,
    /// Amplified weapons point at the tier they upgrade. Not in the plan's example:
    /// MetaForge lists 94 of them and the grid needs to fold them under the base weapon.
    pub variant_of: Option<String>,
    /// Stencils only.
    pub stencil: Option<StencilFit>,
    pub source_refs: SourceRefs,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StencilFit {
    /// Flat pattern card, a URL fetched at runtime like other art.
    pub swatch: String,
    /// Weapon family ids, community-reported. Embark publishes no list.
    pub weapons: Vec<String>,
    /// How to get it, such as "Topside loot".
    pub source: String,
    /// Stencil Parts spent each time it's applied.
    pub parts: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Stats {
    Weapon(WeaponStats),
    Shield(ShieldStats),
    Augment(AugmentStats),
    Consumable(ConsumableStats),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModSlot {
    pub slot: String,
    pub tier_max: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WeaponStats {
    pub damage: Option<f64>,
    pub fire_rate: Option<f64>,
    pub range: Option<f64>,
    pub stability: Option<f64>,
    pub agility: Option<f64>,
    pub stealth: Option<f64>,
    pub magazine_size: Option<u32>,
    pub firing_mode: Option<String>,
    pub arc_armor_penetration: Option<String>,
    pub mod_slots: Vec<ModSlot>,
    pub measured: MeasuredStats,
}

/// Data wanted (plan §10): stays null until community-measured numbers exist.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MeasuredStats {
    pub rounds_per_minute: Option<f64>,
    pub bullet_velocity_ms: Option<f64>,
    pub falloff_start_m: Option<f64>,
    pub falloff_end_m: Option<f64>,
    pub falloff_min_mult: Option<f64>,
    pub headshot_mult: Option<f64>,
    pub reload_s: Option<f64>,
    pub ads_s: Option<f64>,
    pub recoil_pattern_ref: Option<String>,
    pub measured_provenance: Option<Provenance>,
    pub measured_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShieldStats {
    pub shield_hp: Option<f64>,
    pub charge: Option<f64>,
    #[serde(rename = "type")]
    pub shield_type: Option<String>,
    pub movement_penalty: Option<f64>,
    /// Published by MetaForge, absent from the plan's block.
    pub damage_mitigation: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AugmentStats {
    pub backpack_slots: Option<u32>,
    pub weight_limit_kg: Option<f64>,
    pub safe_pocket_slots: Option<u32>,
    pub quick_use_slots: Option<u32>,
    pub shield_compatibility: Option<String>,
    pub trinket_slots: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConsumableStats {
    pub healing: Option<f64>,
    pub healing_per_second: Option<f64>,
    pub use_time: Option<f64>,
    pub duration: Option<f64>,
    pub radius: Option<f64>,
    pub arc_stun: Option<f64>,
    pub raider_stun: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ItemQty {
    pub item_id: String,
    pub qty: u32,
}

/// `salvage` is added to the plan's list: the wiki separates it from `recycle`
/// and the yields differ (Rattler I: 8 metal parts recycled, 4 salvaged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RecipeKind {
    Craft,
    Recycle,
    Salvage,
    Trade,
    ProjectStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Station {
    pub workbench_id: String,
    pub level: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub kind: RecipeKind,
    /// Craft and trade. Recycle and salvage use `outputs`.
    pub output: Option<ItemQty>,
    pub outputs: Vec<ItemQty>,
    pub inputs: Vec<ItemQty>,
    pub workbench_id: Option<String>,
    pub workbench_level: Option<u8>,
    /// Other stations that can make this ("Workbench 1, Gunsmith 1" on the wiki).
    pub alt_stations: Vec<Station>,
    pub requires_blueprint: bool,
    pub blueprint_item_id: Option<String>,
    pub required_skill: Option<String>,
    pub craft_time_s: Option<f64>,
    pub source_refs: SourceRefs,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLevel {
    pub level: u8,
    pub upgrade_inputs: Vec<ItemQty>,
    pub unlocks_recipe_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Workbench {
    pub id: String,
    pub name: String,
    pub max_level: u8,
    pub levels: Vec<WorkbenchLevel>,
}

/// Closed list shared by mods and the weapon calculators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModStat {
    AdsSpeed,
    BaseDispersion,
    BulletVelocity,
    DispersionRecovery,
    DurabilityBurnRate,
    EquipTime,
    FalloffDistance,
    FireRate,
    HorizontalRecoil,
    MaxShotDispersion,
    Noise,
    PerShotDispersion,
    RecoilRecovery,
    UnequipTime,
    VerticalRecoil,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Modifier {
    pub stat: ModStat,
    /// Signed: "15% Reduced" is -15, "15% Increased" is 15.
    pub pct: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FlatModifiers {
    pub magazine_size: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WeaponMod {
    pub id: String,
    pub name: String,
    pub family: String,
    pub tier: u8,
    pub slot: String,
    pub compatible_ammo: Vec<String>,
    /// Weapon family slugs from the wiki; weapons are tiered items, mods fit the family.
    pub compatible_weapons: Vec<String>,
    pub modifiers: Vec<Modifier>,
    pub flat: FlatModifiers,
    pub recipe_id: Option<String>,
    pub gunsmith_level: Option<u8>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MapImage {
    /// Tile template on MetaForge's CDN, fetched and cached at runtime, never committed.
    pub url: Option<String>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub tile_size: Option<u32>,
    pub max_native_zoom: Option<u8>,
    /// [lat, lng, lat, lng] corners of the map in marker coordinates...
    pub world_extent: Option<[f64; 4]>,
    /// ...and the same corners in zoom-0 tile pixels [y, x, y, x].
    pub tile_extent: Option<[f64; 4]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GameMap {
    pub id: String,
    pub name: String,
    pub image: MapImage,
    pub bounds: Option<Bounds>,
    pub poi_ids: Vec<String>,
    pub event_pool: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PoiKind {
    Building,
    Landmark,
    Extract,
    ArcSpawn,
    Cache,
    QuestTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Poi {
    pub id: String,
    pub map_id: String,
    pub name: String,
    pub kind: PoiKind,
    pub x: f64,
    pub y: f64,
    pub loot_areas: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LootArea {
    pub id: String,
    pub name: String,
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuestAction {
    Interact,
    VisitPoi,
    Collect,
    Kill,
    Extract,
    Deliver,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestPrereq {
    pub quest_id: String,
    pub state: String,
}

/// `sequence` nodes group `children`; `atomic` nodes are one objective. The API
/// publishes objectives as plain strings today, so `action` is null until tagged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub id: String,
    pub kind: String,
    pub children: Vec<String>,
    pub round_constraint: Option<String>,
    pub action: Option<QuestAction>,
    pub amount: Option<u32>,
    pub description: Option<String>,
    pub maps: Vec<String>,
    pub poi_ids: Vec<String>,
    pub item_ids: Vec<String>,
    pub maps_provenance: Option<Provenance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub trader: Option<String>,
    pub cadence: String,
    pub prerequisites: Vec<QuestPrereq>,
    pub objective_root: String,
    pub objectives: Vec<Objective>,
    /// Items handed in; the plan's "quest reqs" live here rather than on objectives.
    pub required_items: Vec<ItemQty>,
    pub rewards: Vec<ItemQty>,
    pub xp: Option<u32>,
    pub graph_position: Option<GraphPosition>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectStage {
    pub n: u32,
    pub title: String,
    pub inputs: Vec<ItemQty>,
    pub rewards: Vec<ItemQty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub stages: Vec<ProjectStage>,
    pub deadline_days: Option<u32>,
    pub expedition: bool,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillBranch {
    pub id: String,
    pub color: String,
    pub points_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub branch: String,
    pub kind: String,
    pub description: String,
    /// Player-tested values from the wiki; "may not be completely accurate".
    pub details: Option<String>,
    /// Null where the wiki leaves it blank; planners cap those at 5 and warn.
    pub max_rank: Option<u32>,
    pub per_rank: Vec<String>,
    pub cost_per_rank: u32,
    pub prereq_ids: Vec<String>,
    pub branch_points_gate: u32,
    pub choice_group: Option<String>,
    pub x: f64,
    pub y: f64,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillPoints {
    pub from_levels: u32,
    pub max_level: u32,
    pub expedition_bonus_max: u32,
}

/// Exclusive picks. The Nth pick needs `gates[N]` points in the branch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChoiceGroup {
    pub id: String,
    pub branch: String,
    pub max_picks: u32,
    pub gates: Vec<u32>,
    /// One diamond per pick: the game shows slots, not the options sharing them.
    pub slots: Vec<[f64; 2]>,
}

/// A drawn connection. `from` and `to` are skill or choice group ids; a group wired to
/// itself is the stem joining its slots. Prerequisites are derived from these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillWire {
    pub from: String,
    pub to: String,
    pub points: Vec<[f64; 2]>,
}

/// Nodes come from the wiki. Wiring and in-game positions are hand-entered in
/// skills_overlay.json; the two flags say whether that has happened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillTree {
    pub tree_version: String,
    pub branches: Vec<SkillBranch>,
    pub choice_groups: Vec<ChoiceGroup>,
    pub wires: Vec<SkillWire>,
    pub nodes: Vec<SkillNode>,
    pub points: SkillPoints,
    pub prereqs_entered: bool,
    pub layout_in_game: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArmorZone {
    pub zone: String,
    pub hp: Option<f64>,
    pub weak_point: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArcDrop {
    pub item_id: String,
    pub chance: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arc {
    pub id: String,
    pub name: String,
    pub class: Option<String>,
    pub hp: Option<f64>,
    pub icon: Option<String>,
    pub armor_zones: Vec<ArmorZone>,
    pub drops: Vec<ArcDrop>,
    pub source_refs: SourceRefs,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TraderStock {
    pub item_id: String,
    pub price: u32,
    pub currency: Option<String>,
    pub daily_limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trader {
    pub id: String,
    pub name: String,
    pub inventory: Vec<TraderStock>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatasetVersion {
    pub dataset_version: String,
    pub game_patch: Option<String>,
    pub ingested_from: BTreeMap<String, String>,
}

/// What one ingest changed in one file. Names, capped so a big patch stays readable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileChange {
    pub file: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

/// One row of data/changelog.json, generated by `arc-ingest sync --write` (Phase 6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangelogEntry {
    pub dataset_version: String,
    pub game_patch: Option<String>,
    pub summary: Vec<String>,
    pub files: Vec<FileChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillPlan {
    pub name: String,
    pub tree_version: String,
    #[serde(default)]
    pub expedition_bonus: u32,
    pub ranks: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WeaponBuild {
    pub name: String,
    pub item_id: String,
    pub mod_ids: Vec<String>,
}

/// An entry here means the quest is being tracked.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestProgress {
    pub objectives_done: Vec<String>,
    pub done: bool,
}

/// An entry here means the project is being tracked. `stage` is the 1-based stage in
/// progress; one past the last means complete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectProgress {
    pub stage: u32,
    pub committed: BTreeMap<String, u32>,
}

/// User-local state (plan §4.11). Never written into the repo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, default)]
pub struct Profile {
    pub schema_version: u32,
    pub dataset_version: Option<String>,
    pub skill_tree_version: Option<String>,
    pub blueprints_found: Vec<String>,
    pub quests: BTreeMap<String, QuestProgress>,
    pub projects: BTreeMap<String, ProjectProgress>,
    pub workbenches: BTreeMap<String, u8>,
    pub stash: BTreeMap<String, u32>,
    pub skill_plans: Vec<SkillPlan>,
    pub weapon_builds: Vec<WeaponBuild>,
    pub tracked_item_ids: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            dataset_version: None,
            skill_tree_version: None,
            blueprints_found: Vec::new(),
            quests: BTreeMap::new(),
            projects: BTreeMap::new(),
            workbenches: BTreeMap::new(),
            stash: BTreeMap::new(),
            skill_plans: Vec::new(),
            weapon_builds: Vec::new(),
            tracked_item_ids: Vec::new(),
        }
    }
}
