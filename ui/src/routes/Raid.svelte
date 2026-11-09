<script lang="ts">
  import {
    deleteRaidPlan, farmSpots, getProfile, liveEvents, listItems, mapList, mapMarkers, mapPois, newRaidPlan, raidAuto,
    RAID_DEFAULTS, saveRaidPlan, suggestMaps,
    type EventsView, type GameMap, type ItemSummary, type MapPick, type MarkerSet, type Poi, type Primary, type RaidAuto,
    type RaidPlan, type RaidStop, type TargetArea,
  } from "../lib/api";
  import TileMap, { type MapPoint } from "../lib/components/TileMap.svelte";

  /** `#/raid/<map>` starts a plan there, `#/raid/plan/<id>` opens one, `#/raid/farm[/<item>]` opens the farm finder. */
  let { mapId, planId, farmItem }: { mapId?: string; planId?: string; farmItem?: string } = $props();

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
  let items = $state<ItemSummary[]>([]);
  let plans = $state<RaidPlan[]>([]);
  let events = $state<EventsView | null>(null);
  // Routes remount on a new address, so the first values are the only ones.
  // svelte-ignore state_referenced_locally
  let mode = $state<"plans" | "farm">(farmItem !== undefined ? "farm" : "plans");
  let plan = $state<RaidPlan | null>(null);
  let auto = $state<RaidAuto | null>(null);
  let pois = $state<Poi[]>([]);
  let loadedMap = $state("");
  let floor = $state(0);
  let selected = $state<MapPoint | null>(null);
  let areaIndex = $state(0);
  let newMap = $state("");
  let addItem = $state("");
  let message = $state("");
  let saving = $state(false);
  // svelte-ignore state_referenced_locally
  let farmIds = $state<string[]>(farmItem ? [farmItem] : []);
  let farmResults = $state<TargetArea[] | null>(null);
  let farmPick = $state("");
  let farmIndex = $state(0);
  let picks = $state<MapPick[] | null>(null);
  let suggesting = $state(false);
  let markers = $state<MarkerSet | null>(null);

  Promise.all([mapList(), getProfile(), listItems()]).then(([m, p, i]) => {
    maps = m;
    items = i;
    plans = p.raid_plans ?? [];
    newMap = mapId ?? m[0]?.id ?? "";
    const open = plans.find((x) => x.id === planId) ?? plans.find((x) => x.status === "in_raid");
    if (mapId) start(mapId);
    else if (open && mode === "plans") show(open);
    if (farmItem) runFarm();
  });
  // Live conditions change the plan, so work it out again once they arrive.
  liveEvents(region).then((e) => {
    events = e;
    refresh();
  }, () => {});

  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const itemName = (id: string) => items.find((i) => i.id === id)?.name ?? id;
  const lootable = $derived(items.filter((i) => !["cosmetic", "furniture", "design", "blueprint", "stencil"].includes(i.category)));

  function start(map: string) {
    show(newRaidPlan(map, `${mapName(map)} raid`));
  }

  async function show(p: RaidPlan) {
    plan = structuredClone($state.snapshot(p));
    plan.area ??= "";
    areaIndex = 0;
    selected = null;
    message = "";
    await refresh();
    await loadMap(p.map_id);
  }

  async function loadMap(id: string) {
    if (loadedMap === id) return;
    floor = 0;
    [pois, markers] = await Promise.all([mapPois(id), mapMarkers(id)]);
    loadedMap = id;
  }

  /** Ranks every map for what's tracked, with the same switches a new plan starts with. */
  async function suggest() {
    suggesting = true;
    const now = Object.fromEntries((events?.maps ?? []).map((m) => [m.map_id, m.current.map((e) => e.name)]));
    picks = await suggestMaps(newRaidPlan("", "Raid"), now);
    suggesting = false;
  }

  // The primary objective: a quest step, a named place, an ARC to hunt, or a picked marker.
  const PLACE_KINDS = ["landmark", "building"];
  // One entry per name: some places have a marker per floor or per door.
  const places = $derived.by(() => {
    const seen = new Set<string>();
    return pois
      .filter((p) => PLACE_KINDS.includes(p.kind) && p.name && !/(\s(key|card|floor)|keycard)$/i.test(p.name) && !/spawn|zipline|field crate/i.test(p.name))
      .sort((a, b) => a.name.localeCompare(b.name))
      .filter((p) => !seen.has(p.name.toLowerCase()) && seen.add(p.name.toLowerCase()) !== undefined);
  });
  /** One spot per ARC type: the marker closest to the middle of where that ARC appears. */
  const arcSpots = $derived.by(() => {
    const set = markers;
    if (!set) return [];
    const byType = new Map<number, [number, number, number | null][]>();
    for (const [c, x, y, f] of set.markers) {
      if (set.categories[c]?.group === "arc") byType.set(c, [...(byType.get(c) ?? []), [x, y, f]]);
    }
    return [...byType].map(([c, list]) => {
      const mx = list.reduce((sum, p) => sum + p[0], 0) / list.length;
      const my = list.reduce((sum, p) => sum + p[1], 0) / list.length;
      const away = (p: [number, number, number | null]) => Math.hypot(p[0] - mx, p[1] - my);
      const mid = list.reduce((best, p) => (away(p) < away(best) ? p : best));
      return { id: set.categories[c].id, name: set.categories[c].name, x: mid[0], y: mid[1], floors: mid[2], count: list.length };
    }).sort((a, b) => a.name.localeCompare(b.name));
  });

  function setPrimary(value: string) {
    if (!plan) return;
    const [kind, ...rest] = value.split(":");
    const id = rest.join(":");
    let next: Primary | null = null;
    if (kind === "quest") {
      const q = auto?.quest_stops.find((s) => s.key === id);
      if (q) next = { kind, id: q.key, name: q.items[0] + ": " + q.name, x: q.x, y: q.y, floors: q.floors };
    } else if (kind === "place") {
      const p = pois.find((x) => x.id === id);
      if (p) next = { kind, id: p.id, name: p.name, x: p.x, y: p.y, floors: p.zlayers };
    } else if (kind === "arc") {
      const a = arcSpots.find((x) => x.id === id);
      if (a) next = { kind, id: a.id, name: "Hunt " + a.name, x: a.x, y: a.y, floors: a.floors };
    }
    plan.primary = next;
    save();
  }

  function primaryFromSelected() {
    if (!plan || !selected) return;
    const s = selected;
    plan.primary = { kind: "marker", id: s.key, name: s.label ?? s.title, x: s.x, y: s.y, floors: null };
    selected = null;
    save();
  }

  async function refresh() {
    if (!plan) return;
    const asked = $state.snapshot(plan);
    const result = await raidAuto(asked, live(asked.map_id).map((e) => e.name));
    if (plan && plan.id === asked.id && plan.map_id === asked.map_id) {
      auto = result;
      areaIndex = Math.max(0, result.areas.findIndex((a) => a.label === plan?.area));
    }
  }

  /** Saves after every change, so a plan is never lost. */
  async function save(recompute = true) {
    if (!plan) return;
    saving = true;
    try {
      const saved = await saveRaidPlan($state.snapshot(plan));
      plan.id = saved.id;
      const at = plans.findIndex((x) => x.id === saved.id);
      plans = at < 0 ? [...plans, saved] : plans.map((x) => (x.id === saved.id ? saved : x));
    } catch (e) {
      message = `Couldn't save: ${e}`;
    } finally {
      saving = false;
    }
    if (recompute) await refresh();
  }

  function toggle(key: string, on: boolean) {
    if (!plan) return;
    plan.off = on ? plan.off.filter((k) => k !== key) : [...plan.off, key];
    save();
  }
  const isOn = (key: string) => !plan?.off.includes(key);

  function addExtra() {
    const found = lootable.find((i) => i.name.toLowerCase() === addItem.trim().toLowerCase());
    if (!plan || !found) {
      message = addItem ? `No item called "${addItem}".` : "";
      return;
    }
    if (!plan.extra_items.includes(found.id)) plan.extra_items = [...plan.extra_items, found.id];
    addItem = "";
    message = "";
    save();
  }

  function removeExtra(id: string) {
    if (!plan) return;
    plan.extra_items = plan.extra_items.filter((x) => x !== id);
    plan.off = plan.off.filter((k) => k !== `extra:${id}`);
    save();
  }

  async function remove() {
    if (!plan?.id) {
      plan = null;
      return;
    }
    await deleteRaidPlan(plan.id);
    plans = plans.filter((x) => x.id !== plan?.id);
    plan = null;
    auto = null;
  }

  // Your own stops: from an area's loop, by clicking the map, or from a marker.
  function useArea(a: TargetArea) {
    if (!plan) return;
    plan.stops = a.stops.map((s) => ({ key: s.key, x: s.x, y: s.y, name: s.name, note: s.items.join(", ") }));
    plan.custom = true;
    message = `Area ${a.label}'s stops are now yours to edit.`;
    save(false);
  }

  function place(at: { x: number; y: number }) {
    if (!plan?.custom) return;
    const n = plan.stops.length + 1;
    plan.stops = [...plan.stops, { key: `c${Date.now()}`, x: at.x, y: at.y, name: `Stop ${n}`, note: "" }];
    save(false);
  }

  function addSelected() {
    if (!plan || !selected) return;
    const s = selected;
    plan.custom = true;
    plan.stops = [...plan.stops, { key: s.key, x: s.x, y: s.y, name: s.label ?? s.title, note: "" }];
    selected = null;
    save(false);
  }

  function moveStop(i: number, by: number) {
    if (!plan) return;
    const j = i + by;
    if (j < 0 || j >= plan.stops.length) return;
    const next = [...plan.stops];
    [next[i], next[j]] = [next[j], next[i]];
    plan.stops = next;
    save(false);
  }

  function dropStop(i: number) {
    if (!plan) return;
    plan.stops = plan.stops.filter((_, k) => k !== i);
    save(false);
  }

  function tick(key: string, on: boolean) {
    if (!plan) return;
    plan.checked = on ? [...plan.checked, key] : plan.checked.filter((k) => k !== key);
    save(false);
  }

  function setStatus(status: "planned" | "in_raid") {
    if (!plan) return;
    plan.status = status;
    if (status === "planned") plan.checked = [];
    message = status === "in_raid" ? "Raid started: the Compact Window shows this checklist." : "Raid finished. The plan is ready for next time.";
    save(false);
  }

  async function runFarm() {
    farmResults = null;
    farmIndex = 0;
    if (!farmIds.length) return;
    farmResults = await farmSpots(farmIds, RAID_DEFAULTS);
    if (farmResults[0]) await loadMap(farmResults[0].map_id);
  }

  function addFarm() {
    const found = lootable.find((i) => i.name.toLowerCase() === farmPick.trim().toLowerCase());
    if (!found) return;
    if (!farmIds.includes(found.id)) farmIds = [...farmIds, found.id];
    farmPick = "";
    runFarm();
  }

  async function planFarm(a: TargetArea) {
    const p = newRaidPlan(a.map_id, `Farm ${farmIds.map(itemName).join(", ")}`);
    p.extra_items = [...farmIds];
    // Only the farmed items: everything tracked is switched off for this plan.
    p.off = ["cat:quest", "cat:craft", "cat:project", "cat:bench"];
    p.stops = a.stops.map((s) => ({ key: s.key, x: s.x, y: s.y, name: s.name, note: s.items.join(", ") }));
    p.custom = true;
    mode = "plans";
    await show(p);
    await save(false);
  }

  const current = $derived(maps.find((m) => m.id === (mode === "farm" ? farmResults?.[farmIndex]?.map_id : plan?.map_id)) ?? null);
  const floors = $derived(current?.image.layers ?? []);
  const onFloor = (f: number | null) => !floors.length || f === null || (f & floors[floor].mask) !== 0;
  const areas = $derived(mode === "farm" ? (farmResults?.[farmIndex] ? [farmResults[farmIndex]] : []) : (auto?.areas ?? []));
  const area = $derived(areas[mode === "farm" ? 0 : areaIndex] ?? null);
  const route = $derived.by((): { key: string; x: number; y: number; name: string; detail: string; floors: number | null }[] => {
    if (mode === "plans" && plan?.custom) return plan.stops.map((s) => ({ ...s, detail: s.note, floors: null }));
    return (area?.stops ?? []).map((s: RaidStop) => ({ key: s.key, x: s.x, y: s.y, name: s.name, detail: s.items.join(", "), floors: s.floors }));
  });
  const points = $derived.by((): MapPoint[] => {
    const out: MapPoint[] = pois.filter((p) => p.kind === "extract" && onFloor(p.zlayers))
      .map((p) => ({ key: p.id, x: p.x, y: p.y, icon: "locations/extraction", group: "locations", title: p.name, label: p.name }));
    if (mode === "plans") {
      for (const q of auto?.quest_stops ?? []) {
        if (onFloor(q.floors)) out.push({ key: q.key, x: q.x, y: q.y, icon: "quests", group: "quests", title: `Quest: ${q.items[0]} · ${q.name}` });
      }
      // Other areas' stops stay pickable for your own route.
      if (plan?.custom) {
        for (const a of auto?.areas ?? []) {
          for (const s of a.stops.filter((s) => s.kind !== "quest" && onFloor(s.floors))) {
            out.push({ key: s.key, x: s.x, y: s.y, icon: "containers/base_container", group: "containers", title: `${s.name}: ${s.items.join(", ")}`, label: s.name });
          }
        }
      }
    }
    if (mode === "plans" && auto?.primary && onFloor(auto.primary.floors)) {
      out.push({ key: "primary", x: auto.primary.x, y: auto.primary.y, icon: "quests", group: "route", title: "Primary: " + auto.primary.name, label: "Primary" });
    }
    route.forEach((s, i) => out.push({ key: `r${s.key}`, x: s.x, y: s.y, icon: "route", group: "route", number: i + 1, title: `${i + 1}. ${s.name}${s.detail ? `: ${s.detail}` : ""}` }));
    return out;
  });
  const circles = $derived(areas.map((a, i) => ({ x: a.x, y: a.y, r: a.radius, label: a.label, on: mode === "farm" || (i === areaIndex && !plan?.custom) })));
  const live = (id: string) => events?.maps.find((m) => m.map_id === id)?.current ?? [];
  const KINDS = [
    ["quest", "Quests"], ["craft", "Tracked crafts"], ["project", "Projects"], ["bench", "Workbenches"], ["extra", "Added by hand"],
  ] as const;
  /** "Fabric (likely)" lines, three at most, likeliest first. */
  const short = (list: string[]) => list.length > 3 ? `${list.slice(0, 3).join(", ")} +${list.length - 3} more` : list.join(", ");
  const pct = (v: number) => `${Math.round(Math.min(1, v) * 100)}%`;
