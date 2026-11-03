<script lang="ts">
  import {
    homeView, liveEvents, mapList, projectList, routePlan,
    type EventsView, type Goal, type Home, type Overview, type ProjectView, type RoutePlan,
  } from "../lib/api";

  let { info }: { info: Overview | null } = $props();
  let home = $state<Home | null>(null);
  let events = $state<EventsView | null>(null);
  let plan = $state<RoutePlan | null>(null);
  let projects = $state<ProjectView[]>([]);
  const now = Date.now();
  const DAY = 24 * 3600 * 1000;

  const region = (() => {
    try {
      return localStorage.getItem("event-region") ?? "north-america";
    } catch {
      return "north-america";
    }
  })();

  homeView().then((h) => (home = h));
  routePlan().then((p) => (plan = p));
  projectList().then((p) => (projects = p));
  liveEvents(region).then((e) => (events = e));
  mapList();

  const href = (g: Goal) => ({
    craft: `#/craft/${g.id}`,
    quest: `#/quests/${g.id}`,
    project: `#/projects/${g.id}`,
    bench: "#/profile",
  })[g.kind];
  const verb = (g: Goal) => ({ craft: "Craft", quest: "Turn in", project: "Commit", bench: "Upgrade" })[g.kind];
  const ending = $derived(projects
    .filter((p) => p.ends && Date.parse(`${p.ends}T23:59:59Z`) >= now && !p.paused)
    .map((p) => ({ ...p, days: Math.ceil((Date.parse(`${p.ends}T23:59:59Z`) - now) / DAY) })));
  const liveNow = $derived(events?.maps.flatMap((m) => m.current.map((e) => ({ map: m.map_name, name: e.name }))) ?? []);
  const best = $derived(plan?.needed.length ? plan.covers[0] : null);
  const blank = $derived(home && !home.quests_active && !home.quests_done && !home.bench_levels && !home.items_needed && !home.blueprints_found);
</script>

{#snippet stat(label: string, value: string, hint: string, link: string)}
  <a class="panel stat" href={link}>
    <span class="label">{label}</span>
    <strong>{value}</strong>
    <span class="dim small">{hint}</span>
  </a>
{/snippet}

{#snippet goalRow(g: Goal)}
  <a class="goal" href={href(g)}>
    <span class="name">{g.name}</span>
    <span class="bar"><i style:width="{(g.have / g.need) * 100}%"></i></span>
    <span class="dim small">
      {#if g.missing.length}
        Short: {g.missing.slice(0, 3).map((m) => `${m.qty}× ${m.name}`).join(", ")}{g.missing.length > 3 ? "…" : ""}
      {:else}
        {verb(g)} now
      {/if}
    </span>
  </a>
{/snippet}

<div class="page">
  {#if home}
    <div class="stats">
      {@render stat("Quests", `${home.quests_done}/${home.quests_total}`, `${home.quests_active} in progress`, "#/quests")}
      {@render stat("Needed", String(home.items_needed), "items to find", "#/profile")}
      {@render stat("Blueprints", `${home.blueprints_found}/${home.blueprints_total}`, "found", "#/codex")}
      {@render stat("Workshop", `${home.bench_levels}/${home.bench_levels_total}`, "levels built", "#/profile")}
    </div>

    {#if blank}
      <section class="panel">
        <h3 class="section-title">Get started</h3>
        <a class="step" href="#/profile"><strong>1.</strong> Set your workbench and Outpost levels</a>
        <a class="step" href="#/quests"><strong>2.</strong> Mark your quests. "Done, with all before it" does a whole chain</a>
        <a class="step" href="#/codex"><strong>3.</strong> Import your blueprints from a Codex screenshot</a>
        <a class="step" href="#/items"><strong>4.</strong> Track something you want from its item page</a>
      </section>
    {/if}

    <div class="columns">
      <section class="panel">
        <h3 class="section-title">Ready ({home.ready.length})</h3>
        {#each home.ready as g (`${g.kind}${g.id}`)}{@render goalRow(g)}{:else}
          <p class="dim small">Nothing ready yet. Add what you own to your stash to see what you can finish.</p>
        {/each}
      </section>
      <section class="panel">
        <h3 class="section-title">Almost there</h3>
        {#each home.almost as g (`${g.kind}${g.id}`)}{@render goalRow(g)}{:else}
          <p class="dim small">Goals you have at least half the items for show up here.</p>
        {/each}
      </section>
      <section class="panel">
        <h3 class="section-title">Right now</h3>
        {#if best}
          <a class="goal" href="#/raid/{best.map_id}">
            <span class="name">Best map: {best.map_name}</span>
            <span class="dim small">{best.covered_items.length} of {plan?.needed.length} needed items</span>
          </a>
        {/if}
        {#each ending as p (p.id)}
          <a class="goal" href="#/projects/{p.id}">
            <span class="name">{p.name}</span>
            <span class="small timer">{p.days} {p.days === 1 ? "day" : "days"} left</span>
          </a>
        {/each}
        {#each liveNow as e}
          <p class="small event"><strong>{e.name}</strong> <span class="dim">{e.map}</span></p>
        {/each}
        {#if events && !liveNow.length}<p class="dim small">No events running.</p>{/if}
      </section>
    </div>
    <p class="dim small">Each goal is judged on its own, so two goals can both count the same stack. Reflects patch {info?.game_patch ?? "unknown"}.</p>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .label {
    color: var(--text-dim);
    font-family: var(--font-head);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .stat strong {
    font-family: var(--font-head);
    font-size: 30px;
    line-height: 1.1;
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
    align-items: start;
  }
  .goal {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
  }
  .name {
    font-weight: 600;
    text-transform: capitalize;
  }
  .bar {
    height: 4px;
    overflow: hidden;
    border-radius: 2px;
    background: var(--panel-2);
  }
  .bar i {
    display: block;
    height: 100%;
    background: var(--teal);
  }
  .step {
    display: block;
    padding: 4px 0;
  }
  .timer {
    color: var(--yellow);
  }
  .event {
    margin: 4px 0;
  }
  .small {
    font-size: 13px;
    margin: 0;
  }
  .section-title {
    margin: 0 0 4px;
  }
</style>
