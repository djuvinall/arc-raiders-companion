<script lang="ts" module>
  export interface MapPoint {
    key: string;
    x: number;
    y: number;
    /** Marker type, such as `containers/locker`. */
    icon: string;
    group: string;
    title: string;
    label?: string | null;
    /** Route stop number, drawn large on top. */
    number?: number;
  }
</script>

<script lang="ts">
  import { iconSrc, type GameMap } from "../api";
  import { GROUPS, sprite } from "../mapIcons";

  interface Props {
    map: GameMap;
    /** Tile template of the floor on show. */
    url: string | null;
    points: MapPoint[];
    selected: MapPoint | null;
    onselect: (point: MapPoint | null) => void;
  }
  let { map, url, points, selected, onselect }: Props = $props();

  // Many markers in view draw as small badges, and only a flood of them as dots.
  const SMALL_LIMIT = 1500;
  const DOT_LIMIT = 6000;
  const LABEL_ZOOM = 1.6;
  // Past the sharpest tiles the picture only blurs.
  const OVERZOOM = 1.5;

  let box = $state<HTMLDivElement>();
  let canvas = $state<HTMLCanvasElement>();
  let width = $state(0);
  let height = $state(0);
  // Screen position = zoom-0 pixel × 2^zoom + offset.
  let zoom = $state(2);
  let ox = $state(0);
  let oy = $state(0);
  let fitZoom = 2;
  let drag: { x: number; y: number; moved: boolean } | null = null;
  let hover = $state<MapPoint | null>(null);
  let fitted = "";
  let frame = 0;
  // Screen positions from the last draw, for hit testing without reprojecting.
  let drawn: { point: MapPoint; x: number; y: number; r: number }[] = [];

  const project = (p: { x: number; y: number }): [number, number] => {
    const [la0, ln0, la1, ln1] = map.image.world_extent!;
    const [ty0, tx0, ty1, tx1] = map.image.tile_extent!;
    return [tx0 + ((p.x - ln0) / (ln1 - ln0)) * (tx1 - tx0), ty0 + ((p.y - la0) / (la1 - la0)) * (ty1 - ty0)];
  };
  // Projected once per point list, not on every pan.
  const projected = $derived(points.map((p) => project(p)));

  function fit() {
    if (!projected.length || !width) return;
    let [x0, x1, y0, y1] = [Infinity, -Infinity, Infinity, -Infinity];
    for (const [x, y] of projected) {
      x0 = Math.min(x0, x);
      x1 = Math.max(x1, x);
      y0 = Math.min(y0, y);
      y1 = Math.max(y1, y);
    }
    zoom = fitZoom = Math.log2(Math.min(width / (x1 - x0 || 1), height / (y1 - y0 || 1)) * 0.92);
    ox = width / 2 - ((x0 + x1) / 2) * 2 ** zoom;
    oy = height / 2 - ((y0 + y1) / 2) * 2 ** zoom;
  }

  $effect(() => {
    if (projected.length && width && fitted !== map.id) {
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
          key: `${url}|${z}/${x}/${y}`,
          src: iconSrc(url.replace("{z}", `${z}`).replace("{x}", `${x}`).replace("{y}", `${y}`)),
          left: x * span + ox,
          top: y * span + oy,
          span,
        });
      }
    }
    return out;
  });

  function draw() {
    frame = 0;
    const ctx = canvas?.getContext("2d");
    if (!ctx || !canvas) return;
    const ratio = devicePixelRatio || 1;
    if (canvas.width !== Math.round(width * ratio) || canvas.height !== Math.round(height * ratio)) {
      canvas.width = Math.round(width * ratio);
      canvas.height = Math.round(height * ratio);
    }
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    ctx.clearRect(0, 0, width, height);
    const scale = 2 ** zoom;
    const plain: { point: MapPoint; x: number; y: number }[] = [];
    const stops: { point: MapPoint; x: number; y: number }[] = [];
    for (let i = 0; i < points.length; i++) {
      const [px, py] = projected[i];
      const x = px * scale + ox;
      const y = py * scale + oy;
      if (x < -20 || y < -20 || x > width + 20 || y > height + 20) continue;
      (points[i].number === undefined ? plain : stops).push({ point: points[i], x, y });
    }
    const dots = plain.length > DOT_LIMIT;
    const size = plain.length > SMALL_LIMIT ? 12 : zoom - fitZoom < 0.8 ? 16 : zoom - fitZoom < 2 ? 20 : 26;
    drawn = [];
    for (const v of plain) {
      const focus = v.point === hover || v.point === selected;
      if (dots && !focus) {
        ctx.fillStyle = GROUPS[v.point.group]?.color ?? "#888";
        ctx.fillRect(v.x - 2, v.y - 2, 4, 4);
        drawn.push({ ...v, r: 5 });
      } else {
        const s = focus ? size + 8 : size;
        ctx.drawImage(sprite(v.point.icon, v.point.group, s, ratio), v.x - s / 2, v.y - s / 2, s, s);
        drawn.push({ ...v, r: s / 2 });
      }
    }
    if (size > 12 && zoom - fitZoom >= LABEL_ZOOM) labels(ctx, plain, size);
    ctx.font = "700 13px Barlow, sans-serif";
    ctx.textAlign = "center";
    for (const v of stops) {
      ctx.beginPath();
      ctx.arc(v.x, v.y, 13, 0, Math.PI * 2);
      ctx.fillStyle = GROUPS.route.color;
      ctx.fill();
      ctx.strokeStyle = "#0b0b0f";
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = "#15181f";
      ctx.fillText(String(v.point.number), v.x, v.y + 5);
      drawn.push({ ...v, r: 13 });
    }
  }

  /** Names beside their markers, skipping any that would overlap one already drawn. */
  function labels(ctx: CanvasRenderingContext2D, list: { point: MapPoint; x: number; y: number }[], size: number) {
    ctx.font = "600 12px Barlow, sans-serif";
    ctx.textAlign = "left";
    ctx.lineJoin = "round";
    const placed: [number, number, number, number][] = [];
    for (const v of list) {
      const text = v.point.label;
      if (!text) continue;
      const w = ctx.measureText(text).width;
      const [x, y] = [v.x + size / 2 + 3, v.y + 4];
      if (placed.some(([a, b, c, d]) => x < c && x + w > a && y - 11 < d && y + 3 > b)) continue;
      placed.push([x, y - 11, x + w, y + 3]);
      ctx.strokeStyle = "rgba(8, 10, 14, 0.9)";
      ctx.lineWidth = 3;
      ctx.strokeText(text, x, y);
      ctx.fillStyle = "#f2efe6";
      ctx.fillText(text, x, y);
    }
  }

  // Changes queue one draw for the next frame, however many arrive in between.
  $effect(() => {
    void [points, projected, zoom, ox, oy, width, height, hover, selected, canvas];
    if (!frame) frame = requestAnimationFrame(draw);
  });

  function nearest(event: MouseEvent): MapPoint | null {
    const rect = box!.getBoundingClientRect();
    const [mx, my] = [event.clientX - rect.left, event.clientY - rect.top];
    let best: MapPoint | null = null;
    let distance = Infinity;
    // Last drawn is on top, so search backwards.
    for (let i = drawn.length - 1; i >= 0; i--) {
      const d = drawn[i];
      const gap = Math.hypot(d.x - mx, d.y - my);
      if (gap <= d.r + 3 && gap < distance) [best, distance] = [d.point, gap];
    }
    return best;
  }

  function zoomAt(mx: number, my: number, next: number) {
    next = Math.max(fitZoom - 1.5, Math.min((map.image.max_native_zoom ?? 5) + OVERZOOM, next));
    const k = 2 ** (next - zoom);
    ox = mx - (mx - ox) * k;
    oy = my - (my - oy) * k;
    zoom = next;
  }

  function onwheel(event: WheelEvent) {
    event.preventDefault();
    const rect = box!.getBoundingClientRect();
    zoomAt(event.clientX - rect.left, event.clientY - rect.top, zoom - event.deltaY * 0.002);
  }

  function onmove(event: MouseEvent) {
    if (drag) {
      ox += event.clientX - drag.x;
      oy += event.clientY - drag.y;
      drag = { x: event.clientX, y: event.clientY, moved: true };
    } else {
      const found = nearest(event);
      if (found !== hover) hover = found;
    }
  }

  function onup(event: MouseEvent) {
    if (drag && !drag.moved) onselect(nearest(event));
    drag = null;
  }

  const hoverAt = $derived.by(() => {
    if (!hover) return null;
    const [px, py] = project(hover);
    return [px * 2 ** zoom + ox, py * 2 ** zoom + oy];
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="map" bind:this={box} bind:clientWidth={width} bind:clientHeight={height}
  {onwheel} onmousemove={onmove} onmouseup={onup} onmouseleave={() => { drag = null; hover = null; }}
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
  {#if hover && hoverAt}
    <span class="tip" style:left="{hoverAt[0] + 16}px" style:top="{hoverAt[1] - 12}px">{hover.title}</span>
  {/if}
  <div class="controls" role="toolbar" tabindex="-1" aria-label="Zoom" onmousedown={(e) => e.stopPropagation()}>
    <button aria-label="Zoom in" onclick={() => zoomAt(width / 2, height / 2, zoom + 0.75)}>+</button>
    <button aria-label="Zoom out" onclick={() => zoomAt(width / 2, height / 2, zoom - 0.75)}>−</button>
    <button class="fit" onclick={fit}>Fit</button>
  </div>
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
  .controls {
    position: absolute;
    right: 10px;
    top: 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .controls button {
    min-width: 32px;
    height: 30px;
    border: 0;
    border-radius: 6px;
    background: var(--cream);
    color: var(--on-cream);
    font-family: var(--font-head);
    font-size: 16px;
    font-weight: 700;
    cursor: pointer;
  }
  .controls .fit {
    font-size: 12px;
    text-transform: uppercase;
  }
</style>
