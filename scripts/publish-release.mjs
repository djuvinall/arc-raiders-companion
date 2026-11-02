// Renames the built installer, writes the latest.json the updater reads, and publishes
// the GitHub release. Run by the release workflow: node scripts/publish-release.mjs v0.2.0
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const REPO = "djuvinall/arc-raiders-companion";
const tag = process.argv[2];
const notesFile = `docs/releases/${tag}.md`;
const bundle = "target/release/bundle/nsis";

const setup = readdirSync(bundle).find((f) => f.endsWith("-setup.exe"));
if (!setup) throw new Error(`no installer in ${bundle}`);
const asset = `ARC-Raiders-Companion-${tag}-setup.exe`;
mkdirSync("dist-release", { recursive: true });
copyFileSync(join(bundle, setup), join("dist-release", asset));

const manifest = {
  version: tag.replace(/^v/, ""),
  notes: readFileSync(notesFile, "utf8"),
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: readFileSync(join(bundle, `${setup}.sig`), "utf8"),
      url: `https://github.com/${REPO}/releases/download/${tag}/${asset}`,
    },
  },
};
writeFileSync("dist-release/latest.json", JSON.stringify(manifest, null, 2));

execFileSync("gh", [
  "release", "create", tag, join("dist-release", asset), "dist-release/latest.json",
  "--repo", REPO, "--title", `ARC Raiders Companion ${tag}`, "--notes-file", notesFile, "--latest",
], { stdio: "inherit" });
