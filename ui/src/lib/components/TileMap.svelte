<script lang="ts">
  import { iconSrc, type GameMap, type Poi } from "../api";

  interface Props {
    map: GameMap;
    /** Tile template of the floor on show. */
    url: string | null;
    pois: Poi[];
    /** POI ids on the active route, drawn large and numbered. */
    route: string[];
    selected: Poi | null;
    onselect: (poi: Poi | null) => void;
  }
  let { map, url, pois, route, selected, onselect }: Props = $props();

  const COLORS: Record<string, string> = { cache: "#2ab0c6", extract: "#3fcf6b", landmark: "#9aa0a6", "quest-target": "#e26bff" };

  let box = $state<HTMLDivElement>();
  let canvas = $state<HTMLCanvasElement>();
  let width = $state(0);
  let height = $state(0);
  // Screen position = zoom-0 pixel × 2^zoom + offset.
  let zoom = $state(2);
  let ox = $state(0);
  let oy = $state(0);
  let drag: { x: number; y: number; moved: boolean } | null = null;
  let hover = $state<Poi | null>(null);
  let fitted = "";

  const project = (p: { x: number; y: number }): [number, number] => {
    const [la0, ln0, la1, ln1] = map.image.world_extent!;
    const [ty0, tx0, ty1, tx1] = map.image.tile_extent!;
    return [tx0 + ((p.x - ln0) / (ln1 - ln0)) * (tx1 - tx0), ty0 + ((p.y - la0) / (la1 - la0)) * (ty1 - ty0)];
  };
  const screen = (p: Poi): [number, number] => {
    const [x, y] = project(p);
    return [x * 2 ** zoom + ox, y * 2 ** zoom + oy];
  };

  function fit() {
    const points = pois.map(project);
    if (!points.length || !width) return;
    const xs = points.map((p) => p[0]);
    const ys = points.map((p) => p[1]);
    const [x0, x1, y0, y1] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
    zoom = Math.log2(Math.min(width / (x1 - x0), height / (y1 - y0)) * 0.92);
    ox = width / 2 - ((x0 + x1) / 2) * 2 ** zoom;
    oy = height / 2 - ((y0 + y1) / 2) * 2 ** zoom;
  }

  $effect(() => {
    if (pois.length && width && fitted !== map.id && pois[0].map_id === map.id) {
      fitted = map.id;
      fit();
    }
  });

  const tiles = $derived.by(() => {
    const { tile_size: size, max_native_zoom: maxZ } = map.image;
    if (!url || !size || !width) return [];
    const z = Math.max(1, Math.min(maxZ ?? 5, Math.round(zoom)));
    const span = size * 2 ** (zoom - z);
    const count = 2 ** z;
    const out = [];
    const [x0, y0] = [Math.floor(-ox / span), Math.floor(-oy / span)];
    const [x1, y1] = [Math.floor((width - ox) / span), Math.floor((height - oy) / span)];
    for (let y = Math.max(0, y0); y <= Math.min(count - 1, y1); y++) {
      for (let x = Math.max(0, x0); x <= Math.min(count - 1, x1); x++) {
        out.push({
          key: `${z}/${x}/${y}`,
          src: iconSrc(url.replace("{z}", `${z}`).replace("{x}", `${x}`).replace("{y}", `${y}`)),
          left: x * span + ox,
          top: y * span + oy,
          span,
        });
      }
    }
    return out;
  });

  $effect(() => {
    const ctx = canvas?.getContext("2d");
    if (!ctx || !canvas) return;
    const ratio = devicePixelRatio;
    canvas.width = width * ratio;
    canvas.height = height * ratio;
    ctx.scale(ratio, ratio);
    ctx.clearRect(0, 0, width, height);
    const onRoute = new Map(route.map((id, i) => [id, i + 1]));
    for (const poi of pois) {
      if (onRoute.has(poi.id)) continue;
      const [x, y] = screen(poi);
      if (x < -8 || y < -8 || x > width + 8 || y > height + 8) continue;
      ctx.beginPath();
      ctx.arc(x, y, poi === hover || poi === selected ? 6 : 3.5, 0, Math.PI * 2);
      ctx.fillStyle = COLORS[poi.kind] ?? "#fff";
      ctx.fill();
      ctx.lineWidth = 1;
      ctx.strokeStyle = "#0b0b0f";
      ctx.stroke();
    }
    ctx.font = "700 13px Barlow, sans-serif";
    ctx.textAlign = "center";
    for (const poi of pois) {
      const n = onRoute.get(poi.id);
      if (!n) continue;
      const [x, y] = screen(poi);
      ctx.beginPath();
      ctx.arc(x, y, 12, 0, Math.PI * 2);
      ctx.fillStyle = "#f5c400";
      ctx.fill();
      ctx.strokeStyle = "#0b0b0f";
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = "#15181f";
      ctx.fillText(String(n), x, y + 5);
    }
  });

  function nearest(event: MouseEvent): Poi | null {
    const rect = box!.getBoundingClientRect();
    const [mx, my] = [event.clientX - rect.left, event.clientY - rect.top];
    let best: Poi | null = null;
    let distance = 12;
    for (const poi of pois) {
      const [x, y] = screen(poi);
      const d = Math.hypot(x - mx, y - my);
      if (d < distance) [best, distance] = [poi, d];
    }
    return best;
  }

  function onwheel(event: WheelEvent) {
    event.preventDefault();
    const rect = box!.getBoundingClientRect();
    const [mx, my] = [event.clientX - rect.left, event.clientY - rect.top];
    const next = Math.max(-1, Math.min(7, zoom - event.deltaY * 0.002));
    const k = 2 ** (next - zoom);
    ox = mx - (mx - ox) * k;
    oy = my - (my - oy) * k;
    zoom = next;
  }

  function onmove(event: MouseEvent) {
    if (drag) {
      ox += event.clientX - drag.x;
      oy += event.clientY - drag.y;
      drag = { x: event.clientX, y: event.clientY, moved: true };
    } else {
      hover = nearest(event);
    }
  }

  function onup(event: MouseEvent) {
    if (drag && !drag.moved) onselect(nearest(event));
    drag = null;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="map" bind:this={box} bind:clientWidth={width} bind:clientHeight={height}
  {onwheel} onmousemove={onmove} onmouseup={onup} onmouseleave={() => (drag = null)}
  onmousedown={(e) => (drag = { x: e.clientX, y: e.clientY, moved: false })}
>
  {#each tiles as tile (tile.key)}
    <img
      src={tile.src} alt="" draggable="false"
      style:left="{tile.left}px" style:top="{tile.top}px"
      style:width="{tile.span + 0.5}px" style:height="{tile.span + 0.5}px"
      onerror={(e) => ((e.currentTarget as HTMLElement).style.display = "none")}
    />
  {/each}
  <canvas bind:this={canvas} style:width="{width}px" style:height="{height}px"></canvas>
  {#if hover}
    {@const [x, y] = screen(hover)}
    <span class="tip" style:left="{x + 10}px" style:top="{y - 12}px">{hover.name}</span>
  {/if}
  <button class="reset" onclick={fit} onmousedown={(e) => e.stopPropagation()}>Fit</button>
</div>

<style>
  .map {
    position: relative;
    overflow: hidden;
    height: 100%;
    min-height: 420px;
    background: #0d0f15;
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: grab;
    user-select: none;
  }
  img {
    position: absolute;
    pointer-events: none;
  }
  canvas {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .tip {
    position: absolute;
    padding: 2px 8px;
    background: var(--cream);
    color: var(--on-cream);
    border-radius: 4px;
    font-size: 13px;
    pointer-events: none;
    white-space: nowrap;
  }
  .reset {
    position: absolute;
    right: 10px;
    top: 10px;
    padding: 3px 12px;
    border: 0;
    border-radius: 999px;
    background: var(--cream);
    color: var(--on-cream);
    font-family: var(--font-head);
    font-weight: 700;
    cursor: pointer;
    text-transform: uppercase;
  }
</style>
