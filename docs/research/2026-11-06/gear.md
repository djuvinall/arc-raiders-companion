# ARC Raiders gear research (weapons, mods, shields, augments, gadgets, crafting)

Researched 2026-11-06, live patch 2.4.1 (hotfix 2026-11-05). 2.4.0 and 2.4.1 contain no gear balance changes, so the last gear balance pass is 2.3.0 (2026-10-27).
Format: tip | confidence | sources. Source tags are listed at the bottom with their URL and date.
Confidence: CONFIRMED = official notes, or two independent sources with nothing newer contradicting. LIKELY = one good source, uncontested. DISPUTED/STALE = sources disagree, or a patch changed it.
Independence note: arcraiders.wiki numbers are copied by many guide sites (timesaver.gg, some content farms). Copies are not counted as a second source. mtmmo.com and aoeah.com share a template and count as one source.

## Balance timeline (official patch notes only)

| Date | Patch | Change |
|---|---|---|
| 2025-10-31 / 11-06 | Hotfix, 1.1.0 | Augment effects that weren't working now apply. Integrated Shield Recharger no longer has to be equipped to recharge. Health-regen augments now work if triggered while downed. Barricades can't be placed near zipline exits. [S-1.1] |
| 2025-11-13 | 1.2.0 | Launcher ammo value doubled (100 to 200). Wolfpack costs more to craft. Blaze Grenade Trap over-damage fixed. Anti-Tick Field: Tick removed after 0.5s (was 1s). Smoke clouds now visible at the same range as players. Trigger 'Nade could no longer skip the brief invulnerability on being downed. [S-1.2] |
| 2025-11-20 | 1.3.0 | VENATOR NERF: fire rate gained from upgrades cut from 22/44/60% to 13/26/40%, weight 2 to 5. EXPLOSIVES vs ARC rebalanced: Hullcracker needs more precision, larger ARC (e.g. Bastion) take a bit less. Deadline price 8,100 to 15,000 and trader stock 3 to 1. Launcher ammo 6 for 4,500. [S-1.3] |
| 2025-12-16 | 1.7.0 | BETTINA BUFF: durability burn ~0.43% to ~0.17% per shot, mag 20 to 22, reload 5s to 4.5s. RATTLER BUFF: mag 10 to 12. Deadline no longer damages through walls. Flame Spray over-damage to some ARC fixed. [S-1.7] |
| 2026-01-13 | 1.11.0 | KETTLE fire rate 600 to 450 (anti-macro). TRIGGER 'NADE NERF: damage concentrated toward the centre, and trigger-to-blast delay 0.7s to 1.5s. [S-1.11] |
| 2026-01-27/28 | 1.13 / 1.13.1 | 1.13 cut semi-auto fire rates by accident. 1.13.1 reverted it, but that let spam-clicking reach full cadence (the "shadow buff"). [S-1.13.1][S-1.17] |
| 2026-02-24 | 1.17.0 | STITCHER NERF: headshot multiplier 2.5 to 1.75, damage 7 to 6.5, per-shot dispersion +~50%. KETTLE NERF: damage 10 to 8.5 (keeps its high headshot multiplier). VENATOR NERF #2: headshot 2.5 to 2, damage 9 to 8. JUPITER BUFF: ADS zoom ~1.9x to ~2.2x, faster equip/unequip. APHELION BUFF: reload 4.5s to 3.5s, time between shots 0.9s to 0.7s, vertical recoil about -50%, ADS settle about +35%. Semi-auto input buffer retuned per gun: spam-clicking still works, but paced shots do better. Energy Clip craftable by default. Deadline, Trailblazer and Vita Spray cost more. [S-1.17] |
| 2026-03-17 | 1.20.0 | IL TORO NERF: pellet damage 7.5 to 7, fire rate 43 to 38, dispersion 4.5 to 6, reload 4.3s to 5.7s, falloff loss 40% to 50%. [S-1.20] |
| 2026-03-31 | 1.22.0 | New Dolabra (legendary energy shotgun), Canto (rare medium-ammo SMG) and Surge Coil. [S-1.22] |
| 2026-04-08 | 1.23.0 | Fixed two Trigger 'Nades dealing triple damage together. Barricades can't go on ziplines. [S-1.23] |
| 2026-04-28 | 1.26.0 | WEAPON ECONOMY: upgrading repairs 25% of max durability. Durability loss per shot: Common +75%, Uncommon +50%, Rare +35%, Epic -5%, Legendary -10%. Average spawned durability 50 to 30 (locked rooms still about 60). Durability lost when knocked out: 30% to 15%. TRIGGER 'NADE: 1s lockout after a throw, 1.3s after a trigger. PHOTOELECTRIC CLOAK NERF: weight 1 to 3, power 2.5/s to 10/s. BETTINA BUFF: damage 14 to 16, damage vs ARC armour +~33%, fire rate 285 to 235, bloom -~40%. Heavy ammo stack 40 to 60. Dev note says Combat augments are underperforming. [S-1.26] |
| 2026-05-19 | 1.29.0 | Bettina fire rate 235 to 250. Cloak power 10/s to 5/s. TAC MK3 (HEALING) BUFF: Cloud of Renewal is now AoE, heals 45 (was 20), cooldown 45s (was 30s). COMBAT MK3 (FLANKING) can use Medium Shields. Durability eased on the worst-affected guns. EXTENDED BARREL III: velocity +25% to +30%, falloff distance +15%; tiers I and II added. Mods can now change ADS speed on scoped guns. [A-1.29][S-1.29] |
| 2026-07-07 to 07-28 | 1.36 to 1.39 | A collision change in 1.36 weakened the Seeker by accident (acknowledged in 1.37). 1.39 fixed inconsistent Seeker and Wolfpack damage. [S-1.37][S-1.39] |
| 2026-09-08 | 1.45.0 | Aphelion weapon sway reduced. [S-1.45] |
| 2026-10-08 | 2.0 Frozen Trail | ANVIL moved from Uncommon to Rare ("outperforming weapons within and above its rarity"). Anvil Splitter removed as a mod and Anvil's Tech slot removed. New Bantam (uncommon heavy hand cannon) and Stiletto (light-ammo battle rifle). EXPLOSIVES vs ARC reworked: over 50% more vs armour plates, about 30% less vs fragile parts. JUPITER: double damage propagation fixed, so it's stronger vs armour and slightly weaker vs weak spots. HEAVY SHIELD: decay 0.20 to 0.15, movement penalty 15% to 10%. SNAP HOOK NERF: range 20m to 18m, durability per use 2% to 4%, power 50 to 60. Weapon Amplification, Grappling Hook, Tether Launcher and Yank Grenade added. Weapon-slot weight reduction now applies only to the loadout. Launcher ammo and Energy Clips have daily trader limits. [A-2.0][S-2.0] |
| 2026-10-13 | 2.1.0 | Hairpin semi-auto amplification now has its intended reduced headshot damage. [S-2.1] |
| 2026-10-27 | 2.3.0 | HEAVY SHIELD charge 80 to 90. FLAME SPRAY 10 to 13 dmg/s, durability drain -50%. Adrenaline Shot usable at full stamina and no longer stacks (a second shot refreshes it). Firefly Burner now activates on impact. Bettina, Bobcat, Equalizer, Jupiter, Tempest, Vulcano and Dolabra blueprints can drop in any major map condition. [S-2.3] |
| 2026-11-03/05 | 2.4.0 / 2.4.1 | No gear balance changes. [S-2.4.0][S-2.4.1] |

