# ARC Raiders research: mechanics, quests, skill tree (checked 2026-11-06, patch 2.4.1)

Format: tip | confidence | sources (URL, date)
"Official" means Embark patch notes or dev posts (Steam news feed appid 1808500 or arcraiders.com).
Reddit could not be read: the search API returned HTML only. No Reddit-only claims are included.
Wiki values marked "player-tested" are labelled that way by the wiki itself.

Key sources (short names used below):
- [ST] Steam news feed: https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=1808500&count=210&feeds=steam_community_announcements (read 2026-11-06)
- [2.0] Frozen Trail 2.0 patch notes: https://arcraiders.com/news/frozen-trail-2-0-update (2026-10-08)
- [WIKI-SK] https://arcraiders.wiki/wiki/Skills (read 2026-11-06; the wiki says its post-2.0 values are player-tested)

## Stamina, sprint, dodge roll, movement
- When you are exhausted you can't sprint-climb ladders. | CONFIRMED | [2.0] 2026-10-08
- Sprinting up a steep slope triggers a short "Scrambling" climb, and the Sure-Footed Scrambler skill makes it last longer. | CONFIRMED | [2.0] 2026-10-08; [WIKI-SK]
- Starting a slide costs a little stamina. | CONFIRMED | Official 1.7.0 Cold Snap notes, 2025-12-16 [ST]
- Adrenaline Shot works at full stamina. A second shot does not stack; it only resets the timer. | CONFIRMED | Official 2.3.0, 2026-10-27 [ST]
- You can't use stamina items while the Sprinting on Fumes cooldown is running, and Fight or Flight no longer skips its penalty. | CONFIRMED | Official 2.1.0, 2026-10-13 [ST]
- Going over your augment's weight limit cuts stamina sharply, and Mule's Gallop lets you sprint while over-encumbered. | LIKELY | https://arcraiders.wiki/wiki/Free_Loadout (read 2026-11-06); [WIKI-SK]
- Dodge roll speed depends on your movement state: low or empty stamina makes the roll much slower. A sprint dodge roll goes further and costs more. | LIKELY | https://arcraiders.wiki/wiki/Dodge_Roll , https://arcraiders.wiki/wiki/Sprint_Dodge_Roll (read 2026-11-06; the skill tables on both pages are STALE and list skills removed in 2.0)
- Wall Run: since 2.1 you must hold back to jump off the wall, so you don't trigger it by accident. You can now go from a wall climb onto ladders and ziplines. | CONFIRMED | Official 2.2.0, 2026-10-20 [ST]

## Fall damage, climbing, ladders
- A dodge roll no longer cancels fall damage. Heavy landings now trigger correctly while rolling, so the old exploit is gone. | CONFIRMED | Official 2.3.0, 2026-10-27 [ST]
- A recovery roll on landing exists. Ready To Roll widens its timing window, Stylish Landing lets you use it from higher up, and Sturdy Ankles cuts non-lethal fall damage. | LIKELY | [WIKI-SK] (in-game skill descriptions)
- Fall damage can down you, and the Grapple Hook and Tether now release if that happens. | CONFIRMED | Official 2.1.0, 2026-10-13 [ST]
- Known issue: falling while using the Grappling Hook can bounce you high into the air. | CONFIRMED | Official 2.3.0 known issues, 2026-10-27 [ST]

## Noise and how ARC find you
- Breaching makes noise that nearby ARC react to; a bug that stopped this was fixed in 1.26.0. Gentle Pressure makes breaching quieter. | CONFIRMED | Official 1.26.0 Riven Tides, 2026-04-28 [ST]; [WIKI-SK]
- Footsteps indoors carry further since 1.42.0, and tarp or plastic sheeting makes its own footstep sound. | CONFIRMED | Official 1.42.0, 2026-08-21 [ST]
- 2.1.0 made footsteps quieter outdoors and evened out volume across surfaces. | CONFIRMED | Official 2.1.0, 2026-10-13 [ST]
- Since 2.0, walls and obstacles muffle character sounds more naturally. | CONFIRMED | [2.0] 2026-10-08
- Since 2.0 you can switch off backpack lights to stay hidden. | CONFIRMED | [2.0] 2026-10-08; PC Gamer headline "you can finally turn those lights on your backpack off" (Oct 2026)
- Night Raid maps are darker, so flashlights and listening matter more. | CONFIRMED | Official 1.11.0, 2026-01-13 [ST]; Official 2.1.0 (Pendola night made darker), 2026-10-13
- Known issue: some doors let flashlight beams through. | CONFIRMED (known issue since 1.36; not listed as fixed) | Official 1.36.0 to 1.45.0 known issues, 2026-07-07 to 2026-09-08 [ST]
- Silent Scavenger makes looting quieter. | LIKELY | [WIKI-SK] (in-game description)
- Snitches call in other ARC (since 1.42: a Rocketeer, or a Firefly and Hornet pair). | CONFIRMED | Official 1.42.0, 2026-08-21 [ST]
- Guides say ARC respond to noise more than sight and that gunfire pulls them from further than a flashlight. | LIKELY (guides only, no numbers) | https://noobfeed.com/articles/arc-raiders-win-solo-player (undated); several low-quality guides

