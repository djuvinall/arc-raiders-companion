<script lang="ts">
  import {
    getItem, getProfile, iconSrc, liveEvents, mapList, mapMarkers, mapPois, markerCounts, questList,
    type ItemDetail,
    type EventsView, type GameMap, type MarkerSet, type Poi, type RaidPlan,
  } from "../lib/api";
  import TileMap, { type MapPoint } from "../lib/components/TileMap.svelte";
  import { GROUPS, iconUrl } from "../lib/mapIcons";

  /** `find` opens the map to show where one item turns up, apart from the usual filters. */
  let { mapId, find }: { mapId?: string; find?: string } = $props();
  let finding = $state<ItemDetail | null>(null);
  // svelte-ignore state_referenced_locally
  if (find) getItem(find).then((d) => (finding = d));

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
  let raidPlans = $state<RaidPlan[]>([]);
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

  let perMap = $state<Record<string, number[]>>({});
  markerCounts().then((c) => (perMap = c));

  mapList().then((all) => {
    maps = all;
    open(all.find((m) => m.id === mapId) ?? all.find((m) => m.id === saved.map) ?? all[0]);
  });
  getProfile().then((p) => (raidPlans = p.raid_plans ?? []));
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
    set = null;
    floor = Math.min(savedFloors[map.id] ?? 0, Math.max(0, map.image.layers.length - 1));
    selected = null;
    const [markers, points] = await Promise.all([mapMarkers(map.id), mapPois(map.id)]);
    // A quicker click on another map wins; this one's late answer is dropped.
    if (current?.id !== map.id) return;
    [set, pois] = [markers, points];
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

  // A finding view is a side trip, so it leaves the saved filters alone.
  $effect(() => {
    void [hidden, showLabels, allQuests, collapsed, current, savedFloors];
    if (!find) save();
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

  const findAreas = $derived(finding?.item.loot_areas ?? []);
  const typeOf = (name: string) => set?.categories.find((c) => c.name.toLowerCase() === name.toLowerCase());
  const spots = $derived(pois.filter((p) => p.loot_areas.some((a) => findAreas.includes(a))));
  const areaName = (id: string) => finding?.found_in.find((f) => f.area.id === id)?.area.name ?? id.replace("-", " ");

  // Where the item can be, plus exits for planning the way out.
  const findPoints = $derived.by((): MapPoint[] => {
    if (!set || !finding) return [];
    const out: MapPoint[] = spots.filter((p) => onFloor(p.zlayers)).map((p) => {
      const type = typeOf(p.name);
      const areas = p.loot_areas.filter((a) => findAreas.includes(a)).map(areaName);
      return {
        key: `f${p.id}`, x: p.x, y: p.y, icon: type?.id ?? "containers", group: "containers",
        title: `${p.name}: ${areas.join(", ")}`, label: showLabels ? p.name : null,
      };
    });
    const doors = new Set(doorsHere?.markers ?? []);
    set.markers.forEach(([index, x, y, z, label], i) => {
      const category = set!.categories[index];
      if (doors.has(i) && category && onFloor(z)) {
        const name = label?.trim() || category.name;
        out.push({ key: `d${i}`, x, y, icon: category.id, group: category.group, title: `Opens: ${name}`, label: showLabels ? name : null });
      } else if (category?.id === "locations/extraction" && onFloor(z)) {
        out.push({ key: `x${i}`, x, y, icon: category.id, group: "locations", title: label ?? "Extraction", label: showLabels ? label ?? "Extraction" : null });
      }
    });
    return out;
  });
  // Keys: the doors they open, beside any loot spots.
  const doorsHere = $derived(finding?.opens.find((o) => o.map.id === current?.id) ?? null);
  const doorCount = (mapId: string) => finding?.opens.find((o) => o.map.id === mapId)?.markers.length ?? 0;
  const spotCount = (mapId: string) => (finding?.found_in.reduce((sum, f) => sum + (f.maps.find((m) => m.map.id === mapId)?.poi_count ?? 0), 0) ?? 0) + doorCount(mapId);
  const areaCounts = $derived(findAreas.map((a) => ({ name: areaName(a), count: spots.filter((p) => p.loot_areas.includes(a)).length })).filter((a) => a.count));

  // Quest pins name their quest; the rest are named by their type.
  const points = $derived.by((): MapPoint[] => {
    if (find) return findPoints;
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

  // Every floor counts here: this compares maps, not what's on screen. Quest objectives are
  // left out because they depend on which quests are tracked.
  const tabCount = (id: string) => (perMap[id] ?? []).reduce((sum, count, index) => {
    const category = set?.categories[index];
    return category && category.group !== "quests" && !hidden.has(category.id) ? sum + count : sum;
  }, 0);
  const filtered = $derived(hidden.size > 0);
  const hiddenHere = $derived(legend.flatMap((g) => g.rows).filter((r) => isHidden(r.id)).length);

  const whole = $derived(set?.markers.map(([, x, y]) => ({ x, y })) ?? []);
  const selectedPoi = $derived(selected ? pois.find((p) => p.x === selected?.x && p.y === selected?.y) ?? null : null);
  const plansHere = $derived(raidPlans.filter((r) => r.map_id === current?.id));
  const when = (ms: number) => {
    const minutes = Math.round((ms - now) / 60000);
    if (minutes < 60) return `${Math.max(0, minutes)} min`;
    return `${Math.floor(minutes / 60)} h ${minutes % 60} min`;
  };
  const eventsFor = (id: string) => events?.maps.find((m) => m.map_id === id);
  const liveCount = (id: string) => eventsFor(id)?.current.length ?? 0;
</script>

<div class="page">
  <aside class="left">
    <div class="tabs">
      {#each maps as map (map.id)}
        <button class:on={current?.id === map.id} onclick={() => open(map)} title={map.name}>
          {map.name.replace("The ", "")}{liveCount(map.id) ? " •" : ""}
          {#if find && finding}<span class="tally">{spotCount(map.id)}</span>
          {:else if filtered && set}<span class="tally">{tabCount(map.id).toLocaleString()}</span>{/if}
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

    {#if find}
      <section class="panel finding">
        {#if finding}
          <div class="find-head">
            <img src={iconSrc(finding.item.icon ?? "")} alt="" width="44" height="44" />
            <div>
              <span class="dim small">Finding</span>
              <h3>{finding.item.name}</h3>
            </div>
          </div>
          {#if finding.opens.length}
            {#if doorsHere}
              <p class="small">Opens <strong>{doorsHere.markers.length}</strong> {doorsHere.markers.length === 1 ? "door" : "doors"} on {current?.name ?? "this map"}: {doorsHere.places.join(", ")}.</p>
            {:else}
              <p class="small">Opens doors on {finding.opens.map((o) => o.map.name).join(", ")}, not here.</p>
            {/if}
          {/if}
          {#if findAreas.length}
            <p class="small">Found in {findAreas.map(areaName).join(", ")} areas.</p>
            <p class="small"><strong>{spots.length}</strong> {spots.length === 1 ? "spot" : "spots"} on {current?.name ?? "this map"}{floors.length ? ` (${findPoints.filter((p) => p.group === "containers").length} on this floor)` : ""}.</p>
            {#each areaCounts as area (area.name)}<p class="small dim">{area.name}: {area.count}</p>{/each}
            <p class="small dim">The numbers on the map buttons count spots on each map. Exits are shown for planning the way out.</p>
          {:else if finding.note}
            <p class="small">{finding.note.where}</p>
            <p class="small dim">{finding.note.source}</p>
          {:else if !finding.opens.length}
            <p class="small">No source maps this item to a loot area, so there's nowhere to point to.</p>
          {/if}
          <label class="small"><input type="checkbox" bind:checked={showLabels} /> Labels</label>
          <a class="button dark" href="#/maps/{current?.id ?? ""}">Back to my map filters</a>
          <a class="small" href="#/item/{finding.item.id}">Back to {finding.item.name}</a>
        {:else}
          <p class="dim small">Loading…</p>
        {/if}
      </section>
    {:else}
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
    {/if}
  </aside>

  <div class="center">
    {#if floors.length}
      <div class="floors" role="group" aria-label="Floor">
        {#each floors as layer, i (layer.name)}
          <button class:on={i === floor} aria-pressed={i === floor} onclick={() => { floor = i; selected = null; if (current) savedFloors = { ...savedFloors, [current.id]: i }; }}>{layer.name}</button>
        {/each}
      </div>
    {/if}
    {#if current && set && points.length === 0}
      <p class="empty">
        {#if find}
          {finding?.item.name ?? "This item"} isn't found {floors.length ? "on this floor" : "on this map"}. The numbers on the map buttons show where it is.
        {:else if search.trim()}
          Nothing here matches "{search.trim()}".
        {:else if hiddenHere}
          None of the types you're showing are on this {floors.length ? "floor" : "map"}.
          The numbers on the map buttons show where they are.
        {:else}
          No markers here.
        {/if}
      </p>
    {/if}
    {#if current}
      <TileMap
        map={current} url={floors[floor]?.url ?? current.image.url} {points} {selected} onselect={(p) => (selected = p)}
        bounds={whole} ready={!!set && (!find || !!finding)}
      />
    {/if}
    <span class="dim small">{points.length.toLocaleString()} shown · scroll to zoom, drag to move</span>
  </div>

  <aside class="right">
    {#if selected}
      <section class="panel">
        <h3>{selected.title}</h3>
        <p class="dim small">{GROUPS[selected.group]?.label}</p>
        {#if selectedPoi?.loot_areas.length}<p class="small">Loot: {selectedPoi.loot_areas.join(", ")}</p>{/if}
      </section>
    {/if}

    {#if current}
      <section class="panel">
        <h3 class="section-title">Raid</h3>
        <a class="button primary" href="#/raid/{current.id}">Plan a raid here</a>
        {#each plansHere as r (r.id)}
          <a class="small plan-link" href="#/raid/plan/{r.id}">{r.name}{r.status === "in_raid" ? " · in raid" : ""}</a>
        {/each}
        <p class="dim small">Raid plans pick the areas worth heading for on {current.name}, from what you've tracked. Markers are community-mapped by MetaForge.</p>
      </section>
    {/if}
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
  .finding {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-start;
  }
  .finding h3 {
    margin: 0;
  }
  .find-head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .tally {
    margin-left: 4px;
    padding: 0 5px;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.35);
    font-size: 11px;
  }
  .tabs .on .tally {
    background: rgba(0, 0, 0, 0.12);
  }
  .empty {
    margin: 0;
    padding: 6px 10px;
    border: 1px solid var(--yellow);
    border-radius: 6px;
    color: var(--yellow);
    font-size: 13px;
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
  .plan-link {
    display: block;
    margin-top: 6px;
  }
  .small {
    font-size: 13px;
    margin: 4px 0 0;
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
