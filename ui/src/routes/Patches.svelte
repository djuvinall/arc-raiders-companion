<script lang="ts">
  import { datasetChangelog, patchNotes, type ChangelogEntry, type Overview, type PatchNote } from "../lib/api";

  let { info }: { info: Overview | null } = $props();
  let notes = $state<PatchNote[]>([]);
  let fetched = $state<number | null>(null);
  let loaded = $state(false);
  let changelog = $state<ChangelogEntry[]>([]);
  let selectedId = $state<string | null>(null);

  patchNotes().then((n) => {
    notes = n.notes;
    fetched = n.fetched_ms;
    loaded = true;
    selectedId = n.notes[0]?.gid ?? null;
  });
  datasetChangelog().then((c) => (changelog = c));

  const note = $derived(notes.find((n) => n.gid === selectedId) ?? null);
  const day = (seconds: number) => new Date(seconds * 1000).toLocaleDateString();
</script>

<div class="page">
  <aside>
    <section class="panel">
      <h3 class="section-title">Dataset</h3>
      <p class="reflects">
        {info?.game_patch ? `Reflects patch ${info.game_patch}` : "Patch unknown"}
      </p>
      <p class="dim small">Version {info?.dataset_version}</p>
    </section>
    <section class="panel list">
      <h3 class="section-title">Patch notes</h3>
      {#each notes as n (n.gid)}
        <button class:active={n.gid === selectedId} onclick={() => (selectedId = n.gid)}>
          <strong>{n.title}</strong>
          <span class="dim small">{day(n.date)}</span>
        </button>
      {/each}
      <p class="dim small">
        {#if !loaded}Loading.
        {:else if fetched === null}Offline copy from the app's build.
        {:else}Updated {new Date(fetched).toLocaleString()}.{/if}
      </p>
    </section>
  </aside>

  <div class="content">
    {#if note}
      <article class="panel">
        <h2>{note.title}</h2>
        <p class="dim small">{day(note.date)}</p>
        {#each note.blocks as block}
          {#if block.kind === "heading"}<h3>{block.text}</h3>
          {:else if block.kind === "item"}<p class="item">{block.text}</p>
          {:else}<p>{block.text}</p>{/if}
        {/each}
      </article>
    {/if}
    <section class="panel">
      <h3 class="section-title">Dataset changelog</h3>
      {#each changelog as entry (entry.dataset_version)}
        <h4>{entry.dataset_version}{entry.game_patch ? ` · patch ${entry.game_patch}` : ""}</h4>
        {#each entry.summary as line}<p class="small">{line}</p>{/each}
        {#each entry.files as file}
          <details>
            <summary>{file.file}</summary>
            {#if file.added.length}<p class="small">Added: {file.added.join(", ")}</p>{/if}
            {#if file.removed.length}<p class="small">Removed: {file.removed.join(", ")}</p>{/if}
            {#if file.changed.length}<p class="small">Changed: {file.changed.join(", ")}</p>{/if}
          </details>
        {/each}
      {/each}
    </section>
  </div>
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
  aside {
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow: auto;
    min-height: 0;
  }
  .content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: auto;
    min-height: 0;
  }
  .reflects {
    font-family: var(--font-head);
    font-size: 22px;
    font-weight: 700;
    color: var(--yellow);
    margin: 0;
  }
  .list button {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 7px 8px;
    background: none;
    border: 0;
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .list button.active {
    background: var(--panel-2);
  }
  .small {
    font-size: 13px;
    margin: 3px 0;
  }
  article {
    max-width: 860px;
    user-select: text;
  }
  article h2 {
    font-size: 28px;
  }
  article h3 {
    font-size: 19px;
    margin: 16px 0 4px;
  }
  article p {
    margin: 6px 0;
    line-height: 1.45;
  }
  .item {
    padding-left: 16px;
    position: relative;
  }
  .item::before {
    content: "•";
    position: absolute;
    left: 3px;
    color: var(--yellow);
  }
  h4 {
    margin: 8px 0 2px;
  }
</style>
