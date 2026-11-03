<script lang="ts">
  import QuestParty from "../lib/components/QuestParty.svelte";
  import { completeQuestChain, linkHref, mapList, questList, setQuestObjective, setQuestState, type GameMap, type QuestRow } from "../lib/api";

  // Positions are wide apart, so the graph scrolls rather than shrinking out of legibility.
  const GRAPH_SCALE = 0.32;
  const COLORS: Record<string, string> = {
    done: "#2ab0c6", active: "#f5c400", needed: "#ff8a3d", available: "#ece6d6", locked: "#6b7080",
  };

  let quests = $state<QuestRow[]>([]);
  let maps = $state<GameMap[]>([]);
  let view = $state<"list" | "graph" | "friends">("list");
  let { id }: { id?: string } = $props();
  // Routes remount on a new address, so the first value is the only one.
  // svelte-ignore state_referenced_locally
  let selectedId = $state<string | null>(id ?? null);
  let map = $state("");
  let trader = $state("");
  let status = $state("");
  let search = $state("");

  const load = async () => (quests = await questList());
  load();
  mapList().then((m) => (maps = m));

  const selected = $derived(quests.find((q) => q.quest.id === selectedId) ?? null);
  const traders = $derived([...new Set(quests.map((q) => q.quest.trader).filter(Boolean))].sort() as string[]);
  const atomics = (q: QuestRow) => q.quest.objectives.filter((o) => o.kind === "atomic");
  // With a map chosen, only what's still to do there counts.
  const openOnMap = (q: QuestRow) =>
    atomics(q).filter((o) => o.maps.includes(map) && !q.done_objectives.includes(o.id));
  const visible = $derived(
    quests.filter((q) =>
      (!map || openOnMap(q).length > 0) &&
      (!trader || q.quest.trader === trader) &&
      (!status || q.status === status) &&
      (!search || q.quest.name.toLowerCase().includes(search.toLowerCase())),
    ),
  );
  const objectives = $derived(selected ? (map ? openOnMap(selected) : atomics(selected)) : []);
  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;

  const graph = $derived.by(() => {
    const placed = quests.filter((q) => q.quest.graph_position);
    if (!placed.length) return null;
    const xs = placed.map((q) => q.quest.graph_position!.x);
    const ys = placed.map((q) => q.quest.graph_position!.y);
    const pad = 260;
    const [x0, y0] = [Math.min(...xs) - pad, Math.min(...ys) - pad];
    const [w, h] = [Math.max(...xs) + pad - x0, Math.max(...ys) + pad - y0];
    const edges = placed.flatMap((q) =>
      q.quest.prerequisites.flatMap((p) => {
        const from = placed.find((o) => o.quest.id === p.quest_id);
        return from ? [{ from: from.quest.graph_position!, to: q.quest.graph_position!, id: `${from.quest.id}>${q.quest.id}` }] : [];
      }),
    );
    return { placed, edges, box: `${x0} ${y0} ${w} ${h}`, width: w * GRAPH_SCALE, height: h * GRAPH_SCALE };
  });

  async function tick(objective: string, done: boolean) {
    await setQuestObjective(selectedId!, objective, done);
    await load();
  }

  async function setState(tracked: boolean, done: boolean) {
    await setQuestState(selectedId!, tracked, done);
    await load();
  }
</script>

