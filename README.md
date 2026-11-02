# ARC Raiders Companion

An open-source, offline-first desktop companion for ARC Raiders: every item linked to its recipes, workbenches, quests, projects, loot areas and maps. Unofficial; not affiliated with Embark Studios.

Includes the item database and crafting trees, the 2.0 skill tree planner, weapon sheets and compare, maps with live events and route ranking, quest and project tracking, Codex collections, and patch notes. Phase 7 (the HollowDeck module) is a later project; `arc-core` is ready for it.

## Download

Get the Windows exe from the [latest release](https://github.com/djuvinall/arc-raiders-companion/releases/latest). There is nothing to install. It isn't code-signed, so Windows may show a SmartScreen warning.

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

See [docs/DATA_SOURCES.md](docs/DATA_SOURCES.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/DESIGN.md](docs/DESIGN.md).

## License

Code: MIT. Data in `data/`: CC BY-SA 4.0. Sources are credited in [data/ATTRIBUTION.md](data/ATTRIBUTION.md).
