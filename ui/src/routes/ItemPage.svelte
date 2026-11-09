<script lang="ts">
  import {
    getItem, getProfile, itemVerdict, linkHref, mapList, setStash, toggleFound, toggleTrackedItem,
    type ItemDetail, type Link, type RecipeView, type Verdict,
  } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";
  import LinkRow from "../lib/components/LinkRow.svelte";
  import StencilGuns from "../lib/components/StencilGuns.svelte";
  import StencilPreview from "../lib/components/StencilPreview.svelte";
  import WeaponPicker from "../lib/components/WeaponPicker.svelte";

  let { id }: { id: string } = $props();
  let detail = $state<ItemDetail | null>(null);
  let error = $state("");
  let tracked = $state(false);
  let found = $state(false);
  // Blueprints marked found in the Codex, by hand or from a screenshot.
  let learned = $state<string[]>([]);
  let baseline = $state<ItemDetail | null>(null);
  let verdict = $state<Verdict | null>(null);
  let inStash = $state(0);
  let benches = $state<Record<string, number>>({});
  let mapNames = $state<Record<string, string>>({});
  mapList().then((list) => (mapNames = Object.fromEntries(list.map((m) => [m.id, m.name]))));
  const mapName = (id: string) => mapNames[id] ?? id;

  async function refreshVerdict() {
    verdict = await itemVerdict(id);
  }
  refreshVerdict();

  async function saveStash(value: number) {
    inStash = Math.max(0, Math.floor(value) || 0);
    await setStash(id, inStash);
    await refreshVerdict();
  }

  // A station note reads "Level 2"; compare it with the tracked bench level.
  const shortOf = (station: Link) => {
    const level = Number(station.note?.replace("Level ", "") ?? 0);
    return station.id in benches && benches[station.id] < level;
  };

  getProfile().then((p) => {
    tracked = p.tracked_item_ids.includes(id);
    found = p.blueprints_found.includes(id);
    learned = p.blueprints_found;
    inStash = p.stash[id] ?? 0;
    benches = p.workbenches;
  });

  // Amplified versions compare with their base, levels with level I.
  const baselineId = (d: ItemDetail) => {
    const id = d.base?.id ?? d.family?.levels[0]?.id;
    return id === d.item.id ? null : id ?? null;
  };

  $effect(() => {
    getItem(id).then(async (d) => {
      const other = baselineId(d);
      // Without its baseline the page still shows, just with no comparison.
      baseline = other ? await getItem(other).catch(() => null) : null;
      detail = d;
    }, (e) => (error = String(e)));
  });

  const collectable = $derived(["blueprint", "stencil", "design"].includes(detail?.item.category ?? ""));

  const STAT_LABELS: Record<string, string> = {
    damage: "Damage",
    fire_rate: "Fire rate",
    range: "Range",
    stability: "Stability",
    agility: "Agility",
    stealth: "Stealth",
    magazine_size: "Magazine Size",
    firing_mode: "Firing Mode",
    arc_armor_penetration: "ARC Armor Penetration",
    shield_hp: "Shield HP",
    charge: "Charge",
    type: "Type",
    movement_penalty: "Movement penalty",
    damage_mitigation: "Damage Reduction",
    backpack_slots: "Backpack slots",
    weight_limit_kg: "Max Loadout Weight",
    safe_pocket_slots: "Safe pockets",
    quick_use_slots: "Quick use slots",
    shield_compatibility: "Shield Compatibility",
    trinket_slots: "Trinket slots",
    healing: "Healing",
    healing_per_second: "Healing / s",
    use_time: "Use time (s)",
    duration: "Duration (s)",
    radius: "Radius",
    arc_stun: "ARC stun",
    raider_stun: "Raider stun",
  };

  const stats = $derived(
    Object.entries(detail?.item.stats ?? {}).filter(
      ([key, value]) => key in STAT_LABELS && value !== null,
    ) as [string, string | number][],
  );

  const delta = (key: string, value: string | number) => {
    const before = baseline?.item.stats?.[key];
    if (typeof value !== "number" || typeof before !== "number") return 0;
    return Math.round((value - before) * 10) / 10;
  };
  const upgrades = $derived((detail?.item.stats?.upgrades ?? []) as { label: string; pct: number }[]);
  const anyDelta = $derived(
    stats.some(([key, value]) => delta(key, value) !== 0) ||
      JSON.stringify(upgrades) !== JSON.stringify(baseline?.item.stats?.upgrades ?? []),
  );

  const usedIn = $derived.by((): Link[] => {
    if (!detail) return [];
    const u = detail.used_in;
    const link = (kind: string, id: string, name: string, qty: number, note: string | null, icon: string | null = null): Link => ({
      kind, id, name, qty, note, icon,
    });
    return [
      ...u.recipes.flatMap((r): Link[] =>
        r.output_item_id ? [link("item", r.output_item_id, r.output_name ?? "", r.qty, "Recipe", r.output_icon)] : [],
      ),
      ...u.projects.map((p) => link("project", p.project_id, p.project_name, p.qty, `Project stage ${p.stage}`)),
      ...u.quests.map((q) => link("quest", q.quest_id, q.quest_name, q.qty, "Quest")),
      ...u.workbenches.map((w) => link("workbench", w.workbench_id, w.workbench_name, w.qty, `Upgrade to level ${w.level}`)),
    ];
  });

  // The in-game tooltip's layout: rows of facts, bars for 0-100 weapon stats, slot tiles
  // for augments, and lines for effects that have no number column.
  const BAR_KEYS = ["damage", "fire_rate", "range", "stability", "agility", "stealth"];
  const SLOT_KEYS: [string, string][] = [
    ["backpack_slots", "Backpack"], ["quick_use_slots", "Quick use"], ["safe_pocket_slots", "Safe pocket"], ["trinket_slots", "Trinket"],
  ];
  const LINE_KEYS = ["movement_penalty"];
  const isWeapon = $derived(detail?.item.category === "weapon");
  const bars = $derived(isWeapon ? stats.filter(([k, v]) => BAR_KEYS.includes(k) && typeof v === "number") as [string, number][] : []);
  const slotTiles = $derived(SLOT_KEYS.map(([k, label]) => [label, detail?.item.stats?.[k]] as [string, unknown])
    .filter(([, v]) => typeof v === "number" && v > 0) as [string, number][]);
  const infoRows = $derived(stats.filter(([k]) =>
    !(isWeapon && BAR_KEYS.includes(k)) && !SLOT_KEYS.some(([s]) => s === k) && !LINE_KEYS.includes(k) && k !== "type"));
  const lines = $derived(stats.filter(([k, v]) => LINE_KEYS.includes(k) && Number(v) > 0)
    .map(([, v]) => `${v}% Reduced Movement Speed`));
  const PERCENT_KEYS = ["damage_mitigation"];
  const shown = (key: string, value: string | number) =>
    PERCENT_KEYS.includes(key) ? `${value}%` : key === "arc_armor_penetration" && typeof value === "string" ? value[0].toUpperCase() + value.slice(1) : value;

  const recycle = $derived(detail?.breakdown.find((r) => r.kind !== "salvage") ?? detail?.breakdown[0] ?? null);
  const recyclesInto = $derived(recycle?.outputs ?? []);
  const recycleLabel = $derived(recycle?.kind === "salvage" ? "Salvages into" : "Recycles into");

  const RARITY: Record<string, [string, string]> = {
    common: ["#7d828d", "#ffffff"],
    uncommon: ["#2fcf5a", "#0f1a12"],
    rare: ["#1cb8f0", "#0c1a24"],
    epic: ["#e23fb0", "#ffffff"],
    legendary: ["#f5c400", "#1d1700"],
  };
  const rarityColor = (r: string | null) => RARITY[r ?? ""]?.[0] ?? "#7d828d";
  const rarityInk = (r: string | null) => RARITY[r ?? ""]?.[1] ?? "#ffffff";
  const TYPE_NAMES: Record<string, string> = { weapon_mod: "Modification", "weapon-mod": "Modification" };
  const typeLabel = (i: { category: string; subcategory: string | null }) =>
    (i.category === "weapon" && i.subcategory) || TYPE_NAMES[i.category] || i.category.replace(/[-_]/g, " ");
  // Simple stand-ins for the game's category glyphs.
  const glyphPath = (category: string) => ({
    shield: "M8 1l6 2.2v4.3c0 3.6-2.6 6.4-6 7.5-3.4-1.1-6-3.9-6-7.5V3.2z",
    weapon: "M1 6h11l3-1v3l-3 .5V10H8l-1 3H4l1-3H1z",
    augment: "M3 3h4v4H3zm6 0h4v4H9zM3 9h4v4H3zm6 0h4v4H9z",
  } as Record<string, string>)[category] ?? (category.includes("mod") ? "M1 6h14v4H1z" : "M8 2a6 6 0 1 1 0 12A6 6 0 0 1 8 2z");

  const canCraft = $derived(detail?.crafting.some((r) => r.kind === "craft") ?? false);
  const kindLabel = (r: RecipeView) => (r.kind === "salvage" ? "Salvage" : "Recycle");
