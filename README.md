# ARC Raiders Companion

An open-source, offline-first desktop companion for ARC Raiders: every item linked to its recipes, workbenches, quests, projects, loot areas and maps. Unofficial; not affiliated with Embark Studios.

Includes keep-or-recycle advice, a raid plan, an ARC guide, an always-on-top compact window, the item database and crafting trees, the 2.0 skill tree planner, weapon sheets and compare, maps with live events and route ranking, quest and project tracking, Codex collections, and patch notes. Phase 7 (the HollowDeck module) is a later project; `arc-core` is ready for it.

## Download

Get the Windows installer from the [latest release](https://github.com/djuvinall/arc-raiders-companion/releases/latest). It installs for your user only, needs no admin rights, and updates itself from new releases. It isn't code-signed yet, so Windows may show a SmartScreen warning on the first install. See [docs/SIGNING.md](docs/SIGNING.md).

## Run from source

```bash
cd ui && npm install && npm run build
```

```bash
cargo run --release -p arc-tauri
```

## Data

`data/*.json` is the dataset; the app only reads it. To refresh it:

```bash
cargo run -p arc-ingest -- probe
```

```bash
cargo run -p arc-ingest -- sync
```

```bash
cargo run -p arc-ingest -- sync --write
```

`probe` snapshots the sources into `tests/fixtures`, `sync` shows what would change, and `--write` updates `data/` for review and commit. `cargo test` validates schema and references.

The app downloads newer data from this repository's `main` branch at launch, and uses it after a restart if it validates. The **refresh-data** workflow runs these steps on GitHub.

See [docs/RELEASING.md](docs/RELEASING.md), [docs/DATA_SOURCES.md](docs/DATA_SOURCES.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/DESIGN.md](docs/DESIGN.md).

## License

Code: MIT. Data in `data/`: CC BY-SA 4.0. Sources are credited in [data/ATTRIBUTION.md](data/ATTRIBUTION.md).
