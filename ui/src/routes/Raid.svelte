<script lang="ts">
  import {
    liveEvents, linkHref, mapList, mapPois, questList, routePlan,
    type EventsView, type GameMap, type Poi, type QuestRow, type RoutePlan,
  } from "../lib/api";
  import TileMap, { type MapPoint } from "../lib/components/TileMap.svelte";

  let { mapId }: { mapId?: string } = $props();

  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone ?? "";
  const guess = zone.startsWith("America") ? "north-america" : /^(Australia|Pacific)/.test(zone) ? "oceania" : "europe";
  const region = (() => {
    try {
      return localStorage.getItem("event-region") ?? guess;
    } catch {
      return guess;
    }
  })();

  let maps = $state<GameMap[]>([]);
  let plan = $state<RoutePlan | null>(null);
  let quests = $state<QuestRow[]>([]);
  let events = $state<EventsView | null>(null);
  // Routes remount on a new address, so the first value is the only one.
  // svelte-ignore state_referenced_locally
  let chosen = $state<string | null>(mapId ?? null);
  let pois = $state<Poi[]>([]);
  let selected = $state<MapPoint | null>(null);
  let floor = $state(0);

  Promise.all([mapList(), routePlan(), questList(), liveEvents(region)]).then(([m, p, q, e]) => {
    [maps, plan, quests, events] = [m, p, q, e];
    pick(chosen ?? rows[0]?.map.id ?? m[0].id);
  });

  async function pick(id: string) {
    chosen = id;
    floor = 0;
    selected = null;
    pois = await mapPois(id);
  }

  const working = $derived(quests.filter((q) => q.status === "active" || (q.status === "needed" && q.open)));
  const open = (q: QuestRow, map: string) =>
    q.quest.objectives.filter((o) => o.kind === "atomic" && o.maps.includes(map) && !q.done_objectives.includes(o.id));
  const live = (id: string) => events?.maps.find((m) => m.map_id === id);
  // Quests come first: an objective only happens on its own map, loot is usually in several.
  const rows = $derived(
    maps.map((map) => ({
      map,
      quests: working.filter((q) => open(q, map.id).length).length,
      items: plan?.covers.find((c) => c.map_id === map.id)?.covered_items.length ?? 0,
      events: live(map.id)?.current.length ?? 0,
    })).sort((a, b) => b.quests - a.quests || b.items - a.items || b.events - a.events),
  );
  const current = $derived(maps.find((m) => m.id === chosen) ?? null);
  const cover = $derived(plan?.covers.find((c) => c.map_id === chosen) ?? null);
  const here = $derived(chosen ? working.filter((q) => open(q, chosen!).length) : []);
  const anywhere = $derived(working.filter((q) => !q.maps.length));
  const route = $derived(cover?.pois.map((p) => p.id) ?? []);
  const floors = $derived(current?.image.layers ?? []);
  const onFloor = (p: Poi) => !floors.length || p.zlayers === null || (p.zlayers & floors[floor].mask) !== 0;
  const hereIds = $derived(new Set(here.map((q) => q.quest.id)));
  const points = $derived.by((): MapPoint[] => {
    const out: MapPoint[] = pois.filter((p) => onFloor(p) && (p.kind === "extract" || (!!p.quest_id && hereIds.has(p.quest_id))))
      .map((p) => p.quest_id
        ? { key: p.id, x: p.x, y: p.y, icon: "quests", group: "quests", title: `Quest: ${quests.find((q) => q.quest.id === p.quest_id)?.quest.name ?? p.name}` }
        : { key: p.id, x: p.x, y: p.y, icon: "locations/extraction", group: "locations", title: p.name, label: p.name });
    cover?.pois.forEach((stop, i) => out.push({
      key: `s${stop.id}`, x: stop.x, y: stop.y, icon: "route", group: "route", number: i + 1, title: `Stop ${i + 1}: ${stop.name}`,
    }));
    return out;
  });
  const itemName = (id: string) => plan?.needed.find((n) => n.item.id === id)?.item.name ?? id;
