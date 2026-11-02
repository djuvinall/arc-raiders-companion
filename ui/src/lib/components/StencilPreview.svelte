<script lang="ts">
  import { iconSrc } from "../api";
  import Icon from "./Icon.svelte";

  let { gun, swatch, name, size }: { gun: string | null; swatch: string; name: string; size: number } = $props();
  let canvas = $state<HTMLCanvasElement>();
  let failed = $state(false);

  // The swatch card's corner holds a label, so only this part of it is pattern.
  const PATTERN = { x: 20, y: 20, size: 120 };
  const RESOLUTION = 512;
  const TILE = RESOLUTION / 2;

  const load = (url: string) => new Promise<HTMLImageElement>((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = reject;
    image.src = iconSrc(url);
  });

  // The pattern is cut to the gun's shape, then multiplied so the metal's shading shows through.
  async function paint(target: HTMLCanvasElement, gunUrl: string) {
    const [body, card] = await Promise.all([load(gunUrl), load(swatch)]);
    const layer = document.createElement("canvas");
    layer.width = layer.height = RESOLUTION;
    const pattern = layer.getContext("2d")!;
    for (let x = 0; x < RESOLUTION; x += TILE) {
      for (let y = 0; y < RESOLUTION; y += TILE) {
        pattern.drawImage(card, PATTERN.x, PATTERN.y, PATTERN.size, PATTERN.size, x, y, TILE, TILE);
      }
    }
    pattern.globalCompositeOperation = "destination-in";
    pattern.drawImage(body, 0, 0, RESOLUTION, RESOLUTION);

    const context = target.getContext("2d")!;
    context.drawImage(body, 0, 0, RESOLUTION, RESOLUTION);
    context.globalCompositeOperation = "multiply";
    context.drawImage(layer, 0, 0);
  }

  $effect(() => {
    if (canvas && gun) paint(canvas, gun).catch(() => (failed = true));
  });
</script>

{#if failed || !gun}
  <Icon url={gun} {name} {size} />
{:else}
  <canvas
    bind:this={canvas} width={RESOLUTION} height={RESOLUTION}
    style:width="{size}px" style:height="{size}px" aria-label="{name} with this stencil"
  ></canvas>
{/if}
