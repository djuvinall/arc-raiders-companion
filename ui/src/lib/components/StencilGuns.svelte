<script lang="ts">
  import { deleteStencilShot, linkHref, saveStencilShot, stencilShots, type Link } from "../api";
  import StencilPreview from "./StencilPreview.svelte";

  let { stencilId, swatch, guns }: { stencilId: string; swatch: string; guns: Link[] } = $props();
  let chosenId = $state<string | null>(null);
  let shots = $state<Record<string, string>>({});
  let message = $state("");
  const chosen = $derived(guns.find((g) => g.id === chosenId) ?? guns[0]);
  const load = async () => (shots = await stencilShots(stencilId));
  load();

  const dataUrl = (file: File) => new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });

  async function addShot(event: Event & { currentTarget: HTMLInputElement }) {
    const file = event.currentTarget.files?.[0];
    event.currentTarget.value = "";
    if (!file) return;
    try {
      await saveStencilShot(stencilId, chosen.id, await dataUrl(file));
      message = "";
      await load();
    } catch (e) {
      message = String(e);
    }
  }

  async function removeShot() {
    await deleteStencilShot(stencilId, chosen.id);
    await load();
  }
</script>

{#snippet picture(gun: Link, size: number)}
  {#if shots[gun.id]}
    <img class="shot" src={shots[gun.id]} alt="{gun.name} with this stencil" style:max-height="{size}px" style:max-width="100%" />
  {:else}
    <StencilPreview gun={gun.icon} {swatch} name={gun.name} {size} />
  {/if}
{/snippet}

<section class="panel">
  <h3 class="section-title">Fits {guns.length} {guns.length === 1 ? "weapon" : "weapons"}</h3>
  <div class="stage">
    {#key chosen.id}
      {@render picture(chosen, 340)}
    {/key}
    <a href={linkHref(chosen)}>{chosen.name}</a>
    <div class="actions">
      <label class="button dark">
        {shots[chosen.id] ? "Replace screenshot" : "Add your screenshot"}
        <input type="file" accept="image/png,image/jpeg,image/webp" onchange={addShot} />
      </label>
      {#if shots[chosen.id]}<button class="button dark" onclick={removeShot}>Remove</button>{/if}
    </div>
    {#if message}<p class="note">{message}</p>{/if}
  </div>
  <p class="note">
    {#if shots[chosen.id]}
      Your screenshot, stored only on this computer.
    {:else}
      Approximate: this lays the stencil's pattern over the gun's icon. The game places it differently
      on each gun. Add a screenshot to see the real thing.
    {/if}
    Weapon list, source and cost are community-reported, so confirm them in game.
  </p>
  {#if guns.length > 1}
    <div class="grid">
      {#each guns as gun (gun.id)}
        <button class="gun" class:on={gun.id === chosen.id} aria-pressed={gun.id === chosen.id} title={gun.name} onclick={() => (chosenId = gun.id)}>
          {@render picture(gun, 72)}
          <span>{gun.name}</span>
        </button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .stage a {
    color: var(--teal);
    font-family: var(--font-head);
    font-size: 18px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .shot {
    border-radius: 4px;
    object-fit: contain;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }
  .actions input {
    display: none;
  }
  .actions label {
    cursor: pointer;
  }
  .note {
    margin: 8px 0;
    color: var(--text-dim);
    font-size: 13px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, 96px);
    gap: 8px;
  }
  .gun {
    display: grid;
    justify-items: center;
    gap: 2px;
    padding: 6px 2px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    font-size: 12px;
  }
  .gun.on {
    border-color: var(--cream);
  }
</style>
