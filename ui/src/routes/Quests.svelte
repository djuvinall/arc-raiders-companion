<script lang="ts">
  import QuestParty from "../lib/components/QuestParty.svelte";
  import QuestTree from "../lib/components/QuestTree.svelte";
  import { completeImpliedQuests, completeQuestChain, completeQuestsBefore, exportProfile, importProfile, linkHref, mapList, questList, questProgress, setQuestObjective, STEP_KINDS, setQuestState, type GameMap, type QuestRow } from "../lib/api";

  // Positions are wide apart, so the graph scrolls rather than shrinking out of legibility.
  const GRAPH_SCALE = 0.32;
  const COLORS: Record<string, string> = {
    done: "var(--teal)", active: "var(--yellow)", needed: "var(--orange)", available: "var(--cream)", locked: "var(--locked)",
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

  // Quests marked Needed sit before an active one, so the game says they're already done.
  const implied = $derived(quests.filter((q) => q.status === "needed").length);
  const active = $derived(quests.filter((q) => q.status === "active").length);
  let note = $state("");

  // Progress as it was before the last change that touched many quests or cleared one.
  let undoText = $state<string | null>(null);

  async function undoable(action: () => Promise<string>) {
    const before = await exportProfile();
    note = await action();
    undoText = before;
    await load();
  }

  async function undo() {
    if (!undoText) return;
    await importProfile(undoText);
    undoText = null;
    note = "Undone.";
    await load();
  }

  const fillIn = () => undoable(async () => `Marked ${await completeImpliedQuests()} earlier quests done.`);

  const selected = $derived(quests.find((q) => q.quest.id === selectedId) ?? null);
  const traders = $derived([...new Set(quests.map((q) => q.quest.trader).filter(Boolean))].sort() as string[]);
  const atomics = (q: QuestRow) => q.quest.objectives.filter((o) => STEP_KINDS.includes(o.kind));
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
  // In quest-log order with the game's nesting; a map filter keeps only what's left there.
  // With a map chosen, a flat list of what's left there; otherwise the quest log's tree.
  const openHere = $derived(selected && map ? openOnMap(selected) : []);
  const progress = (q: QuestRow) => questProgress(q.quest, q.done_objectives);
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
    const q = selected;
    // Untracking forgets ticked objectives, so that and Reset can be undone.
    if (q && !tracked && (q.status === "done" || q.done_objectives.length)) {
      await undoable(async () => {
        await setQuestState(q.quest.id, tracked, done);
        return q.status === "done" ? `Reset ${q.quest.name}.` : `Stopped tracking ${q.quest.name}; its ticked objectives were cleared.`;
      });
      return;
    }
    await setQuestState(selectedId!, tracked, done);
    await load();
  }

  const doneBefore = (q: QuestRow) => undoable(async () => {
    const n = await completeQuestsBefore(q.quest.id);
    return `Marked ${n} ${n === 1 ? "quest" : "quests"} before ${q.quest.name} done. ${q.quest.name} is tracked.`;
  });

  // Picking the quests you have in game: tick them, then Track. The game only offers a quest
  // once everything before it is done, so offer to mark those done too.
  let picking = $state(false);
  let picked = $state<string[]>([]);
  let askFill = $state(0);

  function pick(id: string) {
    if (quests.find((q) => q.quest.id === id)?.status === "done") return;
    picked = picked.includes(id) ? picked.filter((p) => p !== id) : [...picked, id];
  }

  async function trackPicked() {
    const ids = picked.filter((id) => quests.find((q) => q.quest.id === id)?.status !== "done");
    await undoable(async () => {
      for (const id of ids) await setQuestState(id, true, false);
      return `Tracking ${ids.length} ${ids.length === 1 ? "quest" : "quests"}.`;
    });
    picked = [];
    picking = false;
    askFill = implied;
  }

  async function track(q: QuestRow) {
    await setState(true, false);
    if (!q.open) askFill = implied;
  }

  async function confirmFill() {
    askFill = 0;
    await fillIn();
  }

  const doneChain = (q: QuestRow) => undoable(async () => {
    await completeQuestChain(q.quest.id);
    return `Marked ${q.quest.name} and everything before it done.`;
  });
</script>

