# Releasing

## A new app version

1. Set the same version in `crates/arc-tauri/tauri.conf.json` and the root `Cargo.toml`.
2. Write the notes in `docs/releases/vX.Y.Z.md`.
3. Commit, then tag and push:

```bash
git tag vX.Y.Z
```

```bash
git push origin main vX.Y.Z
```

The **release** workflow tests, builds the installer, signs it for the updater, and publishes the release with `latest.json`. Every installed copy from v0.2.0 on then offers the update when it starts.

## New item data

Run the **refresh-data** workflow from the Actions tab of [arc-raiders-data](https://github.com/djuvinall/arc-raiders-data). It snapshots the sources, rebuilds `data/` there, runs the tests and commits. Installed apps pick the data up at their next launch, with no new release. It stops without committing if a source changed in a way the tests reject, or a new stencil has no entry in `overlays/stencils_overlay.json`.

Before a release, move the submodule to the newest data so the installer embeds it:

```bash
git submodule update --remote dataset
```

## The updater key

Updates are signed with a private key, stored at `%USERPROFILE%\.tauri\arc-companion.key` and in the `TAURI_SIGNING_PRIVATE_KEY` repository secret. The public half is in `tauri.conf.json`.

Keep an offline backup. If the key is lost, installed copies can't accept updates and everyone must reinstall by hand. If it leaks, someone with access to the release assets could ship an update, so replace it and publish a new release at once.

This key is separate from Windows code signing, covered in [SIGNING.md](SIGNING.md).
