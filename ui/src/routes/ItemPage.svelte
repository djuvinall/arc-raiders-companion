<script lang="ts">
  import {
    getItem, getProfile, linkHref, ROMAN, toggleFound, toggleTrackedItem,
    type ItemDetail, type Link, type RecipeView,
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
  let baseline = $state<ItemDetail | null>(null);

  getProfile().then((p) => {
    tracked = p.tracked_item_ids.includes(id);
    found = p.blueprints_found.includes(id);
  });

  // Amplified versions compare with their base, levels with level I.
  const baselineId = (d: ItemDetail) => {
    const id = d.base?.id ?? d.family?.levels[0]?.id;
    return id === d.item.id ? null : id ?? null;
  };

  $effect(() => {
    getItem(id).then(async (d) => {
      const other = baselineId(d);
      baseline = other ? await getItem(other) : null;
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
    magazine_size: "Magazine",
    firing_mode: "Firing mode",
    arc_armor_penetration: "ARC armor pen.",
    shield_hp: "Shield HP",
    charge: "Charge",
    type: "Type",
    movement_penalty: "Movement penalty",
    damage_mitigation: "Damage mitigation",
    backpack_slots: "Backpack slots",
    weight_limit_kg: "Weight limit (kg)",
    safe_pocket_slots: "Safe pockets",
    quick_use_slots: "Quick use slots",
    shield_compatibility: "Shield",
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
  const anyDelta = $derived(stats.some(([key, value]) => delta(key, value) !== 0));

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

  const canCraft = $derived(detail?.crafting.some((r) => r.kind === "craft") ?? false);
  const kindLabel = (r: RecipeView) => (r.kind === "salvage" ? "Salvage" : "Recycle");
</script>

{#if error}
  <p class="error">{error}</p>
{:else if detail}
  {@const item = detail.item}
  <div class="page">
    <aside class="detail">
      <div class="art" class:blueprint={item.category === "blueprint"}><Icon url={item.icon} name={item.name} size={140} /></div>
      <div class="chips">
        <span class="chip category">{item.category.replace("-", " ")}</span>
        {#if item.rarity}
          <span class="chip" style:color="var(--on-cream)">{item.rarity}</span>
        {/if}
        {#if item.tier}<span class="chip">Tier {ROMAN[item.tier]}</span>{/if}
      </div>
      <h2>{item.name}</h2>
      {#if detail.family && (detail.family.levels.length > 1 || detail.family.amplified.length)}
        <WeaponPicker family={detail.family} {item} baseId={detail.base?.id ?? null} />
      {/if}
      {#if item.description}<p class="description">{item.description}</p>{/if}
      {#if item.flavor_text}<p class="flavor">{item.flavor_text}</p>{/if}
      <hr />
      <dl>
        {#if item.value !== null}<dt>Value</dt><dd>{item.value.toLocaleString()}</dd>{/if}
        {#if item.weight_kg !== null}<dt>Weight</dt><dd>{item.weight_kg} kg</dd>{/if}
        {#if item.stack_size !== null}<dt>Stack</dt><dd>{item.stack_size}</dd>{/if}
        {#if item.ammo_type}<dt>Ammo</dt><dd>{item.ammo_type}</dd>{/if}
        {#each stats as [key, value]}
          {@const d = delta(key, value)}
          <dt>{STAT_LABELS[key]}</dt>
          <dd>{value}{#if d}<span class="delta" class:down={d < 0}>{d > 0 ? "+" : ""}{d}</span>{/if}</dd>
        {/each}
        {#if item.stencil}
          <dt>Source</dt><dd>{item.stencil.source}</dd>
          <dt>Stencil Parts</dt><dd>{item.stencil.parts} per weapon</dd>
        {/if}
      </dl>
      {#if baseline}
        <p class="compared">
          {anyDelta ? `Changes compared with ${baseline.item.name}.` : `Same stats as ${baseline.item.name} in the source data.`}
        </p>
      {/if}
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
                  <a class="chip station" href="#/ref/workbench/{station.id}">
                    {station.name}{station.note ? ` · ${station.note}` : ""}
                  </a>
                {/each}
                {#if recipe.blueprint}
                  <a class="chip blueprint" href="#/item/{recipe.blueprint.id}">{recipe.blueprint.name}</a>
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

      {#if detail.found_in.length}
        <section class="panel">
          <h3 class="section-title">Found in</h3>
          {#each detail.found_in as found}
            <div class="area">
              <LinkRow link={found.area} />
              {#if found.maps.length === 0}
                <p class="sub indent">No mapped spots yet.</p>
              {/if}
              {#each found.maps as presence}
                <p class="indent">
                  <a href="#/ref/map/{presence.map.id}">{presence.map.name}</a>
                  <span class="sub">
                    {presence.poi_count} {presence.poi_count === 1 ? "spot" : "spots"}: {presence.places.slice(0, 5).join(", ")}{presence.places.length > 5 ? "…" : ""}
                  </span>
                </p>
              {/each}
            </div>
          {/each}
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
  .detail {
    position: sticky;
    top: 18px;
    background: var(--cream);
    color: var(--on-cream);
    border-radius: 6px;
    padding: 18px;
    max-height: calc(100vh - 150px);
    overflow: auto;
  }
  .art {
    display: grid;
    place-items: center;
    padding: 6px 0 14px;
  }
  .art.blueprint {
    margin-bottom: 14px;
    padding: 14px 0;
    background: var(--blueprint-bg);
    border-radius: 4px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 10px;
  }
  .category {
    background: #1f8f4d;
    border-color: #1f8f4d;
    color: #fff;
  }
  h2 {
    font-size: 30px;
    line-height: 1.05;
  }
  .description {
    color: var(--on-cream-dim);
    margin: 8px 0 0;
  }
  .flavor {
    color: var(--on-cream-dim);
    font-style: italic;
    margin: 6px 0 0;
  }
  hr {
    border: 0;
    border-top: 1px solid rgba(0, 0, 0, 0.15);
    margin: 14px 0;
  }
  dl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 3px 12px;
    margin: 0 0 14px;
  }
  dt {
    color: var(--on-cream-dim);
  }
  dd {
    margin: 0;
    font-weight: 600;
    text-align: right;
  }
  .delta {
    margin-left: 6px;
    color: #1f8f4d;
    font-size: 12px;
  }
  .delta.down {
    color: #c0392b;
  }
  .compared {
    margin: -8px 0 12px;
    color: var(--on-cream-dim);
    font-size: 13px;
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
  .indent a {
    color: var(--teal);
  }
  .error {
    padding: 20px;
    color: var(--rarity-epic);
  }
</style>
