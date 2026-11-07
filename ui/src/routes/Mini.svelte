<script lang="ts">
  import {
    itemVerdict, listItems, questList, questProgress, routePlan, setQuestObjective,
    type ItemSummary, type QuestRow, type RoutePlan, type Verdict,
  } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";
  import QuestTree from "../lib/components/QuestTree.svelte";

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

  // Folded quests show only their name and progress; remembered per machine.
  const FOLDED_KEY = "mini-folded";
  let folded = $state(new Set<string>(readFolded()));

  function readFolded(): string[] {
    try {
      return JSON.parse(localStorage.getItem(FOLDED_KEY) ?? "[]");
    } catch {
      return [];
    }
  }

  function fold(id: string) {
    const next = new Set(folded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    folded = next;
    try {
      localStorage.setItem(FOLDED_KEY, JSON.stringify([...next]));
    } catch {
      // Remembering is only a convenience.
    }
  }

  const foldAll = (shut: boolean) => {
    folded = new Set(shut ? active.map((q) => q.quest.id) : []);
    try {
      localStorage.setItem(FOLDED_KEY, JSON.stringify([...folded]));
    } catch {
      // Remembering is only a convenience.
    }
  };

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

  <div class="quests-head">
    <h3 class="section-title">Quests ({active.length})</h3>
    {#if active.length}
      {@const allFolded = active.every((q) => folded.has(q.quest.id))}
      <button class="link small" onclick={() => foldAll(!allFolded)}>{allFolded ? "Expand all" : "Collapse all"}</button>
    {/if}
  </div>
  {#each active as q (q.quest.id)}
    {@const progress = questProgress(q.quest, q.done_objectives)}
    {@const open = !folded.has(q.quest.id)}
    <button class="quest" aria-expanded={open} onclick={() => fold(q.quest.id)}>
      <span class="arrow" class:open aria-hidden="true">▸</span>
      <span class="name">{q.quest.name}</span>
      <span class="count">{progress.done}/{progress.total}</span>
    </button>
    {#if open}
      <div class="steps">
        <QuestTree quest={q.quest} done={q.done_objectives} ontick={(id, done) => tick(q.quest.id, id, done)} small />
      </div>
    {/if}
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
  .quests-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .link {
    background: none;
    border: 0;
    padding: 0;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  .link:hover {
    color: var(--text);
  }
  .quest {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 4px 0 0;
    padding: 2px 0;
    background: none;
    border: 0;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    font-weight: 600;
    text-align: left;
    text-transform: capitalize;
  }
  .quest .name {
    flex: 1;
  }
  .quest .count {
    font-weight: 400;
    font-size: 12px;
    color: var(--text-dim);
  }
  .arrow {
    display: inline-block;
    width: 10px;
    transition: transform 0.12s;
  }
  .arrow.open {
    transform: rotate(90deg);
  }
  .steps {
    padding-left: 16px;
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
