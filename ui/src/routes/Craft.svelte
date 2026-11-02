<script lang="ts">
  import { craftPlan, getProfile, setStash, type CraftPlan } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";
  import TreeRow from "../lib/components/TreeRow.svelte";

  let { id }: { id: string } = $props();
  let qty = $state(1);
  let plan = $state<CraftPlan | null>(null);
  let stash = $state<Record<string, number>>({});
  let error = $state("");

  getProfile().then((p) => (stash = p.stash));

  $effect(() => {
    craftPlan(id, qty).then((p) => (plan = p), (e) => (error = String(e)));
  });

  function have(itemId: string, value: number) {
    const n = Math.max(0, Math.floor(value) || 0);
    stash[itemId] = n;
    setStash(itemId, n);
  }

  const remaining = $derived(
    plan?.totals.reduce((sum, t) => sum + Math.max(0, (t.qty ?? 0) - (stash[t.id] ?? 0)), 0) ?? 0,
  );
</script>

{#if error}
  <p class="error">{error}</p>
{:else if plan}
  <div class="page">
    <section class="panel tree">
      <div class="head">
        <h2><a href="#/item/{plan.root.id}">{plan.root.name}</a></h2>
        <label>
          Quantity
          <input class="field qty" type="number" min="1" max="999" bind:value={qty} />
        </label>
      </div>
      <TreeRow node={plan.tree} />
    </section>

    <section class="panel totals">
      <h3 class="section-title">Raw materials</h3>
      <div class="rows">
        {#each plan.totals as total (total.id)}
          {@const held = stash[total.id] ?? 0}
          <div class="row" class:done={held >= (total.qty ?? 0)}>
            <Icon url={total.icon} name={total.name} size={26} />
            <a href="#/item/{total.id}">{total.name}</a>
            <input
              class="field have"
              type="number"
              min="0"
              aria-label="Have {total.name}"
              value={held}
              onchange={(e) => have(total.id, e.currentTarget.valueAsNumber)}
            />
            <span class="need">/ {total.qty}</span>
          </div>
        {/each}
      </div>
      <p class="left">{remaining === 0 ? "Everything in stash" : `${remaining} still to gather`}</p>
    </section>
  </div>
{/if}

<style>
  .page {
    display: grid;
    grid-template-columns: 1fr 420px;
    gap: 18px;
    padding: 18px 20px;
    align-items: start;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  h2 {
    font-size: 26px;
  }
  label {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-dim);
  }
  .qty {
    width: 76px;
  }
  .totals {
    position: sticky;
    top: 18px;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: grid;
    grid-template-columns: 26px 1fr 72px 56px;
    align-items: center;
    gap: 10px;
  }
  .row.done a {
    color: var(--teal);
  }
  .have {
    color: var(--blue);
    padding: 3px 6px;
    text-align: right;
    width: 100%;
  }
  .need {
    color: var(--text-dim);
  }
  .left {
    color: var(--text-dim);
    margin: 12px 0 0;
  }
  .error {
    padding: 20px;
    color: var(--rarity-epic);
  }
</style>
