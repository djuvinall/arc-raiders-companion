<script lang="ts">
  import type { TreeNode } from "../api";
  import TreeRow from "./TreeRow.svelte";

  let { node, depth = 0 }: { node: TreeNode; depth?: number } = $props();
  let open = $state(true);
</script>

<div class="node" style:--depth={depth}>
  <div class="line">
    {#if node.children.length}
      <button aria-label={open ? "Collapse" : "Expand"} aria-expanded={open} onclick={() => (open = !open)}>
        {open ? "−" : "+"}
      </button>
    {:else}
      <span class="leaf"></span>
    {/if}
    <span class="qty">×{node.qty}</span>
    <a href="#/item/{node.item_id}">{node.name}</a>
    {#if node.workbench_id}
      <span class="bench">{node.workbench_id}{node.workbench_level ? ` ${node.workbench_level}` : ""}</span>
    {/if}
    {#if node.cycle}<span class="bench">cycle, not expanded</span>{/if}
  </div>
  {#if open}
    {#each node.children as child}
      <TreeRow node={child} depth={depth + 1} />
    {/each}
  {/if}
</div>

<style>
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 0 3px calc(var(--depth) * 22px);
  }
  button {
    width: 20px;
    height: 20px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--panel-2);
    color: var(--text);
    cursor: pointer;
    line-height: 1;
    padding: 0;
  }
  .leaf {
    width: 20px;
  }
  .qty {
    color: var(--blue);
    font-weight: 600;
    min-width: 44px;
  }
  .bench {
    color: var(--teal);
    font-size: 13px;
  }
</style>
