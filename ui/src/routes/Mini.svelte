<script lang="ts">
  import {
    itemVerdict, listItems, questList, routePlan, setQuestObjective,
    type ItemSummary, type QuestRow, type RoutePlan, type Verdict,
  } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  const REFRESH_MS = 10000;
  let items = $state<ItemSummary[]>([]);
  let plan = $state<RoutePlan | null>(null);
  let quests = $state<QuestRow[]>([]);
  let search = $state("");
  let pick = $state<ItemSummary | null>(null);
  let verdict = $state<Verdict | null>(null);

  const load = async () => {
    [plan, quests] = await Promise.all([routePlan(), questList()]);
  };
  load();
  listItems().then((all) => (items = all.filter((i) => !["cosmetic", "furniture"].includes(i.category) && !i.variant_of)));

  // The main window changes the same profile, so keep up with it.
  $effect(() => {
    const tick = setInterval(load, REFRESH_MS);
    return () => clearInterval(tick);
  });

  const matches = $derived(search.trim().length < 2 ? [] :
    items.filter((i) => i.name.toLowerCase().includes(search.trim().toLowerCase())).slice(0, 6));
  const active = $derived(quests.filter((q) => q.status === "active"));

  async function choose(item: ItemSummary) {
    pick = item;
    search = "";
    verdict = null;
    const v = await itemVerdict(item.id);
    if (pick?.id === item.id) verdict = v;
  }

  async function tick(quest: string, objective: string, done: boolean) {
    await setQuestObjective(quest, objective, done);
    await load();
  }
</script>

<div class="mini">
  <input class="field" placeholder="Should I keep this?" aria-label="Find an item" bind:value={search} />
  {#each matches as item (item.id)}
    <button class="match" onclick={() => choose(item)}><Icon url={item.icon} name={item.name} size={24} />{item.name}</button>
  {/each}
  {#if pick}
    <div class="verdict {verdict?.action ?? ''}">
      <strong>{pick.name}</strong>
      <span>{verdict?.headline ?? "No advice for this item."}</span>
      {#each verdict?.reasons.slice(0, 3) ?? [] as reason}<span class="small">{reason}</span>{/each}
    </div>
  {/if}

  <h3 class="section-title">Quests ({active.length})</h3>
  {#each active as q (q.quest.id)}
    <p class="quest">{q.quest.name}</p>
    {#each q.quest.objectives.filter((o) => o.kind === "atomic") as o (o.id)}
      <label class="small">
        <input type="checkbox" checked={q.done_objectives.includes(o.id)} onchange={(e) => tick(q.quest.id, o.id, e.currentTarget.checked)} />
        <span>{o.description}</span>
      </label>
    {/each}
  {:else}
    <p class="dim small">No quests tracked.</p>
  {/each}

  <h3 class="section-title">Still needed ({plan?.needed.length ?? 0})</h3>
  {#each plan?.needed ?? [] as need (need.item.id)}
    <div class="need"><Icon url={need.item.icon} name={need.item.name} size={24} /><span>{need.qty}× {need.item.name}</span></div>
  {:else}
    <p class="dim small">Nothing tracked.</p>
  {/each}
</div>

<style>
  .mini {
    display: flex;
    flex-direction: column;
    gap: 3px;
    height: 100vh;
    padding: 8px 10px;
    overflow: auto;
    box-sizing: border-box;
  }
  .match {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    background: var(--panel);
    border: 0;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .verdict {
    display: flex;
    flex-direction: column;
    padding: 6px 8px;
    border-radius: 4px;
    background: var(--panel);
  }
  .verdict.keep {
    background: #2a7a4b;
  }
  .verdict.recycle {
    background: #1d6f86;
  }
  .quest {
    margin: 4px 0 0;
    font-weight: 600;
    text-transform: capitalize;
  }
  label {
    display: flex;
    gap: 6px;
    align-items: flex-start;
  }
  .need {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .small {
    font-size: 12px;
    margin: 0;
  }
  .section-title {
    margin: 8px 0 2px;
  }
</style>
