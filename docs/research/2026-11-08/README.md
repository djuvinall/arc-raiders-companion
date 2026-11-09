# Research, 2026-11-08: containers, ARC drops, map conditions

Done to give the raid planner real data instead of guesses, and to check a Gemini summary of
loot in ARC Raiders. Every claim was checked against the wiki (with page revisions), the
official map-conditions tracker and patch notes, and MetaForge. Nothing here has been checked in
game yet. Patch at the time: 2.4.1.

| File | What | Became |
|---|---|---|
| [arc_drops.json](arc_drops.json) | What each ARC, husk, probe, courier, Assessor and Harvester drops, per source | `dataset/overlays/arc_loot.json` |
| [containers.json](containers.json) | 82 wiki containers, and how MetaForge's map markers map onto them | `dataset/overlays/containers_overlay.json` |
| [maps.json](maps.json) | 18 map conditions, per-map notes, rarity rules | `dataset/overlays/conditions_overlay.json` |

## What changed in the data

- **ARC drops** come from each ARC's own wiki page, plus MetaForge where it adds items, instead
  of RaidTheory's lists (frozen since August). The Hydra, Queen, Bully and Turbine had none
  before. The Turbine Compressor, for one, drops from the ARC Turbine; it isn't in any husk.
- **Husks, probes and couriers** use their exact drop lists. They used to count as "anything in
  the ARC loot area", which put every ARC part in every husk.
- **Containers** are data now (`containers.json`): display name, what the wiki says they hold
  as weighted loot areas and item categories plus any items their loot table names, breach,
  skill needed, conditions, confidence, notes. The planner reads it instead of a table in code.
- **Map conditions** are data (`conditions.json`). While one is live on a map the planner
  scales rare-and-up items by its loot value, boosts the marker types it spawns more of, and
  counts its event-only containers fully.
- **Display names**, chosen 2026-11-08 pending an in-game look: Medical Bag (was Med Crate),
  Ammo Case (Ammo Crate), Backpack (Bag), Basket, First Wave Cache (Hurricane Cache), Security
  Locker (Security Breach), Grenade Tube (Utility Crate: MetaForge's own legend says Grenade
  Tube), Container (Base Container).
- **Field Crate** is out of the loot model. It's a crate you carry to a Field Depot, which then
  gives a choice of loot.
- **Earlier findings applied:** the Expedition skill-point cap is 15 (5 per Expedition), the
  Crane Secure Storage Keycard duplicate is gone, nine stale ARC tips are rewritten
  (`overlays/tip_fixes.json`), the Flushing Terminal Key note names Shani's quest, and the
  Epoxy Bucket has loot areas.

## The Gemini summary

Its outline (container type plus zone decides loot) holds. Most specifics didn't:

- Wrong: breaching husks (you loot them; probes are breached); Hornet Husks, "Damaged Hornet
  Drivers", "Motion Cores" and "Light ARC Alloys" (none exist); Baron, Assessor and Deforester as
  enemies (they're lootable wrecks); ARC Circuitry in electrical cabinets (ARC only); Synthesized
  Fuel in cars (Exodus only); "augment boxes" (none); Raider Cache counts per map (guides give
  Dam 47, Buried City 16 or 33 under Uncovered Caches); weapon blueprints from Night Raid or Cold
  Snap (major conditions, the Harvester, Raider Containers).
- Real: the Grenade Tube; weapon cases with a weapon and matching ammo; Uncovered Caches; the
  Harvester's guaranteed Equalizer or Jupiter blueprints.
- Unsupported anywhere: "x2 rare blueprints in major conditions", "4 to 7 items per cache",
  "2 to 5 ammo stacks per case".

## Not settled

- **Drop rates:** no source publishes any. Every weight in the overlays is an estimate.
- **Unconfirmed loot tables:** the wiki flags these unconfirmed: Medical Bag, Android, Rocketeer
  Husk, Server Rack, Seed Vault. Suitcase and 16 others have no table at all.
- **Catch-all markers:** MetaForge's Container (13,000+), Locker (also medical drawers) and
  Breachable Container markers mix many real container types.
- **Conflicts:**
  - The Fireworks Box blueprint: the wiki says Cold Snap only; MetaForge says it's in the
    Industrial pool.
  - Buried City unlocks after 6 rounds (map page) or 8 (Maps page).
  - Two Pendola Pass key rarities differ between pages.
- **2.3.0** lets the Dolabra, Bobcat, Equalizer, Jupiter, Tempest and Vulcano blueprints drop
  in any major condition. It doesn't say whether the Harvester's guarantee still holds.

The in-game checks are listed in `dataset/DATA_AUDIT.md` under "To do in game".
