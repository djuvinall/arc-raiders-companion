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

The dataset lives in [arc-raiders-data](https://github.com/djuvinall/arc-raiders-data), checked out here as a submodule at `dataset/`. Clone with `--recurse-submodules`, or run `git submodule update --init` after cloning. The app only reads it. To refresh it:

```bash
cargo run -p arc-ingest -- probe
```

```bash
cargo run -p arc-ingest -- sync
```

```bash
cargo run -p arc-ingest -- sync --write
```

`probe` snapshots the sources into `dataset/fixtures`, `sync` shows what would change, and `--write` updates `dataset/data` for review and commit there. `cargo test` validates schema and references.

The app downloads newer data from the data repository's `main` branch at launch, and uses it after a restart if it validates. The **refresh-data** workflow in that repository runs these steps on GitHub.

See [docs/RELEASING.md](docs/RELEASING.md), [DATA_SOURCES.md](https://github.com/djuvinall/arc-raiders-data/blob/main/DATA_SOURCES.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/DESIGN.md](docs/DESIGN.md).

## License

Code: MIT. The dataset lives in its own repository, [arc-raiders-data](https://github.com/djuvinall/arc-raiders-data), under CC BY-SA 4.0, with its sources credited in its ATTRIBUTION.md. It is checked out here as a submodule at `dataset/`.
