<script lang="ts">
  import { tick } from "svelte";
  import { arcList, listItems, mapList, projectList, questList } from "../api";
  import Icon from "./Icon.svelte";

  interface Result {
    key: string;
    name: string;
    kind: string;
    icon: string | null;
    href: string;
  }

  const LIMIT = 12;
  let open = $state(false);
  let query = $state("");
  let index = $state(0);
  let all = $state<Result[]>([]);
  let box = $state<HTMLInputElement>();

  async function load() {
    if (all.length) return;
    const [items, quests, maps, arcs, projects] = await Promise.all([listItems(), questList(), mapList(), arcList(), projectList()]);
    const seen = new Set<string>();
    const guns = items.filter((i) => {
      if (!i.family) return !["cosmetic", "furniture"].includes(i.category);
      if (i.variant_of || seen.has(i.family)) return false;
      seen.add(i.family);
      return true;
    });
    all = [
      ...maps.map((m) => ({ key: `m${m.id}`, name: m.name, kind: "Map", icon: null, href: `#/maps/${m.id}` })),
      ...arcs.map(({ arc }) => ({ key: `a${arc.id}`, name: arc.name, kind: "ARC", icon: arc.icon, href: `#/arcs/${arc.id}` })),
      ...guns.map((i) => ({ key: `i${i.id}`, name: i.family_name ?? i.name, kind: i.category.replace("-", " "), icon: i.icon, href: `#/item/${i.id}` })),
      ...quests.map((q) => ({ key: `q${q.quest.id}`, name: q.quest.name, kind: "Quest", icon: null, href: `#/quests/${q.quest.id}` })),
      ...projects.map((p) => ({ key: `p${p.id}`, name: p.name, kind: "Project", icon: null, href: `#/projects/${p.id}` })),
    ];
  }

  // Names that start with the query beat names that merely contain it.
  const results = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    const hits = all.filter((r) => r.name.toLowerCase().includes(q));
    const rank = (r: Result) => (r.name.toLowerCase().startsWith(q) ? 0 : 1);
    return hits.sort((a, b) => rank(a) - rank(b)).slice(0, LIMIT);
  });

  async function show() {
    open = true;
    query = "";
    index = 0;
    await tick();
    box?.focus();
    load();
  }

  function go(result: Result | undefined) {
    if (!result) return;
    open = false;
    location.hash = result.href;
  }

  function onWindow(event: KeyboardEvent) {
    if (event.key.toLowerCase() === "k" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      show();
    }
  }

  function onBox(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") open = false;
    if (event.key === "ArrowDown") index = Math.min(results.length - 1, index + 1);
    if (event.key === "ArrowUp") index = Math.max(0, index - 1);
    if (event.key === "Enter") go(results[index]);
  }
</script>

<svelte:window onkeydown={onWindow} />

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="scrim" onclick={() => (open = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <input
        class="field" placeholder="Search items, quests, maps, ARCs and projects" aria-label="Search everything"
        bind:this={box} bind:value={query} oninput={() => (index = 0)} onkeydown={onBox}
      />
      {#each results as result, i (result.key)}
        <button class:on={i === index} onmouseenter={() => (index = i)} onclick={() => go(result)}>
          <Icon url={result.icon} name={result.name} size={28} />
          <span>{result.name}</span>
          <span class="kind">{result.kind}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    background: rgba(0, 0, 0, 0.55);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: min(560px, 90vw);
    padding: 10px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  input {
    width: 100%;
    box-sizing: border-box;
    font-size: 16px;
  }
  button {
    display: grid;
    grid-template-columns: 32px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    background: none;
    border: 0;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  button.on {
    background: var(--panel-2);
  }
  .kind {
    color: var(--text-dim);
    font-size: 12px;
    text-transform: capitalize;
  }
</style>