## Weapons vs ARC / vs Raiders (current patch only)

- Anvil was officially "outperforming" its own and higher rarities, so 2.0 raised it to Rare instead of nerfing it. It is still strong but costs more now. | CONFIRMED | A-2.0
- The Bantam is the new Uncommon heavy-ammo hand cannon in the Anvil's old slot, with a hip-fire focus. Players say it's weaker and less accurate than the Anvil. | CONFIRMED (role) / LIKELY (weaker than Anvil) | A-2.0, S-FT; G-bantam
- Since 2.0, explosive weapons (Hullcracker, launchers) are the best way to strip armour plates on big ARC. They are worse than before against weak points. | CONFIRMED | A-2.0
- Since 2.0 the Jupiter is stronger vs ARC armour and slightly weaker vs weak spots. Wiki rates its penetration "Very Strong (3x)", the highest listed. | CONFIRMED | A-2.0, W-Jupiter
- The Bettina (Epic, heavy ammo) was buffed vs ARC armour in 1.26 and re-buffed in 1.29. It's a top PvE assault rifle. | CONFIRMED (buffs) / LIKELY (top PvE; MTMMO A tier) | S-1.26, A-1.29, G-mtmmo
- Venator advice from before 1.17 is stale: it was nerfed twice (1.3 upgrade fire rate, 1.17 damage and headshot). | DISPUTED/STALE | S-1.3, S-1.17
- Stitcher and Kettle "low-tier beats high-tier" advice from before 1.17 is stale. Both had their damage cut, and the Stitcher's headshot multiplier dropped to 1.75x. | DISPUTED/STALE | S-1.17
- Il Toro advice from before 1.20 (choke long-range shotgun) is stale. Reload, fire rate and falloff were all nerfed. | DISPUTED/STALE | S-1.20
- Frozen Trail PvP tier lists agree on Bobcat, Vulcano, Tempest, Canto and Dolabra near the top. Treat this as opinion, not data. | LIKELY | G-epiccarry, G-mtmmo
- Light-ammo SMGs (Stitcher, Bobcat, Kettle) are PvP guns. Their Very Weak ARC penetration makes them poor against armoured ARC. | CONFIRMED | W-weapons, G-pen-search (thegamepost/arcraidershub agree)
- Embark admitted in an interview that the cost-to-benefit of the Bobcat and Tempest was "a bit off". | LIKELY | G-gr-dev
- Semi-autos (Ferro, Renegade, Kettle, etc.) reward paced shots over spam-clicking since the 1.17 per-gun input buffer. | CONFIRMED | S-1.17
- Wiki listings may still show Anvil as Uncommon or crafted at Gunsmith 1. Check it in game after 2.0. | DISPUTED/STALE | W-workshop vs A-2.0

