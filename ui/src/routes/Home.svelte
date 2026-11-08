<script lang="ts">
  import {
    getProfile, homeView, linkHref, liveEvents, mapList, patchNotes, projectList,
    type EventsView, type GameMap, type Goal, type Home, type Overview, type PatchNote, type Profile, type ProjectView,
  } from "../lib/api";

  /** One card per tab with the thing worth knowing from it, each linking straight there. */
  let { info }: { info: Overview | null } = $props();
  let home = $state<Home | null>(null);
  let events = $state<EventsView | null>(null);
  let projects = $state<ProjectView[]>([]);
  let profile = $state<Profile | null>(null);
  let maps = $state<GameMap[]>([]);
  let patch = $state<PatchNote | null>(null);
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
  projectList().then((p) => (projects = p));
  getProfile().then((p) => (profile = p));
  mapList().then((m) => (maps = m));
  liveEvents(region).then((e) => (events = e), () => {});
  patchNotes().then((n) => (patch = n.notes.find((x) => x.version) ?? n.notes[0] ?? null), () => {});

  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
  const href = (g: Goal) => ({ craft: `#/craft/${g.id}`, quest: `#/quests/${g.id}`, project: `#/projects/${g.id}`, bench: "#/profile" })[g.kind];
  const verb = (g: Goal) => ({ craft: "Craft", quest: "Turn in", project: "Commit", bench: "Upgrade" })[g.kind];
  const endsAt = (p: ProjectView) => Date.parse(`${p.ends}T23:59:59Z`);
  const daysLeft = (p: ProjectView) => Math.ceil((endsAt(p) - now) / DAY);

  const plans = $derived(profile?.raid_plans ?? []);
  const inRaid = $derived(plans.find((p) => p.status === "in_raid") ?? null);
  const liveNow = $derived(events?.maps.filter((m) => m.current.length) ?? []);
  const projectRows = $derived(projects
    .filter((p) => !p.paused && p.stages.length && p.stage <= p.stages.length && (p.tracked || (p.ends && endsAt(p) >= now)))
    .filter((p) => p.tracked || !p.permanent)
    .map((p) => {
      const stage = p.stages.find((s) => s.n === p.stage)!;
      const left = stage.inputs.filter((i) => i.committed < i.need);
      const task = stage.task;
      const step = task?.objectives.find((o) => o.kind === "atomic" && !task.done.includes(o.id));
      const detail = step?.description ?? (left.length ? `${left.length} ${left.length === 1 ? "item" : "items"} to commit` : "Ready to finish");
      return { ...p, stageName: stage.title, detail, days: p.ends ? daysLeft(p) : null };
    }));
  const benchGoals = $derived([...(home?.ready ?? []), ...(home?.almost ?? [])].filter((g) => g.kind === "bench").slice(0, 3));
  const otherReady = $derived(home?.ready.filter((g) => g.kind !== "bench") ?? []);
  const almost = $derived(home?.almost.filter((g) => g.kind !== "bench").slice(0, 3) ?? []);
  const starred = $derived(profile?.skill_plans.find((p) => p.name === profile?.default_skill_plan) ?? profile?.skill_plans[0] ?? null);
  const points = (ranks: Record<string, number>) => Object.values(ranks).reduce((a, b) => a + b, 0);
  const blank = $derived(home && !home.quests_active && !home.quests_done && !home.bench_levels && !home.items_needed && !home.blueprints_found);
  const date = (secs: number) => new Date(secs * 1000).toLocaleDateString(undefined, { month: "short", day: "numeric" });
</script>

