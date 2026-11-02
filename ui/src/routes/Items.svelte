<script module lang="ts">
  // Lives outside the component so Back returns to the same filtered grid.
  const filters = $state({
    group: "all",
    search: "",
    rarity: "",
    tier: "",
    area: "",
  });
</script>

<script lang="ts">
  import { listItems, type ItemSummary, type Overview } from "../lib/api";
  import ItemTile from "../lib/components/ItemTile.svelte";
  import Rail, { GROUPS } from "../lib/components/Rail.svelte";

  let { info }: { info: Overview | null } = $props();
  let items = $state<ItemSummary[]>([]);
  let searchBox = $state<HTMLInputElement>();

  listItems().then((all) => (items = all));

  const visible = $derived.by(() => {
    const group = GROUPS.find((g) => g.id === filters.group);
    const query = filters.search.trim().toLowerCase();
    return items.filter((item) => {
      if (group?.categories.length && !group.categories.includes(item.category)) return false;
      if (filters.group === "all" && ["cosmetic", "furniture"].includes(item.category) && !query) return false;
      if (filters.rarity && item.rarity !== filters.rarity) return false;
      if (filters.tier && String(item.tier ?? "") !== filters.tier) return false;
      if (filters.area && !item.loot_areas.includes(filters.area)) return false;
      return !query || item.name.toLowerCase().includes(query);
    });
  });

  // One tile per gun, opening the lowest match, so "Tier III" lands on level III.
  const tiles = $derived.by(() => {
    const lead = new Map<string, ItemSummary>();
    for (const item of visible) {
      if (!item.family) continue;
      const held = lead.get(item.family);
      const rank = (i: ItemSummary) => [i.variant_of ? 1 : 0, i.tier ?? 0];
      const [a, b] = [rank(item), held ? rank(held) : [2, 0]];
      if (a[0] < b[0] || (a[0] === b[0] && a[1] < b[1])) lead.set(item.family, item);
    }
    return visible.filter((i) => !i.family || lead.get(i.family) === i)
      .map((i) => (i.family ? { ...i, name: i.family_name ?? i.name, tier: null } : i));
  });

  function onKey(event: KeyboardEvent) {
    if (event.key === "/" && document.activeElement !== searchBox) {
      event.preventDefault();
      searchBox?.focus();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="layout">
  <Rail active={filters.group} onselect={(id) => (filters.group = id)} />
  <section>
    <div class="filters">
      <input
        class="field search"
        type="search"
        placeholder="Search items"
        aria-label="Search items"
        bind:this={searchBox}
        bind:value={filters.search}
      />
      <select class="field" aria-label="Rarity" bind:value={filters.rarity}>
        <option value="">Any rarity</option>
        <option value="common">Common</option>
        <option value="uncommon">Uncommon</option>
        <option value="rare">Rare</option>
        <option value="epic">Epic</option>
        <option value="legendary">Legendary</option>
      </select>
      <select class="field" aria-label="Tier" bind:value={filters.tier}>
        <option value="">Any tier</option>
        <option value="1">Tier I</option>
        <option value="2">Tier II</option>
        <option value="3">Tier III</option>
        <option value="4">Tier IV</option>
      </select>
      <select class="field" aria-label="Loot area" bind:value={filters.area}>
        <option value="">Any loot area</option>
        {#each info?.loot_areas ?? [] as area}
          <option value={area.id}>{area.name}</option>
        {/each}
      </select>
      <span class="count">{tiles.length} items</span>
    </div>
    <div class="grid">
      {#each tiles as item (item.id)}
        <ItemTile {item} />
      {/each}
    </div>
    {#if items.length && !tiles.length}
      <p class="empty">No items match these filters.</p>
    {/if}
  </section>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: auto 1fr;
    min-height: 100%;
  }
  section {
    padding: 14px 20px 20px 6px;
    min-width: 0;
  }
  .filters {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    padding: 0 0 12px;
    background: linear-gradient(#11131c 80%, transparent);
  }
  .search {
    width: 240px;
  }
  .count {
    margin-left: auto;
    color: var(--text-dim);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, 96px);
    gap: 8px;
  }
  .empty {
    color: var(--text-dim);
  }
</style>
