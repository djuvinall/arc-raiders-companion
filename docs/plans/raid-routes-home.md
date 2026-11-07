# Plan: raid planner, farming finder, Home revamp, Nomadic Envoy

Branch: `raid-routes-home`. Written 2026-11-07 from Devon's brief. Nothing here is built yet.

## Goals

1. **Raid owns routes.** The Maps screen stops planning; the Raid screen is where you plan a
   raid you have in mind, write it down and track it.
2. **Auto routes that respect spawn uncertainty**, with categories and single items you can
   switch off, and parameters you can adjust.
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
- **Player spawns:** 160 markers, so a route can start from the spawn you got.
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

**Auto route**

- **What it's for, as switches by category:**
  - quest objectives on this map;
  - tracked crafts;
  - tracked projects;
  - tracked workbench levels;
  - items you add by hand;
  - container types (for example "skip cars").

  Inside each category, every item and stop has its own switch. Switched-off things stay
  listed, dimmed, so you can switch them back.
- **Settings:** stops limit, confidence (chances per item), stay in one area vs spread out,
  start from spawn, end at the nearest exit, floors.
- **Building the route:** pick containers by best chance per metre, cluster them, order with
  nearest-neighbour plus 2-opt, add the exit. Quest objectives are fixed stops; loot fills
  around them.

**Custom route**

- Click any marker or any spot on the map to add a stop. Drag in the list to reorder, add a
  note to a stop, delete a stop.
- **Edit an auto route:** turn it into a custom one and keep changing it. The auto settings
  stay saved, so you can regenerate.

**Using it in a raid**

- **Start raid** marks the plan as in raid. The Compact Window then shows its stops as a
  checklist next to the quests.
- **Finish** asks what you came back with, which ticks quest steps and adds to the stash.
  This is a later hook for the screenshot import we discussed.

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
- Stored in the profile with the week it belongs to. It's marked out of date after the weekly
  reset, so you know to refresh it.
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

## Questions for Devon

1. **Envoy reset:** what day and time does his stock change (your time zone)?
2. **Spawns:** when you load in, do you know which spawn you got? If not, "start from spawn"
   is less useful and routes should start from the stop nearest the map edge you choose.
3. **Raid history:** should finished raids stay listed with notes (a raid log), or disappear?
4. **Containers:** are there types you never bother opening (for example cars)? Those make good
   defaults to switch off.
