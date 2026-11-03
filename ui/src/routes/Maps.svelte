<script lang="ts">
  import {
    iconSrc, liveEvents, mapList, mapPois, questList, routePlan,
    type EventsView, type GameMap, type MapCover, type Poi, type RoutePlan,
  } from "../lib/api";
  import TileMap from "../lib/components/TileMap.svelte";

  let { mapId }: { mapId?: string } = $props();

  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone ?? "";
  const REGIONS = ["europe", "north-america", "oceania"];
  const guess = zone.startsWith("America") ? "north-america" : /^(Australia|Pacific)/.test(zone) ? "oceania" : "europe";

  let maps = $state<GameMap[]>([]);
  let current = $state<GameMap | null>(null);
  let pois = $state<Poi[]>([]);
  let events = $state<EventsView | null>(null);
  let plan = $state<RoutePlan | null>(null);
  let region = $state(remembered() ?? guess);
  let now = $state(Date.now());
  let selected = $state<Poi | null>(null);
  let kinds = $state<Record<string, boolean>>({ cache: true, extract: true, landmark: false });
  let area = $state("");
  let search = $state("");
  let floor = $state(0);
  let tracked = $state(new Set<string>());
  let questPins = $state(true);

  function remembered() {
    try {
      return localStorage.getItem("event-region");
    } catch {
      return null;
    }
  }

  mapList().then((all) => {
    maps = all;
    open(all.find((m) => m.id === mapId) ?? all[0]);
  });
  routePlan().then((p) => (plan = p));
  questList().then((rows) => (tracked = new Set(rows.filter((r) => r.status === "active" || r.status === "needed").map((r) => r.quest.id))));

  $effect(() => {
    try {
      localStorage.setItem("event-region", region);
    } catch {
      // Remembering the region is only a convenience.
    }
    liveEvents(region).then((e) => (events = e));
    const tick = setInterval(() => (now = Date.now()), 30000);
    return () => clearInterval(tick);
  });

  async function open(map: GameMap) {
    current = map;
    floor = 0;
    selected = null;
    pois = await mapPois(map.id);
  }

  const cover = $derived(plan?.covers.find((c) => c.map_id === current?.id) ?? null);
  const route = $derived(cover?.pois.map((p) => p.id) ?? []);
  const areas = $derived([...new Set(pois.flatMap((p) => p.loot_areas))].sort());
  const floors = $derived(current?.image.layers ?? []);
  const onFloor = (p: Poi) => !floors.length || p.zlayers === null || (p.zlayers & floors[floor].mask) !== 0;
  const shown = $derived(
    pois.filter((p) =>
      route.includes(p.id) || (questPins && onFloor(p) && !!p.quest_id && tracked.has(p.quest_id)) || (
        kinds[p.kind] && onFloor(p) &&
        (!area || p.loot_areas.includes(area)) &&
        (!search || p.name.toLowerCase().includes(search.toLowerCase()))
      ),
    ),
  );

  const itemName = (id: string) => plan?.needed.find((n) => n.item.id === id)?.item.name ?? id;
  const when = (ms: number) => {
    const minutes = Math.round((ms - now) / 60000);
    if (minutes < 60) return `${Math.max(0, minutes)} min`;
    return `${Math.floor(minutes / 60)} h ${minutes % 60} min`;
  };
  const eventsFor = (id: string) => events?.maps.find((m) => m.map_id === id);
  // Equal coverage goes to the map with an event running, for more to do on the same trip.
  const liveCount = (id: string) => eventsFor(id)?.current.length ?? 0;
  const ranked = $derived(
    [...(plan?.covers ?? [])].sort((a, b) =>
      b.covered_items.length - a.covered_items.length || liveCount(b.map_id) - liveCount(a.map_id) || a.pois.length - b.pois.length),
  );
  const haveNeeds = $derived((plan?.needed.length ?? 0) > 0);
  const rankLabel = (c: MapCover) => `${c.covered_items.length}/${plan?.needed.length ?? 0} items, ${c.pois.length} stops`;
</script>

