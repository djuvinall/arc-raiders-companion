# Tips research, 2026-11-06 (patch 2.4.1)

Five parallel research passes. Each tip is one line: tip | confidence | sources with dates.

- **CONFIRMED** means Embark's patch notes or a dev post say so, or two independent sources agree and nothing newer contradicts them. A guide that copies the wiki doesn't count as a second source.
- **LIKELY** means one good source and nothing against it.
- **DISPUTED/STALE** means the sources disagree, or a patch changed it.

| File | Topic | Confirmed | Likely | Disputed/stale |
|---|---|---|---|---|
| [arcs.md](arcs.md) | Fighting each ARC, plus existing tips that look stale | 80 | 28 | 15 stale existing tips |
| [maps.md](maps.md) | Per-map extracts, key rooms, conditions, raid flow, key doors | 60 | 43 | 6 |
| [loot.md](loot.md) | Keep/sell, traders, Scrappy, blueprints, stash, Expeditions, currencies | 46 | 22 | 3 |
| [gear.md](gear.md) | Balance timeline, weapons, armor penetration, shields, augments, gadgets | 52 | 24 | 7 |
| [mechanics.md](mechanics.md) | Stamina, noise, downed state, matchmaking, quests, skill tree | 63 | 27 | 5 |

**Coverage gaps:**
- **Not reached:** Reddit, MetaForge's site and the fandom wiki all blocked the agents, so no tip rests on Reddit alone.
- **Official notes:** read from launch (2025-10-30) to hotfix 2.4.1.

## Dataset corrections found

1. **Expedition skill points.** The tree sets `points.expedition_bonus_max = 5`. Embark's 5th Expedition post (2026-08-21) says up to 5 points per Expedition, capped at 15 in total. The planner caps you at 5.
2. **Cassio is missing from traders.** Since 2.0, Cassio (the Nomadic Envoy) replaces Ermal. He trades weekly, by barter, from level 25. `traders.json` has only Apollo, Celeste, Lance, Shani and Tian Wen.
3. **Riven Tides Crane Secure Storage Keycard** looks like a wiki naming slip. Its page is titled Secure Storage Keycard, MetaForge and RaidTheory list only that card, and the Crane House has its own keycard. Merge it into `riven-tides-secure-storage-keycard`.
4. **Flushing Terminal Key.** DATA_AUDIT.md calls it Celeste's quest, but it's Shani's (The Stench of Corruption), and `quests.json` already says Shani. It's a quest item, not a door key. You pick it up in the Departure Building Staff Locker Room and use it on any Flushing Terminal in the tunnels. No source gives the terminal's exact position.
5. **Dam Utility Key.** No door was found anywhere. MetaForge says only "location unknown", and no quest lists it as a reward. It may be unused. The "closets in admin buildings" note has no backing.
6. **Existing ARC tips** (wiki prose): see "Existing tips that look stale or wrong" in arcs.md. The main ones:
   - **Hydra:** standing between its guns stopped working in 2.2.0.
   - **Deadline:** one-shots on the Bastion and Bombardier are disputed since 1.3.0.
   - **Comet:** it does not always drop the Igniter.
   - **Skulker:** one Hullcracker shot doesn't reliably kill it.
   - **Matriarch:** its Dam spawn moved in 1.7.0.

Checked and correct: shield stats (Light 40/40%, Medium 70/42.5%/5%, Heavy 90/52.5%/10%), every workbench and Scrappy upgrade cost, the Anvil at Rare, the Energy Clip at 200, and Riven Tides is present.

## Health values

None could be confirmed.
- **Queen and Matriarch:** disputed. The wiki's 23000 is labelled an estimate, against 34k/21k (Queen) and 35k (Matriarch) elsewhere.
- **Bully:** 3000 comes from one wiki value with no reference.
- **Skulker:** 230 for the body, from one guide.
- **Assessor and Frigate:** have no health; they're an event and a container.
