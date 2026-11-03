<script lang="ts">
  import { overview, type Overview } from "./lib/api";
  import BottomBar from "./lib/components/BottomBar.svelte";
  import QuickSearch from "./lib/components/QuickSearch.svelte";
  import TopBar from "./lib/components/TopBar.svelte";
  import UpdateBanner from "./lib/components/UpdateBanner.svelte";
  import { checkForUpdates } from "./lib/updates.svelte";
  import About from "./routes/About.svelte";
  import Codex from "./routes/Codex.svelte";
  import Craft from "./routes/Craft.svelte";
  import EntityRef from "./routes/EntityRef.svelte";
  import Arcs from "./routes/Arcs.svelte";
  import ItemPage from "./routes/ItemPage.svelte";
  import Mini from "./routes/Mini.svelte";
  import Raid from "./routes/Raid.svelte";
  import Items from "./routes/Items.svelte";
  import Maps from "./routes/Maps.svelte";
  import Patches from "./routes/Patches.svelte";
  import ProfilePage from "./routes/ProfilePage.svelte";
  import Projects from "./routes/Projects.svelte";
  import Quests from "./routes/Quests.svelte";
  import Skills from "./routes/Skills.svelte";
  import Weapons from "./routes/Weapons.svelte";

  const TABS = ["raid", "arcs", "skills", "weapons", "maps", "quests", "projects", "codex", "patches", "profile", "about"];
  const current = () => (location.hash.replace(/^#\/?/, "") || "items").split("/");

  let parts = $state(current());
  let info = $state<Overview | null>(null);
  let main = $state<HTMLElement>();

  overview().then((o) => (info = o));
  // svelte-ignore state_referenced_locally
  if (parts[0] !== "mini") checkForUpdates();

  function onHash() {
    parts = current();
    main?.scrollTo(0, 0);
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape" && parts[0] !== "items") {
      history.back();
    }
  }

  const titles = $derived.by(() => {
    switch (parts[0]) {
      case "item":
        return { title: "Database", subtitle: "Item" };
      case "craft":
        return { title: "Database", subtitle: "Crafting" };
      case "ref":
        return { title: "Database", subtitle: parts[1].replace("-", " ") };
      case "raid":
        return { title: "Raid plan", subtitle: "Next run" };
      case "arcs":
        return { title: "ARCs", subtitle: "Enemy guide" };
      case "skills":
        return { title: "Skills", subtitle: "Planner" };
      case "weapons":
        return { title: "Weapons", subtitle: "Builds" };
      case "maps":
        return { title: "Maps", subtitle: "Events and routes" };
      case "quests":
        return { title: "Quests", subtitle: "Tracker" };
      case "projects":
        return { title: "Projects", subtitle: "Stages" };
      case "codex":
        return { title: "Codex", subtitle: "Collections" };
      case "patches":
        return { title: "Patch notes", subtitle: "Changes" };
      case "profile":
        return { title: "Profile", subtitle: "Tracking" };
      case "about":
        return { title: "About", subtitle: "Attribution" };
      default:
        return { title: "Database", subtitle: "Items" };
    }
  });
</script>

<svelte:window onhashchange={onHash} onkeydown={onKey} />

{#if parts[0] === "mini"}
  <Mini />
{:else}
<QuickSearch />
<div class="shell">
  <TopBar {...titles} canGoBack={parts[0] !== "items"} active={TABS.includes(parts[0]) ? parts[0] : "items"} />
  <div><UpdateBanner /></div>
  <main bind:this={main}>
    {#key parts.join("/")}
      {#if parts[0] === "item"}
        <ItemPage id={parts[1]} />
      {:else if parts[0] === "craft"}
        <Craft id={parts[1]} />
      {:else if parts[0] === "ref"}
        <EntityRef kind={parts[1]} id={parts[2]} />
      {:else if parts[0] === "raid"}
        <Raid mapId={parts[1]} />
      {:else if parts[0] === "arcs"}
        <Arcs id={parts[1]} />
      {:else if parts[0] === "skills"}
        <Skills />
      {:else if parts[0] === "weapons"}
        <Weapons />
      {:else if parts[0] === "maps"}
        <Maps mapId={parts[1]} />
      {:else if parts[0] === "quests"}
        <Quests id={parts[1]} />
      {:else if parts[0] === "projects"}
        <Projects id={parts[1]} />
      {:else if parts[0] === "codex"}
        <Codex />
      {:else if parts[0] === "patches"}
        <Patches {info} />
      {:else if parts[0] === "profile"}
        <ProfilePage />
      {:else if parts[0] === "about"}
        <About {info} />
      {:else}
        <Items {info} />
      {/if}
    {/key}
  </main>
  <BottomBar {info} />
</div>
{/if}

<style>
  .shell {
    display: grid;
    grid-template-rows: auto auto 1fr auto;
    height: 100%;
  }
  main {
    overflow: auto;
    min-height: 0;
  }
</style>
