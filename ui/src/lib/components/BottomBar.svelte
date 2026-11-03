<script lang="ts">
  import { openMini, type Overview } from "../api";

  let { info }: { info: Overview | null } = $props();
</script>

<footer>
  <span><span class="keycap">ESC</span> Back</span>
  <span><span class="keycap">Ctrl K</span> Search anything</span>
  <button class="mini" onclick={openMini}>Compact window</button>
  <span class="status">
    {#if info}
      Dataset {info.dataset_version}
      {#if info.game_patch}· patch {info.game_patch}{/if}
      · {info.item_count} items
    {/if}
  </span>
</footer>

<style>
  .mini {
    padding: 1px 10px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: none;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }
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
