<script lang="ts">
  import { appVersion, type Overview } from "../lib/api";
  import { checkForUpdates, updates } from "../lib/updates.svelte";

  let { info }: { info: Overview | null } = $props();
  let version = $state("");

  appVersion().then((v) => (version = v));
  const upToDate = $derived(updates.checked && !updates.checking && !updates.app && !updates.data && !updates.error);
</script>

<div class="page">
  <h2>ARC Raiders Companion</h2>
  <p>
    An unofficial, open-source companion. Not affiliated with or endorsed by Embark Studios.
    ARC Raiders and its assets belong to their respective owners.
  </p>
  <div class="version">
    <span>Version {version}</span>
    <button class="button dark" disabled={updates.checking} onclick={() => checkForUpdates(false)}>
      {updates.checking ? "Checking…" : "Check for updates"}
    </button>
    {#if upToDate}<span class="dim">Up to date.</span>{/if}
  </div>
  {#if info}
    <p class="dim">
      Dataset {info.dataset_version}: {info.item_count} items, {info.recipe_count} recipes.
      {info.game_patch ? `Reflects patch ${info.game_patch}.` : ""}
    </p>
  {/if}

  <h3 class="section-title">Data sources</h3>
  <ul>
    <li>
      <strong>MetaForge</strong> (<span class="url">https://metaforge.app/arc-raiders</span>):
      item stats, icons, quests, traders, live events, map tiles and map markers.
    </li>
    <li>
      <strong>ARC Raiders Wiki contributors</strong> (<span class="url">https://arcraiders.wiki</span>):
      recipes, recycling, weapon mods, projects, workbench upgrades and skill descriptions, under CC BY-SA 4.0.
    </li>
    <li>
      <strong>Steam news</strong> (<span class="url">https://store.steampowered.com/app/1808500</span>):
      patch notes, as published by Embark Studios.
    </li>
    <li>
      <strong>RaidTheory/arcraiders-data</strong> (<span class="url">https://github.com/RaidTheory/arcraiders-data</span>):
      fallback recipes and ARC drops, under the MIT license.
    </li>
  </ul>

  <h3 class="section-title">Licenses</h3>
  <p>Code: MIT. Dataset in <code>data/</code>: CC BY-SA 4.0. Icons and map tiles are fetched at runtime and cached locally.</p>

  <h3 class="section-title">Privacy</h3>
  <p>
    Your stash, plans, progress and stencil screenshots stay on this computer. Nothing is read from the
    game and nothing is uploaded. The app asks GitHub for new versions and item data when it starts.
  </p>
</div>

<style>
  .page {
    padding: 20px;
    max-width: 780px;
  }
  h2 {
    font-size: 30px;
    margin-bottom: 8px;
  }
  .version {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 12px 0;
  }
  .section-title {
    margin-top: 22px;
  }
  .dim {
    color: var(--text-dim);
  }
  .url {
    color: var(--teal);
    user-select: text;
  }
  li {
    margin-bottom: 8px;
  }
</style>
