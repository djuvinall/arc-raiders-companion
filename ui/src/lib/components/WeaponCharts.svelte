<script lang="ts">
  import type { Sheet } from "../api";

  /** Builds side by side: time to kill through each shield, then one chart per stat. */
  let { builds, hp }: { builds: { name: string; sheet: Sheet }[]; hp: number } = $props();

  // Fixed order, one hue per shield, checked for colour-blind separation on the panel.
  const SHIELD_COLORS: Record<string, string> = {
    none: "#3987e5",
    "light-shield": "#d95926",
    "medium-shield": "#199e70",
    "heavy-shield": "#c98500",
  };
  const NEUTRAL = "#8b93a7";

  let aim = $state<"body" | "head">("body");
  let unit = $state<"shots" | "seconds">("shots");

  const shields = $derived(builds[0]?.sheet.vs_shields ?? []);
  const color = (id: string) => SHIELD_COLORS[id] ?? NEUTRAL;
  const anySeconds = $derived(builds.some((b) => b.sheet.rpm));

  /** The bar's value, or null when that build can't say (no RPM, no headshot data). */
  function value(sheet: Sheet, shield: string): number | null {
    const row = sheet.vs_shields.find((s) => s.shield === shield);
    const kill = aim === "head" ? row?.head : row?.body;
    if (!kill) return null;
    return unit === "seconds" ? kill.seconds : kill.shots;
  }

  const ttkMax = $derived(Math.max(1e-9, ...builds.flatMap((b) => shields.map((s) => value(b.sheet, s.shield) ?? 0))));
  const fmt = (v: number) => (unit === "seconds" ? `${v.toFixed(2)} s` : `${v}`);

  // Grouped horizontal bars: a row per build, a bar per shield.
  const BAR = 14;
  const GAP = 2;
  const ROW_GAP = 14;
  const LABEL_W = 132;
  const PLOT_W = 420;
  const groupH = $derived(shields.length * (BAR + GAP) - GAP);
  const ttkH = $derived(builds.length * (groupH + ROW_GAP));

  interface Stat {
    key: string;
    title: string;
    note: string;
    get: (s: Sheet) => number | null;
    show: (v: number) => string;
  }
  const STATS: Stat[] = [
    { key: "damage", title: "Damage per shot", note: "Shotguns count every pellet.", get: (s) => s.damage, show: (v) => `${v}` },
    { key: "rpm", title: "Fire rate", note: "Rounds per minute, from the wiki. Only some guns publish it.", get: (s) => s.rpm, show: (v) => `${Math.round(v)} RPM` },
    { key: "head", title: "Headshot multiplier", note: "From the wiki.", get: (s) => s.headshot_mult, show: (v) => `${v}×` },
    {
      key: "dps", title: "Damage per second", note: "Damage × RPM ÷ 60, every shot landing.",
      get: (s) => (s.damage && s.rpm ? (s.damage * s.rpm) / 60 : null), show: (v) => v.toFixed(1),
    },
  ];
  const statMax = (stat: Stat) => Math.max(1e-9, ...builds.map((b) => stat.get(b.sheet) ?? 0));
</script>

