# ARC Raiders: maps and raid flow research (patch 2.4.1, researched 2026-11-06)

Format: tip | confidence | sources (URL + date)

Source key (used below to keep lines short):
- [WIKI:<Page>] = https://arcraiders.wiki/wiki/<Page>, wikitext fetched via API 2026-11-06 (page revision dates not exposed).
- [MF-MARK] = MetaForge interactive-map markers, local snapshot C:\LocalRepositories\ArcDatabase\dataset\fixtures\metaforge_markers\2026-11-06.json (pulled from metaforge.app 2026-11-06). Independent of the wiki.
- [MF-ITEM] = MetaForge item/quest data, local snapshot fixtures\metaforge\2026-11-06.json (2026-11-06).
- [RT] = RaidTheory/arctracker data, local snapshot fixtures\raidtheory\2026-11-06.json (2026-11-06).
- Steam official notes (all at https://steamstore-a.akamaihd.net/news/externalpost/steam_community_announcements/<id>):
  - [N2.4.1] id 1848664643341170 (2026-11-05)
  - [N2.3.0] id 1847286114048305 (2026-10-27)
  - [N2.2.0] id 1846651706694342 (2026-10-20)
  - [N2.1.0] id 1846018067943414 (2026-10-13)
  - [NPvE] PvE Toggle Beta Test, id 1846018067943915 (2026-10-13)
  - [NFT] Frozen Trail, id 1846018067928872 (2026-10-08)
  - [N1.42] id 1841579228669036 (2026-08-21)
  - [N1.36] id 1836506165577581 (2026-07-07)
  - [N1.33] id 1835236783573536 (2026-06-16)
  - [N1.26] id 1830797770244665 (2026-04-28)
  - [N1.22] id 1828441623111636 (2026-03-31)
  (Steam API only returns notes back to 2026-03; older patch claims come via the wiki's history tables.)

Current maps (WIKI:Maps, 2026-11-06): Dam Battlegrounds, Buried City, Spaceport, The Blue Gate, Riven Tides, Stella Montis, Pendola Pass, plus Practice Range (no loot, 2-hour sandbox). Riven Tides (added 1.26.0, 2026-04-28) was missing from the requested list.

---

## Dam Battlegrounds

- Starter map: no unlock requirement, 30-minute rounds | CONFIRMED | WIKI:Maps; https://allthings.how/arc-raiders-maps-unlocks-timers-events-and-best-loot-spots (2025-11-02)
- Extracts are 4 cargo elevators/lifts: North Complex Elevator, Water Treatment Elevator, Central Swamp Lift, Red Lakes Balcony Lift | LIKELY (MetaForge names; elevator type also stated by allthings) | [MF-MARK] (2026-11-06); https://allthings.how/arc-raiders-extraction-elevators-hatches-and-timers-explained/ (2025-11-07)
- 4 Raider Hatches: Spillway, Pump House, Sunroof, Good Old Baron's | LIKELY | [MF-MARK] (2026-11-06)
- Key rooms: Control Tower (Epic key), Testing Annex (2 rooms), Controlled Access Zone, Staff Room (Research & Administration bldg), Surveillance (Water Treatment Control) | CONFIRMED | WIKI:Keys; [MF-MARK] (2026-11-06)
- Power Rod (not a key item) opens a door on the east side of the Power Generation Complex | CONFIRMED | WIKI:Keys; [MF-MARK] "Power rod room" (2026-11-06)
- Controlled Access Zone (added 1.17.0): insert a Fuel Cell to power the 4th switch, press all 4 switches near-simultaneously (3 players + Snap Hook works), then feed a panel random items to open the 2nd door | LIKELY (wiki only for steps) | WIKI:Dam_Battlegrounds (2026-11-06)
- Best general loot cited: Research & Administration, Control Tower, Hydroponic Dome Complex; Control Tower is heavily contested | LIKELY (guides from 2025, possibly stale) | allthings.how maps guide (2025-11-02); playnews.gg search snippet (2026, tied to 1.26.0)
- Has Field Depots (6 marked) | CONFIRMED | [MF-MARK]; WIKI:Field_Depot (2026-11-06)
- Beginner-friendly ARC: Comets removed from standard Dam sessions and Fireflies heavily reduced (1.26.0) | CONFIRMED (official) | [N1.26] (2026-04-28)
- ARC Turbines removed from standard Dam sessions and most minor conditions; in the remaining conditions they now almost always spawn | CONFIRMED (official) | [N2.2.0] (2026-10-20)
- Conditions listed for Dam: Lush Blooms, Uncovered Caches, Harvester, Husk Graveyard, Matriarch, Prospecting Probes, Night Raid, Electromagnetic Storm, Cold Snap, Hurricane, Close Scrutiny, ARC Frigate | LIKELY (wiki infobox only; seasonal ones may be out of rotation) | WIKI:Dam_Battlegrounds (2026-11-06)
- Queen, Matriarch and Harvester each have one fixed spawn marker on Dam | LIKELY | [MF-MARK] (2026-11-06); WIKI:Harvester ("fixed location on each map")

## Buried City

- 30-minute rounds; unlock count is inconsistent: 6 rounds (wiki map page, allthings 2025) vs 8 rounds (wiki Maps table) | DISPUTED | WIKI:Buried_City; WIKI:Maps; allthings.how maps guide (2025-11-02)
- Extracts are 4 metro stations: Northern, Southern, Eastern, Western Station | CONFIRMED (metro type in 3 sources; names MetaForge) | [MF-MARK] (2026-11-06); https://www.dexerto.com/wikis/arc-raiders/how-to-extract/ (2025-11-05); allthings extraction guide (2025-11-07)
- 4 Raider Hatches: Highway Overpass, Old Town, Train Station, plus one unnamed | LIKELY | [MF-MARK] (2026-11-06)
- Key rooms: Hospital (3rd-floor lock per allthings), Town Hall (Epic, best key room per AltChar), JKV Employee Access (J Kozma Ventures building), Residential Master Key apartments | CONFIRMED (room list); LIKELY (floor/quality claims) | WIKI:Keys; [MF-MARK]; https://www.altchar.com/guides/arc-raiders-all-key-room-locations-on-buried-city-agzjA1o69Vv2 (date not shown)
- Keyless breach rooms exist (e.g., Pharmacy) | LIKELY | [MF-MARK] (2026-11-06)
- Tight streets and rooftops: expect close fights and flanks from stairwells/roofs | LIKELY | allthings.how maps guide (2025-11-02)
- Bird City (exclusive minor condition): loot chimneys for trinkets/ducks, extra ziplines, many more flying ARC; rare chimney loot slightly lowered in 1.17.0 | LIKELY | WIKI:Bird_City (2026-11-06)
- Comet presence reduced (1.26.0) | CONFIRMED (official) | [N1.26] (2026-04-28)
- ARC Turbines now spawn in almost every Buried City session | CONFIRMED (official) | [N2.2.0] (2026-10-20)
- Has Field Depots (3 marked) | LIKELY | [MF-MARK] (2026-11-06)

## Spaceport

- Unlock after 12 rounds | CONFIRMED | WIKI:Maps; allthings maps guide (2025-11-02)
- Round length: 30 min normally, 40 min during Hidden Bunker (wiki); allthings (Nov 2025) said 40 min base | DISPUTED/STALE (older guide likely pre-change) | WIKI:Maps; WIKI:Hidden_Bunker; allthings maps guide (2025-11-02)
- Extracts are 4 cargo elevators (Central, East, South, West) | CONFIRMED (type in 2 sources) | [MF-MARK]; allthings extraction guide (2025-11-07)
- 4 Raider Hatches: Control, Departures, Trench East, Trench West | CONFIRMED (count also in playnews snippet) | [MF-MARK] (2026-11-06); playnews.gg 2026 POI guide (search snippet)
- Key rooms: Control Tower, Container Storage, Trench Towers (north/south), Shipping Warehouse | CONFIRMED | WIKI:Keys; [MF-MARK] (2026-11-06)
- High-value POIs: Arrival/Departure Buildings, Launch Towers, Control Tower, Container Storage; heavier ARC presence | LIKELY (Nov 2025) | allthings maps guide (2025-11-02)
- Hidden Bunker (exclusive major): activate 4 rooftop antennas (each twice; Wasps/Hornets spawn after first), progress shared by the lobby; bunker opens west/southwest of the Departure Building; 8 data terminals, stay close while downloading; 2 Bastions + 1 Bombardier guard entrances; fewer extracts, no hatches | CONFIRMED (antennas + bunker in 2 sources); LIKELY (guard/terminal counts) | WIKI:Hidden_Bunker (2026-11-06); allthings/deltias search summaries; https://arcraiders.com/en/news/new-map-conditions (2025-11-07)
- Launch Tower Loot (exclusive minor): zipline doors at tower base are locked; power a generator with a fuel cell (found by tunnel entrances or the orb gas tanks) to reach loot at the top | LIKELY | WIKI:Launch_Tower_Loot (2026-11-06)
- Has Field Depots (5 marked) | LIKELY | [MF-MARK] (2026-11-06)
- ARC Turbines now spawn in almost every Spaceport session | CONFIRMED (official) | [N2.2.0] (2026-10-20)

## The Blue Gate

- Unlock after 18 rounds; 30 min, 40 min during Locked Gate | CONFIRMED (unlock); LIKELY (40 min, wiki only) | WIKI:Maps; WIKI:Locked_Gate; allthings maps guide (2025-11-02)
- Extracts are 4 airshafts: Cliffside, Forest, Overlook, Warehouse | CONFIRMED (Cliffside/Overlook also named by guide) | [MF-MARK] (2026-11-06); allthings extraction guide (2025-11-07); Blue Gate guide search summary
- 4 Raider Hatches: Fragrant, Lucky, Prefab, Reinforced | LIKELY | [MF-MARK] (2026-11-06)
- Key rooms: Village, Cellar (several cellar doors), Communication Tower, Confiscation Room (Epic, in the tunnels); Patrol Car Key opens patrol car trunks (also Riven Tides) | CONFIRMED | WIKI:Keys; [MF-MARK] (2026-11-06)
- Checkpoint (map centre) is the gateway to the underground tunnels; tunnels are the best spot for weapons/security loot/blueprints | LIKELY | Blue Gate guide search summaries (nerdschalk/deltias, dates not shown); WIKI:The_Blue_Gate
- Open valley terrain: avoid ridgeline silhouettes, use hard cover between POIs | LIKELY | allthings maps guide (2025-11-02)
- Locked Gate (exclusive major): collect 4 security codes (Raider's Refuge, Pilgrim's Peak, Reinforced Reception, Ancient Fort) from code printers, enter them in the Gate Control Room to open the Checkpoint gate; code sites are mined, Pilgrim's Peak has 2 Rocketeers, campers wait at Ancient Fort; fewer extracts, no hatches | CONFIRMED (codes + console + mines in 2+ sources) | WIKI:Locked_Gate (2026-11-06); Destructoid/Dexerto/GamesRadar Locked Gate guides (search summaries, dates not shown)
- Has no Field Depots marked on MetaForge | LIKELY (contradicts wiki's "nearly every map except Stella Montis") | [MF-MARK] (2026-11-06)

## Riven Tides

- 6th map, added 1.26.0 (2026-04-28); unlock after 20 rounds; 30 min; difficulty 5 | CONFIRMED | [N1.26]; WIKI:Riven_Tides; grindout/dving search summaries (2026)
- Only 2 cargo elevators (Customs, Seaside) plus 3 Raider Hatches (Hilltop, Saltswept, Weighbridge): fewer exits, so plan a fallback | LIKELY (MetaForge; "Customs" extract also in player reports) | [MF-MARK] (2026-11-06); Riven Tides search summary
- Key rooms: 7 Panorama Azzurro hotel suites (Keycards 102/107/113/205/208/311/404), Classified Records and Secure Storage in the Port Authority Building (Secure Storage is inside another key room), middle Crane House | CONFIRMED | WIKI:Keys; [MF-MARK]; [MF-ITEM] (2026-11-06)
- Stacking Yard puzzle: press 4 hidden buttons (room 1), repair the lift with battery/wires and carry the battery back (room 2), which opens a further building | LIKELY (wiki only) | WIKI:Riven_Tides (2026-11-06)
- Beachcombing (exclusive minor): use a Dockmaster's Detector on the north-side sand; finds include Weapon Cases, Suitcases, Ticks and landmines (stand still and hold interact to disarm) | LIKELY | WIKI:Beachcombing; [N1.26] (condition announced)
- ~15% overall loot value increase in Frozen Trail 2.0 | LIKELY (wiki history only) | WIKI:Riven_Tides (2026-11-06)
- ARC Turbines now spawn in almost every Riven Tides session | CONFIRMED (official) | [N2.2.0] (2026-10-20)

## Stella Montis

- Unlock after 24 rounds; 30 min; two levels (upper/lower) | CONFIRMED | WIKI:Maps; WIKI:Stella_Montis; https://www.gamebrief.net/blog/arc-raiders-stella-montis-guide-2026 (2026-10-29, wiki-based)
- Extracts: Lobby and Loading Bay metro stations plus the Seed Vault Airshaft; 3 hatches (Assembly, Sandbox, Eastern Tunnel) | CONFIRMED (MetaForge + gamebrief) | [MF-MARK] (2026-11-06); gamebrief (2026-10-29)
- Hotfix 2.4.1 fixed entering the Seed Vault extract while it was closed: do not rely on it after it shuts | CONFIRMED (official) | [N2.4.1] (2026-11-05)
- Key rooms: Archives (Epic), Assembly Admin, Medical Storage (Medical Research), Security Checkpoint; several fuel-cell doors (Seed Vault, Cafeteria, Atrium/Lobby) | CONFIRMED (keys); LIKELY (fuel-cell doors) | WIKI:Keys; [MF-MARK] (2026-11-06)
- No Field Depots on Stella Montis | CONFIRMED | WIKI:Field_Depot; [MF-MARK] (2026-11-06)
- Shredders patrol indoors (added to all maps in 1.22.0), notably between Seed Vault and the airshaft | LIKELY | [N1.22] (2026-03-31); Last Entry guide search summaries (2026)
- Few extracts on a compact map: extract camping is common | LIKELY | Stella Montis search summaries (2026)
- Conditions: Night Raid (added 1.4.0); wiki lists no others | LIKELY | WIKI:Stella_Montis (2026-11-06)

## Pendola Pass

- Added in Frozen Trail 2.0 (2026-10-08); unlock after 25 rounds; difficulty 3 | CONFIRMED | [NFT]; WIKI:Maps; https://allthings.how/arc-raiders-how-to-extract-from-pendola-pass-ballistic-gondola/ (2026-10-12)
- No elevators or Raider Hatches: you leave on the Ballistic Gondola, called from one of 4 transceivers (Glacial, Northern, Reckless, Supermarket) | CONFIRMED | allthings gondola guide (2026-10-12); [MF-MARK] (2026-11-06); WIKI:Ballistic_Gondola
- Calling the gondola is loud and ARC react to transceiver use | CONFIRMED (official) | [N2.2.0] (2026-10-20); allthings gondola guide (2026-10-12)
- Gondola markers are visible on compass/map out to 175 m (was 120 m) | CONFIRMED (official) | [N2.1.0] (2026-10-13)
- Gondola lands ~100-150 m from the transceiver; 3 one-person pods; breach a red panel to board; a 10 s launch timer starts once someone boards; waits ~2 min if unused; you can ride on top or grapple to it | LIKELY (one guide for timings; boarding method also on wiki) | allthings gondola guide (2026-10-12); WIKI:Ballistic_Gondola
- Flash Freeze: built-in cold hazard (harsher Cold Snap), can hit any time with a short warning; get under cover; some shelters only open as a freeze approaches; ice can seal doors (melt/break with Flame Spray or Raider Tool); Redirection disables it | CONFIRMED (core); LIKELY (details) | [NFT]; WIKI:Pendola_Pass; https://powerupgaming.co.uk/2026/10/12/arc-raiders-flash-freeze-survival-guide-pendola-pass/ (2026-10-12); WIKI:Redirection
- Shelters are contested: expect fights at them | LIKELY | powerupgaming (2026-10-12); allthings search summary
- Storm Huts are always present and hold Outpost materials | LIKELY | powerupgaming (2026-10-12); [N2.3.0] mentions Storm Hut containers
- Fallen Emperor: opened with the Emperor Gateway Conduit (Legendary); since 2.3.0 the best loot sits deeper inside, entrance containers are low-tier; watch the compactor kill zones | CONFIRMED (official loot change) | [N2.3.0] (2026-10-27); WIKI:Pendola_Pass
- Key rooms: Old Town, Railyard, Train Station, Observatory (Epic) | CONFIRMED | WIKI:Pendola_Pass; [MF-MARK] (2026-11-06)
- Dynamic Loot Zones: an area's loot value varies each raid (loot type stays the same); Pendola only for now | LIKELY | https://www.gamebrief.net/blog/arc-raiders-pendola-pass-guide-2026 (2026-10-28)
- Redirection (exclusive minor): loot Emperor Beacons from relays, deploy them outdoors to call down payloads (couplers, Emperor Gateway Conduit blueprint); 2.3.0 lowered the conduit blueprint chance and added ARC parts | CONFIRMED (official change) | [N2.3.0] (2026-10-27); gamebrief Pendola guide (2026-10-28); WIKI:Redirection
- New ARC here: Bully, Skulker; Hydra turrets in the Emperor and on the Frigate; Bullies can wander into gondola landing zones | LIKELY | [NFT] (2026-10-08); gamebrief (2026-10-28); allthings gondola guide (2026-10-12)
- MetaForge marks no Field Depots on Pendola | LIKELY | [MF-MARK] (2026-11-06)

## Map conditions (cross-map)

- Two tiers: minor conditions (loot-focused events) and major conditions (change the whole raid); some majors are "ARC Operations" (Close Scrutiny, ARC Frigate) | CONFIRMED | WIKI:Maps (2026-11-06); [N1.22] (Close Scrutiny = ARC Operation)
- Schedules are regional (offset to each region's peak hours) since 1.42.0 | CONFIRMED (official) | [N1.42] (2026-08-21)
- Night Raid: more loot value (especially locked rooms; keys more common), more ARC, fewer extracts, no Raider Hatches, no Free Loadout | CONFIRMED | WIKI:Night_Raid; https://allthings.how/arc-raiders-night-raids-rules-timers-maps-and-rewards/ (2025-11-10); [N1.33] (2026-06-16, no free loadout)
- Electromagnetic Storm: telegraphed lightning (blue glow on ground) damages and stuns raiders and ARC; small ARC can die, big ARC get stunned, flyers drop; fewer extracts, no hatches; Fossilized Lightning can spawn at strikes | CONFIRMED (lightning hitting both); LIKELY (details) | WIKI:Electromagnetic_Storm; https://arcraiders.com/en/news/new-map-conditions (2025-11-07); deltias/allthings search summaries
- Harvester (Dam, Spaceport, Blue Gate): Queen-guarded structure; get 3 fusion cores from breachable upper containers (one on the floor), insert all 3 quickly, shoot 12 yellow targets; reward 1-3 Equalizer or Jupiter blueprints; flee fire discharge through the opening vents | CONFIRMED | WIKI:Harvester; deltias/nerdschalk/arcraidershub Harvester guides (search summaries, 2025)
- Matriarch (Dam, Spaceport, Blue Gate): spawns the Matriarch, only found in this condition | LIKELY | WIKI:Matriarch_(map_condition)
- Prospecting Probes: many more ARC Probes, often guarded by Wasps | LIKELY | WIKI:Prospecting_Probes
- Husk Graveyard: many more Wasp/Rocketeer/Courier husks; some shock you when breached | LIKELY | WIKI:Husk_Graveyard
- Lush Blooms: more nature loot and Wicker Baskets | LIKELY | WIKI:Lush_Blooms
- Uncovered Caches: many more Raider Caches with better loot, but they explode if left exposed too long; listen for whirring + ticking | CONFIRMED | WIKI:Uncovered_Caches; allthings maps guide (2025-11-02)
- Hurricane (added 1.17.0): wind slows/speeds you and pushes throwables, debris chews shields, more ARC, First Wave Caches better than Raider Caches | LIKELY | WIKI:Hurricane
- Cold Snap: damaging cold outdoors, Candleberries, better Raider Caches (need breaching, no flag); seasonal, removed and re-added before | DISPUTED/STALE (seasonal, check whether it is currently in rotation) | WIKI:Cold_Snap; WIKI:Raider_Cache
- Close Scrutiny: Surveyors call in breachable Assessors; locked doors are open, overall loot lower, ARC loot higher; no Free Loadout; first pylon breach spawns a flying ARC swarm (smoke/Lure/Cloak help) | CONFIRMED (no free loadout official) | [N1.33]; WIKI:Close_Scrutiny
- ARC Frigate (since 2.0, 6 maps): board the hovering Frigate, destroy power cores, release the cargo (ride the cargo down to get off); locked rooms off, loot value lower, no hatches, 35-minute rounds, fewer surface ARC | LIKELY (wiki detail; listed effects echoed by gamebrief) | WIKI:ARC_Frigate; gamebrief Pendola guide (2026-10-28); [NFT]
- Frigate cargo rides: crush damage only at the landing point (2.2.0); Frigate hatches burn when they open (2.1.0) | CONFIRMED (official) | [N2.2.0] (2026-10-20); [N2.1.0] (2026-10-13)
- Since 2.3.0 the Dolabra, Bobcat, Equalizer, Jupiter, Tempest and Vulcano blueprints can drop in any major condition | CONFIRMED (official) | [N2.3.0] (2026-10-27)
- PvE toggle test excluded Night Raid, Locked Gate, Hidden Bunker, Frigate and Redirection, with lower loot in PvE | CONFIRMED (official, beta Oct 13-20) / STALE if PvE launches differently | [NPvE] (2026-10-13)

## General raid

- Rounds last 30 minutes on most maps (exceptions: Hidden Bunker/Locked Gate 40, Frigate 35) | CONFIRMED (30 min); LIKELY (exceptions) | WIKI:Maps; allthings extraction guide (2025-11-07); WIKI:ARC_Frigate
- When the round timer ends, the ARC Orbiter wipes everyone still topside and you lose your gear | CONFIRMED | WIKI:ARC_Orbiter; dexerto how-to-extract (2025-11-05); https://www.pcgamer.com/games/third-person-shooter/arc-raiders-punishes-players-who-fail-to-extract-in-time-by-simply-dropping-a-meteor-on-their-heads-but-there-is-one-way-to-survive-the-blast (date not retrieved)
- Shani warns at 10, 5 and 2 minutes remaining | LIKELY (wiki only) | WIKI:ARC_Orbiter
- Each elevator/metro/airshaft has its own shutdown countdown on the map; once it hits zero that exit is gone for the raid, so the nearest one may already be closed | CONFIRMED | WIKI:Extraction_Points; allthings extraction guide (2025-11-07); dexerto (2025-11-05)
- Interacting with a terminal just before it shuts keeps that extract alive into overtime | LIKELY (wiki only; "grace period" claims elsewhere are unverified) | WIKI:Extraction_Points; studio-asv extraction-timer guide (undated)
- Calling an elevator sets off a loud alarm that draws ARC and raiders; top up health/shields and scout before calling | CONFIRMED | allthings extraction guide (2025-11-07); dexerto (2025-11-05)
- Downed raiders can still use extraction terminals or crawl into the lift | CONFIRMED | WIKI:Extraction_Points; allthings extraction guide (2025-11-07)
- Raider Hatches: silent, no shutdown timer, need a single-use Raider Hatch Key; the hatch stays open only briefly (wiki: about 10-15 s) and anyone nearby can follow you in | CONFIRMED | WIKI:Raider_Hatch; allthings extraction guide (2025-11-07); dexerto (2025-11-05)
- Hatches are off in Night Raid, Electromagnetic Storm, Hidden Bunker, Locked Gate and ARC Frigate; Pendola Pass has none | CONFIRMED (Night Raid, Pendola); LIKELY (others) | WIKI condition pages; allthings Night Raid (2025-11-10); allthings gondola guide (2026-10-12)
- Raider Hatch Key sources: loot, downed raiders, crafting (wiki: 1 Advanced Electrical Components + 3 Sensors), the Hatch Repairs quest, Shani. Shani's terms disagree: wiki says 9,000 coins, 2 per day; Dexerto says level 12, once per 24 h | DISPUTED (Shani terms) | WIKI:Raider_Hatch_Key; dexerto (2025-11-05); allthings extraction guide (2025-11-07)
- Keep a hatch key (and other must-keep items) in your Safe Pocket so you don't lose it on death | CONFIRMED | allthings extraction guide (2025-11-07); WIKI:Loadout
- Late joins: custom-loadout players are biased into fresh servers (1.22.0); Embark says late joiners profit economically from the leftovers of early fights, though key rooms may already be looted | CONFIRMED | [N1.22] (2026-03-31); https://www.gamesradar.com/games/third-person-shooter/that-really-sucks-arc-raiders-lead-responds-to-complaints-about-late-spawns-says-late-joiners-are-important-and-profit-way-more-but-embark-knows-its-annoying-to-have-no-time-for-objectives (date not retrieved); PC Gamer late-spawn articles
- No source shows rarer loot appearing later in a raid; treat "rarer loot late" as unconfirmed | DISPUTED | (absence across all sources checked)
- Locked rooms are the best loot: 1.22.0 raised locked-room loot value scaled by key rarity; 1.26.0 cut outskirts weapon spawns and lowered outdoor weapon durability (average 50 -> 30) vs ~60 in locked rooms | CONFIRMED (official) | [N1.22] (2026-03-31); [N1.26] (2026-04-28)
- Field Depots: carry a nearby Field Crate into the depot's supply station for a reward plus a loot compartment; Field Crates spawn near depots; you are exposed while carrying it, so clear the area or bring a team | CONFIRMED | WIKI:Field_Depot; WIKI:Down_To_Earth; https://allthings.how/arc-raiders-field-crates-and-depots-how-delivery-and-drops-work (date not shown); PC Gamer field crate guide (date not shown)
- Security Lockers need the Security Breach skill; they hold weapons/gun parts, rarely Epic weapons; spawn dynamically since 1.7.0; skill branch varies by source (Conditioning vs Survival) after the 2.0 tree rework | CONFIRMED (skill needed); DISPUTED (branch) | WIKI:Security_Locker; WIKI:Raider_Tool; dataset data\skills.json; Field Depot/locker search summaries
- Raider Caches: ammo, blueprints, grenades, meds, shields, weapons; usually beside a red arrow flag; more and better during Uncovered Caches | CONFIRMED | WIKI:Raider_Cache; WIKI:Uncovered_Caches; allthings maps guide (2025-11-02)
- Prospecting Probes condition = more Probes to breach (guarded); crates from malfunctioning probes are the Field Crates Shani wants delivered | LIKELY | WIKI:Prospecting_Probes; WIKI:Down_To_Earth
- Free Loadout gives no Safe Pocket; going in empty-handed still gets 10 slots + 1 Safe Pocket | LIKELY | WIKI:Loadout; gamebrief Stella Montis guide (2026-10-29, wiki-based)
- Simple loop: spawn, hit one or two main POIs, grab a safer nearby spot, extract before the timers and third parties stack up | LIKELY (advice) | allthings maps guide (2025-11-02); allthings extraction guide (2025-11-07)

---

## Key-door locations (requested)

- Dam Utility Key: no confirmed door. MetaForge's description says "Location is currently unknown except that it is on Dam Battlegrounds" (updated 2026-06-29), and its article claims a Flickering Threat quest reward, but the Flickering Threat rewards on both the wiki and MetaForge's own quest data don't include it. The wiki has no page for it, and RaidTheory doesn't list it. The repo's DATA_AUDIT.md "opens closets and small utility rooms in admin buildings" note comes from an unnamed community key list, and I found nothing to back it up. | DISPUTED (possibly unused/datamined) | [MF-ITEM] (2026-11-06); WIKI:Flickering_Threat (2026-11-06); WebSearch for "Dam Utility Key" (2026-11-06)
- Flushing Terminal Key (Spaceport): a quest item for The Stench Of Corruption (Shani, added 1.7.0), not a door key. Get it in the Staff Locker Room by the southwest lobby of the Departure Building (MetaForge says a clothing rack; MetaForge has a marker for "Departure Building Staff Locker Room"), then use it on any Flushing Terminal in the tunnels under Spaceport. No source gives terminal coordinates; the wiki quest page only has screenshots. | CONFIRMED (where to get it and roughly where to use it); exact terminal position unknown | WIKI:The_Stench_Of_Corruption; WIKI:Flushing_Terminal_Key; [MF-ITEM]; [MF-MARK] (2026-11-06). Note: DATA_AUDIT.md calls it "Celeste's quest"; the wiki says the quest giver is Shani.
- Riven Tides Crane Secure Storage Keycard: almost certainly not a separate item. The wiki page titled "Riven Tides Secure Storage Keycard" has "Riven Tides Crane Secure Storage Keycard" in its infobox name field, so it looks like a wiki naming slip. MetaForge and RaidTheory list only "Riven Tides Secure Storage Keycard" (Epic). It opens the Secure Storage room in the Port Authority Building, which is inside another key room (MetaForge marker; the Classified Records room markers are right next to it). The separate Crane House Keycard (Uncommon) opens the middle Crane House. Recommendation: merge the "Crane Secure Storage" entry into the Secure Storage Keycard. | LIKELY | WIKI:Riven_Tides_Secure_Storage_Keycard; WIKI:Riven_Tides_Crane_House_Keycard; WIKI:Keys; [MF-ITEM]; [RT]; [MF-MARK] (all 2026-11-06)