## ARC armour penetration and ammo

- The ammo type mostly sets ARC armour penetration. Light is very weak, shotgun weak, medium moderate, heavy strong, launcher and energy strong to very strong (with exceptions). | CONFIRMED | W-weapons, W-ammo, G-pen-search
- Wiki per-gun values. Light: Kettle, Stitcher, Bobcat, Hairpin, Burletta 0.2x; Stiletto 0.4x. Shotgun: Il Toro, Vulcano 0.5x. Medium: Rattler, Arpeggio, Torrente 1x; Tempest, Renegade, Canto, Venator, Osprey 0.8x. Heavy: Ferro, Anvil, Bantam 1.5x; Bettina 2x. Energy: Dolabra 1.5x; Aphelion, Equalizer 2.5x; Jupiter 3x. Launcher: Hullcracker, Rascal 2.5x. | LIKELY (single source for the multipliers) | W-weapon pages (2026-10-31)
- Shiny dark-grey plates are armoured: anything below Moderate penetration bounces off, and stripping plates exposes weaker plating. Matte white is unarmoured. Bright yellow is a weak point: shoot those first. | LIKELY | W-ARC (2026-11-06)
- Heavy ammo stack size is 60 (was 40) since 1.26. | CONFIRMED | S-1.26
- Practice Range dummies took no damage from the Dolabra, Hullcracker or Rascal until 1.42. Old "test it on the range" advice for those guns was invalid. | CONFIRMED | S-1.42
- Since 2.0, Launcher Ammo and Energy Clips have a daily purchase limit at traders. | CONFIRMED | A-2.0

## Headshots

