<script lang="ts">
  import { ROMAN, type Item, type WeaponFamily, type WeaponVariant } from "../api";

  let { family, item, baseId }: { family: WeaponFamily; item: Item; baseId: string | null } = $props();

  const current = $derived(family.amplified.find((v) => v.id === item.id) ?? null);
  const on = $derived(current?.amplifications ?? []);
  const options = $derived([...new Set(family.amplified.flatMap((v) => v.amplifications))]);
  const top = $derived(family.levels.at(-1)?.level ?? 1);

  // Replacing the hash keeps Back on the page the gun was opened from.
  const open = (id: string) => location.replace(`#/item/${id}`);

  // The data lists only some combinations, so a click lands on the closest one that exists.
  function toggle(amp: string) {
    const adding = !on.includes(amp);
    const want = adding ? [...on, amp] : on.filter((a) => a !== amp);
    const score = (v: WeaponVariant) =>
      v.amplifications.reduce((total, a) => total + (want.includes(a) ? 2 : -1), 0);
    const [best] = family.amplified
      .filter((v) => v.amplifications.includes(amp) === adding)
      .sort((a, b) => score(b) - score(a));
    if (best) open(best.id);
  }

  function amplify(wanted: boolean) {
    if (wanted && !current) {
      const plain = family.amplified.find((v) => !v.amplifications.length) ?? family.amplified[0];
      open(plain.id);
    } else if (!wanted && current && baseId) {
      open(baseId);
    }
  }
</script>

<div class="picker">
  {#if family.levels.length > 1}
    <div class="row" role="group" aria-label="Level">
      <span class="label">Level</span>
      {#each family.levels as level (level.id)}
        <button
          class="option" class:on={level.level === item.tier} aria-pressed={level.level === item.tier}
          onclick={() => level.level !== item.tier && open(level.id)}
        >{ROMAN[level.level]}</button>
      {/each}
    </div>
  {/if}
  {#if family.amplified.length}
    <div class="row" role="group" aria-label="Version">
      <span class="label">Version</span>
      <button class="option" class:on={!current} aria-pressed={!current} onclick={() => amplify(false)}>Standard</button>
      <button class="option" class:on={!!current} aria-pressed={!!current} onclick={() => amplify(true)}>Amplified</button>
    </div>
    {#if options.length}
      <div class="amps" role="group" aria-label="Amplifications">
        {#each options as amp (amp)}
          <button class="option" class:on={on.includes(amp)} aria-pressed={on.includes(amp)} onclick={() => toggle(amp)}>{amp}</button>
        {/each}
      </div>
    {/if}
    {#if current && family.levels.length > 1}
      <p class="note">Amplified versions are based on level {ROMAN[top]}.</p>
    {/if}
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 12px;
  }
  .row,
  .amps {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
  }
  .label {
    width: 58px;
    color: var(--on-cream-dim);
  }
  .option {
    padding: 3px 10px;
    background: none;
    border: 1px solid var(--on-cream-dim);
    border-radius: 4px;
    color: var(--on-cream);
    cursor: pointer;
    font: inherit;
  }
  .option.on {
    background: var(--on-cream);
    border-color: var(--on-cream);
    color: var(--cream);
  }
  .note {
    margin: 0;
    color: var(--on-cream-dim);
    font-size: 13px;
  }
</style>
