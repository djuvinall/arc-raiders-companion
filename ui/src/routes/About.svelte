<script lang="ts">
  import type { Overview } from "../lib/api";

  let { info }: { info: Overview | null } = $props();
</script>

<div class="page">
  <h2>ARC Raiders Companion</h2>
  <p>
    An unofficial, open-source companion. Not affiliated with or endorsed by Embark Studios.
    ARC Raiders and its assets belong to their respective owners.
  </p>
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
    Your stash, plans and progress stay in <code>profile.json</code> on this computer. Nothing is read from the
    game and nothing is uploaded.
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
