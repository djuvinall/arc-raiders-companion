<script lang="ts">
  import { linkHref, type Link } from "../api";
  import Icon from "./Icon.svelte";

  let { link }: { link: Link } = $props();
</script>

<a class="row" href={linkHref(link)}>
  {#if link.kind === "item"}
    <Icon url={link.icon} name={link.name} size={26} />
  {/if}
  <span class="name">{link.name}</span>
  {#if link.note}<span class="note">{link.note}</span>{/if}
  {#if link.qty !== null}<span class="qty">×{link.qty}</span>{/if}
</a>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 6px;
    border-radius: 4px;
  }
  .row:hover {
    background: var(--panel-2);
  }
  .name {
    flex: 1;
    min-width: 0;
  }
  .note {
    color: var(--text-dim);
    font-size: 13px;
  }
  .qty {
    color: var(--blue);
    font-weight: 600;
  }
</style>
