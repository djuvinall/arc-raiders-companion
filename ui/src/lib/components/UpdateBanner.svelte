<script lang="ts">
  import { applyData, installUpdate, updates } from "../updates.svelte";
</script>

{#if updates.error}
  <div class="banner" role="status"><span>{updates.error}</span></div>
{:else if updates.app}
  <div class="banner" role="status">
    <span>Version {updates.app} is ready to install. The app restarts when it's done.</span>
    <button class="button primary" disabled={updates.installing} onclick={installUpdate}>
      {updates.installing ? "Updating…" : "Update now"}
    </button>
  </div>
{:else if updates.data}
  <div class="banner" role="status">
    <span>New item data ({updates.data}) is ready.</span>
    <button class="button primary" onclick={applyData}>Restart to apply</button>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 6px 16px;
    background: var(--panel-2);
    border-bottom: 1px solid var(--border);
  }
</style>