- Headshot multipliers vary by gun; there is no flat 2.5x. Examples: Stitcher 1.75x, Venator 2x, Kettle still high (wiki 2.5x). Guides from 2025 saying "2.5x for everything" are stale. | CONFIRMED (varies) / DISPUTED/STALE (flat 2.5x) | S-1.17, W-weapon pages, G-ath-hs (2025-11-04)
- Shotguns (Il Toro, Vulcano, Dolabra) get no headshot bonus (1.0x) and the Bantam only 1.5x, so aim centre-mass with them. | LIKELY | W-weapon pages; timesaver repeats it for Bantam (wiki copy)
- Whether the headshot multiplier also drains extra shield charge is disputed. The wiki says charge takes the base damage (multiplier on HP only); allthings.how says otherwise. | DISPUTED | W-shields, G-ath-hs
- The Jupiter is officially the game's top headshot damage dealer in PvP. Since 1.17 it has more zoom and faster swaps for follow-ups. | CONFIRMED | S-1.17

## Weapon tiers I-IV, upgrading and Amplification

- Upgrades are permanent and go I to IV (three upgrades) at the Gunsmith. Bonuses vary per gun (e.g. Stitcher: less horizontal recoil, faster reload, more durability). | CONFIRMED | W-weapons, W-stitcher, G-upgrade-search (deltiasgaming/players.com.ua)
- Since 1.26, each upgrade also repairs 25% of max durability, so upgrade a worn gun instead of repairing then upgrading. | CONFIRMED | S-1.26
- Upgrading is generally worth it. The usual advice: upgrade unless you need the materials elsewhere. | LIKELY | G-upgrade-search, G-pcgamer (headline)
- Commons and uncommons lose durability much faster per shot since 1.26, while Epic and Legendary guns last longer. Weigh that before sinking upgrades into a cheap gun. | CONFIRMED | S-1.26
- Some upgrade bonuses were cut, e.g. Venator upgrade fire rate is now 13/26/40%. | CONFIRMED | S-1.3
- Amplification (2.0) changes a weapon beyond IV and can't be undone. It needs the Outpost plus Gunsmith 4 and Research Bench 4, and is on 15 guns at launch. | CONFIRMED | A-2.0, W-amplification
- Amplified weapons drop to 10% durability on death (wiki). Anti Weak Point gives +67% vs weak points but -60% vs armour (wiki numbers). | LIKELY | W-amplification (2026-10-24)

## Mods

- Extended Barrel III: +30% bullet velocity and +15% distance before damage falloff. Lower tiers I and II exist. | CONFIRMED | A-1.29, W-mods
- Since 1.29, mods can change ADS speed on scoped weapons. | CONFIRMED | A-1.29
- Tier III muzzle mods (Compensator III, Muzzle Brake III, Shotgun Choke III, Silencer III) add +20% durability burn. Pick tier II if durability is tight. | LIKELY | W-mods
- Grips III, Padded Stock and Horizontal Grip slow ADS by 30%. Lightweight Stock gives much faster ADS and swaps at the cost of +50% vertical recoil. | LIKELY | W-mods
- Kinetic Converter (stock): +15% fire rate, +20% recoil both ways. | LIKELY | W-mods
- The Anvil Splitter is no longer a mod (2.0). Old Anvil Splitter builds are dead; the item recycles into Amplified Fragments. | CONFIRMED | A-2.0
- Light mag mods add +5/+10/+15 rounds. Per the wiki, Rattler has no mag slot (base 12 rounds). | LIKELY | W-mods, W-weapons

## Shields

