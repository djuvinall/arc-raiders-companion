<script lang="ts">
  import {
    deleteWeaponBuild, getProfile, listItems, saveWeaponBuild, weaponCompare, weaponSheet, weaponSlots,
    type CompareTable, type ItemSummary, type Sheet, type SlotOptions, type WeaponBuild,
  } from "../lib/api";
  import Icon from "../lib/components/Icon.svelte";

  interface Column {
    build: WeaponBuild;
    slots: SlotOptions[];
    sheet: Sheet | null;
  }

  const MAX = 4;
  let weapons = $state<ItemSummary[]>([]);
  let columns = $state<Column[]>([]);
  let table = $state<CompareTable | null>(null);
  let saved = $state<WeaponBuild[]>([]);
  let targetHp = $state(100);

  listItems().then((all) => {
    weapons = all.filter((i) => i.category === "weapon").sort((a, b) => a.name.localeCompare(b.name));
    const start = weapons.find((w) => w.id === "rattler-iv") ?? weapons[0];
    if (start) addColumn({ name: start.name, item_id: start.id, mod_ids: [] });
  });
  getProfile().then((p) => (saved = p.weapon_builds));

  async function refresh(i: number) {
    const column = columns[i];
    column.sheet = await weaponSheet($state.snapshot(column.build), targetHp);
    if (columns.length > 1) {
      table = await weaponCompare($state.snapshot(columns).map((c) => c.build));
    } else {
      table = null;
    }
  }

  async function addColumn(build: WeaponBuild) {
    if (columns.length >= MAX) return;
    columns.push({ build, slots: await weaponSlots(build.item_id), sheet: null });
    await refresh(columns.length - 1);
  }

  function removeColumn(i: number) {
    columns.splice(i, 1);
    if (columns.length) refresh(0);
    else table = null;
  }

  async function pickWeapon(i: number, id: string) {
    const column = columns[i];
    column.build = { name: weapons.find((w) => w.id === id)?.name ?? id, item_id: id, mod_ids: [] };
    column.slots = await weaponSlots(id);
    await refresh(i);
  }

  function pickMod(i: number, slot: SlotOptions, id: string) {
    const column = columns[i];
    const others = slot.mods.map((m) => m.id);
    column.build.mod_ids = column.build.mod_ids.filter((m) => !others.includes(m));
    if (id) column.build.mod_ids.push(id);
    refresh(i);
  }

  async function save(i: number) {
    await saveWeaponBuild($state.snapshot(columns[i].build));
    saved = (await getProfile()).weapon_builds;
  }

  async function drop(name: string) {
    await deleteWeaponBuild(name);
    saved = (await getProfile()).weapon_builds;
  }

  const num = (v: number | null) => (v === null ? "—" : Number(v.toFixed(1)).toLocaleString());
  const delta = (base: number | null, value: number | null) =>
    base === null || value === null ? 0 : value - base;
  const words = (s: string) => s.replace(/[_-]/g, " ");
  const modText = (slot: SlotOptions, build: WeaponBuild) =>
    slot.mods.find((m) => build.mod_ids.includes(m.id))?.id ?? "";

  $effect(() => {
    targetHp;
    columns.forEach((_, i) => refresh(i));
  });
</script>