{#snippet goalRow(g: Goal)}
  <a class="row" href={href(g)}>
    <span class="name">{g.missing.length ? g.name : `${verb(g)}: ${g.name}`}</span>
    {#if g.missing.length}
      <span class="bar"><i style:width="{(g.have / g.need) * 100}%"></i></span>
      <span class="dim small">Short: {g.missing.slice(0, 3).map((m) => `${m.qty}× ${m.name}`).join(", ")}{g.missing.length > 3 ? "…" : ""}</span>
    {/if}
  </a>
{/snippet}

<div class="page">
  {#if home}
    {#if blank}
      <section class="panel start">
        <h3 class="section-title">Get started</h3>
        <a class="step" href="#/profile"><strong>1.</strong> Set your workbench and Outpost levels</a>
        <a class="step" href="#/quests"><strong>2.</strong> Mark your quests. "Done, with all before it" does a whole chain</a>
        <a class="step" href="#/codex"><strong>3.</strong> Import your blueprints from a Codex screenshot</a>
        <a class="step" href="#/items"><strong>4.</strong> Track something you want from its item page</a>
      </section>
    {/if}

    <div class="cards">
      <section class="panel card">
        <a class="head" href="#/raid"><span>Raid</span><span class="go">›</span></a>
        {#if inRaid}
          <a class="big" href="#/raid/plan/{inRaid.id}">{inRaid.name}</a>
          <span class="dim small">In raid on {mapName(inRaid.map_id)} · {inRaid.checked.length} {inRaid.checked.length === 1 ? "stop" : "stops"} ticked</span>
        {:else if plans.length}
          {#each plans.slice(-3).reverse() as p (p.id)}
            <a class="row" href="#/raid/plan/{p.id}"><span class="name">{p.name}</span><span class="dim small">{mapName(p.map_id)}</span></a>
          {/each}
        {:else}
          <p class="dim small">Plan a raid around what you've tracked: the app picks the areas worth heading for.</p>
        {/if}
        <div class="links"><a href="#/raid">Plan a raid</a><a href="#/raid/farm">Farm an item</a></div>
      </section>

      {#if liveNow.length}
        <section class="panel card">
          <a class="head" href="#/maps"><span>Live events</span><span class="go">›</span></a>
          {#each liveNow as m (m.map_id)}
            <a class="row" href="#/maps/{m.map_id}">
              <span class="name">{m.current.map((e) => e.name).join(", ")}</span>
              <span class="dim small">{m.map_name}{m.next ? ` · next: ${m.next.name}` : ""}</span>
            </a>
          {/each}
        </section>
      {/if}

      <section class="panel card">
        <a class="head" href="#/quests"><span>Quests</span><span class="go">›</span></a>
        <span class="big">{home.quests_active} active</span>
        <span class="dim small">{home.quests_done} of {home.quests_total} done</span>
        {#each home.quest_next.slice(0, 4) as q (q.id)}
          <a class="row" href="#/quests/{q.id}">
            <span class="name">{q.name}</span>
            <span class="dim small">{q.step ?? "Every step done: turn it in"}{q.maps.length ? ` · ${q.maps.map(mapName).join(", ")}` : ""}</span>
          </a>
        {:else}
          <p class="dim small">Track quests to see each one's next step here.</p>
        {/each}
        {#if home.quest_next.length > 4}<a class="small more" href="#/quests">And {home.quest_next.length - 4} more</a>{/if}
      </section>

      {#if projectRows.length}
        <section class="panel card">
          <a class="head" href="#/projects"><span>Projects</span><span class="go">›</span></a>
          {#each projectRows as p (p.id)}
            <a class="row" href="#/projects/{p.id}">
              <span class="name">{p.name} <span class="dim">· stage {p.stage}/{p.stages.length}</span>{#if p.days !== null}<span class="timer">{p.days} {p.days === 1 ? "day" : "days"}</span>{/if}</span>
              <span class="dim small">{p.tracked ? p.detail : "Not tracked"}</span>
            </a>
          {/each}
        </section>
      {/if}

      {#if otherReady.length || almost.length}
        <section class="panel card">
          <span class="head"><span>{otherReady.length ? "Ready now" : "Almost there"}</span></span>
          {#each otherReady.slice(0, 5) as g (`${g.kind}${g.id}`)}{@render goalRow(g)}{/each}
          {#if almost.length}
            {#if otherReady.length}<span class="sub">Almost there</span>{/if}
            {#each almost as g (`${g.kind}${g.id}`)}{@render goalRow(g)}{/each}
          {/if}
        </section>
      {/if}

      {#if home.needed_top.length}
        <section class="panel card">
          <a class="head" href="#/profile"><span>Still needed</span><span class="go">›</span></a>
          <span class="big">{home.items_needed} {home.items_needed === 1 ? "item" : "items"}</span>
          {#each home.needed_top as n (n.item.id)}
            <a class="row" href={linkHref(n.item)}>
              <span class="name">{n.qty}× {n.item.name}</span>
              <span class="dim small">{n.reasons.slice(0, 2).join(" · ")}{n.reasons.length > 2 ? "…" : ""}</span>
            </a>
          {/each}
        </section>
      {/if}

      <section class="panel card">
        <a class="head" href="#/profile"><span>Workbenches</span><span class="go">›</span></a>
        <span class="big">{home.bench_levels}/{home.bench_levels_total}</span>
        <span class="dim small">levels built</span>
        {#each benchGoals as g (g.id)}{@render goalRow(g)}{:else}
          <p class="dim small">Add what you own to your stash to see which bench you can upgrade next.</p>
        {/each}
      </section>

      {#if starred}
        <section class="panel card">
          <a class="head" href="#/skills"><span>Skills</span><span class="go">›</span></a>
          <span class="big">{starred.name}</span>
          <span class="dim small">{points(starred.ranks)} points planned{starred.name === profile?.default_skill_plan ? " · opens by default" : ""}</span>
        </section>
      {/if}

      <section class="panel card">
        <a class="head" href="#/codex"><span>Codex</span><span class="go">›</span></a>
        <span class="big">{home.blueprints_found}/{home.blueprints_total}</span>
        <span class="dim small">blueprints found</span>
      </section>

      <section class="panel card">
        <a class="head" href="#/patches"><span>Patches</span><span class="go">›</span></a>
        {#if patch}
          <a class="row" href="#/patches"><span class="name">{patch.title}</span><span class="dim small">{date(patch.date)}</span></a>
        {/if}
        <span class="dim small">Data {info?.dataset_version ?? ""}, checked against patch {info?.game_patch ?? "unknown"}</span>
      </section>
    </div>
    <p class="dim small">Each goal is judged on its own, so two goals can both count the same stack.</p>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 10px;
    align-items: start;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    margin-bottom: 4px;
    color: var(--text-dim);
    font-family: var(--font-head);
    font-size: 15px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  a.head:hover {
    color: var(--text);
  }
  .go {
    font-size: 18px;
    line-height: 1;
  }
  .big {
    font-family: var(--font-head);
    font-size: 26px;
    line-height: 1.1;
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-of-type {
    border-bottom: 0;
  }
  .name {
    font-weight: 600;
  }
  .timer {
    margin-left: 6px;
    padding: 0 6px;
    border-radius: 3px;
    background: var(--yellow);
    color: var(--on-cream);
    font-size: 12px;
    font-weight: 400;
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
  .sub {
    margin-top: 8px;
    color: var(--text-dim);
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .links {
    display: flex;
    gap: 14px;
    margin-top: 6px;
  }
  .links a,
  .more {
    color: var(--teal);
  }
  .step {
    display: block;
    padding: 4px 0;
  }
  .small {
    font-size: 13px;
    margin: 0;
  }
  .section-title {
    margin: 0 0 4px;
  }
</style>