</script>

{#if error}
  <p class="error">{error}</p>
{:else if detail}
  {@const item = detail.item}
  <div class="page" data-report="items/{item.id}" data-report-label={item.name}>
    <aside class="side">
      <div class="tooltip">
        <div class="art" class:blueprint={item.category === "blueprint"}><Icon url={item.icon} name={item.name} size={112} /></div>
        <div class="tags" style:--tag={rarityColor(item.rarity)} style:--tag-ink={rarityInk(item.rarity)}>
          <span class="tag glyph" aria-hidden="true">
            <svg viewBox="0 0 16 16" width="14" height="14"><path d={glyphPath(item.category)} fill="currentColor" /></svg>
          </span>
          <span class="tag">{typeLabel(item)}</span>
          {#if item.rarity}<span class="tag" data-report="items/{item.id}/rarity" data-report-label="Rarity">{item.rarity}</span>{/if}
        </div>
        <h2 data-report="items/{item.id}/name" data-report-label="Name">{item.name}</h2>
        {#if item.description}<p class="description" data-report="items/{item.id}/description" data-report-label="Description">{item.description}</p>{/if}
        {#if detail.mod_slots.length}
          <div class="mod-slots">
            {#each detail.mod_slots as slot (slot.note)}
              <span class="mod-slot" title="{slot.name} slot">{#if slot.icon}<Icon url={slot.icon} name={slot.name} size={38} />{:else}{slot.name}{/if}</span>
            {/each}
          </div>
        {/if}
        {#if detail.mod_info?.compatible.length}
          <p class="compat">Compatible with: {detail.mod_info.compatible.join(", ")}</p>
        {/if}
        <!-- Mods' flavor text repeats an effect listed below, and weapons' the armor penetration row. -->
        {#if item.flavor_text && !detail.mod_info && !isWeapon}<p class="flavor" class:callout={item.category === "augment"} data-report="items/{item.id}/flavor_text" data-report-label="Flavor text">{item.flavor_text}</p>{/if}

        <div class="rows">
          {#if item.ammo_type}<div class="row"><span>Ammo Type</span><b class="ammo">{#if detail.ammo?.icon}<Icon url={detail.ammo.icon} name={detail.ammo.name} size={20} />{/if}{item.ammo_type.replace(/ ammo$/i, "")} Ammo</b></div>{/if}
          {#each infoRows as [key, value]}
            {@const d = delta(key, value)}
            <div class="row" data-report="items/{item.id}/stats/{key}" data-report-label={STAT_LABELS[key]}>
              <span>{STAT_LABELS[key]}</span>
              <b>{shown(key, value)}{#if d}<span class="delta" class:down={d < 0}>{d > 0 ? "+" : ""}{d}</span>{/if}</b>
            </div>
          {/each}
          {#each lines as line}<div class="row"><span>{line}</span></div>{/each}
          {#each detail.mod_info?.effects ?? [] as effect}<div class="row"><span>{effect}</span></div>{/each}
          {#if item.stack_size !== null && item.stack_size > 1}<div class="row" data-report="items/{item.id}/stack_size" data-report-label="Stack size"><span>Stack size</span><b>{item.stack_size}</b></div>{/if}
          {#if item.stencil}
            <div class="row"><span>Source</span><b>{item.stencil.source}</b></div>
            <div class="row"><span>Stencil Parts</span><b>{item.stencil.parts} per weapon</b></div>
          {/if}
        </div>

        {#if slotTiles.length}
          <div class="slots">
            {#each slotTiles as [label, count]}<span class="slot"><b>×{count}</b>{label}</span>{/each}
          </div>
        {/if}

        {#if bars.length}
          <div class="bars">
            {#each bars as [key, value]}
              {@const before = baseline?.item.stats?.[key]}
              {@const base = typeof before === "number" ? Math.min(before, value) : value}
              <div class="stat-bar">
                <span>{STAT_LABELS[key]}</span>
                <span class="track" title="{STAT_LABELS[key]}: {value}{typeof before === 'number' && before !== value ? ` (was ${before})` : ''}">
                  <span class="fill" style:width="{Math.min(100, base)}%"></span>
                  {#if value > base}<span class="gain" style:width="{Math.min(100, value) - Math.min(100, base)}%"></span>{/if}
                </span>
              </div>
            {/each}
          </div>
        {/if}

        {#if upgrades.length}
          <p class="heading muted">Upgrade modifiers</p>
          {#each upgrades as upgrade}<div class="row"><span>{upgrade.pct}% {upgrade.label}</span></div>{/each}
        {/if}

        {#if recyclesInto.length}
          <p class="heading">{recycleLabel}</p>
          <div class="recycles">
            {#each recyclesInto as out (out.id)}
              <a href={linkHref(out)} title="{out.qty ?? 1}× {out.name}"><Icon url={out.icon} name={out.name} size={40} />{#if (out.qty ?? 1) > 1}<span>{out.qty}</span>{/if}</a>
            {/each}
          </div>
        {/if}

        <div class="footer">
          <label class="cell" title="How many you have in your stash">
            <span class="sr">In your stash</span>
            <input class="stash" type="number" min="0" aria-label="In your stash" value={inStash} onchange={(e) => { const n = Math.max(0, Math.floor(e.currentTarget.valueAsNumber) || 0); e.currentTarget.value = String(n); saveStash(n); }} />
            <small>in stash</small>
          </label>
          {#if item.weight_kg !== null}
            <span class="cell" title="Weight" data-report="items/{item.id}/weight_kg" data-report-label="Weight">
              <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true"><path d="M6 3a2 2 0 0 1 4 0v1h2.2l2.3 10H1.5L3.8 4H6zm1 0v1h2V3a1 1 0 0 0-2 0z" fill="currentColor" /></svg>
              {item.weight_kg.toFixed(1)}
            </span>
          {/if}
          {#if item.value !== null}
            <span class="cell" title="Value" data-report="items/{item.id}/value" data-report-label="Sell value">
              <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true"><circle cx="8" cy="8" r="6.2" fill="none" stroke="currentColor" stroke-width="1.6" /><path d="M8 2v12" stroke="currentColor" stroke-width="1.6" /></svg>
              {item.value.toLocaleString()}
            </span>
          {/if}
        </div>
      </div>

      {#if verdict}
        <div class="verdict {verdict.action}">
          <strong>{verdict.headline}</strong>
          {#each verdict.reasons.slice(0, 3) as reason}<span>{reason}</span>{/each}
          {#if verdict.reasons.length > 3}<span>And {verdict.reasons.length - 3} more</span>{/if}
        </div>
      {/if}
      {#if detail.family && (detail.family.levels.length > 1 || detail.family.amplified.length)}
        <div class="picker"><WeaponPicker family={detail.family} {item} baseId={detail.base?.id ?? null} /></div>
      {/if}
      {#if baseline}
        <p class="compared">
          {anyDelta ? `Blue on the bars and the numbers beside stats are changes from ${baseline.item.name}.` : `Same stats as ${baseline.item.name} in the source data.`}
        </p>
      {/if}
      <div class="actions">
        {#if canCraft}
          <a class="button primary" href="#/craft/{item.id}">Crafting tree</a>
        {/if}
        <button class="button dark" onclick={async () => (tracked = await toggleTrackedItem(item.id))}>
          {tracked ? "Tracked" : "Track resources"}
        </button>
        {#if collectable}
          <button class="button dark" onclick={async () => (found = await toggleFound(item.id))}>
            {found ? "Found ✓" : "Mark found"}
          </button>
        {/if}
      </div>
    </aside>

    <div class="sections">
      {#if item.stencil && detail.stencil_guns.length}
        <StencilGuns stencilId={item.id} swatch={item.stencil.swatch} guns={detail.stencil_guns} />
      {/if}

      {#if detail.gun_stencils.length}
        <section class="panel">
          <h3 class="section-title">Stencils ({detail.gun_stencils.length})</h3>
          <div class="stencils">
            {#each detail.gun_stencils as row (row.stencil.id)}
              <a href={linkHref(row.stencil)} title={row.stencil.name}>
                <StencilPreview gun={item.icon} swatch={row.swatch} name={item.name} size={84} />
                <span>{row.stencil.name.replace(" Stencil", "")}</span>
              </a>
            {/each}
          </div>
          <p class="sub">Approximate previews. The list is community-reported.</p>
        </section>
      {/if}

      {#if detail.crafting.length}
        <section class="panel">
          <h3 class="section-title">Crafting</h3>
          {#each detail.crafting as recipe}
            <div class="recipe">
              <div class="meta">
                {#each recipe.stations as station}
                  <a class="chip station" class:short={shortOf(station)} href="#/ref/workbench/{station.id}">
                    {station.name}{station.note ? ` · ${station.note}` : ""}
                    {shortOf(station) ? `· you have ${benches[station.id]}` : ""}
                  </a>
                {/each}
                {#if recipe.blueprint}
                  <a class="chip blueprint" class:learned={learned.includes(recipe.blueprint.id)} href="#/item/{recipe.blueprint.id}"
                    title={learned.includes(recipe.blueprint.id) ? "Marked found in your Codex" : "Not marked found in your Codex"}>
                    {recipe.blueprint.name}{learned.includes(recipe.blueprint.id) ? " ✓ learned" : ""}
                  </a>
                {:else if recipe.requires_blueprint}
                  <span class="chip blueprint">Blueprint required</span>
                {/if}
                {#if recipe.required_skill}<span class="chip">{recipe.required_skill}</span>{/if}
                {#if recipe.provenance === "community"}<span class="chip stale">RaidTheory, unverified</span>{/if}
              </div>
              {#each recipe.inputs as input}<LinkRow link={input} />{/each}
            </div>
          {/each}
        </section>
      {/if}

      {#if detail.breakdown.length}
        <section class="panel">
          <h3 class="section-title">Breaks down into</h3>
          <div class="columns">
            {#each detail.breakdown as recipe}
              <div>
                <p class="sub">{kindLabel(recipe)}</p>
                {#each recipe.outputs as output}<LinkRow link={output} />{/each}
              </div>
            {/each}
          </div>
        </section>
      {/if}

      {#if usedIn.length}
        <section class="panel">
          <h3 class="section-title">Used in ({usedIn.length})</h3>
          {#each usedIn as link}<LinkRow {link} />{/each}
        </section>
      {/if}

      {#if detail.note}
        <section class="panel">
          <h3 class="section-title">Where to find</h3>
          <p class="note" data-report-label="Where to find">{detail.note.where}</p>
          <p class="sub">{detail.note.source}</p>
        </section>
      {/if}

      {#if detail.opens.length}
        <section class="panel">
          <h3 class="section-title">Opens</h3>
          {#each detail.opens as door}
            <p>
              <a href="#/maps/{door.map.id}/find/{item.id}" title="Show the door on the map">{door.map.name}</a>
              <span class="sub">
                {door.markers.length} {door.markers.length === 1 ? "door" : "doors"}{door.places.length ? `: ${door.places.slice(0, 5).join(", ")}${door.places.length > 5 ? "…" : ""}` : ""}
              </span>
            </p>
          {/each}
        </section>
      {/if}

      {#if detail.found_in.length || detail.sources.length}
        {@const common = detail.sources.filter((s) => s.chance !== "unlikely")}
        {@const rare = detail.sources.filter((s) => s.chance === "unlikely")}
        {@const listed = new Set(detail.sources.map((s) => s.name.toLowerCase()))}
        {@const mapsHere = detail.found_maps.length
          ? detail.found_maps
          : [...new Set(common.flatMap((s) => s.maps))].map((m) => ({ map: { kind: "map", id: m, name: mapName(m), icon: null, qty: null, note: null }, poi_count: 0, places: [] as string[] }))}
        <section class="panel" data-report-label="Found in">
          <h3 class="section-title">Found in</h3>
          <p class="sub"><a href="#/raid/farm/{item.id}">Best places to farm it</a>, across every map.</p>
          {#if detail.sources.length}
            <h4 class="minor">Look in</h4>
            {#each common as source (source.id)}
              <p class="source">
                <span>{source.name}{#if source.notes.length} <span class="sub">· {source.notes.join(" · ")}</span>{/if}</span>
                <span class="chance {source.chance}">{source.chance}</span>
                <span class="sub">{source.maps.length} {source.maps.length === 1 ? "map" : "maps"}</span>
              </p>
            {/each}
            {#if rare.length}
              <p class="sub">{common.length ? "Rarely" : "Only rarely"}: {rare.map((s) => s.name).join(", ")}</p>
            {/if}
            <p class="sub">Chances are the app's estimate from container type, area and rarity. The game publishes no drop rates.</p>
          {/if}
          {#if mapsHere.length}
            <h4 class="minor">Maps</h4>
            {#each mapsHere as presence (presence.map.id)}
              <p class="indent">
                <a href="#/maps/{presence.map.id}/find/{item.id}" title="Show where on the map">{presence.map.name}</a>
                {#if presence.poi_count}
                  {@const places = presence.places.filter((p) => !listed.has(p.toLowerCase()))}
                  <span class="sub">
                    {presence.poi_count} {presence.poi_count === 1 ? "spot" : "spots"}{places.length ? `: ${places.slice(0, 6).join(", ")}${places.length > 6 ? "…" : ""}` : ""}
                  </span>
                {/if}
              </p>
            {/each}
          {/if}
          {#if detail.found_in.length}
            <p class="sub">Loot {detail.found_in.length === 1 ? "area" : "areas"}: {detail.found_in.map((f) => f.area.name).join(", ")}</p>
          {/if}
        </section>
      {/if}

      {#if detail.sold_by.length}
        <section class="panel">
          <h3 class="section-title">Sold by</h3>
          {#each detail.sold_by as link}<LinkRow {link} />{/each}
        </section>
      {/if}

      {#if detail.dropped_by.length}
        <section class="panel">
          <h3 class="section-title">Dropped by</h3>
          {#each detail.dropped_by as link}<LinkRow {link} />{/each}
        </section>
      {/if}

      {#if detail.quest_rewards.length || detail.project_rewards.length}
        <section class="panel">
          <h3 class="section-title">Rewarded by</h3>
          {#each [...detail.quest_rewards, ...detail.project_rewards] as link}<LinkRow {link} />{/each}
        </section>
      {/if}
    </div>
  </div>
{/if}

<style>
  .page {
    display: grid;
    grid-template-columns: 400px 1fr;
    gap: 18px;
    padding: 18px 20px;
    align-items: start;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  /* Modelled on the in-game item tooltip: cream card, rarity tags, ruled rows, footer band. */
  .tooltip {
    background: var(--cream);
    color: var(--on-cream);
    border-radius: 8px;
    overflow: hidden;
    padding: 16px 16px 0;
    font-size: 16px;
  }
  .art {
    display: grid;
    place-items: center;
    padding: 0 0 10px;
  }
  .art.blueprint {
    margin-bottom: 14px;
    padding: 14px 0;
    background: var(--blueprint-bg);
    border-radius: 4px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    margin-bottom: 6px;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    padding: 2px 6px;
    background: var(--tag);
    color: var(--tag-ink);
    font-family: var(--font-body);
    font-weight: 600;
    font-size: 14px;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    line-height: 1.35;
  }
  .tag.glyph {
    padding: 2px 4px;
  }
  /* The game sets item names in a heavy plain sans, not the condensed headline face. */
  h2 {
    margin: 2px 0 0;
    font-family: var(--font-body);
    font-weight: 800;
    font-size: 27px;
    letter-spacing: 0.01em;
    line-height: 1.1;
    text-transform: uppercase;
  }
  .description {
    color: #474b56;
    margin: 6px 0 0;
    font-size: 17px;
    line-height: 1.35;
  }
  .mod-slots {
    display: flex;
    gap: 6px;
    margin-top: 10px;
  }
  .mod-slot {
    display: grid;
    place-items: center;
    width: 50px;
    height: 50px;
    padding: 2px;
    background: #1b1f2e;
    border: 1.5px solid #2f9c55;
    border-radius: 5px;
    color: #c9ccd4;
    font-size: 10px;
    text-align: center;
  }
  .mod-slot :global(img) {
    opacity: 0.55;
    filter: grayscale(1);
  }
  .ammo {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .compat {
    margin: 6px 0 0;
    color: #6b7488;
  }
  .flavor {
    color: var(--on-cream-dim);
    font-style: italic;
    margin: 6px 0 0;
  }
  /* Augments' ability text sits in its own box in game. */
  .flavor.callout {
    margin-top: 10px;
    padding: 8px 12px;
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.35);
    color: var(--on-cream);
    font-style: normal;
    font-size: 15px;
  }
  .rows {
    margin-top: 10px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 10px;
    border-top: 1px solid rgba(0, 0, 0, 0.1);
  }
  .row b {
    font-weight: 500;
    text-align: right;
    text-transform: capitalize;
  }
  .delta {
    margin-left: 6px;
    color: #1f8f4d;
    font-size: 12px;
  }
  .delta.down {
    color: #c0392b;
  }
  .slots {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
    gap: 6px;
    margin: 8px 0;
  }
  .slot {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 6px 4px;
    border: 1px solid rgba(0, 0, 0, 0.3);
    border-radius: 3px;
    font-size: 12px;
    color: var(--on-cream-dim);
  }
  .slot b {
    font-size: 16px;
    color: var(--on-cream);
  }
  .bars {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px 22px;
    padding: 10px 10px 4px;
    border-top: 1px solid rgba(0, 0, 0, 0.1);
  }
  .stat-bar {
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 16px;
  }
  .track {
    display: flex;
    flex: none;
    height: 8px;
    border-radius: 4px;
    overflow: hidden;
    background: #b3b5bc;
  }
  .fill {
    background: #1b1f2e;
  }
  .gain {
    background: #3aa0c8;
  }
  .heading {
    margin: 12px 0 6px;
    font-family: var(--font-body);
    font-weight: 800;
    font-size: 17px;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }
  .heading.muted {
    color: #6b7488;
  }
  .recycles {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-bottom: 12px;
  }
  .recycles a {
    position: relative;
    display: grid;
    place-items: center;
    width: 50px;
    height: 50px;
    background: #1b1f2e;
    border: 1.5px solid #2f9c55;
    border-radius: 5px;
  }
  .recycles span {
    position: absolute;
    right: 3px;
    bottom: 1px;
    color: #fff;
    font-size: 11px;
  }
  .footer {
    display: flex;
    margin: 0 -16px;
    background: #d9cfbd;
  }
  .cell {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 6px;
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 17px;
  }
  .cell + .cell {
    border-left: 2px solid var(--cream);
  }
  .cell small {
    font-family: var(--font-body);
    font-weight: 400;
    font-size: 12px;
    color: var(--on-cream-dim);
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }
  .stash {
    width: 56px;
    padding: 1px 4px;
    border: 1px solid var(--on-cream-dim);
    border-radius: 3px;
    background: none;
    color: var(--on-cream);
    font: inherit;
    text-align: right;
  }
  .verdict {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 8px 10px;
    border-radius: 4px;
    background: #2a7a4b;
    color: #fff;
    font-size: 13px;
  }
  .verdict strong {
    font-size: 15px;
  }
  .verdict.sell {
    background: #2f3442;
  }
  .verdict.recycle {
    background: #1d6f86;
  }
  .picker {
    padding: 10px;
    background: var(--cream);
    color: var(--on-cream);
    border-radius: 6px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .short {
    color: var(--yellow);
  }
  .compared {
    margin: 0;
    color: var(--text-dim);
    font-size: 13px;
  }
  .note {
    margin: 0 0 6px;
    line-height: 1.45;
  }
  .stencils {
    display: grid;
    grid-template-columns: repeat(auto-fill, 104px);
    gap: 8px;
    margin-bottom: 8px;
  }
  .stencils a {
    display: grid;
    justify-items: center;
    gap: 2px;
    padding: 6px 2px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 12px;
  }
  .sections {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .recipe + .recipe {
    border-top: 1px solid var(--border);
    margin-top: 10px;
    padding-top: 10px;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 6px;
  }
  .station {
    color: var(--teal);
  }
  .blueprint {
    color: var(--yellow);
  }
  .blueprint.learned {
    color: var(--teal);
  }
  .stale {
    color: var(--text-dim);
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 14px;
  }
  .sub {
    color: var(--text-dim);
    font-size: 13px;
    margin: 0 0 4px;
  }
  .indent {
    margin: 2px 0 4px 8px;
  }
  .minor {
    margin: 10px 0 4px;
    color: var(--text-dim);
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .source {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 64px;
    max-width: 640px;
    align-items: center;
    gap: 10px;
    margin: 2px 0 2px 8px;
  }
  .source .sub {
    margin: 0;
    text-align: right;
  }
  .chance {
    padding: 0 8px;
    border-radius: 999px;
    font-size: 12px;
    text-transform: capitalize;
  }
  .chance.likely {
    background: var(--teal);
    color: #0b0b0f;
  }
  .chance.possible {
    border: 1px solid var(--yellow);
    color: var(--yellow);
  }
  .indent a {
    color: var(--teal);
  }
  .error {
    padding: 20px;
    color: var(--bad);
  }
</style>
