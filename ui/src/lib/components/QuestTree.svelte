<script lang="ts">
  import QuestTree from "./QuestTree.svelte";
  import { nodeDone, nodeStarted, type Objective } from "../api";

  /**
   * A quest's steps as the in-game log shows them: any-order steps boxed beside a
   * checkbox that fills in once they're all done, and either-or groups that only open
   * the option you've started. Ticking a group's box ticks everything in it.
   */
  let {
    quest, done, ontick, id = quest.objective_root, small = false,
  }: {
    quest: { objectives: Objective[]; objective_root: string };
    done: string[];
    ontick: (id: string, done: boolean) => void;
    id?: string;
    small?: boolean;
  } = $props();

  const node = $derived(quest.objectives.find((o) => o.id === id));
  const isDone = (nodeId: string) => nodeDone(quest, nodeId, done);
  const kind = (nodeId: string) => quest.objectives.find((o) => o.id === nodeId)?.kind ?? "atomic";
  const isStep = (nodeId: string) => ["atomic", "optional"].includes(kind(nodeId));
  /** The first step of an option, shown while that option isn't picked. */
  function head(nodeId: string): string {
    const n = quest.objectives.find((o) => o.id === nodeId);
    return n && !isStep(nodeId) && n.children.length ? head(n.children[0]) : nodeId;
  }
  const picked = $derived(node?.kind === "any" ? node.children.filter((c) => nodeStarted(quest, c, done)) : []);
</script>

{#snippet step(stepId: string)}
  {@const o = quest.objectives.find((x) => x.id === stepId)}
  {#if o}
    <label class="step" class:small class:finished={done.includes(o.id)}>
      <input type="checkbox" checked={done.includes(o.id)} onchange={(e) => ontick(o.id, e.currentTarget.checked)} />
      <span>{#if o.kind === "optional"}<span class="dim">(Optional)&nbsp;</span>{/if}{o.description}{#if o.amount && o.amount > 1}<span class="dim">&nbsp;· {o.amount}</span>{/if}</span>
    </label>
  {/if}
{/snippet}

{#if node}
  {#if isStep(id)}
    {@render step(id)}
  {:else if node.kind === "all"}
    <div class="group">
      <input
        type="checkbox" class="parent" checked={isDone(id)} title="Tick every step in this group"
        onchange={(e) => ontick(id, e.currentTarget.checked)}
      />
      <div class="box">
        {#if node.description}<p class="title" class:small>{node.description}</p>{/if}
        {#each node.children as child (child)}<QuestTree {quest} {done} {ontick} id={child} {small} />{/each}
      </div>
    </div>
  {:else if node.kind === "any"}
    <div class="group">
      <input type="checkbox" class="parent" checked={isDone(id)} disabled title="Done when any one option is" />
      <div class="box">
        <p class="title" class:small>{node.description ?? "Complete one of these"}</p>
        {#each node.children as option, i (option)}
          {#if i > 0}<hr />{/if}
          {#if picked.includes(option)}
            <QuestTree {quest} {done} {ontick} id={option} {small} />
          {:else}
            <div class:dimmed={picked.length > 0}>{@render step(head(option))}</div>
          {/if}
        {/each}
      </div>
    </div>
  {:else}
    {#each node.children as child (child)}<QuestTree {quest} {done} {ontick} id={child} {small} />{/each}
  {/if}
{/if}

<style>
  .step {
    display: flex;
    gap: 8px;
    padding: 3px 0;
    align-items: flex-start;
  }
  .step.small {
    gap: 6px;
    padding: 1px 0;
    font-size: 12px;
  }
  .step.finished > span {
    color: var(--text-dim);
  }
  .group {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin: 2px 0;
  }
  .parent {
    margin-top: 6px;
    flex: none;
  }
  .box {
    flex: 1;
    padding: 2px 8px;
    background: var(--panel-2);
    border-left: 2px solid var(--locked);
    border-radius: 0 4px 4px 0;
  }
  .title {
    margin: 2px 0;
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-dim);
  }
  hr {
    margin: 3px 0;
    border: 0;
    border-top: 1px solid var(--border);
  }
  .dimmed {
    opacity: 0.55;
  }
</style>
