# Plan: route planner and ARC Operations (v0.8, groundwork for v1)

Branch: `v0.8-routes`. Written 2026-11-09 from Devon's brief: plan and theorise the route
planner, account for ARC Operations and other raid-within-a-raid events, and add ARC Operations
to the app as guides (what to expect, minimum gear and the rest). Research is in
`docs/research/2026-11-08/` (containers, ARC drops, conditions) and `docs/research/2026-11-09/`
(operations).

## Where v0.7 leaves the planner

v0.7 plans **target areas**, not routes. It does this because you don't know your spawn.

- Every container, plant and ARC on a map gets a score for each wanted item. The score comes
  from `containers.json`, the loot zone the spot is in, rarity, and any live conditions.
- The best clusters become areas. Each area gets a short loop and its nearest exits.
- A **primary objective** pulls side areas toward it.
- **Suggest a map** ranks maps for the plan.

What v0.7 doesn't know about:
- **Time.** A raid lasts 30 minutes, or 35 with the Frigate. The planner has no idea how long a
  loop takes.
- **Events.** Some conditions are a raid inside the raid: a fixed or moving objective, a time
  window, and the whole lobby heading for it.
- **Kit.** Whether you're carrying enough to take an operation on.
- **Risk.** Where the other players will be.

## How a raid works, as the planner sees it

A raid is a **timeline on a map**:

1. **Drop in**, at a spawn you don't choose. Since 2.1.0, during the Frigate, spawns are moved
   closer to the action.
2. **Travel and loot.**
   - Travel time is distance divided by speed.
   - Loot time is the number of containers times the search time, plus extra time to breach.
3. **Objectives**:
   - Quest steps.
   - A primary objective.
   - An event. Some events have a window: an Assessor pylon is lost about 5 minutes after it
     lands, and the Frigate arrives 1 to 2 minutes in.
4. **Extract** at an exit that is open under today's conditions. For example, the Frigate
   turns off Raider Hatches.

The **risk** along that timeline depends on where the other players are. They gather at
operations, bosses, high-value zones and exits late in the raid.

## The pieces

### 1. ARC Operations data (`operations.json`, built in v0.8)

There is one entry per operation or boss event:
- Close Scrutiny and the ARC Frigate are ARC Operations.
- The Harvester (guarded by the Queen) and the Matriarch are boss conditions.
- Redirection is a payload event on Pendola Pass.

Each entry holds:

- **Identity:** name, kind (`operation`, `boss`, `event`), the matching name in
  `conditions.json` and the schedule, major or minor, the patch it came in, and maps.
- **Guide:** a summary, how it starts, map effects, step-by-step phases, enemies (as ARC ids),
  rewards (item ids, each guaranteed or possible), team size, what to expect (PvP and
  extraction), common mistakes, patch history, conflicts between sources, and in-game checks.
- **Gear:** lines of `minimum` or `recommended` kit. A line names a set of items, or a rule
  such as "a weapon with strong ARC armour penetration", plus a count. The app checks each
  line against your stash.
- **Route facts:**
  - The marker types where it happens (`events/harvester`, `events/assessor`,
    `events/frigate_cargo`, `events/emperor_payload`, `arc/matriarch`).
  - Whether the spot is fixed.
  - When it starts, in raid minutes.
  - How long it holds you.
  - Any window before the objective is lost.
  - Whether it pulls the whole lobby.
  - Whether Raider Hatches are off.
  - The raid length.
- **Provenance:** confidence and sources (URL, date retrieved, revision).

Everything is hand-curated in `overlays/operations_overlay.json`. Ingest checks it: every item
name, ARC and marker type must resolve, or the build fails.

Like `containers.json`, this is a new file. Older apps ignore it, and the app treats it as
optional, so v0.7 keeps taking data updates.

### 2. Time budget (v0.8)

The raid model gains a clock:

| Setting | Default | Source |
|---|---|---|
| Raid length | 30 min, Frigate 35 | wiki; checked per operation |
| Travel speed | 4.5 map units/s (jogging) | **estimate**: calibrate in game by timing a run between two markers |
| Search a container | 4 s | estimate |
| Breach | +8 s | estimate |
| Extraction call | 30 s at the exit | estimate |

Each area gets an **estimated time**: the walk to its loop and around it, plus the loot time.
The plan adds the areas up against the raid clock and marks the point where you should head
for an exit.

Speeds are settings, not facts. The planner says they're estimates until checked in game.

### 3. A route as phases (v0.8 builds the operation primary; the rest in v1)

A route is an ordered list of phases, not just areas:

1. **Approach.** From wherever you land, go to the nearest worthwhile area. You still don't
   know your spawn, so this stays a ranked list of areas.
2. **Primary.** A quest step, an ARC, a place or, new in v0.8, an **operation**. An operation
   primary sits at its marker type's spots. If the operation has a window (such as the Frigate
   arriving at minute 1 to 2), the plan says to go there first.
