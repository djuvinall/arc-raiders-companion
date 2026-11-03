<script lang="ts">
  import { iconSrc } from "../api";

  // With a cold cache and no network the image 404s; initials keep the tile usable.
  let { url, name, size = 32 }: { url: string | null; name: string; size?: number } = $props();
  let failed = $state(false);
  const initials = $derived(
    name.split(/[\s-]+/).slice(0, 2).map((w) => w[0]?.toUpperCase() ?? "").join(""),
  );
</script>

{#if url && !failed}
  <img
    crossorigin="anonymous"
    src={iconSrc(url)}
    alt=""
    width={size}
    height={size}
    loading="lazy"
    decoding="async"
    onerror={() => (failed = true)}
  />
{:else}
  <span class="initials" style:width="{size}px" style:height="{size}px" style:font-size="{size * 0.36}px">
    {initials}
  </span>
{/if}

<style>
  img {
    object-fit: contain;
    display: block;
  }
  .initials {
    display: grid;
    place-items: center;
    font-family: var(--font-head);
    font-weight: 700;
    color: var(--text-dim);
    opacity: 0.8;
  }
</style>