- Shields reduce damage by a percentage; they are not extra HP. Each hit drains charge, and mitigation stays full while 1+ charge remains. | LIKELY (wiki mechanic, uncontested in recent sources) | W-shields
- Current stats (wiki). Light: 40 charge, 40%, no penalty, weight 5. Medium: 70 charge, 42.5%, 5% slower, weight 7. Heavy: 90 charge, 52.5%, 10% slower, weight 9. Launch-era guides show other numbers (e.g. 50/23%, 80/45%, 120/65%) and are stale. | LIKELY (current values) / DISPUTED/STALE (2025 guides) | W-heavy, W-medium, W-light; G-ath-shields (2025-10-28) conflicts
- Heavy Shield buffs: 2.0 cut its penalty 15% to 10% and decay 0.20 to 0.15, and 2.3 raised charge 80 to 90. Any guide quoting 15% or 80 is stale. The wiki's own Shields overview still showed 80/15%. | CONFIRMED | A-2.0, S-2.3
- Embark says Heavy Shield usage is still low and it doesn't want too big a gap between Heavy and Medium. Medium is the common default for mobile and solo play, Heavy for holding ground. | CONFIRMED (dev statement) / LIKELY (role advice) | S-2.3, G-timesaver (2026-11-01)
- You need a compatible augment to wear Medium or Heavy shields. Heavy: Combat Mk2, Combat Mk3 Aggressive, Tactical Mk3 Defensive, Looting Mk3 Safekeeper. | LIKELY | W-heavy, G-brokenbuilds
- The Used to the Weight skill reduces Medium and Heavy movement penalties (wiki, pre-2.0 skill tree; check after the 2.0 skill rework). | DISPUTED/STALE (unverified post-2.0) | W-shields
- Shields lose durability as they absorb; about five full depletions break one at 100% durability (player test). | LIKELY | W-shields

## Augments

- Tactical Mk3 (Healing): when you're revived, an AoE cloud heals 45 (was 20) on a 45s cooldown (was 30s). It also heals enemies in range. The wiki infobox and augment table still say 20/30s. | CONFIRMED (new values) / STALE (wiki table) | A-1.29, G-gfinity-survivor (2026-05-20), G-brokenbuilds
- Combat Mk3 (Flanking): Light or Medium shields since 1.29. Stowed pistols and hand cannons equip 33% faster. | CONFIRMED (shield) / LIKELY (perk) | A-1.29, W-aug-table, G-brokenbuilds
- Looting Mk3 (Survivor): highest weight limit (80) and regenerates you up to 75% downed health while downed and still. Seen as dominant; Embark buffed other augments instead of nerfing it. | CONFIRMED | W-aug-table, G-brokenbuilds, G-gfinity-survivor
- Tactical Mk3 (Defensive): integrated Shield Recharger, lets you use rechargers while running, takes Heavy shields. | CONFIRMED | W-aug-table, G-brokenbuilds
- Looting Mk3 (Safekeeper): its safe pocket holds any item (including weapons) and it takes Heavy shields. | CONFIRMED | W-aug-table, G-gr-safekeeper (headline), S-1.18 (exploit fix references it)
- Looting Mk3 (Cautious): biggest backpack (24), integrated binoculars, weak Adrenaline Shot when your shield breaks. Light shield only. | CONFIRMED | W-aug-table, G-brokenbuilds
- Tactical Mk2 and Tactical Mk3 (Smoke) drop smoke when your shield breaks. Combat Mk2 / Mk3 Aggressive regen 1 or 2 HP per 5s, paused 30s after taking damage. | CONFIRMED | W-aug-table, G-brokenbuilds
- Looting Mk2 throws off attached Ticks after 1s. | CONFIRMED | W-aug-table, G-brokenbuilds
- Augments lose 30 durability each time you're knocked out and can't be swapped topside. | LIKELY | W-augments
- Apollo sells Epic augments since 2.0. | CONFIRMED | A-2.0

## Grenades and gadgets

