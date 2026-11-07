<script lang="ts">
  import {
    iconSrc, decodeSkillPlan, deleteSkillPlan, encodeSkillPlan, getProfile, saveSkillPlan, setDefaultSkillPlan, skillPlanState,
    skillTree, type PlanState, type SkillGroup, type SkillNode, type SkillPlan, type SkillTree,
  } from "../lib/api";

  // Coordinates are the in-game screen at 2000 wide; this frames the tree.
  const VIEW = "70 45 1620 790";
  const LABELS: Record<string, [number, number]> = {
    conditioning: [651, 770],
    mobility: [872, 530],
    survival: [1138, 770],
  };

  let tree = $state<SkillTree | null>(null);
  let plan = $state<SkillPlan>({ name: "New plan", tree_version: "", expedition_bonus: 0, ranks: {} });
  let view = $state<PlanState | null>(null);
  let saved = $state<SkillPlan[]>([]);
  let selected = $state<SkillNode | null>(null);
  let group = $state<SkillGroup | null>(null);
  let brokenIcons = $state<Record<string, boolean>>({});
  let code = $state("");
  let message = $state("");

  let defaultName = $state<string | null>(null);

  // Opens on the default plan when there is one, else a blank tree.
  Promise.all([skillTree(), getProfile()]).then(([t, p]) => {
    tree = t;
    saved = p.skill_plans;
    defaultName = p.default_skill_plan;
    const start = saved.find((s) => s.name === defaultName);
    if (start) plan = structuredClone($state.snapshot(start));
    else plan.tree_version = t.tree_version;
  });

  // Plans set to save automatically are saved when you leave the page.
  $effect(() => () => {
    if (plan.auto_save && plan.tree_version && unsaved()) {
      const snapshot = $state.snapshot(plan);
      snapshot.expedition_bonus = Math.max(0, Math.floor(Number(snapshot.expedition_bonus)) || 0);
      saveSkillPlan(snapshot);
    }
  });

  async function makeDefault(name: string | null) {
    await setDefaultSkillPlan(name);
    defaultName = (await getProfile()).default_skill_plan;
    message = name ? `"${name}" opens whenever you come to Skills.` : "Skills opens on a blank tree.";
  }

  /** The flag is part of the saved plan, so turning it on or off saves the plan. */
  async function toggleAutoSave() {
    plan.auto_save = !plan.auto_save;
    if (saved.some((s) => s.name === plan.name)) await save();
  }

  // Only the newest answer counts, so a slow one can't overwrite a later plan.
  let asked = 0;
  $effect(() => {
    if (plan.tree_version) {
      const n = ++asked;
      const snapshot = $state.snapshot(plan);
      snapshot.expedition_bonus = Math.max(0, Math.floor(Number(snapshot.expedition_bonus)) || 0);
      skillPlanState(snapshot).then((s) => {
        if (n === asked) view = s;
      });
    }
  });
  let confirmDrop = $state<string | null>(null);

  const rank = (id: string) => plan.ranks[id] ?? 0;
  const color = (node: SkillNode) => tree?.branches.find((b) => b.id === node.branch)?.color ?? "#fff";
  const branchColor = (id: string) => tree?.branches.find((b) => b.id === id)?.color ?? "#fff";
  const options = (g: SkillGroup) => tree?.nodes.filter((n) => n.choice_group === g.id) ?? [];
  const picks = (g: SkillGroup) => options(g).filter((n) => rank(n.id) > 0);
  const lit = (id: string) => {
    const g = tree?.choice_groups.find((c) => c.id === id);
    return g ? picks(g).length > 0 : rank(id) > 0;
  };
  const wireColor = (w: { from: string; to: string }) => {
    const node = tree?.nodes.find((n) => n.id === w.from);
    const g = tree?.choice_groups.find((c) => c.id === w.from);
    return lit(w.from) ? branchColor(node?.branch ?? g?.branch ?? "") : "#3a3d47";
  };
  const glyph = (name: string) =>
    name.replace(/['’]/g, "").split(/[\s-]+/).slice(0, 2).map((w) => w[0]).join("").toUpperCase();
  const slotOpen = (g: SkillGroup, i: number) =>
    i === picks(g).length && options(g).some((n) => view?.addable.includes(n.id));

  function pickSlot(g: SkillGroup, i: number) {
    group = g;
    selected = picks(g)[i] ?? null;
    message = "";
  }

  function add(node: SkillNode) {
    selected = node;
    if (!node.choice_group) group = null;
    message = "";
    // The view can lag a fast double click, so the rank cap is checked here too.
    if (view?.addable.includes(node.id) && rank(node.id) < limit(node)) plan.ranks[node.id] = rank(node.id) + 1;
    else message = `${node.name} can't take a point right now.`;
  }

  function remove(node: SkillNode, event?: Event) {
    event?.preventDefault();
    selected = node;
    if (!node.choice_group) group = null;
    message = "";
    if (!view?.removable.includes(node.id)) {
      if (rank(node.id) > 0) message = `Other skills depend on ${node.name}.`;
      return;
    }
    if (rank(node.id) === 1) delete plan.ranks[node.id];
    else plan.ranks[node.id] = rank(node.id) - 1;
  }

  // A plan with points that matches no saved plan would be lost, so New asks twice.
  let confirmNew = $state(false);
  const unsaved = () => (Object.keys(plan.ranks).length > 0 || saved.some((p) => p.name === plan.name)) &&
    !saved.some((p) => JSON.stringify({ ...p, auto_save: !!p.auto_save }) === JSON.stringify({ ...$state.snapshot(plan), auto_save: !!plan.auto_save }));

  function fresh() {
    if (!confirmNew && unsaved()) {
      confirmNew = true;
      message = "This plan isn't saved. Click New again to start over anyway.";
      return;
    }
    confirmNew = false;
    plan = { name: "New plan", tree_version: tree!.tree_version, expedition_bonus: 0, ranks: {} };
    group = null;
    selected = null;
    code = "";
    message = "";
  }

  async function save() {
    const snapshot = $state.snapshot(plan);
    snapshot.expedition_bonus = Math.max(0, Math.floor(Number(snapshot.expedition_bonus)) || 0);
    await saveSkillPlan(snapshot);
    saved = (await getProfile()).skill_plans;
    message = `Saved "${plan.name}".`;
  }

  function open(p: SkillPlan) {
    plan = structuredClone($state.snapshot(p));
    message = "";
  }

  async function drop(name: string) {
    await deleteSkillPlan(name);
    saved = (await getProfile()).skill_plans;
  }

  async function share() {
    code = await encodeSkillPlan($state.snapshot(plan));
    await navigator.clipboard?.writeText(code).catch(() => {});
    message = "Plan code ready.";
  }

  async function load() {
    try {
      plan = await decodeSkillPlan(code);
      message = `Loaded "${plan.name}".`;
    } catch (e) {
      message = String(e);
    }
  }

  const limit = (node: SkillNode) => node.max_rank ?? 5;
  const pointList = (points: [number, number][]) => points.map((p) => p.join(",")).join(" ");
  const diamond = (x: number, y: number, r: number) =>
    `${x},${y - r} ${x + r},${y} ${x},${y + r} ${x - r},${y}`;
</script>

{#snippet art(node: SkillNode, x: number, y: number, size: number, on: boolean)}
  {#if node.icon && !brokenIcons[node.id]}
    <image
      class="art" class:on href={iconSrc(node.icon)} x={x - size / 2} y={y - size / 2}
      width={size} height={size} onerror={() => (brokenIcons[node.id] = true)}
    />
  {:else}
    <text class="glyph" class:on {x} y={y + 6} text-anchor="middle" fill={color(node)}>{glyph(node.name)}</text>
  {/if}
{/snippet}

{#if tree && view}
  <div class="page">
    <div class="board">
      {#if !tree.prereqs_entered || !tree.layout_in_game}
        <p class="banner" role="status">
          Provisional tree: skills, ranks, point gates, choice limits and the budget are enforced.
          Links between skills aren't entered yet, so skill order isn't checked, and the layout
          isn't the in-game one.
        </p>
      {/if}
      <svg viewBox={VIEW} role="group" aria-label="Skill tree">
        {#each tree.wires as wire}
          <polyline
            points={pointList(wire.points)} fill="none" stroke-linejoin="round"
            stroke={wireColor(wire)} stroke-width={lit(wire.from) ? 8 : 5}
          />
        {/each}
        {#each tree.branches as branch}
          {@const [x, y] = LABELS[branch.id] ?? [0, 0]}
          <text class="branch" {x} {y} text-anchor="middle" fill={branch.color}>{branch.points_label}</text>
          <text class="branch-total" {x} y={y + 36} text-anchor="middle" fill={branch.color}>
            {view.report.branch_points[branch.id] ?? 0}
          </text>
        {/each}
        {#each tree.nodes.filter((n) => !n.choice_group) as node (node.id)}
          {@const taken = rank(node.id) > 0}
          {@const open = view.addable.includes(node.id)}
          {@const full = rank(node.id) >= limit(node)}
          {@const ring = taken || open ? color(node) : "#6b7080"}
          {@const radius = node.kind === "keystone" ? 40 : 25}
          <g
            class="node"
            class:locked={!taken && !open}
            role="button"
            tabindex="0"
            aria-label="{node.name}, {rank(node.id)} of {limit(node)}"
            onclick={() => add(node)}
            oncontextmenu={(e) => remove(node, e)}
            onmouseenter={() => { selected = node; group = null; }}
            onfocus={() => { selected = node; group = null; }}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === "+") add(node);
              if (e.key === "Backspace" || e.key === "-") remove(node);
            }}
          >
            <circle
              cx={node.x} cy={node.y} r={radius}
              fill={full ? ring : "#14161f"} stroke={ring} stroke-width="3"
            />
            {@render art(node, node.x, node.y, radius * 1.15, full)}
            {#if !full && (taken || open)}
              <rect x={node.x - 22} y={node.y + 14} width="44" height="22" rx="10" fill="#14161f" stroke={ring} stroke-width="2" />
              <text class="rank" x={node.x} y={node.y + 31} text-anchor="middle" fill={ring}>
                {rank(node.id)}/{node.max_rank ?? "?"}
              </text>
            {/if}
          </g>
        {/each}
        {#each tree.choice_groups as g (g.id)}
          {#each g.slots as [x, y], i}
            {@const pick = picks(g)[i]}
            {@const open = slotOpen(g, i)}
            {@const ring = pick || open ? branchColor(g.branch) : "#6b7080"}
            <g
              class="node"
              class:locked={!pick && !open}
              role="button"
              tabindex="0"
              aria-label={pick ? `${pick.name}, chosen` : "Choice slot, empty"}
              onclick={() => pickSlot(g, i)}
              oncontextmenu={(e) => pick ? remove(pick, e) : e.preventDefault()}
              onfocus={() => pickSlot(g, i)}
              onmouseenter={() => pickSlot(g, i)}
              onkeydown={(e) => {
                if (e.key === "Enter") pickSlot(g, i);
                if (pick && (e.key === "Backspace" || e.key === "-")) remove(pick);
              }}
            >
              <polygon
                points={diamond(x, y, 42)} stroke-linejoin="round"
                fill={pick ? ring : "#1b1f2e"} stroke={ring} stroke-width="3"
              />
              {#if pick}
                {@render art(pick, x, y, 54, true)}
              {:else}
                <text class="glyph slot" {x} y={y + 8} text-anchor="middle" fill={ring}>?</text>
              {/if}
              {#if !pick && !open}
                <circle cx={x - 28} cy={y + 28} r="13" fill="#6b7080" />
                <rect x={x - 34} y={y + 26} width="12" height="9" fill="#14161f" />
                <path d="M{x - 31} {y + 26} v-3 a3 3 0 0 1 6 0 v3" fill="none" stroke="#14161f" stroke-width="2" />
              {/if}
            </g>
          {/each}
        {/each}
      </svg>
    </div>

    <aside>
      <section class="panel">
        <input class="field name" aria-label="Plan name" bind:value={plan.name} />
        <p class="points" class:over={!view.report.ok}>
          {view.report.spent} / {view.report.budget} points
        </p>
        <label>
          Expedition bonus
          <input
            class="field bonus" type="number" min="0" max={tree.points.expedition_bonus_max}
            bind:value={plan.expedition_bonus}
          />
        </label>
        <div class="actions">
          <button class="button primary" onclick={save}>Save</button>
          <button class="button" onclick={share}>Share</button>
          <button class="button" class:danger={confirmNew} onclick={fresh}>{confirmNew ? "Discard and start new" : "New"}</button>
        </div>
        <label class="check">
          <input type="checkbox" checked={!!plan.auto_save} onchange={toggleAutoSave} />
          Save changes when I leave this page
        </label>
        {#if message}<p class="message" role="status">{message}</p>{/if}
        {#each view.report.errors as error}<p class="error">{error}</p>{/each}
        {#each view.report.warnings as warning}<p class="warning">{warning}</p>{/each}
      </section>

      <section class="panel detail">
        {#if group}
          <h3>{group.id.endsWith("mid") ? "Large node" : "End node"}</h3>
          <p class="meta">
            {group.branch} · {group.max_picks === 1 ? "pick 1" : `pick up to ${group.max_picks}`} ·
            needs {group.gates.join(" / ")} points
          </p>
          {#each options(group) as option (option.id)}
            {@const taken = rank(option.id) > 0}
            <button
              class="option" class:taken
              disabled={!taken && !view.addable.includes(option.id)}
              onclick={() => (taken ? remove(option) : add(option))}
            >
              <strong>{option.name}</strong>
              <span>{option.description}</span>
              {#if option.details}<span class="meta">{option.details}</span>{/if}
            </button>
          {/each}
        {:else if selected}
          <h3>{selected.name}</h3>
          <p class="meta">
            {selected.branch} · {rank(selected.id)}/{selected.max_rank ?? "? (unconfirmed)"}
          </p>
          <p>{selected.description}</p>
          {#if selected.details}<p class="meta">{selected.details}</p>{/if}
        {:else}
          <p class="meta">Click a skill to add a point, right-click to remove one.</p>
        {/if}
      </section>

      <section class="panel">
        <h3 class="section-title">Plan code</h3>
        <textarea class="field code" rows="3" aria-label="Plan code" bind:value={code}></textarea>
        <button class="button" onclick={load}>Load code</button>
      </section>

      {#if saved.length}
        <section class="panel">
          <h3 class="section-title">Saved plans</h3>
          <p class="meta">★ opens whenever you come to Skills.</p>
          {#each saved as item (item.name)}
            {@const isDefault = defaultName === item.name}
            <div class="saved">
              <button
                class="link star" class:on={isDefault} aria-pressed={isDefault}
                title={isDefault ? "Open a blank tree instead" : "Open this plan whenever you come to Skills"}
                onclick={() => makeDefault(isDefault ? null : item.name)}
              >{isDefault ? "★" : "☆"}</button>
              <button class="link" onclick={() => open(item)}>{item.name}{item.auto_save ? " · auto-saves" : ""}</button>
              <button class="link dim" class:confirm={confirmDrop === item.name} aria-label="Delete {item.name}" onclick={() => (confirmDrop === item.name ? (confirmDrop = null, drop(item.name)) : (confirmDrop = item.name))}>{confirmDrop === item.name ? "Click again to delete" : "Delete"}</button>
            </div>
          {/each}
        </section>
      {/if}
    </aside>
  </div>
{/if}

<style>
  .check {
    display: flex;
    justify-content: flex-start;
    gap: 6px;
    align-items: center;
    margin-top: 8px;
    font-size: 13px;
  }
  .star {
    flex: none;
    color: var(--text-dim);
  }
  .star.on {
    color: var(--yellow);
  }
  .page {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 280px;
    gap: 12px;
    padding: 12px;
    height: 100%;
    min-height: 0;
    align-items: start;
  }
  .board {
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    min-height: 0;
  }
  svg {
    flex: 1;
    width: 100%;
    min-height: 0;
    display: block;
  }
  .banner {
    margin: 0 0 10px;
    padding: 8px 12px;
    border: 1px solid var(--yellow);
    border-radius: 6px;
    color: var(--yellow);
  }
  .branch {
    font-family: var(--font-head);
    font-size: 24px;
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .node {
    cursor: pointer;
    outline: none;
  }
  .node.locked {
    cursor: not-allowed;
  }
  .node:focus-visible circle,
  .node:focus-visible polygon {
    stroke: #fff;
  }
  .branch-total {
    font-family: var(--font-head);
    font-size: 34px;
    font-weight: 700;
  }
  .glyph {
    font-family: var(--font-head);
    font-size: 17px;
    font-weight: 700;
    pointer-events: none;
  }
  .glyph.slot {
    font-size: 24px;
  }
  .art {
    pointer-events: none;
  }
  .art.on {
    filter: brightness(0);
  }
  .node.locked .art {
    opacity: 0.4;
  }
  .glyph.on {
    fill: #101218 !important;
  }
  .rank {
    font-size: 15px;
    pointer-events: none;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 3px;
    width: 100%;
    margin-top: 8px;
    padding: 8px 10px;
    background: none;
    border: 1px solid var(--panel-border, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .option.taken {
    border-color: var(--yellow);
  }
  .option:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .name {
    width: 100%;
  }
  .points {
    font-family: var(--font-head);
    font-size: 26px;
    font-weight: 700;
    margin: 8px 0;
  }
  .points.over {
    color: var(--bad);
  }
  label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--text-dim);
  }
  .bonus {
    width: 70px;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 10px;
  }
  .actions .button {
    padding: 6px 12px;
  }
  .message,
  .warning,
  .error {
    margin: 8px 0 0;
    font-size: 13px;
  }
  .message {
    color: var(--teal);
  }
  .warning {
    color: var(--yellow);
  }
  .error {
    color: var(--bad);
  }
  .detail {
    min-height: 130px;
  }
  .detail h3 {
    font-size: 20px;
  }
  .detail p {
    margin: 6px 0 0;
  }
  .meta {
    color: var(--text-dim);
    font-size: 13px;
    text-transform: none;
  }
  .code {
    width: 100%;
    resize: vertical;
    font-size: 12px;
    margin-bottom: 8px;
  }
  .saved {
    display: flex;
    gap: 6px;
    align-items: baseline;
  }
  .saved .link:last-child {
    margin-left: auto;
  }
  .link {
    background: none;
    border: 0;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    padding: 3px 0;
  }
  .link.dim {
    color: var(--text-dim);
  }
  .link.confirm {
    color: var(--bad);
  }
</style>
