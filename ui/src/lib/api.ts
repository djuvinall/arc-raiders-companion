import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export type Rarity = "common" | "uncommon" | "rare" | "epic" | "legendary";

export interface ItemSummary {
  id: string;
  name: string;
  category: string;
  subcategory: string | null;
  rarity: Rarity | null;
  tier: number | null;
  icon: string | null;
  loot_areas: string[];
  variant_of: string | null;
  value: number | null;
  weight_kg: number | null;
  family: string | null;
  family_name: string | null;
}

export interface WeaponVariant {
  id: string;
  amplifications: string[];
}

export interface WeaponFamily {
  name: string;
  levels: { id: string; level: number }[];
  amplified: WeaponVariant[];
}

export interface Link {
  kind: string;
  id: string;
  name: string;
  icon: string | null;
  qty: number | null;
  note: string | null;
}

export interface RecipeView {
  id: string;
  kind: string;
  inputs: Link[];
  outputs: Link[];
  stations: Link[];
  blueprint: Link | null;
  requires_blueprint: boolean;
  required_skill: string | null;
  provenance: string;
}

export interface FoundIn {
  area: Link;
  maps: { map: Link; poi_count: number; places: string[] }[];
}

export interface UsedIn {
  recipes: { recipe_id: string; output_item_id: string | null; output_name: string | null; output_icon: string | null; qty: number }[];
  projects: { project_id: string; project_name: string; stage: number; qty: number }[];
  quests: { quest_id: string; quest_name: string; qty: number }[];
  workbenches: { workbench_id: string; workbench_name: string; level: number; qty: number }[];
}

export interface Item extends ItemSummary {
  description: string;
  flavor_text: string;
  value: number | null;
  weight_kg: number | null;
  stack_size: number | null;
  stats: ({ kind: string } & Record<string, unknown>) | null;
  ammo_type: string | null;
  stencil: { swatch: string; weapons: string[]; source: string; parts: number } | null;
  blueprint_required: boolean;
  provenance: string;
}

export interface ItemDetail {
  item: Item;
  crafting: RecipeView[];
  breakdown: RecipeView[];
  used_in: UsedIn;
  found_in: FoundIn[];
  sold_by: Link[];
  dropped_by: Link[];
  base: Link | null;
  family: WeaponFamily | null;
  stencil_guns: Link[];
  gun_stencils: { stencil: Link; swatch: string }[];
  quest_rewards: Link[];
  project_rewards: Link[];
}

export interface TreeNode {
  item_id: string;
  name: string;
  qty: number;
  recipe_id: string | null;
  workbench_id: string | null;
  workbench_level: number | null;
  cycle: boolean;
  owned: boolean;
  children: TreeNode[];
}

export interface CraftPlan {
  tree: TreeNode;
  totals: Link[];
  root: Link;
}

export interface EntityPage {
  kind: string;
  id: string;
  name: string;
  subtitle: string;
  sections: { title: string; text: string[]; links: Link[] }[];
}

export interface Verdict {
  action: "keep" | "recycle" | "sell";
  headline: string;
  reasons: string[];
  sell_value: number | null;
  recycle_value: number | null;
  gives: Link[];
}

export interface Goal {
  kind: "craft" | "quest" | "project" | "bench";
  id: string;
  name: string;
  have: number;
  need: number;
  missing: Link[];
}

export interface Home {
  ready: Goal[];
  almost: Goal[];
  quests_active: number;
  quests_done: number;
  quests_total: number;
  items_needed: number;
  blueprints_found: number;
  blueprints_total: number;
  bench_levels: number;
  bench_levels_total: number;
}

export interface ArcView {
  arc: {
    id: string;
    name: string;
    hp: number | null;
    icon: string | null;
    armor: string | null;
    attack: string | null;
    ability: string | null;
    weak_points: string[];
    tips: string | null;
  };
  drops: Link[];
}

export interface BenchView {
  id: string;
  name: string;
  level: number;
  max_level: number;
  tracked: boolean;
  next: { item: Link; need: number; in_stash: number }[];
  requires: string | null;
}