<div class="page">
  <div class="main">
    <div class="filters">
      <div class="switch">
        <button class:on={view === "list"} onclick={() => (view = "list")}>List</button>
        <button class:on={view === "graph"} onclick={() => (view = "graph")}>Graph</button>
        <button class:on={view === "friends"} onclick={() => (view = "friends")}>Friends</button>
      </div>
      <input class="field" placeholder="Search quests" bind:value={search} />
      <select class="field" aria-label="Map" bind:value={map}>
        <option value="">All maps</option>
        {#each maps as m}<option value={m.id}>{m.name}</option>{/each}
      </select>
      <select class="field" aria-label="Trader" bind:value={trader}>
        <option value="">All traders</option>
        {#each traders as t}<option value={t}>{t}</option>{/each}
      </select>
      <select class="field" aria-label="Status" bind:value={status}>
        <option value="">Any status</option>
        {#each Object.keys(COLORS) as s}<option value={s}>{s}</option>{/each}
      </select>
      <span class="dim">{visible.length} of {quests.length}</span>
    </div>

    {#if view === "friends"}
      <QuestParty {maps} {selectedId} onselect={(id) => (selectedId = id)} />
    {:else if view === "list"}
      <div class="list">
        {#each visible as q (q.quest.id)}
          <button class="row" class:active={selectedId === q.quest.id} onclick={() => (selectedId = q.quest.id)}>
            <i style:background={COLORS[q.status]}></i>
            <strong>{q.quest.name}</strong>
            <span class="dim">{q.quest.trader ?? ""}</span>
            <span class="dim">{q.done_objectives.length}/{atomics(q).length}</span>
          </button>
        {/each}
      </div>
    {:else if graph}
      <div class="graph-scroll">
      <svg width={graph.width} height={graph.height} viewBox={graph.box} role="group" aria-label="Quest graph">
        {#each graph.edges as edge (edge.id)}
          <line x1={edge.from.x} y1={edge.from.y} x2={edge.to.x} y2={edge.to.y} stroke="#3a3d47" stroke-width="6" />
        {/each}
        {#each graph.placed as q (q.quest.id)}
          {@const p = q.quest.graph_position!}
          <g
            role="button" tabindex="0" aria-label={q.quest.name} class="gnode"
            class:dimmed={!visible.includes(q)}
            onclick={() => (selectedId = q.quest.id)}
            onkeydown={(e) => e.key === "Enter" && (selectedId = q.quest.id)}
          >
            <circle cx={p.x} cy={p.y} r={selectedId === q.quest.id ? 36 : 24} fill={COLORS[q.status]} />
            <title>{q.quest.name}</title>
            <text x={p.x} y={p.y + 64} text-anchor="middle">{q.quest.name}</text>
          </g>
        {/each}
      </svg>
      </div>
    {/if}
  </div>

  <aside class="panel">
    {#if selected}
      <h2>{selected.quest.name}</h2>
      <p class="dim">
        {selected.quest.trader ?? "No trader"} · {selected.status}
        {#if selected.quest.xp}· {selected.quest.xp.toLocaleString()} XP{/if}
      </p>
      {#if selected.maps.length}<p class="dim small">{selected.maps.map(mapName).join(", ")}</p>{/if}

      <h3 class="section-title">Objectives</h3>
      {#each objectives as o (o.id)}
        <label class="objective">
          <input
            type="checkbox" checked={selected.done_objectives.includes(o.id)}
            onchange={(e) => tick(o.id, e.currentTarget.checked)}
          />
          <span>{o.description}</span>
        </label>
      {:else}
        <p class="dim small">{map ? "Nothing left to do on this map." : "No objectives listed."}</p>
      {/each}

      {#if selected.needed_for.length}
        <h3 class="section-title">Needed for</h3>
        {#each selected.needed_for as link}
          <button class="line plain" onclick={() => (selectedId = link.id)}>{link.name}</button>
        {/each}
        {#if !selected.open}<p class="dim small">An earlier quest has to come first.</p>{/if}
      {/if}

      {#if selected.required.length}
        <h3 class="section-title">Needs</h3>
        {#each selected.required as link}<a class="line" href={linkHref(link)}>{link.qty}× {link.name}</a>{/each}
      {/if}
      {#if selected.prerequisites.length}
        <h3 class="section-title">After</h3>
        {#each selected.prerequisites as link}
          <button class="line plain" onclick={() => (selectedId = link.id)}>{link.name}</button>
        {/each}
      {/if}
      {#if selected.rewards.length}
        <h3 class="section-title">Rewards</h3>
        {#each selected.rewards as link}<a class="line" href={linkHref(link)}>{link.qty}× {link.name}</a>{/each}
      {/if}

      <div class="actions">
        {#if selected.status === "done"}
          <button class="button dark" onclick={() => setState(false, false)}>Reset</button>
        {:else}
          <button class="button" onclick={() => setState(selected?.status !== "active", false)}>
            {selected.status === "active" ? "Stop tracking" : "Track"}
          </button>
          <button class="button primary" onclick={() => setState(true, true)}>Mark done</button>
        {/if}
        {#if selected.status !== "done" && !selected.open}
          <button class="button dark" onclick={async () => { await completeQuestChain(selectedId!); await load(); }}>Done, with all before it</button>
        {/if}
      </div>
    {:else}
      <p class="dim">Pick a quest to see its objectives. Tracked quests feed the map route.</p>
    {/if}
  </aside>
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 360px;
    gap: 12px;
    padding: 12px 16px;
    height: 100%;
    min-height: 0;
  }
  .main {
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 8px;
    min-height: 0;
  }
  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  select {
    text-transform: capitalize;
  }
  .switch {
    display: flex;
    gap: 2px;
  }
  .switch button {
    padding: 4px 14px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  .switch .on {
    border-color: var(--text);
    color: var(--text);
  }
  .list {
    overflow: auto;
    min-height: 0;
  }
  .row {
    display: grid;
    grid-template-columns: 12px 1fr 90px 50px;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
    text-transform: capitalize;
  }
  .row.active {
    background: var(--panel-2);
  }
  .row i {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }
  .graph-scroll {
    overflow: auto;
    min-height: 0;
    background: var(--panel);
    border-radius: 6px;
  }
  .gnode {
    cursor: pointer;
  }
  .gnode.dimmed {
    opacity: 0.25;
  }
  .gnode text {
    fill: var(--text);
    font-size: 36px;
    pointer-events: none;
  }
  aside {
    overflow: auto;
    min-height: 0;
  }
  h2 {
    font-size: 24px;
  }
  .section-title {
    margin: 14px 0 6px;
  }
  .small {
    font-size: 13px;
  }
  p {
    margin: 4px 0;
  }
  .objective {
    display: flex;
    gap: 8px;
    padding: 3px 0;
    align-items: flex-start;
  }
  .line {
    display: block;
    padding: 2px 0;
  }
  .plain {
    background: none;
    border: 0;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
  }
</style>
