<script lang="ts">
  import { arcList, linkHref, type ArcView } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  let { id }: { id?: string } = $props();
  let arcs = $state<ArcView[]>([]);
  // Routes remount on a new address, so the first value is the only one.
  // svelte-ignore state_referenced_locally
  let selectedId = $state<string | null>(id ?? null);

  arcList().then((all) => {
    arcs = [...all].sort((a, b) => (b.arc.hp ?? 0) - (a.arc.hp ?? 0));
    selectedId ??= arcs[0]?.arc.id ?? null;
  });

  const selected = $derived(arcs.find((a) => a.arc.id === selectedId) ?? null);
</script>

<div class="page">
  <aside>
    {#each arcs as { arc } (arc.id)}
      <button class="row" class:on={arc.id === selectedId} onclick={() => (selectedId = arc.id)}>
        <Icon url={arc.icon} name={arc.name} size={36} />
        <span class="name">{arc.name}</span>
        <span class="dim small">{arc.hp?.toLocaleString() ?? "—"}</span>
      </button>
    {/each}
  </aside>

  {#if selected}
    {@const arc = selected.arc}
    <section class="panel detail">
      <div class="head">
        <Icon url={arc.icon} name={arc.name} size={96} />
        <div>
          <h2>{arc.name}</h2>
          <p class="dim">{arc.hp ? `${arc.hp.toLocaleString()} health` : "Health not listed"}{arc.armor ? ` · ${arc.armor} armor` : ""}</p>
        </div>
      </div>

      <h3 class="section-title">Weak points</h3>
      {#each arc.weak_points as point}<p class="point">{point}</p>{:else}<p class="dim">Not listed.</p>{/each}

      {#if arc.attack || arc.ability}
        <h3 class="section-title">Attacks</h3>
        {#if arc.attack}<p>{arc.attack}</p>{/if}
        {#if arc.ability}<p>{arc.ability}</p>{/if}
      {/if}
      {#if arc.tips}
        <h3 class="section-title">Tips</h3>
        <p>{arc.tips}</p>
      {/if}
      {#if selected.drops.length}
        <h3 class="section-title">Drops</h3>
        <div class="drops">
          {#each selected.drops as drop (drop.id)}
            <a href={linkHref(drop)} title={drop.name}><Icon url={drop.icon} name={drop.name} size={40} /><span>{drop.name}</span></a>
          {/each}
        </div>
      {/if}
      <p class="dim small">From the ARC Raiders Wiki, CC BY-SA 4.0.</p>
    </section>
  {/if}
</div>

<style>
  .page {
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr);
    gap: 12px;
    padding: 12px 16px;
    height: 100%;
    min-height: 0;
  }
  aside,
  .detail {
    overflow: auto;
    min-height: 0;
  }
  .row {
    display: grid;
    grid-template-columns: 40px 1fr auto;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 4px 8px;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .row.on {
    background: var(--panel-2);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  h2 {
    font-size: 28px;
  }
  p {
    margin: 4px 0;
    max-width: 70ch;
  }
  .point {
    color: var(--yellow);
    font-size: 18px;
    font-weight: 600;
  }
  .section-title {
    margin: 14px 0 4px;
  }
  .small {
    font-size: 13px;
  }
  .drops {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .drops a {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px 3px 4px;
    background: var(--panel-2);
    border-radius: 4px;
  }
</style>