<div class="page-pad page">
  <div class="columns">
    {#each columns as column, i}
      <section class="panel build">
        <div class="head">
          {#if column.sheet}<Icon url={column.sheet.weapon.icon} name={column.sheet.weapon.name} size={48} />{/if}
          <input class="field name" aria-label="Build name" bind:value={column.build.name} />
          <button class="link dim" aria-label="Remove build" onclick={() => removeColumn(i)}>×</button>
        </div>
        <select
          class="field" aria-label="Weapon" value={column.build.item_id}
          onchange={(e) => pickWeapon(i, e.currentTarget.value)}
        >
          {#each weapons as weapon (weapon.id)}<option value={weapon.id}>{weapon.name}</option>{/each}
        </select>
        {#each column.slots as slot}
          <label>
            <span>{words(slot.slot)}</span>
            <select class="field" value={modText(slot, column.build)} onchange={(e) => pickMod(i, slot, e.currentTarget.value)}>
              <option value="">Empty</option>
              {#each slot.mods as m (m.id)}<option value={m.id}>{m.name}</option>{/each}
            </select>
          </label>
        {:else}
          <p class="dim small">No mod slots.</p>
        {/each}

        {#if column.sheet}
          {@const sheet = column.sheet}
          <p class="dim small">
            {[sheet.firing_mode, sheet.ammo_type && `${sheet.ammo_type} ammo`, sheet.arc_armor_penetration && `${sheet.arc_armor_penetration} ARC pen.`].filter(Boolean).join(" · ")}
          </p>
          <dl>
            {#each sheet.rows as row (row.key)}
              {@const d = delta(row.base, row.value)}
              <dt>{row.label}</dt>
              <dd>
                {num(row.value)}
                {#if Math.abs(d) > 1e-9}<span class:up={d > 0} class:down={d < 0}>{d > 0 ? "+" : ""}{num(d)}</span>{/if}
              </dd>
            {/each}
          </dl>
          {#if sheet.upgrades.length}
            <h3 class="section-title">Upgrade modifiers</h3>
            <dl>
              {#each sheet.upgrades as upgrade}<dt>{upgrade.label}</dt><dd>{upgrade.pct}%</dd>{/each}
            </dl>
          {/if}
          {#if sheet.effects.length}
            <h3 class="section-title">Mod effects</h3>
            <dl>
              {#each sheet.effects as effect}
                <dt>{words(effect.stat)}</dt>
                <dd>{effect.pct > 0 ? "+" : ""}{num(effect.pct)}%</dd>
              {/each}
            </dl>
          {/if}
          {#each sheet.warnings as warning}<p class="warning small">{warning}</p>{/each}

          <h3 class="section-title">Time to kill</h3>
          {#if sheet.ttk}
            <p class="small">{sheet.ttk.shots} shots, {sheet.ttk.seconds.toFixed(2)} s against {sheet.ttk.target_hp} HP</p>
          {:else}
            <p class="dim small">Data wanted: no measured fire rate for this weapon yet.</p>
          {/if}
          <h3 class="section-title">Falloff</h3>
          {#if sheet.measured.some((m) => m.key.startsWith("falloff"))}
            <dl>
              {#each sheet.measured.filter((m) => m.key.startsWith("falloff")) as row}
                <dt>{row.label}</dt><dd>{num(row.value)}</dd>
              {/each}
            </dl>
          {:else}
            <p class="dim small">Data wanted: no measured falloff yet.</p>
          {/if}
          <button class="button dark" onclick={() => save(i)}>Save build</button>
        {/if}
      </section>
    {/each}

    {#if columns.length < MAX}
      <button
        class="add panel"
        onclick={() => addColumn({ ...$state.snapshot(columns.at(-1)?.build ?? { name: "", item_id: weapons[0].id, mod_ids: [] }), mod_ids: [] })}
      >+ Add build to compare</button>
    {/if}
  </div>

  {#if table}
    <section class="panel">
      <h3 class="section-title">Compare</h3>
      <table>
        <thead>
          <tr><th></th>{#each table.columns as name}<th>{name}</th>{/each}</tr>
        </thead>
        <tbody>
          {#each table.rows as row}
            <tr>
              <td class="dim">{row.label}</td>
              {#each row.values as value, c}<td class:best={row.best === c}>{num(value)}</td>{/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {/if}

  <section class="panel foot">
    <p class="dim small">
      Stat DPS is damage × the fire rate stat, including level upgrades. It's derived from UI stats, so it compares
      weapons but isn't real damage per second. Percent mods multiply in sequence, which is an
      assumption: the game's stacking rule isn't published. To contribute measured numbers, edit a
      weapon's <code>measured</code> block in <code>data/items.json</code> and open a pull request.
    </p>
    <label class="hp">
      Target HP for time to kill
      <input class="field" type="number" min="1" bind:value={targetHp} />
    </label>
    {#if saved.length}
      <h3 class="section-title">Saved builds</h3>
      {#each saved as build (build.name)}
        <div class="saved">
          <button class="link" onclick={() => addColumn(structuredClone($state.snapshot(build)))}>{build.name}</button>
          <button class="link dim" aria-label="Delete {build.name}" onclick={() => drop(build.name)}>Delete</button>
        </div>
      {/each}
    {/if}
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
    align-items: start;
  }
  .build {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .name {
    flex: 1;
    min-width: 0;
  }
  select.field {
    width: 100%;
  }
  label {
    display: grid;
    grid-template-columns: 84px 1fr;
    align-items: center;
    gap: 8px;
    color: var(--text-dim);
    font-size: 13px;
    text-transform: capitalize;
  }
  dl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 10px;
    margin: 0;
  }
  dt {
    color: var(--text-dim);
    text-transform: capitalize;
  }
  dd {
    margin: 0;
    text-align: right;
  }
  .up {
    color: var(--rarity-uncommon);
    font-size: 12px;
  }
  .down {
    color: var(--rarity-epic);
    font-size: 12px;
  }
  .small {
    font-size: 13px;
    margin: 0;
  }
  .warning {
    color: var(--yellow);
  }
  .section-title {
    margin: 6px 0 0;
  }
  .add {
    min-height: 120px;
    color: var(--text-dim);
    cursor: pointer;
    font: inherit;
    border-style: dashed;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    padding: 4px 10px;
    text-align: right;
    border-bottom: 1px solid var(--border);
  }
  th:first-child,
  td:first-child {
    text-align: left;
  }
  .best {
    color: var(--yellow);
    font-weight: 700;
  }
  .foot {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .hp {
    display: flex;
    gap: 10px;
    text-transform: none;
  }
  .hp input {
    width: 90px;
  }
  .saved {
    display: flex;
    justify-content: space-between;
    max-width: 420px;
  }
  .link {
    background: none;
    border: 0;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    padding: 2px 0;
  }
  .link.dim {
    color: var(--text-dim);
  }
  @media (max-width: 1240px) {
    .columns {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
