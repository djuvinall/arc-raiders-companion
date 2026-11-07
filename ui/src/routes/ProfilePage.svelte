<script lang="ts">
  import {
    exportProfile, getProfile, importProfile, linkHref, listItems, routePlan, setStash, setWorkbenchLevel,
    toggleTrackedBench, toggleTrackedItem, workbenchList,
    type BenchView, type ItemSummary, type Profile, type RoutePlan,
  } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  let profile = $state<Profile | null>(null);
  let plan = $state<RoutePlan | null>(null);
  let benches = $state<BenchView[]>([]);
  let items = $state<ItemSummary[]>([]);
  let text = $state("");
  let message = $state("");

  const load = async () => {
    [profile, plan, benches] = await Promise.all([getProfile(), routePlan(), workbenchList()]);
  };
  load();
  listItems().then((all) => (items = all));

  const summary = (id: string) => items.find((i) => i.id === id);

  async function untrack(id: string) {
    await toggleTrackedItem(id);
    await load();
  }

  async function have(id: string, value: number) {
    await setStash(id, Math.max(0, Math.floor(value) || 0));
    await load();
  }

  let benchNote = $state("");
  // Off while entering levels reached earlier, so setting up doesn't empty the stash.
  const SPEND_KEY = "bench-use-stash";
  let spend = $state((() => {
    try {
      return localStorage.getItem(SPEND_KEY) !== "off";
    } catch {
      return true;
    }
  })());
  function setSpend(on: boolean) {
    spend = on;
    try {
      localStorage.setItem(SPEND_KEY, on ? "on" : "off");
    } catch {
      // Remembering the choice is only a convenience.
    }
  }

  async function level(bench: BenchView, value: number) {
    const moved = await setWorkbenchLevel(bench.id, Math.min(bench.max_level, Math.max(0, value)), spend);
    const list = (sign: number) => moved.items.filter(([, q]) => Math.sign(q) === sign)
      .map(([id, q]) => `${Math.abs(q)}× ${summary(id)?.name ?? id}`).join(", ");
    const took = list(1);
    const back = list(-1);
    benchNote = took ? `Took ${took} from your stash for ${bench.name}. Lower it again to put them back.`
      : back ? `Put ${back} back in your stash.` : "";
    await load();
  }

  async function doExport() {
    text = await exportProfile();
    await navigator.clipboard?.writeText(text).catch(() => {});
    message = "Profile JSON is below and copied to the clipboard. Save it anywhere.";
  }

  // The profile as it was before the last import, so one click puts it back.
  let before = $state<string | null>(null);

  async function doImport() {
    try {
      const previous = await exportProfile();
      await importProfile(text);
      before = previous;
      await load();
      message = "Profile imported. It replaced what was here.";
    } catch (e) {
      message = String(e);
    }
  }

  async function undoImport() {
    if (!before) return;
    await importProfile(before);
    before = null;
    await load();
    message = "Import undone. Your previous profile is back.";
  }
</script>