## Downed state, revives, self-revive, defib
- You can be looted while downed: other Raiders can take your weapon, stencil included. | CONFIRMED | [2.0] 2026-10-08; Official 1.2.0 (fix for looting downed players on slopes), 2025-11-13
- You can open the map while downed. | CONFIRMED | [2.0] 2026-10-08
- You can still extract while downed; reaching the extract while crawling saves your run. Known issue: you can still die inside the Ballistic Gondola if you are almost knocked out. | CONFIRMED | Official 2.2.0/2.3.0 known issues, 2026-10-20/27 [ST]; PC Gamer "crawled for 15 minutes to an extraction", 2025-11/12
- Stay still while downed: moving drains the downed timer much faster. | LIKELY (one creator test) | https://www.pcgamer.com/games/third-person-shooter/please-stop-scooting-around-while-youre-downed-in-arc-raiders-its-making-you-die-faster (2025-11-25)
- Bandaging a downed player buys them more time before they bleed out. | CONFIRMED | PC Gamer (2025-11-25, same URL); geekchamp healing guide (https://geekchamp.com/?p=234479, undated)
- Base downed timer is about 100 s; each rank of Downed But Determined adds about 6 s. | LIKELY (player-tested) | [WIKI-SK]; https://coe.gg/skills/ (copies wiki, not independent)
- You can revive your own squad without items. Reviving anyone outside your squad, including enemies, needs a Defibrillator. A defib revive is faster and restores more health. | CONFIRMED | https://www.gamesradar.com/games/third-person-shooter/arc-raiders-revive-enemies-dead-teammates (2025-11-06); https://arcraiders.wiki/wiki/Defibrillator (read 2026-11-06)
- Lance sells Defibrillators for 3,000 Coins, and the Nomadic Crafting end skill lets you craft them in-raid. | CONFIRMED (price may change) | GamesRadar 2025-11-06; wiki Defibrillator page; [WIKI-SK]
- Two people using defibs on the same player at once no longer gives a full-HP revive. | CONFIRMED | Official 1.13.0 Headwinds, 2026-01-27 [ST]
- A knocked-out (dead) Raider can't be brought back that round. | CONFIRMED | GamesRadar 2025-11-06; https://allthings.how/arc-raiders-revive-rules-allies-enemies-and-when-death-is-permanent (2025-11-06)
- No true self-revive was found in any source. The closest options are Looting Mk.3 (Survivor), which regenerates downed health to 75% while you stay still, and Back On Your Feet, which regenerates health when you are critically hurt but not downed. | LIKELY | https://arcraiders.wiki/wiki/Looting_Mk._3_(Survivor) (read 2026-11-06); [2.0]; PC Gamer 15-minute crawl article
- Back On Your Feet heals up to 40, up from 30 in 2.0. | CONFIRMED | [2.0] 2026-10-08
- A red flare goes up from a downed Raider; the Remote Raider Flare fires the same flare on command, so you can use it as bait. | LIKELY (no official statement on downed vs knocked out) | https://arcraiders.wiki/wiki/Remote_Raider_Flare (read 2026-11-06); PC Gamer "paramedic" article (Nov 2025)
- During the PvE test, ARC ignored downed players for 8 s and then could attack them again. | STALE (PvE test ran 2026-10-13 to 10-20 and is over) | Official PvE Toggle Beta post, 2026-10-13 [ST]

## Shields and recharging
- Shields cut a percentage of damage instead of acting as extra HP. Any charge left blocks the full percentage, and each full drain costs about 20% durability (roughly 5 full drains to break). | LIKELY (formula from player testing) | https://arcraiders.wiki/wiki/Shields (read 2026-11-06)
- The wiki's Shields overview table is STALE for Heavy; the Heavy Shield page is current. Heavy now has 90 charge (was 80, changed 2.3.0) and a 10% move penalty (was 15%), and durability decay dropped from 0.20 to 0.15 (changed 2.0). | CONFIRMED | Official 2.3.0, 2026-10-27; [2.0] 2026-10-08; https://arcraiders.wiki/wiki/Heavy_Shield
- A Shield Recharger restores 40 charge over 10 s, enough to refill a Light Shield. A Surge Shield Recharger gives 50 after a 5 s use. | LIKELY | https://arcraiders.wiki/wiki/Shield_Recharger , https://arcraiders.wiki/wiki/Surge_Shield_Recharger (read 2026-11-06)
- Used To The Weight is the Conditioning starting node and reduces the shield slowdown. | LIKELY | [WIKI-SK]

## Doors, breaching, locked rooms
- Breaching a door takes about 10 s, or about 7 s with Proficient Pryer maxed (about +5% per rank, +10% on the last). | LIKELY (player-tested) | [WIKI-SK]
- Security Lockers need the Security Breach end skill (Conditioning), which also lets you open lockers someone else has already breached. | LIKELY | [WIKI-SK]
- You can't breach while downed, and opening your inventory cancels a breach. | CONFIRMED | Official 1.7.0, 2025-12-16 [ST]
- Some locked-room quests now have spare keys you can find. | CONFIRMED | Official 1.7.0, 2025-12-16 [ST]
- Keys stay in your Safe Pocket when you use "Unload backpack". | CONFIRMED | Official 1.7.0, 2025-12-16 [ST]

## Death, what you lose, Safe Pocket, free loadout
- If you are knocked out Topside, you lose everything except what's in your Safe Pocket. | CONFIRMED | Embark FAQ https://id.embark.games/arc-raiders/support/faq/135-your-first-raid/ (updated 2025-10-01); allthings safe-pocket guide
- A knocked-out Raider's weapons lose 15% durability (was 30%, changed 1.26.0). Augments lose 30 durability on a knockout. | CONFIRMED (weapons) / LIKELY (augments) | Official 1.26.0, 2026-04-28 [ST]; https://arcraiders.wiki/wiki/Augments
- The Free Loadout augment has no Safe Pocket, 14 slots and a 35 weight limit. Going in with nothing still gives 10 slots and 1 Safe Pocket. | CONFIRMED (no safe pocket) / LIKELY (numbers) | https://arcraiders.wiki/wiki/Free_Loadout ; https://allthings.how/arc-raiders-free-loadouts-how-they-work-and-when-to-use-them
- Custom-loadout players are more likely to get fresh servers; free-loadout players more often join rounds already in progress. | CONFIRMED | Official 1.22.0, https://arcraiders.com/news/patch-notes-1-22-0 (2026-03-31)
- Safe Pockets range from 1 to 3 slots depending on augment, and weapons only fit with Looting Mk.3 (Safekeeper). | CONFIRMED | Official notes: fixed exploit putting weapons in Safe Pocket "without the Safekeeper Augment" (1.18.0, 2026-03-03) [ST]; https://arcraiders.wiki/wiki/Looting_Mk._3_(Safekeeper)
- No gear insurance system was found in any official note or guide. | LIKELY (absence of evidence) | Steam feed read in full 2025-10-30 to 2026-11-05

## Proximity chat and other Raiders
- Teaming up with strangers you meet over proximity chat is allowed. Pre-arranged teaming groups are detected and kept apart in matchmaking. | CONFIRMED | Official "An Update on Anti-Cheat", 2026-09-30 [ST]
- Proximity chat has an optional voice-changer (Raider Voice), improved in 2.0. | CONFIRMED | [2.0] 2026-10-08; Official 1.4.0, 2025-11-27
- Embark's own figures: spoiled raids from teaming fell from about 1 in 50 to 1 in 300. | CONFIRMED | Official, 2026-09-30 [ST]

## Matchmaking (Embark's statements)
- Matchmaking puts you with players of a similar playstyle on a sliding scale; there are no separate "friendly" and "aggressive" lobbies, and similar players are only more likely, never guaranteed. | CONFIRMED | Official "Notes on The Matchmaking System", 2026-05-20 [ST]
- Your loadout, your end-of-round survey, looting downed players and crossplay do not change your playstyle score. Patches don't reset it, and every squad member counts the same as the leader. | CONFIRMED | Official, 2026-05-20 [ST]
- One kill doesn't flip you into PvP lobbies; the score shifts slowly. Rounds where little happens (like spawning and surrendering) count less. | CONFIRMED | Official, 2026-05-20 [ST]
- Solo, duo and trio playstyles are tracked separately, and equal squad sizes are matched where possible. | CONFIRMED | Official 1.36.0, 2026-07-07; 2026-05-20 [ST]
- A test started 2026-07-28 where self-defense (including knocking out whoever shot first) doesn't count at all; only fights you start do, and third-partying counts as starting one. No later post says whether it was kept. | LIKELY (test status unknown) | Official 1.39.0, 2026-07-28 [ST]
- The "Solo vs Squads" toggle (added 1.17.0 for level 40+) was removed in 2.0. | STALE for any guide still mentioning it | [2.0] 2026-10-08; Official 1.17.0, 2026-02-24
- The PvE toggle is not live. It was a beta from 2026-10-13 to 10-20, with a decision promised in November. | CONFIRMED | Official 2.1.0 / 2.2.0, 2026-10-13/20 [ST]

## Quests
- Since 2.0, quests and quest objectives give XP. | CONFIRMED | [2.0] 2026-10-08 (PC Gamer's "missions don't give XP" line is STALE)
- 2.0 added 22 quests, and five older ones now need the new Camera for photo steps: The League, Reduced to Rubble, Life of a Pharmacist, Battening Down, Groundbreaking. | CONFIRMED | [2.0] 2026-10-08
- If you don't extract, you can lose quest progress; the progress bar shows what's at risk. | CONFIRMED | Official 1.22.0, 2026-03-31
- Never sell or lose quest items. Known issue: losing one, such as the Equatorial Sundial for "Clean Hands", can make the quest impossible to finish. Keep them in your Safe Pocket. | CONFIRMED | Official 2.2.0/2.3.0 known issues, 2026-10-20/27 [ST]
- Items already in your stash can complete "collect" steps, and they are not used up when you turn the quest in. | LIKELY | https://allthings.how/all-arc-raiders-quests-and-rewards-objectives-traders-and-loot/ (2025-11-04); a fandom wiki quote seen in search results only (could not be opened)
- Not every quest accepts stash items: some must be looted or finished within one round. Unexpected Initiative and Broken Monument ("In One Round") are reported examples. | LIKELY | allthings Broken Monument guide; fragster Unexpected Initiative guide (dates not shown in search results)
- "To the Skies": finish all installations without extracting partway, because a partial extract can bug it. Since 2.2 it can be completed on any map. | CONFIRMED | Official 2.2.0 fix and known issue, 2026-10-20 [ST]
- "When Everything Falls": the "Photograph Port A" step may not complete (known issue). | CONFIRMED | Official 2.2.0/2.3.0, 2026-10-20/27
- "Belly of the Beast": the optional step needs the exact item named in its text. | CONFIRMED | Official 2.2.0, 2026-10-20
- "Picking Up the Pieces" now counts visiting loot-icon areas on Pendola Pass (fixed 2.1). | CONFIRMED | Official 2.1.0, 2026-10-13
- "Worth Your Salt": the final stage works even if someone else did it in the same raid (fixed). | CONFIRMED | Official 1.18.0, 2026-03-03
- Some quest steps are just an interaction: look for yellow glows or icons. Quest pickups show an icon on the ground. | CONFIRMED | Official 1.7.0 (quest pickups show icons), 2025-12-16; allthings 2025-11-04
- Since 1.45, an Expedition no longer resets your quests unless you choose to. | CONFIRMED | Official 1.45.0 / 5th Expedition post, 2026-08-21 to 09-08
- Quest chain order: no source gives one. The wiki builds its quest tree from a widget, and there is no API text to check against. | (open) | https://arcraiders.wiki/wiki/Quests

## Skill tree 2.0 (Frozen Trail)
- Three branches (Conditioning, Mobility, Survival) with 16 skills each, 48 in total. | CONFIRMED | [WIKI-SK] tables; https://allthings.how/arc-raiders-2-0-skill-tree-all-48-skills-and-every-choice-node-explained/ (updated 2026-10-09)
- In the middle of each branch you pick one of two skills. At the end you take up to two of three, once you have spent enough points. You can only change picks by resetting the tree. | CONFIRMED | [2.0] 2026-10-08
- Gates are points spent in that branch: 10 for the choice node, 30 for the first end skill, 40 for the second. Embark's notes give no numbers. | CONFIRMED | [WIKI-SK]; allthings 2026-10-09; https://thearcraiders.com/guides/skill-tree/ (2026-10-08/09); https://arcraidersultimate.com/blog/arc-raiders-skill-tree/ (2026-10-11, citing Dot Esports)
- The old tree's gates (15 and 36) are STALE; they appear on the wiki's Dodge Roll and Sprint Dodge Roll pages and in older guides. | STALE | thearcraiders.com 2026-10-09; wiki Sprint_Dodge_Roll
- Open question: does the point spent on the first end skill count toward the 40 gate? Not settled. arcraidersultimate says one end skill costs 30 and both cost 40. Another guide (in search results) says 31 and 41, and thearcraiders' sample build uses 41. The wiki and allthings don't say. | DISPUTED | sources above; needs an in-game check
- A reset costs 2,000 Coins per spent point and clears the whole tree; you can't reset one branch or skill. | CONFIRMED | Official 1.7.0, 2025-12-16 [ST]; [WIKI-SK]; SCUF guide (updated 2026-09-23)
- A full 76-point reset costs about 152,000 Coins. | LIKELY (arithmetic on the 76 total) | thearcraiders.com; arcraidersultimate.com
- You get 76 points from leveling to the cap of 75. Caveat: the wiki's XP table runs from level 0 to 75, which is 75 level-ups, and some older guides say 75 points. | LIKELY | [WIKI-SK]; SCUF guide 2026-09-23; thearcraiders.com 2026-10-09
- Expeditions give up to 5 bonus points each, capped at 15 in total, with catch-up for points you missed. That puts the maximum at about 91. Expeditions are paused until early 2027. | CONFIRMED | Official "The 5th Expedition", 2026-08-21 [ST]
- 2.0 reportedly refunded all points for a free respec. Not in the patch notes. | LIKELY (weak) | thearcraiders.com 2026-10-09; arcraidersultimate.com says unconfirmed
- Since 2.2, a node's tooltip shows how many ranks it has, and the button reads "Spend Point". | CONFIRMED | Official 2.2.0, 2026-10-20
- 2.0 removed 13 skills: Blast-Born, Survivor's Stamina, Unburdened Roll, Effortless Swing, Sky-Clearing Swing, Flyswatter, Carry The Momentum, Calming Stroll, Heroic Leap, Revitalizing Squat, Suffer In Silence, Good As New, One Raider's Scraps. Any build that uses them is STALE (PC Gamer 2025-12-02, GamesRadar 2026-05-07, PCGamesN 2026-06-24). | CONFIRMED | [WIKI-SK] history; [2.0]
- Open question: the 2.1 notes mention a "Field Salvaging" skill that isn't on the wiki's list (it may be another name for Road Warrior). | (open) | Official 2.1.0, 2026-10-13

### Recommended skills (2.0)
- Youthful Lungs is the 1-point Mobility starting node, and most builds take it first. | LIKELY | allthings 2026-10-09; https://arcraiders.uk/skills/best-skill-tree/ (2026)
- Popular end skills are Security Breach (for looters), Sprinting on Fumes (for running/PvP), and Road Warrior or Field Repair (for PvE). The guides recommending them are small sites with untested builds. | LIKELY | arcraiders.uk; arcraidersultimate.com 2026-10-11; thearcraiders.com 2026-10-09
- Before 2.0, guides agreed on Marathon Runner, Looter's Instincts, Broad Shoulders and In-Round Crafting. All still exist, but the layout and costs changed. | STALE (pre-2.0 tree) | PC Gamer 2025-12-02; GamesRadar 2026-05-07; PCGamesN 2026-06-24

### Per-rank values
- Fight or Flight: about 25% more stamina per rank (2.0). Stubborn Mule: about 33% more stamina regen (2.0). | CONFIRMED | [2.0]
- Field Repair repairs gear in-raid up to 30% durability. | CONFIRMED | [WIKI-SK]; allthings 2026-10-09 (in-game text)
- Broad Shoulders: +2 max weight per rank, up to +10. | LIKELY | [WIKI-SK]; allthings 2026-10-09
- Fight or Flight restores 5 stamina at rank 1, up to 25 at rank 5. | LIKELY (one source) | allthings 2026-10-09
- Sprinting on Fumes: about 36 s of sprint gives about 5 s of recovery, and about 72 s gives about 14 s. | LIKELY (player-tested) | [WIKI-SK]
- One site's table: Effortless Roll -7% to -34% roll stamina; Marathon Runner -3% to -15%; Nimble Climber +3% to +15%; Silent Scavenger -10% to -50% looting noise; Looter's Luck 6.6% to 33%; Swift Start +2.5 s to +10 s; Stylish Landing +50 cm to +150 cm; Sure-Footed Scrambler +0.5 s to +2 s. | LIKELY (unverified, single source, don't ship as fact) | thearcraiders.com 2026-10-09