3. **Sides.** Areas ranked by what they give per minute, against the detour from the primary.
   This is v0.7's ranking, which v1 will convert to time.
4. **Extract.** The nearest exit open under the live conditions (no Raider Hatches during the
   Frigate), and when to leave by.

### 4. Heat: where the players go (v1, theory)

There is no player data, so heat is a model. Each spot gets a heat value:

- **Operation and boss sites** are hot for the length of the event. The Frigate and Close
  Scrutiny pull the whole lobby.
- **High-value zones** are warm. These are the places with many rare-loot containers.
- **Exits** warm up late in the raid.
- **Spawn clusters** are warm early on.

A plan setting picks **quiet** (rank cold areas up), **balanced** or **hot** (head for the
action). Areas show their heat. Once the raid log exists, the model can learn from your own
raids where you met players.

### 5. Readiness (v0.8)

Before a raid built around an operation, the plan checks your stash against the operation's
gear lines:

- **Minimum lines** you're missing are shown as "short".
- **Recommended lines** are shown as advice.

Team size is compared with your party (friends), and quests you share with friends on that map
are listed. The party plan already finds these.

### 6. Paths (v1, theory)

Distances are still straight lines. v1 can build a rough walk graph from what the map data has:

- POIs, roads, ziplines (`locations/zipline`), metro entrances and stations, gondola landings
  and mast lifts as nodes.
- Edges between nodes that are close and on the same floor.
- Ziplines and gondolas as fast edges.

That's good enough to avoid "through the cliff" routes on Pendola Pass and Stella Montis
without real navmesh data.

## v0.8 scope (this branch)

1. `operations.json`: overlay, ingest, schema, model and tests.
2. An **Operations** tab under ARCs:
   - A list showing which operations are live now or next, from the schedule.
   - A guide page per operation: summary, map effects, step by step, enemies, rewards, team
     size, gear (with stash ticks), what to expect, mistakes, patches, conflicts and sources.
3. Live events on Home and Maps link to the operation's guide.
4. Raid planner:
   - An operation can be the primary objective, placed at its nearest marker.
   - Its route facts apply: raid length, Raider Hatches off for exits, and a note about its
     window.
   - Gear readiness is shown on the plan.
   - A time estimate per area, with the raid clock.
5. Docs: in-game checks added to `DATA_AUDIT.md`.

## Later (v1)

- The heat model and the quiet / balanced / hot setting.
- A walk graph instead of straight lines.
- Phase timeline UI: a single strip from drop-in to extract, with event windows on it.
- Raid log, which also calibrates speeds and heat.
- A community operations feed, if Embark publishes Assessor or Cargo spots.

## Unknowns (checked in game, not guessed)

- **Travel speed in map units per second.** All the time estimates depend on it.
- **Free loadouts:** whether the ban during operations is permanent. 1.33.0 called it a
  three-week test.
- **Frigate:** the Power Core count (18 or 20), the exit (ride the Cargo or take the
  Reclaimer), and the players needed at the levers.
- **Harvester:** how many blueprints are guaranteed and how they split.
- **Close Scrutiny:** Assessors per raid and their landing spots.
- **The cloak's name:** Photoelectric Cloak, as our item data says. One guide says Prophetic.

## Done on this branch (2026-11-09)

- `operations.json`: five entries (Close Scrutiny, ARC Frigate, Harvester, Matriarch,
  Redirection). Ingest resolves every item, ARC, map, marker type and condition name, and
  fails on a typo. Weapon names expand to every tier and amplified variant. Tests are in
  `crates/arc-core/tests/operations.rs`.
- **Operations tab** under ARCs (`#/arcs/ops/<id>`). It shows what's live or next from the
  schedule, then each guide: maps with their marked spots, team, raid timing, map effects,
  steps, gear with stash ticks, enemies, rewards, what to expect, mistakes, unsettled points,
  patches and sources.
- Live events on Home and Maps link to their guide.
- **Raid planner:**
  - Operation spots (Frigate Cargo, Assessor landings, the Harvester, the Matriarch) can be
    picked as the primary objective.
  - A live or primary operation shows its gear readiness (what you're short of), team size,
    timing and notes.
  - It sets the raid length (35 minutes for the Frigate) and drops Raider Hatches from exits.
  - Exits are labelled when they're a Raider Hatch, and every area keeps at least one exit
    that isn't.
  - Each area shows a rough time in minutes.

## Before releasing v0.8

- Merge `v0.8-routes` in arc-raiders-data into its `main` *before* tagging the app. The app
  lists `operations.json` in its data files, so downloads stay on the shipped copy until the
  data repo has the file. Then re-run `sync --write` there, so the changelog collapses the
  branch's interim versions into one entry.
- Point the app's `dataset` submodule at that merge.