</script>

<div class="page">
  <aside class="panel">
    <h3 class="section-title">Where to go</h3>
    {#each rows as row, i (row.map.id)}
      <button class="map" class:on={row.map.id === chosen} onclick={() => pick(row.map.id)}>
        <strong>{row.map.name}{i === 0 && (row.quests || row.items) ? " · best" : ""}</strong>
        <span class="dim small">{row.quests} {row.quests === 1 ? "quest" : "quests"}, {row.items} items</span>
        {#each live(row.map.id)?.current ?? [] as event}<span class="small live">{event.name}</span>{/each}
      </button>
    {/each}
    <a class="small" href="#/quests">Track quests</a>
  </aside>

  <div class="center">
    {#if floors.length}
      <div class="floors" role="group" aria-label="Floor">
        {#each floors as layer, i (layer.name)}
          <button class:on={i === floor} onclick={() => (floor = i)}>{layer.name}</button>
        {/each}
      </div>
    {/if}
    {#if current}
      <TileMap map={current} url={floors[floor]?.url ?? current.image.url} {points} {selected} onselect={(p) => (selected = p)} />
    {/if}
    <p class="dim small">Numbers are loot stops, pink pins are quest objectives, yellow pins are exits.</p>
  </div>

  <aside class="panel side">
    {#if selected}<p class="small"><strong>{selected.title}</strong></p>{/if}
    <h3 class="section-title">Quests here</h3>
    {#each here as q (q.quest.id)}
      <a href="#/quests/{q.quest.id}"><strong>{q.quest.name}</strong></a>
      {#each open(q, chosen!) as objective (objective.id)}<p class="small dim">{objective.description}</p>{/each}
    {:else}
      <p class="dim small">No tracked quests on this map.</p>
    {/each}
    {#if anywhere.length}
      <h3 class="section-title">Any map</h3>
      {#each anywhere as q (q.quest.id)}<a class="small" href="#/quests/{q.quest.id}">{q.quest.name}</a>{/each}
    {/if}

    <h3 class="section-title">Loot stops</h3>
    {#each cover?.pois ?? [] as stop, i (stop.id)}
      <p class="small"><strong>{i + 1}. {stop.name}:</strong> {stop.covers.map(itemName).join(", ")}</p>
    {:else}
      <p class="dim small">Track items or projects to get stops.</p>
    {/each}
    {#if cover?.uncovered.length}
      <p class="dim small">Not on this map: {cover.uncovered.map(itemName).join(", ")}</p>
    {/if}

    <h3 class="section-title">Bring back ({plan?.needed.length ?? 0})</h3>
    {#each plan?.needed.slice(0, 12) ?? [] as need (need.item.id)}
      <a class="small" href={linkHref(need.item)}>{need.qty}× {need.item.name}</a>
    {/each}
  </aside>
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: 230px minmax(0, 1fr) 280px;
    gap: 10px;
    padding: 10px 12px;
    height: 100%;
    min-height: 0;
  }
  aside {
    overflow: auto;
    min-height: 0;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .center {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    min-width: 0;
  }
  .center :global(.map) {
    flex: 1;
    min-height: 0;
  }
  .map {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 6px 8px;
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .map.on {
    background: var(--panel-2);
    border-color: var(--border);
  }
  .floors {
    display: flex;
    gap: 2px;
  }
  .floors button {
    padding: 3px 12px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  .floors .on {
    background: var(--cream);
    color: var(--on-cream);
  }
  .live {
    color: var(--yellow);
  }
  .small {
    font-size: 13px;
    margin: 0;
  }
  .section-title {
    margin: 10px 0 2px;
  }
  @media (max-width: 1160px) {
    .page {
      grid-template-columns: 190px minmax(0, 1fr) 220px;
    }
  }
</style>
