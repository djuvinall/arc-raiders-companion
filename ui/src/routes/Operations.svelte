<script lang="ts">
  import { linkHref, liveEvents, operationList, type EventsView, type GearCheck, type OperationView } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  let { id }: { id?: string } = $props();
  let ops = $state<OperationView[]>([]);
  let events = $state<EventsView | null>(null);
  // Routes remount on a new address, so the first value is the only one.
  // svelte-ignore state_referenced_locally
  let selectedId = $state<string | null>(id ?? null);

  const region = (() => {
    try {
      return localStorage.getItem("event-region") ?? "north-america";
    } catch {
      return "north-america";
    }
  })();

  operationList().then((all) => {
    ops = all;
    selectedId ??= all[0]?.op.id ?? null;
  });
  liveEvents(region).then((e) => (events = e), () => {});

  const selected = $derived(ops.find((o) => o.op.id === selectedId) ?? null);
  const KIND: Record<string, string> = { operation: "ARC Operation", boss: "Boss event", event: "Map event" };

  /** Maps where it's on now, and the next slot elsewhere. */
  function when(o: OperationView) {
    const now: string[] = [];
    let next: { map: string; at: number } | null = null;
    for (const m of events?.maps ?? []) {
      if (m.current.some((e) => e.name.toLowerCase() === o.op.condition.toLowerCase())) now.push(m.map_name);
      if (m.next && m.next.name.toLowerCase() === o.op.condition.toLowerCase() && (!next || m.next.startTime < next.at)) {
        next = { map: m.map_name, at: m.next.startTime };
      }
    }
    return { now, next };
  }
  const clock = (ms: number) => new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  const minimum = $derived(selected?.gear.filter((g) => g.need === "minimum") ?? []);
  const recommended = $derived(selected?.gear.filter((g) => g.need === "recommended") ?? []);
  const gearLine = (g: GearCheck) =>
    g.names.length ? g.names.join(" or ") : `A weapon with ${g.armor_pen} ARC armour penetration or better`;
</script>

