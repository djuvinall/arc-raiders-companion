<script lang="ts">
  import { craftPlan, getItem, getProfile, ROMAN, setStash, setTrackedFrom, type CraftPlan } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";
  import TreeRow from "../lib/components/TreeRow.svelte";

  let { id }: { id: string } = $props();
  let qty = $state(1);
  let plan = $state<CraftPlan | null>(null);
  let stash = $state<Record<string, number>>({});
  let error = $state("");
  let owned = $state<string | null>(null);
  let lower = $state<{ id: string; level: number }[]>([]);

  // svelte-ignore state_referenced_locally
  getProfile().then((p) => {
    stash = p.stash;
    owned = p.tracked_from[id] ?? null;
  });
  // svelte-ignore state_referenced_locally
  getItem(id).then((d) => (lower = d.family?.levels.filter((l) => l.level < (d.item.tier ?? 0)) ?? []));

  $effect(() => {
    // An empty or odd quantity while typing plans for one, rather than failing the page.
    const n = Math.min(999, Math.max(1, Math.floor(qty) || 1));
    craftPlan(id, n, owned).then((p) => {
      plan = p;
      error = "";
    }, (e) => (error = String(e)));
  });

  function startFrom(value: string) {
    owned = value || null;
    setTrackedFrom(id, owned);
  }

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
        {#if lower.length}
          <select class="field" aria-label="Already own" value={owned ?? ""} onchange={(e) => startFrom(e.currentTarget.value)}>
            <option value="">Starting from nothing</option>
            {#each lower as l (l.id)}<option value={l.id}>I own level {ROMAN[l.level]}</option>{/each}
          </select>
        {/if}
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
              onchange={(e) => { const n = Math.max(0, Math.floor(e.currentTarget.valueAsNumber) || 0); e.currentTarget.value = String(n); have(total.id, n); }}
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
    color: var(--bad);
  }
</style>