export interface Overview {
  dataset_version: string;
  game_patch: string | null;
  item_count: number;
  recipe_count: number;
  loot_areas: { id: string; name: string }[];
}

export interface SkillNode {
  id: string;
  name: string;
  branch: string;
  kind: "rank" | "toggle" | "choice" | "keystone";
  description: string;
  details: string | null;
  max_rank: number | null;
  per_rank: string[];
  prereq_ids: string[];
  branch_points_gate: number;
  choice_group: string | null;
  x: number;
  y: number;
  icon: string | null;
}

export interface SkillGroup {
  id: string;
  branch: string;
  max_picks: number;
  gates: number[];
  slots: [number, number][];
}

export interface SkillTree {
  tree_version: string;
  branches: { id: string; color: string; points_label: string }[];
  choice_groups: SkillGroup[];
  wires: { from: string; to: string; points: [number, number][] }[];
  nodes: SkillNode[];
  points: { from_levels: number; max_level: number; expedition_bonus_max: number };
  prereqs_entered: boolean;
  layout_in_game: boolean;
}

export interface SkillPlan {
  name: string;
  tree_version: string;
  expedition_bonus: number;
  ranks: Record<string, number>;
}

export interface PlanState {
  report: {
    ok: boolean;
    spent: number;
    budget: number;
    branch_points: Record<string, number>;
    errors: string[];
    warnings: string[];
  };
  addable: string[];
  removable: string[];
}

export interface WeaponBuild {
  name: string;
  item_id: string;
  mod_ids: string[];
}

export interface Profile {
  stash: Record<string, number>;
  skill_plans: SkillPlan[];
  weapon_builds: WeaponBuild[];
  tracked_item_ids: string[];
  blueprints_found: string[];
  player_name: string | null;
  friends: { name: string; done: string[]; active: string[] }[];
  workbenches: Record<string, number>;
  tracked_from: Record<string, string>;
}

export interface PartyQuest {
  quest: Link;
  maps: string[];
  depth: number;
  active: boolean;
}

export interface PartyPlan {
  friend: string;
  together: PartyQuest[];
  you_catch_up: PartyQuest[];
  friend_catch_up: PartyQuest[];
  maps: [string, number][];
}

export interface WeaponMod {
  id: string;
  name: string;
  tier: number;
  slot: string;
  modifiers: { stat: string; pct: number }[];
  flat: { magazine_size: number };
}

export interface SlotOptions {
  slot: string;
  tier_max: number;
  mods: WeaponMod[];
}

export interface StatRow {
  key: string;
  label: string;
  base: number | null;
  value: number | null;
}

export interface Sheet {
  weapon: Link;
  firing_mode: string | null;
  arc_armor_penetration: string | null;
  ammo_type: string | null;
  rows: StatRow[];
  upgrades: { label: string; pct: number }[];
  effects: { stat: string; pct: number }[];
  measured: StatRow[];
  ttk: { target_hp: number; shots: number; seconds: number } | null;
  warnings: string[];
}

export interface CompareTable {
  columns: string[];
  rows: { label: string; values: (number | null)[]; best: number | null }[];
}

export interface GameMap {
  id: string;
  name: string;
  image: {
    url: string | null;
    tile_size: number | null;
    max_native_zoom: number | null;
    world_extent: [number, number, number, number] | null;
    tile_extent: [number, number, number, number] | null;
    layers: { name: string; url: string; mask: number }[];
  };
}

export interface MarkerCategory {
  id: string;
  name: string;
  group: "locations" | "quests" | "containers" | "arc" | "events" | "nature";
}

/** [category index, x, y, floors bitmask, label]. */
export type MapMarker = [number, number, number, number | null, string | null];

export interface MarkerSet {
  categories: MarkerCategory[];
  markers: MapMarker[];
}

export interface Poi {
  id: string;
  map_id: string;
  name: string;
  kind: string;
  x: number;
  y: number;
  loot_areas: string[];
  zlayers: number | null;
  quest_id: string | null;
}