- Trigger 'Nade: sticks to surfaces and ARC, remote detonation, 1.5s fuse after triggering, damage concentrated at the centre. It can't be chain-thrown and triggered (1s and 1.3s lockouts). Old "air-burst spam" advice is stale. | CONFIRMED | S-1.11, S-1.26, W-trigger
- Seeker Grenade: homes on the nearest ARC within about 20m and is great vs Wasps, Hornets and Snitches. It was weakened by a bug in 1.36 and fixed in 1.39. | CONFIRMED (ARC homing, fix) / LIKELY (20m) | W-seeker, S-1.37, S-1.39
- Lure Grenade: sticks to surfaces or ARC (not raiders) and pulls ARC aggro (wiki: 15s). Shredders track it (1.3), and it no longer cancels a Snitch's call (1.13), so don't use it to stop a Snitch. | CONFIRMED | S-1.3, S-1.13, W-lure
- Deadline: 6s fixed timer, can't be disarmed or recovered, huge blast that can delete big ARC. Since 1.7 it no longer goes through walls, so hard cover saves you. | CONFIRMED (walls) / LIKELY (6s, 1,000 dmg) | S-1.7, W-deadline
- Blaze Grenade: fire area that deals damage over time to ARC and raiders (wiki: 5/s, 10s, 10m). | LIKELY | W-blaze
- Snap Blast Grenade: sticks to ARC, raiders or surfaces and blows after a short delay (wiki: 3s, 70 dmg). | LIKELY | W-snapblast
- Smoke Grenade: blocks sight for ARC and raiders (wiki: 20s, 7.5m). Since 1.2, item smoke renders at the same range as players, so it isn't see-through from far away. | CONFIRMED (render range) / LIKELY (stats) | S-1.2, W-smoke
- Jolt Mine: proximity stun (wiki: ARC 7s, raiders 4s, 5m). Mines can be set off by shooting them. | LIKELY | W-jolt, S-1.7 (shoot-trigger fix)
- Showstopper: stun grenade, detonates on hit or after a delay (wiki: ARC 7s, raiders 2s). | LIKELY | W-showstopper
- Zipline: up to about 60m. Can't anchor on carryables or barricades. Breaks if something cuts through it after placement (2.0). | CONFIRMED (placement/break rules) / LIKELY (60m) | S-1.17, S-1.23, A-2.0, W-zipline
- Snap Hook: since 2.0 it's a burst-mobility tool. 18m range, 4% durability per use, 60 power, so no back-to-back uses. It can break falls (wiki). | CONFIRMED (nerf) / LIKELY (fall trick) | A-2.0, W-snaphook
- Grappling Hook (2.0) is the climbing/abseil/swing option. It can't be used inside the Matriarch's shield. | CONFIRMED | A-2.0, S-2.1
- Barricade Kit: deployable cover (wiki: 500 HP). Any raider can dismantle it in 5s. Can't overlap walls, tight spaces or ziplines. | CONFIRMED (placement rules) / LIKELY (HP, 5s) | S-1.3, S-1.23, S-2.3, W-barricade
- Photoelectric Cloak: hides you from ARC only and slows you to a walk. After the 1.26 nerf and 1.29 partial revert it drains 5 power/s and weighs 3. | CONFIRMED | S-1.26, A-1.29, W-cloak
- Flame Spray: buffed in 2.3 to 13 dmg/s with half the durability drain. | CONFIRMED | S-2.3
- Tether Launcher and Yank Grenade (2.0) root or pull targets. Yank can drag flying ARC to the ground or pull raiders out of cover. | CONFIRMED | S-FT
- Wolfpack scatters homing missiles at ARC and had the same damage fix as the Seeker in 1.39. | CONFIRMED | S-1.39, W-grenades

## Healing and recovery

- Health over time (wiki): Bandage 2/s, Herbal Bandage 3.5/s, Sterilized Bandage 5/s, each 10s with a 1.5s use. Vita Shot gives 50 HP (4s use). Vita Spray 10/s continuous and works on allies. | LIKELY | W-healing; timesaver repeats Bandage (wiki copy)
- Vita Spray heals continuously only while you're using it (1.17). | CONFIRMED | S-1.17
- Shield Recharger gives 4/s over 10s (2s use); Surge Shield Recharger gives 50 instantly (5s use). Tactical Mk3 Defensive has an integrated Surge recharger. | LIKELY | W-healing
- Adrenaline Shot: full stamina restore. Since 2.3 it works at full stamina, and a second shot refreshes the timer instead of stacking. | CONFIRMED | S-2.3
- 2.0 skill changes: Back on Your Feet heals 40 (was 30), Fight or Flight gives about 25% more stamina per level, Stubborn Mule regen about 33% higher. | CONFIRMED | A-2.0

## Workbench and crafting priorities

