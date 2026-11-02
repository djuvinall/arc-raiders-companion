<script lang="ts">
  import { collectionList, toggleFound, type Collection } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  let collections = $state<Collection[]>([]);
  let activeId = $state("blueprints");
  let hideFound = $state(false);
  let search = $state("");

  collectionList().then((c) => (collections = c));

  const active = $derived(collections.find((c) => c.id === activeId));
  const entries = $derived(
    active?.entries.filter((e) =>
      (!hideFound || !e.found) && (!search || e.name.toLowerCase().includes(search.toLowerCase())),
    ) ?? [],
  );

  async function toggle(id: string) {
    const found = await toggleFound(id);
    const collection = collections.find((c) => c.id === activeId)!;
    collection.entries.find((e) => e.id === id)!.found = found;
    collection.found += found ? 1 : -1;
  }
</script>

<div class="page">
  <div class="cards">
    {#each collections as c (c.id)}
      <button class="card" class:active={c.id === activeId} onclick={() => (activeId = c.id)}>
        <strong>{c.title}</strong>
        <span class="count">{c.found}/{c.total}</span>
        <span class="bar"><i style:width="{c.total ? (c.found / c.total) * 100 : 0}%"></i></span>
      </button>
    {/each}
  </div>

  <div class="filters">
    <input class="field" placeholder="Search" bind:value={search} />
    <label><input type="checkbox" bind:checked={hideFound} /> Hide found</label>
    <span class="dim">Click a tile to mark it found.</span>
  </div>

  <div class="grid">
    {#each entries as entry (entry.id)}
      <div class="tile" class:found={entry.found}>
        <button class:blueprint={activeId === "blueprints"} aria-label="{entry.name}, {entry.found ? 'found' : 'not found'}" title={entry.name} onclick={() => toggle(entry.id)}>
          <Icon url={entry.icon} name={entry.name} size={64} />
          {#if entry.rarity}<span class="wedge" style:background="var(--rarity-{entry.rarity})"></span>{/if}
          {#if entry.found}<span class="check">✓</span>{/if}
        </button>
        <a href="#/item/{entry.id}">{entry.name}</a>
      </div>
    {/each}
  </div>
</div>

<style>
  .page {
    padding: 14px 20px;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
  }
  .card {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    padding: 14px 16px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
  }
  .card.active {
    border-color: var(--cream);
  }
  .card strong {
    font-family: var(--font-head);
    font-size: 19px;
    letter-spacing: 0.05em;
    text-align: left;
    text-transform: uppercase;
  }
  .count {
    color: var(--teal);
  }
  .card .bar {
    grid-column: span 2;
  }
  .filters {
    display: flex;
    align-items: center;
    gap: 14px;
    margin: 12px 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, 112px);
    gap: 10px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 12px;
  }
  .tile button {
    position: relative;
    display: grid;
    place-items: center;
    width: 112px;
    height: 96px;
    padding: 0;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
    cursor: pointer;
    opacity: 0.6;
  }
  .tile button.blueprint {
    background: var(--blueprint-bg);
  }
  .tile.found button {
    opacity: 1;
    border-color: var(--teal);
  }
  .wedge {
    position: absolute;
    left: 0;
    bottom: 0;
    width: 28px;
    height: 28px;
    clip-path: polygon(0 0, 100% 100%, 0 100%);
  }
  .check {
    position: absolute;
    top: 3px;
    right: 3px;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--teal);
    color: #0b0b0f;
    font-size: 13px;
  }
  .tile a {
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