<div class="page">
  <div class="main">
    <div class="filters">
      <div class="switch">
        <button class:on={view === "list"} onclick={() => (view = "list")}>List</button>
        <button class:on={view === "graph"} onclick={() => (view = "graph")}>Graph</button>
        <button class:on={view === "friends"} onclick={() => (view = "friends")}>Friends</button>
      </div>
      <input class="field" placeholder="Search quests" aria-label="Search quests" bind:value={search} />
      <select class="field" aria-label="Map" bind:value={map}>
        <option value="">All maps</option>
        {#each maps as m}<option value={m.id}>{m.name}</option>{/each}
      </select>
      <select class="field" aria-label="Trader" bind:value={trader}>
        <option value="">All traders</option>
        {#each traders as t}<option value={t}>{t.replace("-", " ")}</option>{/each}
      </select>
      <select class="field" aria-label="Status" bind:value={status}>
        <option value="">Any status</option>
        {#each Object.keys(COLORS) as s}<option value={s}>{s}</option>{/each}
      </select>
      <span class="dim">{visible.length} of {quests.length}</span>
      {#if view === "list"}
        <button class="button small" class:primary={picking} onclick={() => { picking = !picking; picked = []; }}>
          {picking ? "Cancel" : "Pick my current quests"}
        </button>
      {/if}
    </div>

    <div class="slot">
    {#if picking}
      <div class="implied" role="status">
        <span>Tick every quest you have open in game, then track them all at once. {picked.length} picked.</span>
        <button class="button primary" disabled={!picked.length} onclick={trackPicked}>Track {picked.length || ""}</button>
      </div>
    {:else if implied}
      <div class="implied" role="status">
        <span>
          Your {active} active {active === 1 ? "quest means" : "quests mean"} {implied} earlier
          {implied === 1 ? "quest is" : "quests are"} already done, since the game only offers a quest once everything before it is finished.
        </span>
        <button class="button primary" onclick={fillIn}>Mark them done</button>
      </div>
    {:else if note}
      <p class="dim small note" role="status">{note} {#if undoText}<button class="button small dark" onclick={undo}>Undo</button>{/if}</p>
    {/if}
    </div>

    {#if view === "friends"}
      <QuestParty {maps} {selectedId} onselect={(id) => (selectedId = id)} onchange={load} />
    {:else if view === "list"}
      <div class="list">
        {#each visible as q (q.quest.id)}
          <button class="row" class:picking class:active={selectedId === q.quest.id} onclick={() => (picking ? pick(q.quest.id) : (selectedId = q.quest.id))}>
            {#if picking}<input type="checkbox" tabindex="-1" aria-label="Pick {q.quest.name}" checked={picked.includes(q.quest.id)} disabled={q.status === "done"} />{/if}
            <i style:background={COLORS[q.status]}></i>
            <strong>{q.quest.name}</strong>
            <span class="dim">{q.quest.trader?.replace("-", " ") ?? ""}</span>
            <span class="dim">{progress(q).done}/{progress(q).total}</span>
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
            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), (selectedId = q.quest.id))}
          >
            <circle cx={p.x} cy={p.y} r={selectedId === q.quest.id ? 36 : 24} style:fill={COLORS[q.status]} />
            <title>{q.quest.name}</title>
            <text x={p.x} y={p.y + 64} text-anchor="middle">{q.quest.name}</text>
          </g>
        {/each}
      </svg>
      </div>
    {/if}
  </div>

  <aside class="panel" data-report={selected ? `quests/${selected.quest.id}` : undefined} data-report-label={selected?.quest.name}>
    {#if selected}
      <h2 data-report="quests/{selected.quest.id}/name" data-report-label="Name">{selected.quest.name}</h2>
      <p class="dim">
        <span data-report="quests/{selected.quest.id}/trader" data-report-label="Trader">{selected.quest.trader?.replace("-", " ") ?? "No trader"}</span> · {selected.status}
        {#if selected.quest.xp}· <span data-report="quests/{selected.quest.id}/xp" data-report-label="XP">{selected.quest.xp.toLocaleString()} XP</span>{/if}
      </p>
      {#if selected.maps.length}<p class="dim small">{selected.maps.map(mapName).join(", ")}</p>{/if}

      <h3 class="section-title">Objectives</h3>
      <div data-report-label="Objectives">
      {#if map}
        {#each openHere as o (o.id)}
          <label class="objective">
            <input type="checkbox" checked={false} onchange={(e) => tick(o.id, e.currentTarget.checked)} />
            <span>{o.description}</span>
          </label>
        {:else}
          <p class="dim small">Nothing left to do on this map.</p>
        {/each}
      {:else if selected.quest.objectives.length > 1}
        <QuestTree quest={selected.quest} done={selected.done_objectives} ontick={tick} />
      {:else}
        <p class="dim small">No objectives listed.</p>
      {/if}
      </div>

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
          {#if selected.status === "active"}
            <button class="button" onclick={() => setState(false, false)}>Stop tracking</button>
          {:else}
            <button class="button" onclick={() => selected && track(selected)}>Track</button>
          {/if}
          <button class="button primary" onclick={() => setState(true, true)}>Mark done</button>
        {/if}
        {#if selected.status !== "done" && !selected.open}
          <button class="button dark" title="Marks every earlier quest done and tracks this one" onclick={() => selected && doneBefore(selected)}>I'm on this one: all before it done</button>
          <button class="button dark" onclick={() => selected && doneChain(selected)}>Done, with all before it</button>
        {/if}
      </div>
    {:else}
      <p class="dim">Pick a quest to see its objectives. Tracked quests feed the map route.</p>
    {/if}
  </aside>
</div>

<svelte:window onkeydowncapture={(e) => { if (askFill && e.key === "Escape") { e.preventDefault(); e.stopPropagation(); askFill = 0; } }} />

{#if askFill}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="scrim" onclick={() => (askFill = 0)}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="dialog panel" role="dialog" aria-modal="true" aria-label="Earlier quests" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <h3 class="section-title">Mark earlier quests done?</h3>
      <p>
        The game only offers a quest once everything before it is finished, so the {askFill} earlier
        {askFill === 1 ? "quest is" : "quests are"} already done. Mark {askFill === 1 ? "it" : "them"} done here too?
      </p>
      <p class="dim small">You can undo this.</p>
      <div class="actions">
        <button class="button" onclick={() => (askFill = 0)}>Not now</button>
        <button class="button primary" onclick={confirmFill}>Mark {askFill} done</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 22;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 14vh;
    background: rgba(0, 0, 0, 0.55);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(480px, 92vw);
  }
  .dialog .actions {
    justify-content: flex-end;
  }
  .row.picking {
    grid-template-columns: 16px 12px 1fr 90px 50px;
  }
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
    grid-template-rows: auto auto 1fr;
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
  }
  .row.active {
    background: var(--panel-2);
  }
  .row i {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }
  .implied {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border: 1px solid var(--yellow);
    border-radius: 6px;
    font-size: 13px;
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
  .row:hover:not(.active) {
    background: var(--panel-2);
  }
</style>
