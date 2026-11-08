# Plan: raid planner, farming finder, Home revamp, Nomadic Envoy

Branch: `raid-routes-home`. Written 2026-11-07 from Devon's brief, revised the same day with
Devon's answers (see the end). Nothing here is built yet.

## Goals

1. **Raid owns routes.** The Maps screen stops planning; the Raid screen is where you plan a
   raid you have in mind, write it down and track it.
2. **Auto plans that respect spawn uncertainty**: since you don't know where you'll load in,
   a plan picks **target areas** worth heading to, not a route from a fixed start. Categories
   and single items can be switched off, and the settings can be adjusted.
3. **Custom routes** you draw yourself, or start from an auto route and edit.
4. **Farm finder:** the best places on any map to farm one or more resources.
5. **Home shows the most important thing from each tab** and links straight into it.
6. **Nomadic Envoy's rotating stock** somewhere useful without a new tab.

## What the data can and can't do (checked today)

- **Containers:** the map has about 30,900 container markers across 7 maps (13,280 base
  containers, 5,018 lockers, 3,300 raider caches, 1,723 ARC probes, 1,349 couriers, 994 cars,
  754 ammo crates and more). Only 670 named spots carry the loot-type tags today's routes
  use. So today's routes see about 2% of the places loot can spawn.
- **No spawn rates exist anywhere.** No source says "a Medical bag holds Antiseptic 12% of the
  time". Any chance we show is a model, and the app must say so.
- **Loot types:** 13 loot areas (Mechanical, Electrical, Medical, Residential, Industrial, ARC,
  Old World, Exodus, Nature, Technological, Commercial and others) list which items they can
  drop. The in-game map legend shows the same types as zones (your Pendola Pass screenshot).
- **Player spawns:** 160 markers, but you don't know which one you got until you're in, so
  plans can't start from a spawn.
- **Extractions:** every map's exits are markers, so a route can end at the nearest one.
- **No walkable paths.** Distances are straight lines. Good for ordering stops and comparing
  areas; it won't route around cliffs. The app says so.
- **Nomadic Envoy:** no source publishes his weekly stock. Guide sites post dated snapshots
  only. It has to come from you (typed in, or read from a screenshot).

## 1. Spawn model (arc-core, the foundation for 2 and 3)

An item is never "at" one spot. It can turn up in any container whose loot pool includes it.
The model gives every container a score for every wanted item:

- **Container type:** what the type can hold. Ammo crates hold ammo, weapon cases weapons,
  med bags medical, ARC probes and couriers ARC parts, and plain containers most things. This
  needs a loot pool per container type, which no source we use has. Research step: the wiki's
  container pages and MetaForge's container data. Hand-entered gaps go in an overlay, as
  quests and doors do now.
- **Zone:** a container inside a Medical zone leans Medical. Needs the zone outlines.
  Research step: MetaForge's map data may carry them (the in-game legend draws them). Fallback:
  the 670 tagged spots as zone seeds, nearest-seed wins.
- **Rarity:** rarer items are less likely per container.

The output is a **relative chance per container per item**: "likely", "possible" or "unlikely",
never a promise. A route then aims for **enough chances**, not one stop per item. A
"confidence" setting asks for, for example, 3 likely containers per copy you need. More
containers mean better odds, and the screen shows the expected count.

Tests pin the model's ordering (a med bag beats a locker for Antiseptic) rather than exact
numbers.

## 2. Raid screen: planned raids

A **raid plan** is saved in the profile: name, map, start spawn (optional), stops in order,
notes, the settings that made it, and a status (planned, in raid, done). The Raid screen
lists your plans and opens one on its map.

**Auto plan: target areas, not a route from spawn**

You don't know your spawn, so the plan's core is a ranked list of **target areas** (tight
clusters of good containers and quest stops). Inside each area the stops get a short loop
order. Once you're in and know roughly where you are, you head for the nearest good area; the
plan shows each area's nearest exits.

- **What it's for, as switches by category:**
  - quest objectives on this map;
  - tracked crafts;
  - tracked projects;
  - tracked workbench levels;
  - items you add by hand;
  - container types (for example "skip cars").

  Inside each category, every item and stop has its own switch. Switched-off things stay
  listed, dimmed, so you can switch them back.
- **Settings:** how many areas, area radius, confidence (chances per item), floors.
- **Building the plan:** score every container for what you want, cluster them into areas,
  rank areas by combined chance per metre, order the stops inside each area with
  nearest-neighbour plus 2-opt, and list its nearest exits. Quest objectives are fixed stops
  and pull their area up the list.
- **Container types:** all on by default, since you open everything at some point. Each
  type's weight comes from the loot it can drop, so a medical run leans on med bags without
  you switching anything. You can still switch a type off for one plan.

**Custom route**

- Click any marker or any spot on the map to add a stop, or draw an area to target. Drag in the list to reorder, add a
  note to a stop, delete a stop.
- **Edit an auto route:** turn it into a custom one and keep changing it. The auto settings
  stay saved, so you can regenerate.

**Using it in a raid**

- **Start raid** marks the plan as in raid. The Compact Window then shows its stops as a
  checklist next to the quests.
