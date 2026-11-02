<script module lang="ts">
  export const GROUPS: { id: string; label: string; categories: string[]; path: string }[] = [
    { id: "all", label: "All", categories: [], path: "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z" },
    { id: "weapons", label: "Weapons", categories: ["weapon"], path: "M3 10h14l4-2v4l-4 1h-5l-2 6H6l1-6H3z" },
    { id: "mods", label: "Weapon Mods", categories: ["weapon-mod"], path: "M12 3l2 3 4-1 1 4 3 2-3 2-1 4-4-1-2 3-2-3-4 1-1-4-3-2 3-2 1-4 4 1z" },
    { id: "gear", label: "Shields and Augments", categories: ["shield", "augment"], path: "M12 3l8 3v6c0 5-4 8-8 9-4-1-8-4-8-9V6z" },
    { id: "quick", label: "Quick Use", categories: ["healing", "gadget", "grenade", "tool"], path: "M10 3h4v7h7v4h-7v7h-4v-7H3v-4h7z" },
    { id: "ammo", label: "Ammo", categories: ["ammo"], path: "M9 3h6l2 5v13H7V8z" },
    { id: "materials", label: "Materials", categories: ["material-basic", "material-topside", "material-refined", "nature"], path: "M12 3l9 5-9 5-9-5zM3 13l9 5 9-5v3l-9 5-9-5z" },
    { id: "loot", label: "Recyclables and Trinkets", categories: ["recyclable", "trinket", "valuable", "food", "misc"], path: "M5 8h14l-2 13H7zM9 8a3 3 0 0 1 6 0" },
    { id: "keys", label: "Keys and Quest Items", categories: ["key", "quest-item"], path: "M8 3a5 5 0 0 1 4 8l9 9-2 2-2-2-2 2-2-2 1-1-5-5A5 5 0 1 1 8 3z" },
    { id: "blueprints", label: "Blueprints and Research", categories: ["blueprint", "research", "stencil", "design"], path: "M4 3h16v18H4zM8 7v10M12 7v10M16 7v10" },
    { id: "cosmetics", label: "Cosmetics and Furniture", categories: ["cosmetic", "furniture"], path: "M9 3l3 3 3-3 6 4-3 4-2-1v11H8V10l-2 1-3-4z" },
  ];
</script>

<script lang="ts">
  let { active, onselect }: { active: string; onselect: (id: string) => void } = $props();
</script>

<nav aria-label="Categories">
  {#each GROUPS as group}
    <button
      class:active={active === group.id}
      title={group.label}
      aria-label={group.label}
      aria-pressed={active === group.id}
      onclick={() => onselect(group.id)}
    >
      <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
        <path d={group.path} fill="currentColor" fill-rule="evenodd" />
      </svg>
    </button>
  {/each}
</nav>

<style>
  nav {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 12px;
  }
  button {
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--panel);
    color: var(--text-dim);
    cursor: pointer;
  }
  button:hover {
    color: var(--text);
  }
  button.active {
    background: var(--cream);
    color: var(--on-cream);
  }
</style>