export interface LiveEvent {
  name: string;
  map: string;
  icon: string | null;
  startTime: number;
  endTime: number;
}

export interface MapEvents {
  map_id: string;
  map_name: string;
  current: LiveEvent[];
  next: LiveEvent | null;
}

export interface EventsView {
  maps: MapEvents[];
  region: string;
  fetched_ms: number | null;
  stale: boolean;
}

export interface MapCover {
  map_id: string;
  map_name: string;
  pois: { id: string; name: string; x: number; y: number; covers: string[] }[];
  covered_items: string[];
  uncovered: string[];
}

export interface RoutePlan {
  needed: { item: Link; qty: number; have: number; reasons: string[] }[];
  covers: MapCover[];
}

export interface Objective {
  id: string;
  kind: string;
  description: string | null;
  maps: string[];
}

export interface QuestRow {
  quest: {
    id: string;
    name: string;
    trader: string | null;
    objectives: Objective[];
    prerequisites: { quest_id: string }[];
    xp: number | null;
    graph_position: { x: number; y: number } | null;
  };
  status: "done" | "active" | "needed" | "available" | "locked";
  needed_for: Link[];
  open: boolean;
  prerequisites: Link[];
  required: Link[];
  rewards: Link[];
  maps: string[];
  done_objectives: string[];
}

export interface StageView {
  n: number;
  title: string;
  inputs: { item: Link; need: number; committed: number; in_stash: number }[];
  rewards: Link[];
}

export interface ProjectView {
  id: string;
  name: string;
  expedition: boolean;
  deadline_days: number | null;
  starts: string | null;
  ends: string | null;
  permanent: boolean;
  paused: boolean;
  tracked: boolean;
  stage: number;
  stages: StageView[];
}

export interface Collection {
  id: string;
  title: string;
  found: number;
  total: number;
  entries: { id: string; name: string; icon: string | null; rarity: Rarity | null; found: boolean }[];
}

export interface PatchNote {
  gid: string;
  title: string;
  url: string;
  date: number;
  version: string | null;
  blocks: { kind: "heading" | "item" | "text"; text: string }[];
}

export interface ChangelogEntry {
  dataset_version: string;
  game_patch: string | null;
  summary: string[];
  files: { file: string; added: string[]; removed: string[]; changed: string[] }[];
}

export const overview = () => invoke<Overview>("overview");
export const listItems = () => invoke<ItemSummary[]>("list_items");
export const getItem = (id: string) => invoke<ItemDetail>("get_item", { id });
export const craftPlan = (id: string, qty: number, owned: string | null) =>
  invoke<CraftPlan>("craft_plan", { id, qty, owned });
export const setTrackedFrom = (id: string, owned: string | null) => invoke<void>("set_tracked_from", { id, owned });
export const itemVerdict = (id: string) => invoke<Verdict | null>("item_verdict", { id });
export const homeView = () => invoke<Home>("home_view");
export const setFound = (add: string[], remove: string[]) => invoke<void>("set_found", { add, remove });
export const arcList = () => invoke<ArcView[]>("arc_list");
export const workbenchList = () => invoke<BenchView[]>("workbench_list");
export const setWorkbenchLevel = (id: string, level: number) => invoke<void>("set_workbench_level", { id, level });
export const toggleTrackedBench = (id: string) => invoke<void>("toggle_tracked_bench", { id });
export const completeQuestChain = (questId: string) => invoke<void>("complete_quest_chain", { questId });
export const openMini = () => invoke<void>("open_mini");
export const getEntity = (kind: string, id: string) => invoke<EntityPage>("get_entity", { kind, id });
export const getProfile = () => invoke<Profile>("get_profile");
export const setStash = (itemId: string, qty: number) => invoke<void>("set_stash", { itemId, qty });