{#snippet gearRow(g: GearCheck)}
  <div class="gear" class:met={g.met}>
    <span class="tick" aria-label={g.met ? "In your stash" : "Not in your stash"}>{g.met ? "✓" : "–"}</span>
    <span class="icons">{#each g.examples.slice(0, 3) as ex (ex.id)}<Icon url={ex.icon} name={ex.name} size={28} />{/each}</span>
    <span>
      {gearLine(g)}{#if g.qty > 1} ×{g.qty}{/if}
      <span class="dim small"> · have {g.have}{#if g.note} · {g.note}{/if}</span>
    </span>
  </div>
{/snippet}

<div class="page">
  <aside>
    <nav class="switch" aria-label="Guide">
      <a href="#/arcs">ARCs</a>
      <a href="#/arcs/ops" class="on" aria-current="page">Operations</a>
    </nav>
    {#each ops as o (o.op.id)}
      {@const w = when(o)}
      <button class="row" class:on={o.op.id === selectedId} onclick={() => (selectedId = o.op.id)}>
        <span class="name">{o.op.name}</span>
        <span class="dim small">{KIND[o.op.kind] ?? o.op.kind}</span>
        {#if w.now.length}<span class="live small">Live: {w.now.join(", ")}</span>
        {:else if w.next}<span class="dim small">Next: {w.next.map}, {clock(w.next.at)}</span>{/if}
      </button>
    {/each}
  </aside>

  {#if selected}
    {@const op = selected.op}
    {@const w = when(selected)}
    <section class="panel detail" data-report="operations/{op.id}" data-report-label={op.name}>
      <div class="head">
        <div>
          <h2>{op.name}</h2>
          <p class="dim">
            <span class="kind {op.kind}">{KIND[op.kind] ?? op.kind}</span>
            {#if op.introduced}· since {op.introduced}{/if}
            · <span class="confidence {op.confidence}" title="How well the sources agree">{op.confidence === "in-game" ? "checked in game" : op.confidence}</span>
          </p>
          {#if w.now.length}<p class="live">Live now on {w.now.join(", ")}.</p>
          {:else if w.next}<p class="dim">Next: {w.next.map} at {clock(w.next.at)}.</p>{/if}
        </div>
      </div>

      <p class="summary">{op.summary}</p>
      <p class="dim">{op.starts}</p>

      <div class="facts">
        <div>
          <h3 class="section-title">Maps</h3>
          {#each selected.maps as m (m.map_id)}
            <p>
              <a href="#/raid/{m.map_id}" title="Plan a raid here">{m.map_name}</a>
              {#if m.spots}<span class="dim small"> · {m.spots} marked {m.spots === 1 ? "spot" : "spots"}</span>{/if}
            </p>
          {/each}
        </div>
        <div>
          <h3 class="section-title">Team</h3>
          <p>{op.team.min === op.team.best ? op.team.best : `${op.team.min} to ${op.team.best}`} {op.team.best === 1 ? "player" : "players"}</p>
          {#if op.team.note}<p class="dim small">{op.team.note}</p>{/if}
        </div>
        <div>
          <h3 class="section-title">In the raid</h3>
          {#if op.route.raid_minutes && op.route.raid_minutes !== 30}<p>The raid lasts {op.route.raid_minutes} minutes.</p>{/if}
          {#if op.route.start_minute}<p>Starts about {op.route.start_minute} minutes in.</p>{/if}
          {#if op.route.minutes}<p>Takes about {op.route.minutes} minutes.</p>{/if}
          {#if op.route.window_minutes}<p>About {op.route.window_minutes} minutes before it's lost.</p>{/if}
          {#if op.route.pulls_lobby}<p>The whole lobby heads for it.</p>{/if}
          {#if op.route.no_hatches}<p>Raider Hatches don't work.</p>{/if}
        </div>
      </div>

      {#if op.map_effects.length}
        <h3 class="section-title">What changes on the map</h3>
        <ul>{#each op.map_effects as e}<li>{e}</li>{/each}</ul>
      {/if}

      <h3 class="section-title">Step by step</h3>
      <ol class="steps">
        {#each op.steps as step, i (i)}
          <li><strong>{step.name}.</strong> {step.what}</li>
        {/each}
      </ol>

      <h3 class="section-title">Gear</h3>
      <p class="verdict" class:ready={selected.ready}>
        {selected.ready ? "Your stash covers the minimum." : "Your stash is short of the minimum below."}
      </p>
      {#if minimum.length}<h4 class="minor">Minimum</h4>{#each minimum as g, i (i)}{@render gearRow(g)}{/each}{/if}
      {#if recommended.length}<h4 class="minor">Recommended</h4>{#each recommended as g, i (i)}{@render gearRow(g)}{/each}{/if}
      {#each op.gear_notes as note}<p class="dim small">{note}</p>{/each}
      <p class="dim small">No source publishes a minimum. These lines are our reading of the guides for patch 2.4.1, and the ticks count what's in your stash.</p>

      {#if selected.enemies.length}
        <h3 class="section-title">Enemies</h3>
        <div class="chips">
          {#each selected.enemies as e (e.id)}
            <a href="#/arcs/{e.id}"><Icon url={e.icon} name={e.name} size={32} /><span>{e.name}</span></a>
          {/each}
        </div>
        {#each op.enemy_notes as note}<p class="small">{note}</p>{/each}
      {/if}

      {#if selected.rewards.length}
        <h3 class="section-title">Rewards</h3>
        <div class="chips">
          {#each selected.rewards as r (r.item.id)}
            <a href={linkHref(r.item)} title={r.from}>
              <Icon url={r.item.icon} name={r.item.name} size={32} />
              <span>{r.item.name}</span>
              <span class="chance {r.chance}">{r.chance}</span>
            </a>
          {/each}
        </div>
        {#each op.reward_notes as note}<p class="small">{note}</p>{/each}
      {/if}

      <h3 class="section-title">What to expect</h3>
      {#if op.expect.pvp}<p><strong>Other players.</strong> {op.expect.pvp}</p>{/if}
      {#if op.expect.extraction}<p><strong>Getting out.</strong> {op.expect.extraction}</p>{/if}
      {#each op.expect.other as line}<p>{line}</p>{/each}
      {#each op.route.notes as line}<p class="dim">{line}</p>{/each}

      {#if op.mistakes.length}
        <h3 class="section-title">Common mistakes</h3>
        <ul>{#each op.mistakes as m}<li>{m}</li>{/each}</ul>
      {/if}

      {#if op.conflicts.length || op.checks.length}
        <h3 class="section-title">Not settled</h3>
        {#each op.conflicts as c}<p class="small">{c}</p>{/each}
        {#if op.checks.length}<p class="small dim">To check in game: {op.checks.join(" ")}</p>{/if}
      {/if}

      {#if op.patches.length}
        <h3 class="section-title">Patch history</h3>
        {#each op.patches as p (p.patch)}
          <p class="small"><strong>{p.patch}</strong>{#if p.date} <span class="dim">({p.date})</span>{/if}: {p.change}</p>
        {/each}
      {/if}

      <h3 class="section-title">Sources</h3>
      {#each op.sources as s (s.url)}<p class="small dim"><span class="url">{s.url}</span> · {s.revision}, read {s.retrieved}</p>{/each}
      <p class="dim small">Written from the ARC Raiders Wiki (CC BY-SA 4.0), Embark's patch notes and the official map-conditions schedule, plus the dated guides above. Not yet checked in game.</p>
    </section>
  {:else if !ops.length}
    <p class="dim pad">No operations in this data yet.</p>
  {/if}
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr);
    gap: 12px;
    padding: 12px 16px;
    height: 100%;
    min-height: 0;
  }
  aside,
  .detail {
    overflow: auto;
    min-height: 0;
  }
  .switch {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
  }
  .switch a {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
    font-family: var(--font-head);
    letter-spacing: 0.06em;
    text-align: center;
    text-transform: uppercase;
  }
  .switch a.on {
    background: var(--panel-2);
    border-color: var(--yellow);
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 1px;
    width: 100%;
    padding: 6px 8px;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .row.on,
  .row:hover {
    background: var(--panel-2);
  }
  .name {
    font-weight: 600;
  }
  .live {
    color: var(--yellow);
  }
  h2 {
    font-size: 28px;
  }
  p {
    margin: 4px 0;
    max-width: 75ch;
  }
  .summary {
    font-size: 17px;
  }
  .kind {
    font-family: var(--font-head);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .kind.operation {
    color: var(--yellow);
  }
  .confidence.conflicting,
  .confidence.unverified {
    color: #d9a441;
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 6px 24px;
  }
  .section-title {
    margin: 16px 0 4px;
  }
  h4.minor {
    margin: 8px 0 2px;
  }
  ul,
  .steps {
    margin: 4px 0;
    padding-left: 20px;
    max-width: 75ch;
  }
  li {
    margin: 3px 0;
  }
  .verdict {
    font-weight: 600;
    color: #d9a441;
  }
  .verdict.ready {
    color: #7fc97f;
  }
  .gear {
    display: grid;
    grid-template-columns: 18px auto 1fr;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
  }
  .gear .tick {
    color: var(--dim, #888);
    font-weight: 700;
  }
  .gear.met .tick {
    color: #7fc97f;
  }
  .icons {
    display: flex;
    gap: 2px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chips a {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px 3px 4px;
    background: var(--panel-2);
    border-radius: 4px;
  }
  .chance {
    padding: 0 6px;
    border-radius: 999px;
    font-size: 12px;
    background: var(--border);
  }
  .chance.guaranteed {
    background: #2f6b3a;
    color: #fff;
  }
  .chance.likely {
    background: #5a6b2f;
    color: #fff;
  }
  .small {
    font-size: 13px;
  }
  .url {
    user-select: all;
  }
  .pad {
    padding: 16px;
  }
</style>