<div class="page">
  <aside class="left">
    <section class="panel">
      <div class="region">
        <h3 class="section-title">Maps and events</h3>
        <select class="field" aria-label="Event region" bind:value={region}>
          {#each REGIONS as r}<option value={r}>{r.replace("-", " ")}</option>{/each}
        </select>
      </div>
      {#each maps as map (map.id)}
        {@const live = eventsFor(map.id)}
        <button class="map-row" class:active={current?.id === map.id} onclick={() => open(map)}>
          <strong>{map.name}</strong>
          {#each live?.current ?? [] as event}
            <span class="event now">
              {#if event.icon}<img src={iconSrc(event.icon)} alt="" width="16" height="16" />{/if}
              {event.name} · ends in {when(event.endTime)}
            </span>
          {/each}
          {#if live?.next}
            <span class="event">Next: {live.next.name} in {when(live.next.startTime)}</span>
          {/if}
        </button>
      {/each}
      {#if events && events.fetched_ms === null}
        <p class="dim small">No event schedule cached yet; connect once to fetch it.</p>
      {:else if events?.stale}
        <p class="warning small">Offline: showing the schedule from {new Date(events.fetched_ms ?? 0).toLocaleString()}.</p>
      {/if}
    </section>
  </aside>

  <div class="center">
    <div class="filters">
      {#if floors.length}
        <div class="floors" role="group" aria-label="Floor">
          {#each floors as layer, i (layer.name)}
            <button class:on={i === floor} aria-pressed={i === floor} onclick={() => { floor = i; selected = null; }}>{layer.name}</button>
          {/each}
        </div>
      {/if}
      {#each Object.keys(kinds) as kind}
        <label><input type="checkbox" bind:checked={kinds[kind]} /> {kind}</label>
      {/each}
      <label><input type="checkbox" bind:checked={questPins} /> Tracked quests</label>
      <select class="field" aria-label="Loot area" bind:value={area}>
        <option value="">All loot areas</option>
        {#each areas as a}<option value={a}>{a.replace("-", " ")}</option>{/each}
      </select>
      <input class="field" placeholder="Search places" bind:value={search} />
      <span class="dim small">{shown.length} shown</span>
    </div>
    {#if current}
      <TileMap map={current} url={floors[floor]?.url ?? current.image.url} pois={shown} {route} {selected} onselect={(p) => (selected = p)} />
    {/if}
  </div>

  <aside class="right">
    {#if selected}
      <section class="panel">
        <h3>{selected.name}</h3>
        <p class="dim small">{selected.kind}</p>
        {#if selected.loot_areas.length}<p class="small">Loot: {selected.loot_areas.join(", ")}</p>{/if}
      </section>
    {/if}

    <section class="panel">
      <h3 class="section-title">Needed gear</h3>
      {#if haveNeeds}
        <p class="small">{plan?.needed.length} items still to find.</p>
        {#each ranked as c (c.map_id)}
          <button class="cover" class:active={c.map_id === current?.id} onclick={() => { const m = maps.find((x) => x.id === c.map_id); if (m) open(m); }}>
            <strong>{c.map_name}</strong>
            <span class="dim small">{rankLabel(c)}</span>
            {#each eventsFor(c.map_id)?.current ?? [] as event}<span class="small live">{event.name}</span>{/each}
          </button>
        {/each}
        {#if cover}
          <h3 class="section-title stops">Stops on {cover.map_name}</h3>
          {#each cover.pois as poi, i (poi.id)}
            <p class="small"><b>{i + 1}.</b> {poi.name}: {poi.covers.map(itemName).join(", ")}</p>
          {/each}
          {#if cover.uncovered.length}
            <p class="dim small">Not found in any loot area here: {cover.uncovered.map(itemName).join(", ")}</p>
          {/if}
        {/if}
      {:else}
        <p class="dim small">
          Track items, quests or projects and this ranks the maps that cover what you still need.
        </p>
      {/if}
      <p class="dim small">Stops are a set of places, not a walking path.</p>
    </section>
  </aside>
</div>

<style>
  .live {
    display: block;
    color: var(--yellow);
  }
  .floors {
    display: flex;
    gap: 2px;
  }
  .floors button {
    padding: 4px 14px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  .floors .on {
    border-color: var(--cream);
    background: var(--cream);
    color: var(--on-cream);
  }
  .page {
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr) 280px;
    gap: 12px;
    padding: 12px 16px;
    height: 100%;
    min-height: 0;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow: auto;
    min-height: 0;
  }
  .center {
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 8px;
    min-height: 0;
  }
  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }
  .filters label {
    text-transform: capitalize;
    color: var(--text-dim);
  }
  .region {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  .region .section-title {
    margin: 0;
  }
  .map-row,
  .cover {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 8px;
    background: none;
    border: 1px solid transparent;
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .map-row.active,
  .cover.active {
    background: var(--panel-2);
    border-color: var(--border);
  }
  .event {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--text-dim);
    font-size: 13px;
  }
  .event.now {
    color: var(--yellow);
  }
  .small {
    font-size: 13px;
    margin: 4px 0 0;
  }
  .warning {
    color: var(--yellow);
  }
  .stops {
    margin-top: 12px;
  }
  @media (max-width: 1240px) {
    .page {
      grid-template-columns: 230px minmax(0, 1fr) 220px;
    }
  }
</style>