export const skillTree = () => invoke<SkillTree>("skill_tree");
export const skillPlanState = (plan: SkillPlan) => invoke<PlanState>("skill_plan_state", { plan });
export const saveSkillPlan = (plan: SkillPlan) => invoke<void>("save_skill_plan", { plan });
export const deleteSkillPlan = (name: string) => invoke<void>("delete_skill_plan", { name });
export const encodeSkillPlan = (plan: SkillPlan) => invoke<string>("encode_skill_plan", { plan });
export const decodeSkillPlan = (code: string) => invoke<SkillPlan>("decode_skill_plan", { code });

export const weaponSlots = (id: string) => invoke<SlotOptions[]>("weapon_slots", { id });
export const weaponSheet = (build: WeaponBuild, targetHp: number) =>
  invoke<Sheet>("weapon_sheet", { build, targetHp });
export const weaponCompare = (builds: WeaponBuild[]) => invoke<CompareTable>("weapon_compare", { builds });
export const saveWeaponBuild = (build: WeaponBuild) => invoke<void>("save_weapon_build", { build });
export const deleteWeaponBuild = (name: string) => invoke<void>("delete_weapon_build", { name });

export const mapList = () => invoke<GameMap[]>("map_list");
export const mapPois = (id: string) => invoke<Poi[]>("map_pois", { id });
export const mapMarkers = (id: string) => invoke<MarkerSet>("map_markers", { id });
export const liveEvents = (region: string) => invoke<EventsView>("live_events", { region });
export const routePlan = () => invoke<RoutePlan>("route_plan");
export const toggleTrackedItem = (id: string) => invoke<boolean>("toggle_tracked_item", { id });

export const questList = () => invoke<QuestRow[]>("quest_list");
export const setQuestObjective = (questId: string, objectiveId: string, done: boolean) =>
  invoke<void>("set_quest_objective", { questId, objectiveId, done });
export const setQuestState = (questId: string, tracked: boolean, done: boolean) =>
  invoke<void>("set_quest_state", { questId, tracked, done });

export const questShareCode = (name: string) => invoke<string>("quest_share_code", { name });
export const addFriend = (code: string) => invoke<string>("add_friend", { code });
export const removeFriend = (name: string) => invoke<void>("remove_friend", { name });
export const partyPlan = (name: string) => invoke<PartyPlan>("party_plan", { name });

export const projectList = () => invoke<ProjectView[]>("project_list");
export const commitResource = (projectId: string, itemId: string, qty: number) =>
  invoke<number>("commit_resource", { projectId, itemId, qty });
export const setProjectTracked = (projectId: string, tracked: boolean) =>
  invoke<void>("set_project_tracked", { projectId, tracked });

export const collectionList = () => invoke<Collection[]>("collection_list");
export const toggleFound = (id: string) => invoke<boolean>("toggle_found", { id });

export const patchNotes = () => invoke<{ notes: PatchNote[]; fetched_ms: number | null }>("patch_notes");
export const datasetChangelog = () => invoke<ChangelogEntry[]>("dataset_changelog");
export const exportProfile = () => invoke<string>("export_profile");
export const importProfile = (text: string) => invoke<void>("import_profile", { text });

export const appVersion = () => invoke<string>("app_version");
export const checkAppUpdate = () => invoke<string | null>("check_app_update");
export const installAppUpdate = () => invoke<void>("install_app_update");
export const checkDataUpdate = () => invoke<string | null>("check_data_update");
export const restartApp = () => invoke<void>("restart_app");

export const stencilShots = (stencilId: string) => invoke<Record<string, string>>("stencil_shots", { stencilId });
export const saveStencilShot = (stencilId: string, gunId: string, image: string) =>
  invoke<void>("save_stencil_shot", { stencilId, gunId, image });
export const deleteStencilShot = (stencilId: string, gunId: string) =>
  invoke<void>("delete_stencil_shot", { stencilId, gunId });

export const iconSrc = (url: string) => convertFileSrc(url, "arcicon");

export const linkHref = (link: Pick<Link, "kind" | "id">) =>
  link.kind === "item" ? `#/item/${link.id}` : `#/ref/${link.kind}/${link.id}`;

export const ROMAN = ["", "I", "II", "III", "IV"];
