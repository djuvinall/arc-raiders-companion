<script lang="ts">
  import type { Overview } from "../api";

  let { info }: { info: Overview | null } = $props();
</script>

<footer>
  <span><span class="keycap">ESC</span> Back</span>
  <span><span class="keycap">/</span> Search</span>
  <span class="status">
    {#if info}
      Dataset {info.dataset_version}
      {#if info.game_patch}· patch {info.game_patch}{/if}
      · {info.item_count} items
    {/if}
  </span>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 8px 20px;
    border-top: 1px solid var(--border);
    font-family: var(--font-head);
    font-size: 14px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
  }
  .status {
    margin-left: auto;
  }
</style>
