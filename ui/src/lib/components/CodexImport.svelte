<script lang="ts">
  import { setFound, type Collection } from "../api";
  import { scanCodex, type ScanResult } from "../codexScan";

  let { collection, onchange }: { collection: Collection; onchange: () => void } = $props();
  let rows = $state<(ScanResult & { keep: boolean; pick: string })[]>([]);
  let busy = $state(false);
  let message = $state("");
  let replace = $state(false);

  const LOW = 0.35;

  async function read(file: Blob | null | undefined) {
    if (!file) return;
    busy = true;
    message = "";
    rows = [];
    try {
      const results = await scanCodex(file, $state.snapshot(collection.entries));
      rows = results.map((r) => ({ ...r, keep: !!r.entry, pick: r.entry?.id ?? "" }));
      if (!rows.length) message = "No learned blueprints found. Use a full screenshot of the Blueprints tab in the game's Codex.";
    } catch (e) {
      message = String(e);
    }
    busy = false;
  }

  function onpaste(event: ClipboardEvent) {
    const file = [...(event.clipboardData?.items ?? [])].find((i) => i.type.startsWith("image/"))?.getAsFile();
    if (file) read(file);
  }

  const chosen = $derived(rows.filter((r) => r.keep && r.pick).map((r) => r.pick));
  const marked = $derived(collection.entries.filter((e) => e.found).map((e) => e.id));
  const adds = $derived(chosen.filter((id) => !marked.includes(id)));
  const removes = $derived(marked.filter((id) => !chosen.includes(id)));
  const unsure = $derived(rows.filter((r) => r.confidence < LOW).length);

  async function apply() {
    await setFound(adds, replace ? removes : []);
    message = `Marked ${adds.length} found${replace ? ` and cleared ${removes.length}` : ""}.`;
    rows = [];
    onchange();
  }
</script>

<svelte:window {onpaste} />

<section class="panel">
  <h3 class="section-title">Import from a screenshot</h3>
  <p class="dim small">
    Screenshot the Blueprints tab in the game's Codex, then press Ctrl+V here or pick the file.
    It's read on this PC and nothing is uploaded. You review the result before anything changes.
  </p>
  <label class="button dark">
    {busy ? "Reading…" : "Choose screenshot"}
    <input type="file" accept="image/*" disabled={busy} onchange={(e) => read(e.currentTarget.files?.[0])} />
  </label>
  {#if message}<p class="message small" role="status">{message}</p>{/if}

  {#if rows.length}
    <p class="small">
      Found {rows.length} learned blueprints: {adds.length} new, {removes.length} marked here but not in the screenshot.
      {#if unsure}<strong>{unsure} look-alikes need a check</strong>, outlined in yellow.{/if}
    </p>
    <div class="grid">
      {#each rows as row, i (i)}
        <div class="row" class:unsure={row.confidence < LOW}>
          <img src={row.crop} alt="" width="56" height="56" />
          <select class="field" aria-label="Blueprint {i + 1}" bind:value={row.pick}>
            <option value="">Skip this tile</option>
            {#each collection.entries as e (e.id)}<option value={e.id}>{e.name.replace(" Blueprint", "")}</option>{/each}
          </select>
        </div>
      {/each}
    </div>
    <label class="check"><input type="checkbox" bind:checked={replace} /> Also clear the {removes.length} not in the screenshot</label>
    <button class="button primary" disabled={!adds.length && !(replace && removes.length)} onclick={apply}>Apply</button>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: flex-start;
    margin-bottom: 12px;
  }
  label.button {
    cursor: pointer;
  }
  label.button input {
    display: none;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 4px;
    width: 100%;
  }
  .row {
    display: grid;
    grid-template-columns: 56px 1fr;
    align-items: center;
    gap: 6px;
    padding: 2px;
    border: 1px solid transparent;
    border-radius: 4px;
  }
  .row.unsure {
    border-color: var(--yellow);
  }
  select {
    min-width: 0;
    width: 100%;
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .small {
    font-size: 13px;
    margin: 0;
  }
  .message {
    color: var(--teal);
  }
</style>
