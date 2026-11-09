# Research, 2026-11-09: ARC Operations and boss events

Gathered for the ARC Operations guide section and the route planner. Claims were checked against
arcraiders.wiki (page revisions in the JSON), Embark's patch notes and news on arcraiders.com, the
official tracker at arcraiders.com/map-conditions, and dated guides. Nothing has been checked in
game. Patch at the time: 2.4.1.

| File | What |
|---|---|
| [arc_operations.json](arc_operations.json) | Definition, one entry per operation or equivalent, sources and confidence per entry |

## Findings

- **"ARC Operation" is an official term.** Flashpoint (1.22.0, 2026-03-31) introduced it as a new
  type of major map condition. They are scheduled conditions on the hourly, regional rotation.
  They are not in-raid triggers and you don't queue for them separately.
- **There are two as of 2.4.1:** Close Scrutiny (ARC Assessor, 1.22.0) and the ARC Frigate (2.0,
  2026-10-08). Both cut ambient loot, open or disable locked rooms and ban free loadouts, so the
  whole lobby heads for one objective.
- **Harvester and Matriarch are not operations.** The tracker lists them as minor conditions,
  each adding one fixed boss encounter to a normal raid. The Queen has no condition of her own;
  she only guards the Harvester. Redirection (Pendola Pass) is minor too, but it changes routing,
  so it's included.
- **Night Raid, Cold Snap, Electromagnetic Storm and Hurricane** are map-wide modifiers (weather,
  darkness, loot value) with no single objective.
- **Routing:**
  - The Frigate arrives 1 to 2 minutes into a 35-minute raid (5 minutes longer than usual). Raider
    Hatches are off and surface ARC is mostly gone.
  - Close Scrutiny Assessors are marked by red beams visible from anywhere on the map. Each pylon
    lasts about 5 minutes from touchdown.
  - The Harvester sits at a fixed spot on each of its three maps.
  - Redirection beacons let you choose where a payload lands.
- **Patch history** is in each entry: 1.2.0, 1.13.0, 1.22.0, 1.33.0, 2.0, 2.1.0, 2.2.0 and 2.3.0.
  2.4.0 and 2.4.1 changed nothing here. Since 2.3.0, the Dolabra, Bobcat, Equalizer, Jupiter,
  Tempest and Vulcano blueprints can drop in any major condition.

## Conflicts

- **Free loadouts:** 1.33.0 announced a three-week test of no free loadouts in Close Scrutiny and
  Night Raid. The wiki still shows the ban and the Frigate launched with it, but I found no note
  making it permanent.
- **Frigate Power Cores:** the wiki gives 18 in one place and 20 in another.
- **Leaving the Frigate:** the wiki and Beebom say to ride the Cargo down (made safe in 2.2.0).
  KeenGamer, writing on launch day, says to take the Reclaimer back down.
- **Players needed on the Frigate:** three for the levers (KeenGamer) vs two or more for some
  buttons (wiki).
- **Harvester maps:** the wiki infobox lists 3 maps, but its intro says "each map".
- **Harvester blueprints:** the wiki says 1 to 3 are guaranteed (Equalizer or Jupiter). Guides
  say the Equalizer is rare.
- **Cloak name:** the wiki says Photoelectric Cloak, arcraidershub says Prophetic Cloak.
- **Frigate classification:** Embark's 2.0 notes call the Frigate just a "map condition". It
  carries the ARC Operation label only on the wiki and the tracker.
- **Wiki patch dates:** the wiki's patch-list dates for 2.x are day-month swapped (for example
  2.1.0 shows as 2026-13-09). The official date is 2026-10-13.

## Gaps

- **Frequency:** none of the sources says how often a condition comes up; the tracker only shows
  each one's next slot.
- **Gear:** no minimum gear tier anywhere. Every gear field is a guide recommendation, and
  augments are rarely named. The Matriarch and Queen gear guides date from November 2025,
  before the 2.0 explosive rework.
- **Close Scrutiny:** the number of Assessors per raid and where they land aren't documented.
- **Frigate:** the flight path and Cargo drop points aren't documented.
- **Harvester:** no patch says when it was added. A guide from 2025-11-06 already covers it.
- **Unreadable sources:** MetaForge returned 403 on every page and the fandom wiki returned 402,
  so neither was used.

## Next

Check in game: the free loadout ban, the Frigate core count and exit, the Harvester blueprint
count, and the cloak's name.
