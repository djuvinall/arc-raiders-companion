<script lang="ts">
  import { commitResource, linkHref, projectList, reopenProjectStage, setProjectObjective, setProjectTracked, uncommitResource, type ProjectView } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";
  import QuestTree from "../lib/components/QuestTree.svelte";

  let { id }: { id?: string } = $props();
  let projects = $state<ProjectView[]>([]);
  // Routes remount on a new address, so the first value is the only one.
  // svelte-ignore state_referenced_locally
  let selectedId = $state<string | null>(id ?? null);
  let inputId = $state<string | null>(null);
  let amount = $state(1);
  let message = $state("");
  let showPast = $state(false);
  // The last commit, so one click can undo it, even when it finished the stage.
  let last = $state<{ project: string; item: string; name: string; qty: number; stage: number } | null>(null);
  let confirmClear = $state(false);
  // One action at a time, so a double click can't commit twice or step back two stages.
  let busy = $state(false);

  async function run(action: () => Promise<void>) {
    if (busy) return;
    busy = true;
    try {
      await action();
    } catch (e) {
      message = `Couldn't save that: ${e}`;
    } finally {
      busy = false;
    }
  }
  const count = (n: number, most: number) => Math.min(Math.max(most, 1), Math.max(1, Math.floor(n) || 1));
  const now = Date.now();
  const DAY = 24 * 3600 * 1000;

  // Dates are whole days, so an event runs until its last day is over.
  const endsAt = (p: ProjectView) => Date.parse(`${p.ends}T23:59:59Z`);
  const live = (p: ProjectView) => !!p.ends && endsAt(p) >= now;
  const daysLeft = (p: ProjectView) => Math.ceil((endsAt(p) - now) / DAY);
  const upcoming = (p: ProjectView) => !!p.starts && Date.parse(p.starts) > now;
  const closed = (p: ProjectView) => p.paused || (!p.permanent && !live(p)) || upcoming(p);

  const expeditions = $derived(projects.filter((p) => p.expedition));
  const permanent = $derived(projects.filter((p) => p.permanent));
  const events = $derived(projects.filter((p) => !p.expedition && !p.permanent && live(p)));
  const past = $derived(
    projects.filter((p) => !p.expedition && !p.permanent && !live(p))
      .sort((a, b) => (b.ends ?? "").localeCompare(a.ends ?? "")),
  );

  let loaded = $state(false);
  const load = async () => {
    projects = await projectList();
    loaded = true;
    selectedId ??= (projects.find((p) => !p.expedition && !p.permanent && live(p)) ?? projects.find((p) => p.permanent) ?? projects[0])?.id ?? null;
  };
  load();

  const project = $derived(projects.find((p) => p.id === selectedId) ?? null);
  const finished = (p: ProjectView) => p.stages.length > 0 && p.stage > p.stages.length;
  const stage = $derived(project?.stages.find((s) => s.n === project?.stage) ?? null);
  const input = $derived(stage?.inputs.find((i) => i.item.id === inputId) ?? stage?.inputs[0] ?? null);
  const left = $derived(input ? input.need - input.committed : 0);
  const stageFull = $derived(!!stage?.inputs.length && stage.inputs.every((i) => i.committed >= i.need));
  const hasProgress = $derived(!!project && (project.stage > 1 || !!stage?.inputs.some((i) => i.committed > 0) || !!stage?.task?.done.length));

  function select(id: string) {
    selectedId = id;
    inputId = null;
    message = "";
    last = null;
    confirmClear = false;
  }

  const commit = () => run(async () => {
    if (!project || !input) return;
    const before = project.stage;
    const taken = await commitResource(project.id, input.item.id, count(amount, left));
    last = taken ? { project: project.id, item: input.item.id, name: input.item.name, qty: taken, stage: before } : null;
    message = `Committed ${taken} ${input.item.name}.`;
    amount = 1;
    await load();
    if (project && project.stage > before) message += finished(project) ? " Project complete." : ` Stage ${before} done.`;
  });

  const undo = () => run(async () => {
    if (!last) return;
    const { project: id, item, name, qty, stage: was } = last;
    last = null;
    if (projects.find((p) => p.id === id)?.stage !== was) await reopenProjectStage(id);
    const back = await uncommitResource(id, item, qty);
    message = `Took back ${back} ${name}.`;
    await load();
  });

  const takeBack = () => run(async () => {
    if (!project || !input) return;
    const back = await uncommitResource(project.id, input.item.id, count(amount, input.committed));
    message = `Took back ${back} ${input.item.name}.`;
    last = null;
    amount = 1;
    await load();
  });

  const reopen = () => run(async () => {
    if (!project) return;
    const now = await reopenProjectStage(project.id);
    message = `Back on stage ${now}, as it was when it finished. Take back what you need to.`;
    last = null;
    await load();
  });

  // Recommitting nothing re-checks a reopened stage that still has everything in.
  const finishStage = () => run(async () => {
    if (!project || !stage?.inputs[0]) return;
    await commitResource(project.id, stage.inputs[0].item.id, 0);
    message = `Stage ${stage.n} done.`;
    last = null;
    await load();
  });

  const tick = (objectiveId: string, done: boolean) => run(async () => {
    if (!project || !stage) return;
    const n = stage.n;
    const now = await setProjectObjective(project.id, objectiveId, done);
    last = null;
    await load();
    message = now > n ? (finished(project) ? "Project complete." : `Stage ${n} done.`) : "";
  });

  async function track(on: boolean) {
    if (!project) return;
    if (!on && hasProgress && !confirmClear) {
      confirmClear = true;
      return;
    }
    confirmClear = false;
    last = null;
    await setProjectTracked(project.id, on);
    message = on ? "Tracked: its current stage feeds the map route." : "Tracking and progress cleared.";
    await load();
  }
