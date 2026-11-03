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
          {#if selected.threat}<span class="threat {selected.threat.toLowerCase()}">{selected.threat} threat</span>{/if}
        </div>
      </div>

      <div class="facts">
        <div>
          <h3 class="section-title">Weak points</h3>
          {#each arc.weak_points as point}<p class="point">{point}</p>{:else}<p class="dim">Not listed.</p>{/each}
        </div>
        {#if arc.attack || arc.ability}
          <div>
            <h3 class="section-title">Attacks</h3>
            {#if arc.attack}<p>{arc.attack}</p>{/if}
            {#if arc.ability}<p class="dim">{arc.ability}</p>{/if}
          </div>
        {/if}
      </div>

      {#if selected.strategy.length}
        <h3 class="section-title">Strategy</h3>
        <ul class="strategy">
          {#each selected.strategy as line}<li>{line}</li>{/each}
        </ul>
      {/if}
      {#if arc.tips}
        <h3 class="section-title">Field notes</h3>
        <p>{arc.tips}</p>
      {/if}
      {#if arc.armor === "Heavy" && selected.counters.length}
        <h3 class="section-title">Guns that punch through</h3>
        <div class="drops">
          {#each selected.counters as gun (gun.id)}
            <a href={linkHref(gun)} title={gun.note ?? gun.name}><Icon url={gun.icon} name={gun.name} size={40} /><span>{gun.name}<small class="dim"> · {gun.note?.replace(" ARC armor penetration", "")}</small></span></a>
          {/each}
        </div>
      {/if}
      {#if selected.spawns.length}
        <h3 class="section-title">Where it spawns</h3>
        <div class="spawns">
          {#each selected.spawns as spawn (spawn.map_id)}
            <a href="#/maps/{spawn.map_id}" class="spawn"><strong>{spawn.map_name}</strong><span class="dim small">{spawn.count} mapped</span></a>
          {/each}
        </div>
      {/if}
      {#if selected.drops.length}
        <h3 class="section-title">Drops</h3>
        <div class="drops">
          {#each selected.drops as drop (drop.id)}
            <a href={linkHref(drop)} title={drop.name}><Icon url={drop.icon} name={drop.name} size={40} /><span>{drop.name}</span></a>
          {/each}
        </div>
      {/if}
      <p class="dim small">Stats and field notes from the ARC Raiders Wiki (CC BY-SA 4.0); spawns from MetaForge's community map. The strategy is written from that data.</p>
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
  .threat {
    display: inline-block;
    margin-top: 4px;
    padding: 1px 10px;
    border-radius: 999px;
    background: var(--panel-2);
    font-family: var(--font-head);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .threat.boss,
  .threat.heavy {
    background: #8a2a2f;
    color: #fff;
  }
  .threat.medium {
    background: #8a5a1f;
    color: #fff;
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 6px 24px;
  }
  .strategy {
    margin: 4px 0;
    padding-left: 20px;
    max-width: 75ch;
  }
  .strategy li {
    margin: 3px 0;
  }
  .spawns {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .spawn {
    display: flex;
    flex-direction: column;
    padding: 4px 10px;
    background: var(--panel-2);
    border-radius: 4px;
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
  .row:hover {
    background: var(--panel-2);
  }
</style>