</script>

{#snippet covers(a: TargetArea)}
  {@const good = a.covers.filter((c) => c.enough)}
  <details class="covers">
    <summary class="small">Good chances for {good.length} of {a.covers.length} items</summary>
    {#each a.covers as c (c.item_id)}
      <p class="small" class:short={!c.enough}>{c.enough ? "✓" : "·"} {c.name}: {c.likely} likely, {c.possible} possible</p>
    {/each}
  </details>
{/snippet}

<datalist id="raid-items">
  {#each lootable as i (i.id)}<option value={i.name}></option>{/each}
</datalist>

<div class="page">
  <aside class="panel left">
    <div class="tabs" role="tablist">
      <button role="tab" aria-selected={mode === "plans"} class:on={mode === "plans"} onclick={() => (mode = "plans")}>Plans</button>
      <button role="tab" aria-selected={mode === "farm"} class:on={mode === "farm"} onclick={() => (mode = "farm")}>Farm</button>
    </div>
    {#if mode === "plans"}
      {#each plans as p (p.id)}
        <button class="plan" class:on={p.id === plan?.id} onclick={() => show(p)}>
          <strong>{p.name}</strong>
          <span class="dim small">{mapName(p.map_id)}{p.custom ? " · my stops" : ""}</span>
          {#if p.status === "in_raid"}<span class="small badge">In raid</span>{/if}
        </button>
      {:else}
        <p class="dim small">No plans yet. Pick a map to start one.</p>
      {/each}
      <h3 class="section-title">New plan</h3>
      <select class="field" bind:value={newMap} aria-label="Map">
        {#each maps as m (m.id)}<option value={m.id}>{m.name}{live(m.id).length ? ` · ${live(m.id).map((e) => e.name).join(", ")}` : ""}</option>{/each}
      </select>
      <button class="button small" onclick={() => start(newMap)}>Plan a raid here</button>
      <button class="button small dark" disabled={suggesting} onclick={suggest}>{suggesting ? "Ranking maps…" : "Suggest a map"}</button>
      {#if picks}
        <p class="dim small">Best for what you're tracking, quest steps included.</p>
        {#each picks as pick, i (pick.map_id)}
          <button class="plan" onclick={() => start(pick.map_id)} title="Plan a raid here">
            <strong>{i + 1}. {pick.map_name}</strong>
            <span class="dim small">
              {pick.best ? "Best: " + (pick.best.name || "open ground") : "Nothing tracked here"}{pick.quest_steps ? " · " + pick.quest_steps + (pick.quest_steps === 1 ? " quest step" : " quest steps") : ""}
            </span>
            {#if pick.items_here.length}<span class="dim small">Good for {short(pick.items_here)}</span>{/if}
            {#if pick.conditions.length}<span class="small badge" title={pick.conditions.flatMap((c) => c.effects).join("\n")}>{pick.conditions.map((c) => c.name).join(", ")}</span>{/if}
          </button>
        {/each}
      {/if}
    {:else}
      <h3 class="section-title">Farm items</h3>
      <p class="dim small">The best places on any map for what you pick.</p>
      <div class="add">
        <input class="field" list="raid-items" placeholder="Add an item" bind:value={farmPick} onkeydown={(e) => e.key === "Enter" && addFarm()} />
        <button class="button small" onclick={addFarm}>Add</button>
      </div>
      <div class="chips">
        {#each farmIds as id (id)}
          <button class="chip" title="Remove" onclick={() => { farmIds = farmIds.filter((x) => x !== id); runFarm(); }}>{itemName(id)} ×</button>
        {/each}
      </div>
      {#each farmResults ?? [] as a, i (`${a.map_id}${a.label}`)}
        <button class="plan" class:on={i === farmIndex} onclick={() => { farmIndex = i; loadMap(a.map_id); }}>
          <strong>{a.map_name}: {a.name || "open ground"}</strong>
          <span class="dim small" title="Loop length is a straight line in map units">{a.depth}× the chances wanted · {a.stops.length} stops · loop {a.walk}</span>
        </button>
      {:else}
        {#if farmIds.length && farmResults}<p class="dim small">Nothing in the model holds these.</p>{/if}
      {/each}
    {/if}
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
      <TileMap
        map={current} url={floors[floor]?.url ?? current.image.url} {points} {selected} onselect={(p) => (selected = p)}
        bounds={pois} ready={loadedMap === current.id} areas={circles} path={route}
        onplace={mode === "plans" && plan?.custom ? place : undefined}
      />
    {:else}
      <div class="empty panel"><p class="dim">Open a plan, start one, or farm an item.</p></div>
    {/if}
    <p class="dim small">
      {#if mode === "plans" && plan?.custom}Click the map to add a stop. Click a marker, then "Add as stop".{:else}Circles are target areas, numbers the loop through the one picked, yellow pins exits.{/if}
      Chances come from a model of container types and zones: no source publishes drop rates. Distances are straight lines.
    </p>
  </div>

  <aside class="panel side">
    {#if mode === "farm"}
      {#if area}
        <h3 class="section-title">{area.map_name}: {area.name}</h3>
        {@render covers(area)}
        <p class="small dim">Nearest exits: {area.exits.map((e) => e.name).join(", ") || "none mapped"}</p>
        <button class="button small primary" onclick={() => planFarm(area)}>Plan this</button>
        <h3 class="section-title">Stops</h3>
        {#each area.stops as s, i (s.key)}<p class="small"><b>{i + 1}.</b> {s.name}{s.items.length ? `: ${short(s.items)}` : ""}</p>{/each}
      {/if}
    {:else if plan}
      <input class="field name" aria-label="Plan name" bind:value={plan.name} onchange={() => save(false)} />
      <p class="dim small">{mapName(plan.map_id)}{saving ? " · saving…" : ""}</p>
      <div class="row">
        {#if plan.status === "in_raid"}
          <button class="button small primary" onclick={() => setStatus("planned")}>Finish raid</button>
        {:else}
          <button class="button small primary" onclick={() => setStatus("in_raid")}>Start raid</button>
        {/if}
        <div class="seg" role="group" aria-label="Route">
          <button class:on={!plan.custom} onclick={() => { if (plan) { plan.custom = false; save(false); } }}>Auto areas</button>
          <button class:on={plan.custom} onclick={() => { if (plan) { plan.custom = true; save(false); } }}>My stops</button>
        </div>
      </div>
      {#if message}<p class="small message" role="status">{message}</p>{/if}
      {#if auto?.conditions.length}
        <div class="live">
          {#each auto.conditions as c (c.name)}
            <p class="small"><strong>Live now: {c.name}</strong></p>
            {#each c.effects as e}<p class="dim small">{e}</p>{/each}
          {/each}
        </div>
      {/if}

      <h3 class="section-title">Primary objective</h3>
      {#if plan.primary}
        <p class="small"><strong>{plan.primary.name}</strong> <button class="link" onclick={() => { if (plan) { plan.primary = null; save(); } }}>clear</button></p>
        <p class="dim small">The areas below are side trips: ranked by what they give against how far they are from it, and each loop passes it.</p>
      {:else}
        <p class="dim small">Optional. Pick what the raid is mainly for, and the areas become side trips around it.</p>
      {/if}
      <select class="field" aria-label="Primary objective" value="" onchange={(e) => { const v = e.currentTarget.value; e.currentTarget.value = ""; if (v) setPrimary(v); }}>
        <option value="">{plan.primary ? "Change…" : "Choose…"}</option>
        {#if auto?.quest_stops.length}
          <optgroup label="Quest steps">
            {#each auto.quest_stops as q (q.key)}<option value={"quest:" + q.key}>{q.items[0]}: {q.name}</option>{/each}
          </optgroup>
        {/if}
        {#if arcSpots.length}
          <optgroup label="Hunt an ARC">
            {#each arcSpots as a (a.id)}<option value={"arc:" + a.id}>{a.name} ({a.count} spawns)</option>{/each}
          </optgroup>
        {/if}
        {#if places.length}
          <optgroup label="Go to a place">
            {#each places as p (p.id)}<option value={"place:" + p.id}>{p.name}</option>{/each}
          </optgroup>
        {/if}
      </select>
      {#if selected && selected.group !== "route"}
        <p class="small"><button class="link" onclick={primaryFromSelected}>Make "{selected.label ?? selected.title}" the primary objective</button></p>
      {/if}

      {#if selected && plan.custom && selected.group !== "route"}
        <p class="small"><strong>{selected.title}</strong> <button class="link" onclick={addSelected}>Add as stop</button></p>
      {/if}

      {#if plan.custom}
        <h3 class="section-title">My stops ({plan.stops.length})</h3>
        {#each plan.stops as s, i (s.key)}
          <div class="stop">
            {#if plan.status === "in_raid"}<input type="checkbox" checked={plan.checked.includes(s.key)} onchange={(e) => tick(s.key, e.currentTarget.checked)} aria-label="Done" />{/if}
            <b>{i + 1}.</b>
            <input class="field stop-name" bind:value={s.name} onchange={() => save(false)} aria-label="Stop name" />
            <button class="icon" aria-label="Move up" onclick={() => moveStop(i, -1)}>↑</button>
            <button class="icon" aria-label="Move down" onclick={() => moveStop(i, 1)}>↓</button>
            <button class="icon" aria-label="Remove" onclick={() => dropStop(i)}>×</button>
          </div>
          {#if s.note}<p class="dim small note">{s.note}</p>{/if}
        {:else}
          <p class="dim small">Click the map to add stops, or copy an area's loop below.</p>
        {/each}
      {/if}

      <h3 class="section-title">{plan.primary ? "Side areas" : "Target areas"}</h3>
      {#each auto?.areas ?? [] as a, i (a.label)}
        <div class="area" class:on={i === areaIndex && !plan.custom}>
          <button class="area-head" onclick={() => { areaIndex = i; if (plan) { plan.custom = false; plan.area = a.label; save(false); } }}>
            <strong>{a.label}. {a.name || "Open ground"}</strong>
            <span class="dim small" title="Covered: how much of what you want has enough chances here. Chances: how many, against what the plan asks for. Loop length is a straight line in map units.">{pct(a.value)} covered · {a.depth}× chances · {a.stops.length} stops · loop {a.walk}{a.detour === null || a.detour === undefined ? "" : a.detour ? " · " + a.detour + " off the objective" : " · around the objective"}</span>
          </button>
          {#if i === areaIndex}
            {@render covers(a)}
            <p class="small dim">Nearest exits: {a.exits.map((e) => e.name).join(", ") || "none mapped"}</p>
            {#if !plan.custom}
              {#each a.stops as s, k (s.key)}
                <label class="small check">
                  {#if plan.status === "in_raid"}<input type="checkbox" checked={plan.checked.includes(s.key)} onchange={(e) => tick(s.key, e.currentTarget.checked)} />{/if}
                  <span><b>{k + 1}.</b> {s.name}{s.items.length ? `: ${short(s.items)}` : ""}</span>
                </label>
              {/each}
            {/if}
            <button class="link small" onclick={() => useArea(a)}>Edit as my stops</button>
          {/if}
        </div>
      {:else}
        <p class="dim small">{auto ? "Nothing switched on can be found on this map. Track something, or add an item below." : "Working it out…"}</p>
      {/each}

      <details open>
        <summary class="section-title">What it's for</summary>
        {#each KINDS as [kind, label]}
          {@const list = auto?.sources.filter((s) => s.kind === kind) ?? []}
          {#if list.length}
            <label class="cat"><input type="checkbox" checked={isOn(`cat:${kind}`)} onchange={(e) => toggle(`cat:${kind}`, e.currentTarget.checked)} /> {label}</label>
            {#each list as s (s.key)}
              <label class="small sub" class:off={!s.on}>
                <input type="checkbox" checked={isOn(s.key)} disabled={!isOn(`cat:${kind}`)} onchange={(e) => toggle(s.key, e.currentTarget.checked)} />
                <span>{s.name}{#if kind === "extra"} <button class="link" onclick={() => removeExtra(s.key.slice(6))}>remove</button>{/if}</span>
              </label>
            {/each}
          {/if}
        {/each}
        <div class="add">
          <input class="field" list="raid-items" placeholder="Add an item" bind:value={addItem} onkeydown={(e) => e.key === "Enter" && addExtra()} />
          <button class="button small" onclick={addExtra}>Add</button>
        </div>
      </details>

      <details>
        <summary class="section-title">Items ({auto?.wants.filter((w) => w.on).length ?? 0})</summary>
        {#each auto?.wants ?? [] as w (w.item.id)}
          <label class="small sub" class:off={!w.on}>
            <input type="checkbox" checked={isOn(`item:${w.item.id}`)} onchange={(e) => toggle(`item:${w.item.id}`, e.currentTarget.checked)} />
            <span>{w.qty}× {w.item.name}{#if !w.placeable} <span class="dim">· no known spawn</span>{/if}</span>
          </label>
        {/each}
      </details>

      <details>
        <summary class="section-title">Containers</summary>
        {#each auto?.boxes ?? [] as b (b.key)}
          <label class="small sub" class:off={!b.on}>
            <input type="checkbox" checked={b.on} onchange={(e) => toggle(b.key, e.currentTarget.checked)} />
            <span>{b.name} <span class="dim">{b.count}</span></span>
          </label>
        {/each}
      </details>

      <details>
        <summary class="section-title">Settings</summary>
        <label class="setting small">Areas <input class="field" type="number" min="1" max="8" bind:value={plan.settings.areas} onchange={() => save()} /></label>
        <label class="setting small">Area radius <input class="field" type="number" min="60" max="1500" step="10" bind:value={plan.settings.radius} onchange={() => save()} /></label>
        <label class="setting small">Stops per area <input class="field" type="number" min="1" max="30" bind:value={plan.settings.stops} onchange={() => save()} /></label>
        <label class="setting small" title="Good containers wanted for each copy you need">Chances per item <input class="field" type="number" min="1" max="10" bind:value={plan.settings.confidence} onchange={() => save()} /></label>
      </details>

      <h3 class="section-title">Notes</h3>
      <textarea class="field notes" rows="3" bind:value={plan.notes} onchange={() => save(false)} aria-label="Notes"></textarea>
      <button class="link small danger" onclick={remove}>{plan.id ? "Delete this plan" : "Discard"}</button>
    {/if}
  </aside>
</div>

<style>
  .live {
    padding: 6px 8px;
    border: 1px solid var(--yellow);
    border-radius: 6px;
  }
  .live p {
    margin: 0;
  }
  .page {
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr) 320px;
    gap: 10px;
    padding: 10px 12px;
    height: 100%;
    min-height: 0;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: auto;
    min-height: 0;
  }
  .center {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    min-width: 0;
  }
  .center :global(.map),
  .empty {
    flex: 1;
    min-height: 0;
  }
  .empty {
    display: grid;
    place-items: center;
  }
  .tabs,
  .seg {
    display: flex;
    gap: 2px;
  }
  .tabs button,
  .seg button,
  .floors button {
    padding: 3px 12px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
    font-size: 13px;
  }
  .tabs .on,
  .seg .on,
  .floors .on {
    background: var(--cream);
    color: var(--on-cream);
  }
  .tabs {
    margin-bottom: 6px;
  }
  .plan,
  .area-head {
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
  .plan.on,
  .area.on {
    background: var(--panel-2);
    border-color: var(--border);
  }
  .area {
    border: 1px solid transparent;
    border-radius: 4px;
    padding-bottom: 4px;
  }
  .area > :not(.area-head) {
    margin-left: 8px;
  }
  .badge {
    align-self: flex-start;
    padding: 0 6px;
    border-radius: 3px;
    background: var(--yellow);
    color: var(--on-cream);
  }
  .floors {
    display: flex;
    gap: 2px;
  }
  .row,
  .add {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .add .field {
    flex: 1;
    min-width: 0;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin: 4px 0;
  }
  .chip {
    padding: 1px 8px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    font-size: 13px;
  }
  .name {
    font-weight: 600;
  }
  .stop {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .stop-name {
    flex: 1;
    min-width: 0;
    padding: 1px 6px;
  }
  .icon {
    width: 22px;
    padding: 0;
    background: none;
    border: 0;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  .note {
    margin-left: 22px;
  }
  .check,
  .sub {
    display: flex;
    align-items: flex-start;
    gap: 6px;
  }
  .sub {
    margin-left: 18px;
  }
  .cat {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
    font-weight: 600;
  }
  .off {
    opacity: 0.5;
  }
  .short {
    color: var(--text-dim);
  }
  .covers summary {
    cursor: pointer;
    color: var(--teal);
  }
  .setting {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .setting .field {
    width: 76px;
  }
  .notes {
    width: 100%;
    resize: vertical;
  }
  details summary {
    cursor: pointer;
  }
  .message {
    color: var(--teal);
  }
  .danger {
    align-self: flex-start;
    margin-top: 8px;
    color: var(--red, #e5484d);
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
      grid-template-columns: 200px minmax(0, 1fr) 280px;
    }
  }
</style>