</script>

{#snippet card(p: ProjectView)}
  <button class="card" class:active={p.id === selectedId} onclick={() => select(p.id)}>
    <strong>{p.name}</strong>
    {#if p.paused}
      <span class="status">Temporarily paused</span>
    {:else if upcoming(p)}
      <span class="timer">Starts {p.starts}</span>
    {:else if live(p)}
      <span class="timer">{daysLeft(p)} {daysLeft(p) === 1 ? "day" : "days"}</span>
    {:else if p.ends}
      <span class="status">Ended {p.ends}</span>
    {/if}
    <span class="pips">
      {#each p.stages as s (s.n)}
        <span class="pip" class:done={s.n < p.stage} class:now={s.n === p.stage}>{s.n < p.stage ? "✓" : s.n}</span>
      {/each}
    </span>
  </button>
{/snippet}

<div class="page">
  <aside>
    <h3 class="section-title">Expedition</h3>
    {#each expeditions as p (p.id)}{@render card(p)}{/each}
    <h3 class="section-title">Permanent</h3>
    {#each permanent as p (p.id)}{@render card(p)}{/each}
    <h3 class="section-title">Events</h3>
    {#each events as p (p.id)}{@render card(p)}{:else}{#if loaded}<p class="dim">No events running right now.</p>{/if}{/each}
    <label class="toggle"><input type="checkbox" bind:checked={showPast} /> Show past projects ({past.length})</label>
    {#if showPast}
      <h3 class="section-title">Past</h3>
      {#each past as p (p.id)}{@render card(p)}{/each}
    {/if}
  </aside>

  {#if project}
    <section class="detail">
      <h2>{project.name}</h2>
      {#if project.paused}
        <p class="dim">Temporarily paused in the game. Stages are shown for planning.</p>
      {:else if upcoming(project)}
        <p class="dim">This event hasn't started yet.</p>
      {:else if closed(project)}
        <p class="dim">No longer running{project.ends ? `: it ended ${project.ends}` : ""}. Stages are shown for reference.</p>
      {/if}
      {#if !project.stages.length}
        <p class="dim">No source publishes this project's stages yet.</p>
      {:else if finished(project)}
        <p class="dim">Project complete.</p>
        {#if project.tracked && !closed(project)}<button class="button small" disabled={busy} onclick={reopen}>Reopen the last stage</button>{/if}
      {:else if stage}
        <div class="stage-head">
          <h3 class="stage">Stage {stage.n}{stage.title && stage.title !== `Stage ${stage.n}` ? `: ${stage.title}` : ""}</h3>
          {#if project.stage > 1 && project.tracked && !closed(project)}
            <button class="link" disabled={busy} onclick={reopen} title="Go back to stage {stage.n - 1} to take something back">Back to stage {stage.n - 1}</button>
          {/if}
        </div>
        {#if stageFull && !closed(project)}
          <p class="full-note">Everything for this stage is in. <button class="button small primary" disabled={busy} onclick={finishStage}>Finish stage {stage.n}</button></p>
        {/if}
        {#if stage.task}
          <div class="task">
            {#if stage.task.map}<a class="map" href="#/maps/{stage.task.map.id}">{stage.task.map.name}</a>{/if}
            {#if stage.task.description}<p>{stage.task.description}</p>{/if}
            <QuestTree quest={stage.task} done={stage.task.done} ontick={closed(project) ? () => load() : tick} />
            {#if closed(project)}<p class="dim">Shown for reference.</p>{/if}
          </div>
        {/if}
        <div class="inputs">
          {#each stage.inputs as row (row.item.id)}
            {@const full = row.committed >= row.need}
            <button class="input" class:on={input?.item.id === row.item.id} onclick={() => { inputId = row.item.id; amount = 1; }}>
              <Icon url={row.item.icon} name={row.item.name} size={52} />
              <span class="body">
                <strong>{row.item.name}</strong>
                <span class="bar"><i style:width="{(row.committed / row.need) * 100}%"></i></span>
                <span class="counts" class:full>
                  <span>×{row.in_stash} in stash</span>
                  <span>{row.committed}/{row.need}</span>
                </span>
              </span>
            </button>
          {/each}
        </div>
        {#if input && !closed(project)}
          <div class="commit">
            <h3>Commit {input.item.name}</h3>
            <p>{input.in_stash} in your stash · <a href={linkHref(input.item)}>item page</a></p>
            <div class="stepper">
              <button aria-label="Less" onclick={() => (amount = Math.max(1, amount - 1))}>−</button>
              <input class="field" type="number" min="1" max={left} aria-label="Amount" bind:value={amount} />
              <button aria-label="More" onclick={() => (amount = Math.min(Math.max(left, input?.committed ?? 0, 1), amount + 1))}>+</button>
            </div>
            <div class="actions">
              <button class="button primary" disabled={left <= 0 || busy} onclick={commit}>Commit resource</button>
              {#if input.committed > 0}
                <button class="button dark" disabled={busy} onclick={takeBack}>Take back</button>
              {/if}
            </div>
            <p class="note">Takes the amount from your tracked stash. Take back returns it, and stash items go back into the stash.</p>
          </div>
        {/if}
        <h3 class="section-title">Rewards</h3>
        <div class="rewards">
          {#each stage.rewards as reward (reward.id)}
            <a href={linkHref(reward)} title={reward.name}><Icon url={reward.icon} name={reward.name} size={56} /></a>
          {:else}
            <p class="dim">None listed.</p>
          {/each}
        </div>
      {/if}
      <div class="foot">
        {#if confirmClear}
          <span class="warn">Stop tracking and clear all progress on this project?</span>
          <button class="button danger" onclick={() => track(false)}>Clear progress</button>
          <button class="button" onclick={() => (confirmClear = false)}>Keep it</button>
        {:else}
          <button class="button" onclick={() => track(!project?.tracked)}>
            {project.tracked ? "Stop tracking" : "Track resources"}
          </button>
          {#if message}<span class="message" role="status">{message}</span>{/if}
          {#if last}<button class="link" disabled={busy} onclick={undo}>Undo</button>{/if}
        {/if}
      </div>
    </section>
  {/if}
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: 340px minmax(0, 1fr);
    gap: 14px;
    padding: 12px 16px;
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
  .card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .card strong {
    font-family: var(--font-head);
    font-size: 19px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .card.active {
    border-color: var(--cream);
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
    color: var(--text-dim);
  }
  .status {
    color: var(--text-dim);
    font-size: 13px;
  }
  .timer {
    align-self: flex-start;
    padding: 1px 8px;
    border-radius: 3px;
    background: var(--yellow);
    color: var(--on-cream);
    font-size: 13px;
  }
  .detail {
    overflow: auto;
    min-height: 0;
    max-width: 720px;
  }
  h2 {
    font-size: 28px;
  }
  .stage {
    font-size: 20px;
    margin: 8px 0;
  }
  .task {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 12px;
    padding: 12px 14px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .task p {
    margin: 0;
  }
  .task .map {
    color: var(--text-dim);
    font-family: var(--font-head);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .input {
    display: grid;
    grid-template-columns: 64px 1fr;
    align-items: center;
    gap: 10px;
    padding: 6px 12px 6px 6px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .input.on {
    background: var(--cream);
    color: var(--on-cream);
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .body strong {
    text-transform: uppercase;
  }
  .counts {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }
  .counts.full {
    color: var(--teal);
  }
  .commit {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
    padding: 14px;
    background: var(--cream);
    color: var(--on-cream);
    border-radius: 6px;
  }
  .commit p {
    margin: 0;
    font-size: 13px;
  }
  .commit a {
    text-decoration: underline;
  }
  .stepper {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .stepper button {
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 50%;
    background: var(--panel);
    color: var(--text);
    font-size: 18px;
    cursor: pointer;
  }
  .stepper input {
    width: 90px;
    text-align: center;
  }
  .note {
    color: var(--on-cream-dim);
  }
  .section-title {
    margin: 14px 0 6px;
  }
  .rewards {
    display: flex;
    gap: 6px;
  }
  .rewards a {
    display: grid;
    place-items: center;
    width: 72px;
    height: 72px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 4px;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 14px 0;
  }
  .message {
    color: var(--teal);
  }
  .warn {
    color: var(--yellow);
  }
  .stage-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .full-note {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 0 8px;
    color: var(--teal);
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .card:hover:not(.active) {
    background: var(--panel-2);
  }
</style>