<section class="panel charts">
  <div class="chart-head">
    <h3 class="section-title">Time to kill through each shield</h3>
    <div class="toggles" role="group" aria-label="Chart options">
      <button class="chip" class:on={aim === "body"} aria-pressed={aim === "body"} onclick={() => (aim = "body")}>Body</button>
      <button class="chip" class:on={aim === "head"} aria-pressed={aim === "head"} onclick={() => (aim = "head")}>Headshots</button>
      <span class="sep"></span>
      <button class="chip" class:on={unit === "shots"} aria-pressed={unit === "shots"} onclick={() => (unit = "shots")}>Shots</button>
      <button
        class="chip" class:on={unit === "seconds"} aria-pressed={unit === "seconds"} disabled={!anySeconds}
        title={anySeconds ? "" : "None of these guns has a published fire rate in RPM"}
        onclick={() => (unit = "seconds")}
      >Seconds</button>
    </div>
  </div>
  <div class="legend" aria-label="Shields">
    {#each shields as s (s.shield)}
      <span class="key"><span class="swatch" style:background={color(s.shield)}></span>{s.name}{s.charge ? ` (${s.charge} charge, ${s.reduction_pct}%)` : ""}</span>
    {/each}
  </div>

  <svg class="plot" viewBox="0 0 {LABEL_W + PLOT_W + 70} {ttkH}" role="img" aria-label="Time to kill by build and shield">
    {#each builds as b, row (b.name)}
      {@const top = row * (groupH + ROW_GAP)}
      <text class="row-label" x="0" y={top + groupH / 2} dominant-baseline="middle">{b.name}</text>
      {#each shields as s, i (s.shield)}
        {@const v = value(b.sheet, s.shield)}
        {@const y = top + i * (BAR + GAP)}
        {#if v !== null}
          {@const w = Math.max(4, (v / ttkMax) * PLOT_W)}
          <g class="bar">
            <title>{b.name} vs {s.name}: {fmt(v)}{aim === "head" ? " (every shot a headshot)" : ""}</title>
            <rect x={LABEL_W} {y} width={PLOT_W + 70} height={BAR} fill="transparent" />
            <path
              d="M{LABEL_W} {y} h{w - 4} a4 4 0 0 1 4 4 v{BAR - 8} a4 4 0 0 1 -4 4 h{-(w - 4)} z"
              fill={color(s.shield)}
            />
            <text class="value" x={LABEL_W + w + 6} y={y + BAR / 2} dominant-baseline="middle">{fmt(v)}</text>
          </g>
        {:else}
          <text class="value dim" x={LABEL_W} y={y + BAR / 2} dominant-baseline="middle">
            {unit === "seconds" && !b.sheet.rpm ? "no RPM published" : aim === "head" ? "no headshot data" : "—"}
          </text>
        {/if}
      {/each}
    {/each}
  </svg>
  <p class="dim small">
    Against {hp} health. A shield takes its percentage of each hit until its charge runs out, then
    health takes all of it: the community's reading of the shield text, not a published rule. Seconds count one shot
    interval per shot and assume every shot lands.
  </p>
</section>

<div class="stats">
  {#each STATS as stat (stat.key)}
    {@const max = statMax(stat)}
    <section class="panel stat">
      <h3 class="section-title">{stat.title}</h3>
      <div class="hbars" role="img" aria-label={stat.title}>
        {#each builds as b (b.name)}
          {@const v = stat.get(b.sheet)}
          <span class="name">{b.name}</span>
          {#if v !== null && v !== undefined}
            <span class="track" title="{b.name}: {stat.show(v)}"><span class="fill" style:width="{Math.max(2, (v / max) * 100)}%"></span></span>
            <span class="num">{stat.show(v)}</span>
          {:else}
            <span class="dim small none">not published</span>
          {/if}
        {/each}
      </div>
      <p class="dim small">{stat.note}</p>
    </section>
  {/each}
  <section class="panel stat">
    <h3 class="section-title">Damage falloff by range</h3>
    <p class="dim small">
      No source publishes falloff distances yet, so there's nothing to draw. Mods that change falloff list it under
      Mod effects. Measured numbers can go in a weapon's <code>measured</code> block in the arc-raiders-data repository.
    </p>
  </section>
</div>

<style>
  .charts {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .chart-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }
  .toggles {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .sep {
    width: 8px;
  }
  .chip {
    padding: 3px 10px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: none;
    color: var(--text-dim);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .chip.on {
    background: var(--cream);
    color: var(--on-cream);
  }
  .chip:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 14px;
    font-size: 12px;
    color: var(--text-dim);
  }
  .key {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }
  svg {
    width: 100%;
    height: auto;
    overflow: visible;
  }
  .plot {
    max-width: 760px;
  }
  .row-label {
    font-size: 12px;
    fill: var(--text);
  }
  .value {
    font-size: 11px;
    fill: var(--text-dim);
  }
  .bar:hover path {
    filter: brightness(1.15);
  }
  .hbars {
    display: grid;
    grid-template-columns: minmax(0, 7em) 1fr auto;
    align-items: center;
    gap: 6px 8px;
    font-size: 13px;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .track {
    height: 10px;
    border-radius: 0 4px 4px 0;
  }
  .fill {
    display: block;
    height: 100%;
    background: #8b93a7;
    border-radius: 0 4px 4px 0;
  }
  .num {
    color: var(--text-dim);
    font-size: 12px;
    text-align: right;
  }
  .none {
    grid-column: span 2;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 12px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .small {
    font-size: 12px;
    margin: 0;
  }
  .section-title {
    margin: 0;
  }
</style>