<div class="page">
  <section class="panel">
    <h3 class="section-title">Tracked items</h3>
    {#each profile?.tracked_item_ids ?? [] as id (id)}
      <div class="row">
        <Icon url={summary(id)?.icon ?? null} name={summary(id)?.name ?? id} size={32} />
        <a href="#/item/{id}">{summary(id)?.name ?? id}</a>
        <button class="link" onclick={() => untrack(id)}>Remove</button>
      </div>
    {:else}
      <p class="dim">Use Track resources on an item page.</p>
    {/each}
  </section>

  <section class="panel">
    <h3 class="section-title">Still needed</h3>
    {#each plan?.needed ?? [] as need (need.item.id)}
      <div class="row">
        <Icon url={need.item.icon} name={need.item.name} size={32} />
        <a href={linkHref(need.item)}>{need.qty}× {need.item.name}</a>
        <span class="dim small">{need.reasons.join(", ")}</span>
        <input class="field have" type="number" min="0" aria-label="{need.item.name} in stash" title="In your stash" value={need.have} onchange={(e) => { const n = Math.max(0, Math.floor(e.currentTarget.valueAsNumber) || 0); e.currentTarget.value = String(n); have(need.item.id, n); }} />
      </div>
    {:else}
      <p class="dim">Nothing outstanding from tracked items, quests or projects.</p>
    {/each}
    {#if plan?.needed.length}<a class="button" href="#/maps">Rank maps for this</a>{/if}
  </section>

  <section class="panel wide">
    <h3 class="section-title">Workbenches</h3>
    <p class="dim small">Set your levels so items show what you can craft and what to save. Track one to add its next level to what's needed.</p>
    <label class="small spend">
      <input type="checkbox" checked={spend} onchange={(e) => setSpend(e.currentTarget.checked)} />
      Use materials from my stash when raising a level
    </label>
    <p class="dim small">
      {spend
        ? "On: raising a level takes its materials from your stash, as upgrading does in game. Lowering it puts back what it took."
        : "Off: raising only records the level, for benches you upgraded before using this app. Nothing leaves your stash."}
    </p>
    {#if benchNote}<p class="small" role="status">{benchNote}</p>{/if}
    <div class="benches">
      {#each benches as bench (bench.id)}
        <div class="bench">
          <strong>{bench.name}</strong>
          <div class="stepper">
            <button aria-label="Lower {bench.name}" disabled={bench.level <= 0} onclick={() => level(bench, bench.level - 1)}>−</button>
            <span>{bench.level}/{bench.max_level}</span>
            <button aria-label="Raise {bench.name}" disabled={bench.level >= bench.max_level} onclick={() => level(bench, bench.level + 1)}>+</button>
          </div>
          {#each bench.next as row (row.item.id)}
            <a class="small" class:met={row.in_stash >= row.need} href={linkHref(row.item)} title="{row.in_stash} in your stash">{row.in_stash >= row.need ? "✓ " : ""}{Math.min(row.in_stash, row.need)}/{row.need} {row.item.name}</a>
          {:else}
            <span class="dim small">{bench.level >= bench.max_level ? "Fully upgraded" : "No upgrade cost listed"}</span>
          {/each}
          {#if bench.requires}<span class="small requires">Requires {bench.requires}</span>{/if}
          {#if bench.next.length}
            <button class="button small dark track" class:on={bench.tracked} onclick={async () => { await toggleTrackedBench(bench.id); await load(); }}>{bench.tracked ? "Stop tracking" : "Track next level"}</button>
          {/if}
        </div>
      {/each}
    </div>
  </section>

  <section class="panel wide">
    <h3 class="section-title">Backup</h3>
    <p class="dim small">
      Your stash, skill plans, weapon builds and progress live in profile.json in this app's data
      folder. Export copies them as JSON; import replaces everything here with pasted JSON.
    </p>
    <textarea class="field" rows="6" aria-label="Profile JSON" bind:value={text}></textarea>
    <div class="actions">
      <button class="button" onclick={doExport}>Export</button>
      <button class="button dark" disabled={!text.trim()} onclick={doImport}>Import</button>
    </div>
    {#if message}
      <p class="message" role="status">{message} {#if before}<button class="button small dark" onclick={undoImport}>Undo import</button>{/if}</p>
    {/if}
  </section>
</div>

<style>
  .row:has(.have) {
    grid-template-columns: 32px 1fr auto 64px;
  }
  .requires {
    color: var(--yellow);
  }
  .have {
    width: 64px;
    padding: 2px 6px;
  }
  .spend {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-top: 6px;
  }
  .benches {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 10px;
  }
  .bench {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 8px 10px;
    background: var(--panel-2);
    border-radius: 4px;
  }
  .stepper {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .stepper button:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .stepper button {
    width: 24px;
    height: 24px;
    border: 1px solid var(--border);
    border-radius: 50%;
    background: none;
    color: var(--text);
    cursor: pointer;
  }
  .page {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    padding: 14px 20px;
    align-items: start;
  }
  .wide {
    grid-column: span 2;
  }
  .row {
    display: grid;
    grid-template-columns: 32px 1fr auto;
    align-items: center;
    gap: 10px;
    padding: 3px 0;
  }
  .small {
    font-size: 13px;
  }
  .met {
    color: var(--teal);
  }
  .track {
    align-self: flex-start;
    margin-top: 4px;
  }
  .track.on {
    border-color: var(--teal);
  }
  .link {
    background: none;
    border: 0;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
  }
  textarea {
    width: 100%;
    font-size: 12px;
    resize: vertical;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
  .message {
    color: var(--teal);
  }
</style>
