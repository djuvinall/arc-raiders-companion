<script lang="ts">
  import { getEntity, type EntityPage } from "../lib/api";
  import LinkRow from "../lib/components/LinkRow.svelte";

  let { kind, id }: { kind: string; id: string } = $props();
  let page = $state<EntityPage | null>(null);
  // Entity kinds as dataset files, for the Report window.
  const FILES: Record<string, string> = {
    workbench: "workbenches", "loot-area": "loot_areas", map: "maps", trader: "traders",
    arc: "arcs", quest: "quests", project: "projects",
  };
  let error = $state("");

  $effect(() => {
    getEntity(kind, id).then((p) => (page = p), (e) => (error = String(e)));
  });
</script>

{#if error}
  <p class="error">{error}</p>
{:else if page}
  <div class="page" data-report={FILES[kind] ? `${FILES[kind]}/${id}` : undefined} data-report-label={page.name}>
    <h2>{page.name}</h2>
    <p class="subtitle">{page.subtitle}</p>
    <div class="sections">
      {#each page.sections as section}
        <section class="panel" data-report-label={section.title}>
          <h3 class="section-title">{section.title}</h3>
          {#each section.text as line}<p class="text">{line}</p>{/each}
          {#each section.links as link}<LinkRow {link} />{/each}
        </section>
      {/each}
    </div>
  </div>
{/if}

<style>
  .page {
    padding: 18px 20px;
    max-width: 1100px;
  }
  h2 {
    font-size: 30px;
  }
  .subtitle {
    color: var(--text-dim);
    margin: 2px 0 14px;
    text-transform: capitalize;
  }
  .sections {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: 12px;
    align-items: start;
  }
  .text {
    margin: 0 0 6px;
  }
  .error {
    padding: 20px;
    color: var(--bad);
  }
</style>