- Station costs. Gunsmith 1: 20 Metal + 30 Rubber. Gunsmith 2: 3 Rusted Tools, 5 Mech Components, 8 Wasp Drivers. Gunsmith 3: 3 Rusted Gear, 5 Adv Mech Components, 4 Sentinel Firing Cores. Gear Bench 1: 25 Plastic + 30 Fabric. Gear Bench 2: 3 Power Cable, 5 Elec Components, 5 Hornet Drivers. Gear Bench 3: 3 Industrial Battery, 5 Adv Elec Components, 6 Bastion Cells. | CONFIRMED (no later changes found) | W-workshop (2026-10-31), G-gfinity-ws (2025-11-05)
- Gunsmith 4 (needs Outpost: Radial Press, Magnetic Accelerator, Emperor Modulator) unlocks Amplification. | LIKELY | W-workshop, A-2.0
- Upgrade order is opinion and guides disagree: Gear Bench 1 first (Medium Shield + augments), or Gunsmith first (guns + mods). Both put Gunsmith 2 and Gear Bench 2 early. | DISPUTED | G-lootmgr (2026-01-25), G-ws-search
- The basic Workbench already crafts Ferro, Kettle, Stitcher, Hairpin, all ammo types, Light Shield, Shield Recharger, Bandages and Light Impact grenades. You don't need a bench upgrade to start running. | LIKELY | W-workshop
- Launcher ammo is craftable at the Workbench without a blueprint (1.3), and Energy Clips without a blueprint since 1.17. Energy Clips are no longer a money craft (sell 1,000 to 200 in 1.20). | CONFIRMED | S-1.3, S-1.17, S-1.20
- ARC Motion Core and ARC Circuitry unlock at Refiner 2 (was 3) since 1.2. | CONFIRMED | S-1.2
- Locked rooms still spawn guns around 60 durability vs as low as 15 on the outskirts (1.26). Loot weapons there. | CONFIRMED | S-1.26

## Sources

