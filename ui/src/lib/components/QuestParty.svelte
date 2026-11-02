<script lang="ts">
  import {
    addFriend, getProfile, partyPlan, questShareCode, removeFriend,
    type GameMap, type PartyPlan, type PartyQuest, type Profile,
  } from "../api";

  let { maps, selectedId, onselect }: { maps: GameMap[]; selectedId: string | null; onselect: (id: string) => void } = $props();

  let profile = $state<Profile | null>(null);
  let name = $state("");
  let code = $state("");
  let pasted = $state("");
  let friend = $state("");
  let plan = $state<PartyPlan | null>(null);
  let message = $state("");

  async function load() {
    profile = await getProfile();
    name ||= profile.player_name ?? "";
    if (!profile.friends.some((f) => f.name === friend)) friend = profile.friends[0]?.name ?? "";
    plan = friend ? await partyPlan(friend) : null;
  }
  load();

  async function share() {
    code = await questShareCode(name);
    await navigator.clipboard.writeText(code).then(
      () => (message = "Code copied. Send it to your friend."),
      () => (message = "Copy the code below and send it to your friend."),
    );
  }

  async function add() {
    try {
      friend = await addFriend(pasted);
      pasted = "";
      message = `Updated ${friend}'s progress.`;
      await load();
    } catch (e) {
      message = String(e);
    }
  }

  async function remove() {
    await removeFriend(friend);
    message = "";
    await load();
  }

  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
</script>

{#snippet column(title: string, hint: string, rows: PartyQuest[])}
  <section class="panel">
    <h3 class="section-title">{title} ({rows.length})</h3>
    <p class="dim small">{hint}</p>
    {#each rows as row (row.quest.id)}
      <button class="row" class:on={row.quest.id === selectedId} onclick={() => onselect(row.quest.id)}>
        <strong>{row.quest.name}</strong>
        <span class="dim small">
          {row.active ? "Tracked · " : ""}{row.maps.map(mapName).join(", ") || "Any map"}
        </span>
      </button>
    {:else}
      <p class="dim small">Nothing here.</p>
    {/each}
  </section>
{/snippet}

<div class="party">
  <section class="panel setup">
    <div class="line">
      <input class="field" placeholder="Your name" aria-label="Your name" bind:value={name} />
      <button class="button dark" disabled={!name.trim()} onclick={share}>Copy my code</button>
      <input class="field paste" placeholder="Paste a friend's code" aria-label="Friend's code" bind:value={pasted} />
      <button class="button dark" disabled={!pasted.trim()} onclick={add}>Add or update friend</button>
    </div>
    {#if code}<textarea class="field" readonly aria-label="Your code" rows="2" value={code}></textarea>{/if}
    {#if message}<p class="message small" role="status">{message}</p>{/if}
    <p class="dim small">
      A code holds your name and quest progress and nothing else. It's a snapshot, so swap new codes when you
      want to sync. It goes only where you paste it.
    </p>
    {#if profile?.friends.length}
      <div class="line">
        <select class="field" aria-label="Friend" bind:value={friend} onchange={load}>
          {#each profile.friends as f (f.name)}<option value={f.name}>{f.name}</option>{/each}
        </select>
        <button class="button dark" onclick={remove}>Remove</button>
      </div>
    {/if}
  </section>

  {#if plan}
    {#if plan.maps.length}
      <p class="small">Best map together: <strong>{mapName(plan.maps[0][0])}</strong> ({plan.maps[0][1]} of your shared quests)</p>
    {/if}
    <div class="columns">
      {@render column("Do together", "You can both do these now. Oldest first.", plan.together)}
      {@render column(`${plan.friend} catches up`, "You've finished these. Help them get level with you.", plan.friend_catch_up)}
      {@render column("You catch up", `${plan.friend} has finished these. Do them to get level.`, plan.you_catch_up)}
    </div>
  {:else}
    <p class="dim">Add a friend's code to see which quests to do together.</p>
  {/if}
</div>

<style>
  .party {
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow: auto;
    min-height: 0;
  }
  .setup {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .paste {
    flex: 1;
    min-width: 180px;
  }
  textarea {
    width: 100%;
    resize: none;
    word-break: break-all;
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
    align-items: start;
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 1px;
    width: 100%;
    padding: 5px 6px;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
    text-transform: capitalize;
  }
  .row.on {
    background: var(--panel-2);
  }
  .small {
    font-size: 13px;
    margin: 0;
  }
  .message {
    color: var(--teal);
  }
</style>
