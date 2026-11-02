<script lang="ts">
  import { getItem, getProfile, ROMAN, toggleTrackedItem, type ItemDetail, type Link, type RecipeView } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";
  import LinkRow from "../lib/components/LinkRow.svelte";
  import WeaponPicker from "../lib/components/WeaponPicker.svelte";

  let { id }: { id: string } = $props();
  let detail = $state<ItemDetail | null>(null);
  let error = $state("");
  let tracked = $state(false);

  getProfile().then((p) => (tracked = p.tracked_item_ids.includes(id)));

  $effect(() => {
    getItem(id).then((d) => (detail = d), (e) => (error = String(e)));
  });

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
      <div class="art"><Icon url={item.icon} name={item.name} size={140} /></div>
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
          <dt>{STAT_LABELS[key]}</dt><dd>{value}</dd>
        {/each}
      </dl>
      {#if canCraft}
        <a class="button primary" href="#/craft/{item.id}">Crafting tree</a>
      {/if}
      <button class="button dark" onclick={async () => (tracked = await toggleTrackedItem(item.id))}>
        {tracked ? "Tracked" : "Track resources"}
      </button>
    </aside>

    <div class="sections">
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
