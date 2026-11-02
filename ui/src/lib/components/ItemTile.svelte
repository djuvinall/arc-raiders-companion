<script lang="ts">
  import { ROMAN, type ItemSummary } from "../api";
  import Icon from "./Icon.svelte";

  let { item, found = false }: { item: ItemSummary; found?: boolean } = $props();
</script>

<a class="tile" class:blueprint={item.category === "blueprint"} href="#/item/{item.id}" title={item.name}>
  <Icon url={item.icon} name={item.name} size={64} />
  {#if item.rarity}
    <span class="wedge" style:background="var(--rarity-{item.rarity})"></span>
  {/if}
  {#if item.tier}
    <span class="tier">{ROMAN[item.tier]}</span>
  {/if}
  {#if found}<span class="check" aria-label="Found">✓</span>{/if}
  <span class="name">{item.name}</span>
</a>

<style>
  .tile {
    position: relative;
    display: grid;
    place-items: center;
    width: 96px;
    height: 96px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
    content-visibility: auto;
    contain-intrinsic-size: 96px 96px;
  }
  .blueprint {
    background: var(--blueprint-bg);
  }
  .tile:hover,
  .tile:focus-visible {
    background: var(--cream);
    color: var(--on-cream);
    outline: none;
  }
  .wedge {
    position: absolute;
    left: 0;
    bottom: 0;
    width: 22px;
    height: 22px;
    clip-path: polygon(0 0, 100% 100%, 0 100%);
  }
  .tier {
    position: absolute;
    right: 5px;
    bottom: 2px;
    font-family: var(--font-head);
    font-size: 13px;
    font-weight: 700;
  }
  .check {
    position: absolute;
    top: 3px;
    right: 3px;
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--teal);
    color: #0b0b0f;
    font-size: 12px;
  }
  .name {
    position: absolute;
    inset: auto 0 0 0;
    padding: 2px 4px;
    background: rgba(11, 11, 15, 0.92);
    color: var(--text);
    font-size: 11px;
    line-height: 1.15;
    text-align: center;
    transform: translateY(100%);
    transition: transform 0.08s linear;
  }
  .tile:hover .name,
  .tile:focus-visible .name {
    transform: none;
  }
</style>
