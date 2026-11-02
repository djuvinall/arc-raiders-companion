<script lang="ts">
  import { commitResource, linkHref, projectList, setProjectTracked, type ProjectView } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  let projects = $state<ProjectView[]>([]);
  let selectedId = $state<string | null>(null);
  let inputId = $state<string | null>(null);
  let amount = $state(1);
  let message = $state("");

  const load = async () => {
    projects = await projectList();
    selectedId ??= projects[0]?.id ?? null;
  };
  load();

  const project = $derived(projects.find((p) => p.id === selectedId) ?? null);
  const finished = (p: ProjectView) => p.stages.length > 0 && p.stage > p.stages.length;
  const stage = $derived(project?.stages.find((s) => s.n === project?.stage) ?? null);
  const input = $derived(stage?.inputs.find((i) => i.item.id === inputId) ?? stage?.inputs[0] ?? null);
  const left = $derived(input ? input.need - input.committed : 0);

  async function commit() {
    if (!project || !input) return;
    const taken = await commitResource(project.id, input.item.id, Math.min(amount, left));
    message = `Committed ${taken} ${input.item.name}.`;
    amount = 1;
    inputId = null;
    await load();
  }

  async function track(on: boolean) {
    if (!project) return;
    await setProjectTracked(project.id, on);
    message = on ? "Tracked: its current stage feeds the map route." : "Tracking and progress cleared.";
    await load();
  }
</script>

<div class="page">
  <aside>
    {#each projects as p (p.id)}
      <button class="card" class:active={p.id === selectedId} onclick={() => { selectedId = p.id; inputId = null; message = ""; }}>
        <strong>{p.name}</strong>
        {#if p.deadline_days}<span class="timer">{p.deadline_days} days</span>{/if}
        <span class="pips">
          {#each p.stages as s (s.n)}
            <span class="pip" class:done={s.n < p.stage} class:now={s.n === p.stage}>{s.n < p.stage ? "✓" : s.n}</span>
          {/each}
        </span>
      </button>
    {/each}
  </aside>

  {#if project}
    <section class="detail">
      <h2>{project.name}</h2>
      {#if !project.stages.length}
        <p class="dim">No source publishes this project's stages yet.</p>
      {:else if finished(project)}
        <p class="dim">Project complete.</p>
      {:else if stage}
        <h3 class="stage">Stage {stage.n}{stage.title && stage.title !== `Stage ${stage.n}` ? `: ${stage.title}` : ""}</h3>
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
        {#if input}
          <div class="commit">
            <h3>Commit {input.item.name}</h3>
            <p>{input.in_stash} in your stash · <a href={linkHref(input.item)}>item page</a></p>
            <div class="stepper">
              <button aria-label="Less" onclick={() => (amount = Math.max(1, amount - 1))}>−</button>
              <input class="field" type="number" min="1" max={left} aria-label="Amount" bind:value={amount} />
              <button aria-label="More" onclick={() => (amount = Math.min(Math.max(left, 1), amount + 1))}>+</button>
            </div>
            <button class="button primary" disabled={left <= 0} onclick={commit}>Commit resource</button>
            <p class="note">Takes the amount from your tracked stash and can't be undone except by clearing the project.</p>
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
        <button class="button" onclick={() => track(!project?.tracked)}>
          {project.tracked ? "Stop tracking" : "Track resources"}
        </button>
        {#if message}<span class="message" role="status">{message}</span>{/if}
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
</style>