- **Finish** asks what you came back with, which ticks quest steps and adds to the stash.
  This is a later hook for the screenshot import we discussed.
- **Raid log: not in v1.** A finished plan goes back to "planned" so you can reuse it. A log
  of finished raids with notes can come later, filled in automatically if the screenshot
  import lands.

**Maps screen changes**

- Remove the Needed gear panel and the route stops.
- Keep markers, filters, item finding, comparing maps and live events.
- Add **Plan a raid here**, which opens Raid on that map.

## 3. Farm finder

- Pick one or more items, from the Raid screen's **Farm** mode or from an item page.
- Every map is scanned for **hotspots**: the tightest clusters of containers with the best
  combined chance for those items, within a radius you set.
- The result is a list across all maps: map, area, how many chances, how compact. **Plan this**
  turns a hotspot into a raid plan.
- The item page's **Found in** gains a short "Best places to farm" line linking here.

## 4. Home revamp

One card per tab with its single most useful fact, each linking to the exact place:

| Card | Shows | Opens |
|---|---|---|
| Next raid | your planned or in-raid plan, stop count | Raid, on that plan |
| Live events | what's on now and next per map | Maps, on that map |
| Quests | active quests with their next step (from the quest tree) | Quests, on that quest |
| Projects | current stage and what's still missing | Projects |
| Workbenches | benches ready to upgrade, or the closest | Profile, Workbenches |
| Still needed | top items left to find, and where | the item's map find view |
| Nomadic Envoy | this week's stock, with useful items flagged | the item pages |
| Skills | the starred plan's points | Skills |
| Patches | newest patch and what changed in the data | Patches |

Today's Ready / Almost there / Right now lists fold into these cards. Cards with nothing to
say hide, so a new player sees a short page.

## 5. Nomadic Envoy (recommendation: a Home card, no new tab)

- No feed exists, so the week's stock is **entered by you**:
  - **Read it from a screenshot** of his shop, using the same reader as the Codex import (best).
  - **Or type it in:** pick items and their trade prices.
- Stored in the profile with the date you entered it. The reset time isn't known yet, so the
  card says how old the entry is ("entered 3 days ago") rather than guessing when it expires.
  Once the reset day is confirmed in game, the card can mark it out of date automatically.
- The card flags what you need (tracked, quests, projects, benches) using the existing
  keep/sell advice.
- Later, optional: a community `envoy.json` in arc-raiders-data, so one person's entry reaches
  everyone. That needs someone to update it weekly, so not in the first pass.

## Order of work

1. Spawn model in arc-core: research the container loot pools and zone outlines, add the
   overlay, write the model and its tests. Everything else depends on this.
2. Raid plans: model, saving, auto route, custom route editing, Compact Window checklist.
   Remove routing from Maps.
3. Farm finder, using the same model.
4. Home revamp.
5. Nomadic Envoy card and screenshot reading.

Each step ships on this branch with tests and a headless screenshot check, and is merged to
`main` for a release when it's whole.

## Devon's answers (2026-11-07)

1. **Envoy reset:** not known yet. The card shows how old the entry is; find the reset in
   game when convenient.
2. **Spawns:** you don't know which spawn you got, so plans target areas rather than routes
   from a spawn (section 2).
3. **Raid log:** nice to have, not v1.
4. **Containers:** everything gets opened at some point, depending on what you're after. No
   type is off by default; the loot pools weight them per target.

## Done on this branch already

- **Item card polish.** Names and tags in the game's plain heavy font, larger description,
  mod slot tiles for weapons (greyed icons of the lowest mod that fits), the ammo icon on
  Ammo Type, and no tier tag or weapon flavor line, matching the in-game tooltip.
- **Spawn model and raid plans** (`arc-core/src/raid.rs`). Every container, plant and ARC on
  a map is scored per wanted item from a container-type table (`POOLS`), its zone (loot-area
  tags of spots within 200 units) and rarity. Plans rank target areas by how much of what you
  want has enough chances, then by depth; each area has a short loop (nearest-neighbour plus
  2-opt, ending near an exit) and its nearest exits. Plans are saved in the profile with
  switches per category, source, item and container type, settings, your own stops, notes,
  and in-raid ticks.
- **Raid page** rebuilt around plans: auto areas or your own stops (click the map or copy an
  area), and a **Farm** tab that ranks areas across every map for the items you pick. Item
  pages link to it.
- **Maps** no longer plans routes; it has "Plan a raid here" and lists that map's plans.
- **Home** is one card per tab: raid, live events, quests with their next step, projects,
  ready now, still needed, workbenches, skills, Codex and patches.
- **Compact Window** shows the in-raid plan's stops as a checklist.

Not done yet: the Nomadic Envoy card (on hold), a raid log, and tuning `POOLS` against the game.

- **Project stages played as missions.** Soul's Offering stage 4 ("Honor The Fallen") has
  objectives instead of materials, so it showed an empty page. Stages can now carry
  objectives, nested like quests, from a new `project_tasks.json` (its own file, so apps from
  before it keep taking data updates). Ticking the last step finishes the stage.
