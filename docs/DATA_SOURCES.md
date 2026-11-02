# Data sources

Probe of 2026-11-02 (UTC). Reproduce with `cargo run -p arc-ingest -- probe`.

## Recipe source decision

**Wiki Cargo is the recipe source.** RaidTheory fills gaps; MetaForge has no recipes.

| Source | Craft outputs | Recycle inputs | Last updated |
|---|---|---|---|
| Wiki Cargo (`crafting`, `recycling`) | 291 | 386 | live |
| RaidTheory | 156 | 359 | 2026-08-19 |
| MetaForge | 0 (176 items have only a free-text `workbench` string) | 0 | live |

Of the 155 recipes both sources have, 151 match exactly. The 4 that differ (Anvil I, Snap Hook, Stable Stock I, Zipline) are resolved in the wiki's favour because RaidTheory stopped updating in August; none was checked in game. 285 of the 291 wiki outputs match a MetaForge name directly. The other 6 are `User:` sandbox rows and naming variants the resolver handles.

## Access

| Source | Endpoint | Notes |
|---|---|---|
| MetaForge items, quests, arcs | `https://metaforge.app/api/arc-raiders/{items,quests,arcs}?page=N&limit=100` | 1,512 items (507 cosmetic), 123 quests, 26 arcs |
| MetaForge traders, events | `/traders`, `/events-schedule?region=` | Trader prices carry no currency. `region` accepts `europe`, `north-america` and `oceania`; other values fall back to Europe |
| MetaForge map tiles | `static.metaforge.app/arc-raiders/maps/<map>/<version>/{z}/{x}/{y}.webp` | Read from its map page config on 2026-11-02 into `crates/arc-ingest/maps_overlay.json`. Fetched and cached at runtime, never committed. A re-render changes the path, so a 404 means re-reading the config |
| MetaForge map markers | `https://metaforge.app/api/game-map-data?mapID=dam&tableID=arc_map_data` | `tableID` was found by probing and is undocumented. 43,039 markers across 7 maps; fixtures keep the 2,161 with loot areas or named locations |
| Wiki Cargo | `https://arcraiders.wiki/w/api.php?action=cargoquery` | Tables: augments, crafting, items, projectItems, projects, questItems, quests, recycling, trials, weaponMods, weeklyTrials, workshopUpgrades. Rows on `User:` pages are ignored |
| RaidTheory | Git tree plus raw JSON at a pinned commit | The repo is ~228 MB, mostly images, so only JSON is fetched and only English strings are kept |
| Steam news | `ISteamNews/GetNewsForApp/v2/?appid=1808500` | Only `steam_community_announcements` count as patch notes; the feed also carries press items. The newest versioned title sets `game_patch` |

## Skill tree 2.0

- **Nodes:** names, descriptions, ranks and choice groups come from the wiki's Skills article (CC BY-SA 4.0), saved in `tests/fixtures/wiki_skills/`. The count is 48, 16 per branch. RaidTheory's `skillNodes.json` is the removed pre-2.0 tree and is not used.
- **Gates:** first choice at 10 branch points (one of two); end choices at 30 and 40 (two of three). Wiki, allthings.how and Neonsect agree.
- **Max ranks:** the wiki leaves six Mobility ranks blank. Effortless Roll is 5 per allthings.how. The other five come from in-game tooltips: Slip And Slide, Off The Wall, Marathon Runner and Swift Start show V, and Right Back At It shows no numeral, so it's a single point. Together they match the 21 points the reference screenshot's Mobility total needs.
- **Wiring and layout:** hand-entered from an in-game screenshot of tree 2.0 into `crates/arc-ingest/skills_overlay.json`, in the screenshot's 2000-wide coordinates. Nodes were named by matching in-game icons to the wiki's per-skill icons, which are identical. `real_tree_reproduces_the_screenshot_build` checks the screenshot's 41/10/9 build and its locks.
- **Icons:** the wiki's per-skill files, referenced by URL and fetched at runtime like other art.
- **Do not use the wiki's `Widget:SkillTree` SVG.** Its header reads "Can not be used for other purposes without given permission." Ingest does not fetch it. Using it would need permission from its author, wiki user Dread.

## Plan assumptions corrected by the probe

- MetaForge quests are not all empty on location: 8 of 123 have coordinates and a map. With wiki locations, 70 of 123 quests now have a map.
- MetaForge quest objectives are plain strings, not a graph, so `objectives[].action` stays null until tagged.
- Weapons are separate items per tier (`rattler-i` to `rattler-iv`), not one `rattler` item.
- Trader prices are not all coins. Currency and daily limits come from RaidTheory, so they are as of August.
- `game-map-data` carries markers only; map images are separate tile pyramids (see Access).
- Spaceport, Blue Gate and Stella Montis have two floors each, as separate tile paths. A marker's `zlayers` is a bitmask of the floors it's on: bit 1 is the lower floor and bit 2 the upper, and 2147483647 means every floor.

## Field precedence

Declared in `crates/arc-ingest/precedence.toml`. Unresolved references are listed in `data/coverage.json` rather than dropped silently.
