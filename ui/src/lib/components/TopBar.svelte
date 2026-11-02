<script lang="ts">
  interface Props {
    title: string;
    subtitle: string;
    canGoBack: boolean;
    active: string;
  }
  let { title, subtitle, canGoBack, active }: Props = $props();
</script>

<header>
  <button class="back" aria-label="Back" disabled={!canGoBack} onclick={() => history.back()}>
    <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
      <path d="M15 5l-7 7 7 7" fill="none" stroke="currentColor" stroke-width="2.5" />
    </svg>
  </button>
  <h1>{title} <span>— {subtitle}</span></h1>
  <nav>
    {#each ["items", "skills", "weapons", "maps", "quests", "projects", "codex", "patches", "profile", "about"] as tab}
      <a class="tab" class:active={active === tab} href="#/{tab}">{tab}</a>
    {/each}
  </nav>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 20px;
    border-bottom: 1px solid var(--border);
  }
  .back {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.3);
    background: transparent;
    color: var(--text);
    cursor: pointer;
  }
  .back:disabled {
    opacity: 0.3;
    cursor: default;
  }
  h1 {
    font-size: 22px;
    white-space: nowrap;
  }
  @media (max-width: 1240px) {
    h1 span {
      display: none;
    }
  }
  h1 span {
    color: var(--text-dim);
    font-weight: 600;
  }
  nav {
    margin-left: auto;
    display: flex;
    gap: 2px;
  }
  .tab {
    padding: 4px 10px;
    border-radius: 999px;
    border: 1px solid transparent;
    font-family: var(--font-head);
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-dim);
  }
  .tab.active {
    border-color: var(--text);
    color: var(--text);
  }
</style>
