<script lang="ts">
  import {
    iconSrc, liveEvents, mapList, mapMarkers, mapPois, questList, routePlan,
    type EventsView, type GameMap, type MapCover, type MarkerSet, type Poi, type RoutePlan,
  } from "../lib/api";
  import TileMap, { type MapPoint } from "../lib/components/TileMap.svelte";
  import { GROUPS, iconUrl } from "../lib/mapIcons";

  let { mapId }: { mapId?: string } = $props();

  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone ?? "";
  const REGIONS = ["europe", "north-america", "oceania"];
  const guess = zone.startsWith("America") ? "north-america" : /^(Australia|Pacific)/.test(zone) ? "oceania" : "europe";
  // Hidden until switched on: very common, so they bury everything else. A group name
  // stands for every type in it.
  const QUIET = ["nature", "events/snow_pile", "events/security_camera", "locations/player_spawn", "containers/base_container"];
  const SAVED_KEY = "map-filters";

  interface Saved {
    hidden: string[];
    labels: boolean;
    allQuests: boolean;
    collapsed: string[];
    map: string | null;
    floors: Record<string, number>;
  }

  // Read once per visit; older saves held only the hidden list.
  const saved: Partial<Saved> = (() => {
    try {
      const raw = localStorage.getItem(SAVED_KEY);
      if (raw) return JSON.parse(raw);
      const old = localStorage.getItem("map-hidden");
      return old ? { hidden: JSON.parse(old) } : {};
    } catch {
      return {};
    }
  })();

  let maps = $state<GameMap[]>([]);
  let current = $state<GameMap | null>(null);
  let set = $state<MarkerSet | null>(null);
  let pois = $state<Poi[]>([]);
  let events = $state<EventsView | null>(null);
  let plan = $state<RoutePlan | null>(null);
  let region = $state(remembered("event-region") ?? guess);
  let now = $state(Date.now());
  let selected = $state<MapPoint | null>(null);
  let search = $state("");
  let floor = $state(0);
  let questNames = $state(new Map<string, string>());
  let tracked = $state(new Set<string>());
  let allQuests = $state(saved.allQuests ?? false);
  let showLabels = $state(saved.labels ?? true);
  // Holds marker type ids only once the types are known; group names are expanded on load.
  let hidden = $state(new Set<string>(saved.hidden ?? QUIET));
  let collapsed = $state(new Set<string>(saved.collapsed ?? []));
  let savedFloors = $state<Record<string, number>>(saved.floors ?? {});

  function remembered(key: string) {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  }

  function remember(key: string, value: string) {
    try {
      localStorage.setItem(key, value);
    } catch {
      // Remembering is only a convenience.
    }
  }

  mapList().then((all) => {
    maps = all;
    open(all.find((m) => m.id === mapId) ?? all.find((m) => m.id === saved.map) ?? all[0]);
  });
  routePlan().then((p) => (plan = p));
  questList().then((rows) => {
    questNames = new Map(rows.map((r) => [r.quest.id, r.quest.name]));
    tracked = new Set(rows.filter((r) => r.status === "active" || r.status === "needed").map((r) => r.quest.id));
  });

  $effect(() => {
    remember("event-region", region);
    liveEvents(region).then((e) => (events = e));
    const tick = setInterval(() => (now = Date.now()), 30000);
    return () => clearInterval(tick);
  });

  async function open(map: GameMap) {
    current = map;
    floor = Math.min(savedFloors[map.id] ?? 0, Math.max(0, map.image.layers.length - 1));
    selected = null;
    [set, pois] = await Promise.all([mapMarkers(map.id), mapPois(map.id)]);
    // A group name in the saved list means all of that group's types.
    const groups = [...hidden].filter((h) => h in GROUPS && h !== "quests");
    if (groups.length) {
      hidden = new Set([...hidden].filter((h) => !groups.includes(h))
        .concat(set.categories.filter((c) => groups.includes(c.group)).map((c) => c.id)));
    }
  }

  function save() {
    const state: Saved = {
      hidden: [...hidden], labels: showLabels, allQuests, collapsed: [...collapsed],
      map: current?.id ?? null, floors: savedFloors,
    };
    remember(SAVED_KEY, JSON.stringify(state));
  }

  $effect(() => {
    void [hidden, showLabels, allQuests, collapsed, current, savedFloors];
    save();
  });

  const floors = $derived(current?.image.layers ?? []);
  const onFloor = (z: number | null) => !floors.length || z === null || (z & floors[floor].mask) !== 0;
  const isHidden = (id: string) => hidden.has(id);
  const idsIn = (group: string) => (set?.categories ?? []).filter((c) => c.group === group).map((c) => c.id);
  const groupHidden = (group: string) => idsIn(group).every((id) => hidden.has(id));

  function toggle(id: string) {
    const next = new Set(hidden);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    hidden = next;
  }

  /** Hides or shows every type in a group, leaving the rest alone. */
  function setGroup(group: string, show: boolean) {
    const next = new Set(hidden);
    for (const id of idsIn(group)) {
      if (show) next.delete(id);
      else next.add(id);
    }
    hidden = next;
  }

  /** Just this type, for finding one thing such as ARC Courier spawns. */
  function only(id: string) {
    hidden = new Set((set?.categories ?? []).map((c) => c.id).filter((c) => c !== id));
  }

  function setAll(show: boolean) {
    hidden = new Set(show ? [] : (set?.categories ?? []).map((c) => c.id));
  }

  // Quest pins name their quest; the rest are named by their type.
  const points = $derived.by((): MapPoint[] => {
    if (!set) return [];
    const query = search.trim().toLowerCase();
    const out: MapPoint[] = [];
    set.markers.forEach(([index, x, y, z, label], i) => {
      const category = set!.categories[index];
      if (!category || !onFloor(z)) return;
      const quest = category.group === "quests";
      if (isHidden(category.id) || (quest && !allQuests && !tracked.has(label ?? ""))) return;
      const name = quest ? questNames.get(label ?? "") ?? "Quest objective" : label ?? category.name;
      if (query && !name.toLowerCase().includes(query) && !category.name.toLowerCase().includes(query)) return;
      out.push({
        key: `m${i}`, x, y, icon: category.id, group: category.group,
        title: quest ? `Quest: ${name}` : label ? `${label} (${category.name})` : category.name,
        label: showLabels && (quest || category.group === "locations" || category.group === "events") ? name : null,
      });
    });
    cover?.pois.forEach((stop, i) => out.push({
      key: `s${stop.id}`, x: stop.x, y: stop.y, icon: "route", group: "route", number: i + 1,
      title: `Stop ${i + 1}: ${stop.name}`,
    }));
    return out;
  });

  // Counts on this floor, for the legend.
  const counts = $derived.by(() => {
    const out = new Map<number, number>();
    for (const [index, , , z] of set?.markers ?? []) {
      if (onFloor(z)) out.set(index, (out.get(index) ?? 0) + 1);
    }
    return out;
  });
  const legend = $derived(Object.keys(GROUPS).filter((g) => g !== "route").map((group) => ({
    group,
    rows: (set?.categories ?? []).map((c, index) => ({ ...c, index, count: counts.get(index) ?? 0 }))
      .filter((c) => c.group === group && c.count > 0),
  })).filter((g) => g.rows.length));

  const cover = $derived(plan?.covers.find((c) => c.map_id === current?.id) ?? null);
  const stopPoi = $derived(selected?.group === "route" ? pois.find((p) => `s${p.id}` === selected?.key) ?? null : null);
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
    <div class="tabs">
      {#each maps as map (map.id)}
        <button class:on={current?.id === map.id} onclick={() => open(map)} title={map.name}>
          {map.name.replace("The ", "")}{liveCount(map.id) ? " •" : ""}
        </button>
      {/each}
    </div>
    {#if current}
      {@const live = eventsFor(current.id)}
      <div class="events">
        {#each live?.current ?? [] as event}
          <span class="event now">
            {#if event.icon}<img src={iconSrc(event.icon)} alt="" width="16" height="16" />{/if}
            {event.name} · ends in {when(event.endTime)}
          </span>
        {/each}
        {#if live?.next}<span class="event">Next: {live.next.name} in {when(live.next.startTime)}</span>{/if}
        <select class="field region" aria-label="Event region" bind:value={region}>
          {#each REGIONS as r}<option value={r}>{r.replace("-", " ")}</option>{/each}
        </select>
      </div>
    {/if}

    <input class="field" placeholder="Search places and types" aria-label="Search the map" bind:value={search} />
    <div class="bulk">
      <button class="link" onclick={() => setAll(true)}>Show all</button>
      <button class="link" onclick={() => setAll(false)}>Hide all</button>
      <label><input type="checkbox" bind:checked={showLabels} /> Labels</label>
    </div>

    <div class="legend">
      {#each legend as { group, rows } (group)}
        <section>
          <div class="group">
            <button class="group-name" onclick={() => { const n = new Set(collapsed); if (n.has(group)) n.delete(group); else n.add(group); collapsed = n; }}>
              {collapsed.has(group) ? "▸" : "▾"} {GROUPS[group].label}
            </button>
            <button class="link" class:off={groupHidden(group)} onclick={() => setGroup(group, groupHidden(group))}>{groupHidden(group) ? "Show" : "Hide"}</button>
          </div>
          {#if group === "quests"}
            <label class="row"><input type="checkbox" bind:checked={allQuests} /> All quests, not just tracked</label>
          {/if}
          {#if !collapsed.has(group)}
            <div class="rows">
              {#each rows as row (row.id)}
                <div class="entry">
                  <button class="row" class:off={isHidden(row.id)} onclick={() => toggle(row.id)} aria-pressed={!isHidden(row.id)} title="Show or hide {row.name}">
                    <img src={iconUrl(row.id, row.group)} alt="" width="18" height="18" />
                    <span class="name">{row.name}</span>
                    <span class="count">{row.count}</span>
                  </button>
                  <button class="only" onclick={() => only(row.id)} title="Show only {row.name}" aria-label="Show only {row.name}">only</button>
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/each}
    </div>
  </aside>

  <div class="center">
    {#if floors.length}
      <div class="floors" role="group" aria-label="Floor">
        {#each floors as layer, i (layer.name)}
          <button class:on={i === floor} aria-pressed={i === floor} onclick={() => { floor = i; selected = null; if (current) savedFloors = { ...savedFloors, [current.id]: i }; }}>{layer.name}</button>
        {/each}
      </div>
    {/if}
    {#if current}
      <TileMap map={current} url={floors[floor]?.url ?? current.image.url} {points} {selected} onselect={(p) => (selected = p)} />
    {/if}
    <span class="dim small">{points.length.toLocaleString()} shown · scroll to zoom, drag to move</span>
  </div>

  <aside class="right">
    {#if selected}
      <section class="panel">
        <h3>{selected.title}</h3>
        <p class="dim small">{GROUPS[selected.group]?.label}</p>
        {#if stopPoi?.loot_areas.length}<p class="small">Loot: {stopPoi.loot_areas.join(", ")}</p>{/if}
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
        <p class="dim small">Track items, quests or projects and this ranks the maps that cover what you still need.</p>
      {/if}
      <p class="dim small">Stops are a set of places, not a walking path. Markers are community-mapped by MetaForge.</p>
    </section>
  </aside>
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: 290px minmax(0, 1fr) 270px;
    gap: 10px;
    padding: 10px 12px;
    height: 100%;
    min-height: 0;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: auto;
    min-height: 0;
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tabs button,
  .floors button {
    padding: 4px 10px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
    font-size: 13px;
  }
  .tabs .on,
  .floors .on {
    border-color: var(--cream);
    background: var(--cream);
    color: var(--on-cream);
  }
  .events {
    display: flex;
    flex-direction: column;
    gap: 2px;
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
  .region {
    align-self: flex-start;
    margin-top: 4px;
    text-transform: capitalize;
  }
  .bulk {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
  }
  .bulk label {
    display: flex;
    gap: 4px;
    margin-left: auto;
    color: var(--text-dim);
  }
  .link {
    padding: 0;
    background: none;
    border: 0;
    color: var(--teal);
    cursor: pointer;
    font: inherit;
    font-size: 13px;
  }
  .link.off {
    color: var(--text-dim);
  }
  .legend section {
    margin-bottom: 8px;
  }
  .group {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    padding-bottom: 2px;
    margin-bottom: 3px;
  }
  .group-name {
    padding: 0;
    background: none;
    border: 0;
    color: var(--text);
    cursor: pointer;
    font-family: var(--font-head);
    font-size: 14px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .rows {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px 6px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
    padding: 2px 3px;
    background: none;
    border: 0;
    border-radius: 3px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    text-align: left;
  }
  .entry {
    position: relative;
    min-width: 0;
  }
  .entry .row {
    width: 100%;
  }
  .only {
    position: absolute;
    right: 2px;
    top: 2px;
    display: none;
    padding: 0 5px;
    border: 0;
    border-radius: 3px;
    background: var(--cream);
    color: var(--on-cream);
    cursor: pointer;
    font: inherit;
    font-size: 11px;
  }
  .entry:hover .only,
  .only:focus-visible {
    display: block;
  }
  .row:hover {
    background: var(--panel-2);
  }
  .row.off {
    opacity: 0.4;
  }
  .row .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .row .count {
    color: var(--text-dim);
    font-size: 11px;
  }
  label.row {
    color: var(--text-dim);
    margin-bottom: 3px;
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
  .floors {
    display: flex;
    gap: 2px;
  }
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
  .cover.active {
    background: var(--panel-2);
    border-color: var(--border);
  }
  .live {
    display: block;
    color: var(--yellow);
  }
  .small {
    font-size: 13px;
    margin: 4px 0 0;
  }
  .stops {
    margin-top: 12px;
  }
  @media (max-width: 1240px) {
    .page {
      grid-template-columns: 250px minmax(0, 1fr) 210px;
    }
    .rows {
      grid-template-columns: 1fr;
    }
  }
</style>
