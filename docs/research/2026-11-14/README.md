# In-game weapon check, 2026-11-14

Devon's screenshots of weapon cards, upgrade trees and stencils. These are first-hand and
outrank every source.

- Stat, modifier and slot fixes are in
  `dataset/overlays/corrections/ingame-2026-11-14-weapons.json` (43 corrections).
- Amplification trees, costs and open checks are in `weapons.json`. The app has no field
  for amplification effects yet, so nothing reads this file.

Main finding: the sources give every weapon level the level IV numbers. Each level's own
fire rate, stability, agility and range differ, so levels below IV are wrong unless a
correction covers them.
