# Architecture

- `crates/arc-core`: schema (`model`), loading and integrity (`dataset`), calculators (`calc`), UI read models (`views`). No Tauri and no network; HollowDeck will wrap this crate.
- `crates/arc-ingest`: CLI. Fetch to `tests/fixtures`, normalize, merge by `precedence.toml`, diff, write `data/`. Deterministic, no LLM (D6).
- `crates/arc-tauri`: commands over `arc-core`, `profile.json`, and the `arcicon:` cache protocol. It embeds `data/` into the exe.
- `ui`: Svelte 5, TypeScript, plain CSS, hash routes. Plain Vite rather than SvelteKit.

Runtime network calls all live in `arc-tauri`: `icons.rs` serves icons and map tiles from `static.metaforge.app` and skill icons from the wiki; `feeds.rs` fetches the event schedule (15 min TTL) and Steam news (6 h TTL). Every call is cached, stale data beats none, and patch notes also ship as a snapshot in the exe. The webview CSP blocks direct remote calls. `ARC_OFFLINE=1` forces the no-network path.

## Skill planner rules

`arc-core/src/skills.rs` validates a plan as a final state. Prerequisites are derived from the wires in `skills_overlay.json`. Large nodes are slots shared by options, as in the game: one slot of two options at 10 points, two slots of three options at 30 and 40. Options open from any one prior node, other nodes need all of theirs, and a skill after a large node opens from whichever option was picked. A choice pick's gate counts points in the branch excluding that pick and later picks, so the first end pick counts towards the second's 40. Share codes are `ARC1-` plus base64url of skill ids, so they survive layout changes.

The budget is `76 + expedition_bonus`, capped at 5 as the plan specifies. The wiki says up to 5 per completed Expedition, which may mean the real total can go higher.

## Weapon calculators

`arc-core/src/weapons.rs`. Only `fire_rate` and flat magazine size change a UI stat; every other mod stat has no UI counterpart and is listed as an effect. Percent modifiers on one stat multiply in sequence, which is a guess: the game's rule isn't published. Stat DPS is damage × fire rate stat. Time to kill and falloff stay hidden until `measured` values exist.

## Tracking and routing

An entry in `profile.quests` or `profile.projects` means it is tracked. `needed_items` combines tracked crafts (down to raw materials), tracked open quests and the current stage of tracked projects, minus the stash. `poi_cover` is greedy set cover per map over `poi.loot_areas → loot_area.item_ids`, ranked by items covered then fewest stops. It picks places, not a path.

Committing to a project takes the amount from the tracked stash, but doesn't require it: tracking the stash is optional.

## Additions to the plan's schema

The sources required these; each is commented in `model.rs`.

| Addition | Why |
|---|---|
| Categories `weapon-mod`, `recyclable`, `nature`, `research`, `quest-item`, `furniture`, `misc` | Real MetaForge types with no home in the plan's enum |
| Recipe kind `salvage` | Yields differ from recycling (Rattler I: 8 vs 4 metal parts) |
| `Recipe.alt_stations`, `blueprint_item_id`, `required_skill` | The wiki lists several stations and names the blueprint |
| `Item.variant_of` | Amplified entries fold under the top-tier base weapon; tier and rarity come from that base |
| `SoldBy.currency`, `TraderStock.currency` | Prices are not all coins |
| `Quest.required_items` | Requirements are quest-level in both sources, not per objective |
| `WeaponMod.compatible_weapons` | Sources list weapon families; `compatible_ammo` is derived from it |
| `ShieldStats.damage_mitigation` | Published by MetaForge |
| Provenance `community` | Marks RaidTheory rows |
| `SkillTree.choice_groups`, `prereqs_entered`, `layout_in_game` | Pick limits and stepped gates are per group; the flags drive the app's banner |
| `SkillNode.description`, `details`, nullable `max_rank` | The wiki publishes them; some ranks are blank |
| `ChoiceGroup.slots`, `SkillTree.wires` | The game draws shared slots and routed lines, not one node per option |
| `MapImage.tile_size`, `max_native_zoom`, `world_extent`, `tile_extent` | Maps are tile pyramids, not single images |
| `data/changelog.json` | Phase 6: written by `sync --write`, listing what each ingest changed |

Canonical ids are slugs of the display name, because MetaForge ids carry typos and duplicates (`burltetta-recipe`, `burletta-iiii`). MetaForge ids stay in `source_refs`.