Official (Steam announcements, appid 1808500, base URL https://steamstore-a.akamaihd.net/news/externalpost/steam_community_announcements/<id>; full notes on arcraiders.com):
- S-1.1: 1815580768256227 (2025-11-06); Hotfix 1815034432902750 (2025-10-31)
- S-1.2: 1816307528908374 (2025-11-13)
- S-1.3: 1816849002009972 (2025-11-20)
- S-1.7: 1818752592135186 (2025-12-16)
- S-1.11: 1821288646587673 (2026-01-13)
- S-1.13 / S-1.13.1: 1822556746168686 (2026-01-27) / 1823191198599856 (2026-01-28)
- S-1.17: 1825093633197017 (2026-02-24)
- S-1.18: 1825727806719675 (2026-03-03)
- S-1.20: 1826992588603398 (2026-03-17)
- S-1.22: 1828441623111636 (2026-03-31); full: https://arcraiders.com/news/patch-notes-1-22-0
- S-1.23: 1829528821303625 (2026-04-08)
- S-1.26: 1830797770244665 (2026-04-28)
- S-1.29: 1832700592800429 (2026-05-19); A-1.29: https://arcraiders.com/news/patch-notes-1-29-0 (2026-05-19)
- S-1.37: 1837955055365541 (2026-07-14)
- S-1.39: 1839041357037759 (2026-07-28)
- S-1.42: 1841579228669036 (2026-08-21)
- S-1.45: 1842846814455073 (2026-09-08)
- S-FT: 1846018067928872 "Frozen Trail" overview (2026-10-08)
- S-2.0: 1846018067928877 (2026-10-08); A-2.0: https://arcraiders.com/news/frozen-trail-2-0-update (2026-10-08)
- S-2.1: 1846018067943414 (2026-10-13)
- S-2.3: 1847286114048305 (2026-10-27)
- S-2.4.0: 1847920302251347 (2026-11-03); S-2.4.1: 1848664643341170 (2026-11-05)

arcraiders.wiki (via https://arcraiders.wiki/w/api.php?action=parse&page=<Name>&prop=wikitext&format=json; last revision date shown):
- W-shields: Shields (2026-07-02; overview table stale for Heavy). W-light: Light_Shield (2026-06-23). W-medium: Medium_Shield (2026-10-23). W-heavy: Heavy_Shield (2026-10-28)
- W-weapons: Weapons (2026-10-21). W-ammo: Ammo (2026-05-21). W-ARC: ARC (2026-11-06). Per-weapon pages Stitcher/Kettle/Anvil/Ferro/Bettina/Bantam/Jupiter/Hullcracker (2026-10-31), others same period
- W-mods: Weapon_Mods (2026-02-11, rendered table). W-amplification: Amplification (2026-10-24). W-workshop: Workshop (2026-10-31)
- W-augments: Augments (2026-03-18). W-aug-table: Template:Augments_table (2026-07-02; Healing row stale)
- W-healing: Healing (2026-04-02). W-grenades: Grenades. Gadget pages: Deadline (2026-11-04), Snap_Hook (2026-11-03), Trigger_'Nade, Seeker, Lure, Blaze, Smoke, Jolt_Mine, Zipline, Barricade_Kit, Photoelectric_Cloak (all 2026-10-23), Snap_Blast_Grenade (2026-06-23), Showstopper (2026-10-10)

Guides / third party:
- G-epiccarry: https://epiccarry.com/blogs/arc-raiders-weapon-tier-list/ (updated 2026-10-13; seen via search snippet only, page returned 403)
- G-mtmmo: https://www.mtmmo.com/news/2338--arc-raiders-20-best-gun-tier-list-for-pve--pvp-frozen-trail (2026-10-16); same template as aoeah.com/news/4879 (2026-10-12)
- G-dexerto-pvp: https://www.dexerto.com/wikis/arc-raiders/weapon-tier-list-pvp-arc-raiders (undated; S: Anvil, Renegade, Tempest, Venator, Dolabra; not used for current claims)
- G-brokenbuilds: https://brokenbuilds.gg/arc-raiders/tier-lists/arc-raiders-augment-tier-list (2026-05-03, updated 2026-07-28, patch 1.39)
- G-gfinity-survivor: https://www.gfinityesports.com/article/arc-raiders-devs-refuse-to-nerf-the-op-survivor-augment-so-they-buffed-everything-else-instead (2026-05-20)
- G-gfinity-ws: https://www.gfinityesports.com/article/arc-raiders-all-workshop-scrappy-and-expedition-upgrades-guide (2025-11-05)
- G-lootmgr: https://arc-raiders-loot-manager.pages.dev/guides/workshop-upgrade-priority (2026-01-25)
- G-timesaver: https://timesaver.gg/blog/arc-raiders-pvp-tips (2026-11-01; numbers match the wiki, so not independent)
- G-ath-shields: https://allthings.how/arc-raiders-shields-types-mitigation-stats-and-recharge-options/ (2025-10-28; STALE stats)
- G-ath-hs: https://allthings.how/arc-raiders-headshot-damage-multipliers-shields-and-ttk/ (2025-11-04; flat 2.5x is stale)
- G-gr-dev: https://www.gamesradar.com/games/third-person-shooter/cost-should-not-inherently-fill-a-skill-gap-arc-raiders-dev-admits-were-definitely-a-bit-off-on-some-of-the-cost-to-benefit-ratios-for-guns-like-the-bobcat-and-tempest-but-says-all-weapons-have-to-be-usable (date not retrieved)
- G-gr-safekeeper: https://www.gamesradar.com/games/third-person-shooter/arc-raiders-answers-the-communitys-prayers-new-epic-looting-augment-lets-you-put-weapons-in-your-safe-pocket-and-even-use-heavy-shields-to-boot (headline only, date not retrieved)
- G-pcgamer: https://www.pcgamer.com/games/third-person-shooter/arc-raiders-upgrade-weapons (headline only)
- G-bantam: search summary of games.gg / beebom / gamebrief Bantam guides (Oct 2026), incl. a quoted player complaint; not verified on Reddit
- G-pen-search / G-upgrade-search / G-ws-search: WebSearch summaries 2026-11-06 (thegamepost, arcraidershub, deltiasgaming, players.com.ua, etc.); supporting only

Not reached: Reddit (JSON API blocked, site: search returned no threads), metaforge.app (403), arc-raiders.fandom.com (402).
