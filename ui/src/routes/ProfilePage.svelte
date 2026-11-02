<script lang="ts">
  import {
    exportProfile, getProfile, importProfile, linkHref, listItems, routePlan, toggleTrackedItem,
    type ItemSummary, type Profile, type RoutePlan,
  } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  let profile = $state<Profile | null>(null);
  let plan = $state<RoutePlan | null>(null);
  let items = $state<ItemSummary[]>([]);
  let text = $state("");
  let message = $state("");

  const load = async () => {
    [profile, plan] = await Promise.all([getProfile(), routePlan()]);
  };
  load();
  listItems().then((all) => (items = all));

  const summary = (id: string) => items.find((i) => i.id === id);

  async function untrack(id: string) {
    await toggleTrackedItem(id);
    await load();
  }

  async function doExport() {
    text = await exportProfile();
    await navigator.clipboard?.writeText(text).catch(() => {});
    message = "Profile JSON is below and copied to the clipboard. Save it anywhere.";
  }

  async function doImport() {
    try {
      await importProfile(text);
      await load();
      message = "Profile imported. It replaced what was here.";
    } catch (e) {
      message = String(e);
    }
  }
</script>

<div class="page">
  <section class="panel">
    <h3 class="section-title">Tracked items</h3>
    {#each profile?.tracked_item_ids ?? [] as id (id)}
      <div class="row">
        <Icon url={summary(id)?.icon ?? null} name={summary(id)?.name ?? id} size={32} />
        <a href="#/item/{id}">{summary(id)?.name ?? id}</a>
        <button class="link" onclick={() => untrack(id)}>Remove</button>
      </div>
    {:else}
      <p class="dim">Use Track resources on an item page.</p>
    {/each}
  </section>

  <section class="panel">
    <h3 class="section-title">Still needed</h3>
    {#each plan?.needed ?? [] as need (need.item.id)}
      <div class="row">
        <Icon url={need.item.icon} name={need.item.name} size={32} />
        <a href={linkHref(need.item)}>{need.qty}× {need.item.name}</a>
        <span class="dim small">{need.reasons.join(", ")}</span>
      </div>
    {:else}
      <p class="dim">Nothing outstanding from tracked items, quests or projects.</p>
    {/each}
    {#if plan?.needed.length}<a class="button" href="#/maps">Rank maps for this</a>{/if}
  </section>

  <section class="panel wide">
    <h3 class="section-title">Backup</h3>
    <p class="dim small">
      Your stash, skill plans, weapon builds and progress live in profile.json in this app's data
      folder. Export copies them as JSON; import replaces everything here with pasted JSON.
    </p>
    <textarea class="field" rows="6" aria-label="Profile JSON" bind:value={text}></textarea>
    <div class="actions">
      <button class="button" onclick={doExport}>Export</button>
      <button class="button dark" disabled={!text.trim()} onclick={doImport}>Import</button>
    </div>
    {#if message}<p class="message" role="status">{message}</p>{/if}
  </section>
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    padding: 14px 20px;
    align-items: start;
  }
  .wide {
    grid-column: span 2;
  }
  .row {
    display: grid;
    grid-template-columns: 32px 1fr auto;
    align-items: center;
    gap: 10px;
    padding: 3px 0;
  }
  .small {
    font-size: 13px;
  }
  .link {
    background: none;
    border: 0;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  textarea {
    width: 100%;
    font-size: 12px;
    resize: vertical;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
  .message {
    color: var(--teal);
  }
</style>
