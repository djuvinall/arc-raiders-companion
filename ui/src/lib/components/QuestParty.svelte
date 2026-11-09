<script lang="ts">
  import {
    addFriend, getProfile, partyPlan, questShareCode, removeFriend, setFriendQuest,
    type GameMap, type PartyPlan, type PartyQuest, type Profile,
  } from "../api";

  let { maps, selectedId, onselect, onchange }: {
    maps: GameMap[]; selectedId: string | null; onselect: (id: string) => void; onchange?: () => void;
  } = $props();

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

  let confirmRemove = $state(false);
  async function remove() {
    if (!confirmRemove) {
      confirmRemove = true;
      return;
    }
    confirmRemove = false;
    await removeFriend(friend);
    message = "";
    await load();
  }

  // Between code swaps: tick off what you finished together, or what they told you they did.
  async function tickFriend(row: PartyQuest, me: boolean) {
    await setFriendQuest(friend, row.quest.id, true, me);
    message = me ? `Marked ${row.quest.name} done for you and ${friend}.` : `Marked ${row.quest.name} done for ${friend}.`;
    await load();
    if (me) onchange?.();
  }

  const mapName = (id: string) => maps.find((m) => m.id === id)?.name ?? id;
</script>

{#snippet column(title: string, hint: string, rows: PartyQuest[], action: "both" | "friend" | null)}
  <section class="panel">
    <h3 class="section-title">{title} ({rows.length})</h3>
    <p class="dim small">{hint}</p>
    {#each rows as row (row.quest.id)}
      <div class="entry" class:on={row.quest.id === selectedId}>
        <button class="row" onclick={() => onselect(row.quest.id)}>
          <strong>{row.quest.name}</strong>
          <span class="dim small">
            {row.active ? "Tracked · " : ""}{row.maps.map(mapName).join(", ") || "Any map"}
          </span>
        </button>
        {#if action}
          <button class="tick" title={action === "both" ? "You both finished it" : `${plan?.friend} finished it`} onclick={() => tickFriend(row, action === "both")}>
            {action === "both" ? "✓ Both" : "✓ Done"}
          </button>
        {/if}
      </div>
    {:else}
      <p class="dim small">Nothing here.</p>
    {/each}
  </section>
{/snippet}

<div class="party">
  <div class="setup-grid">
    <section class="panel setup">
      <h3 class="section-title">Your code</h3>
      <p class="dim small">Send this to a friend so they can see your quests. The name is just how you show up in their list.</p>
      <div class="line">
        <input class="field" placeholder="Your name" aria-label="Your name" bind:value={name} />
        <button class="button dark" disabled={!name.trim()} onclick={share}>Copy my code</button>
      </div>
      {#if code}<textarea class="field" readonly aria-label="Your code" rows="2" value={code}></textarea>{/if}
    </section>

    <section class="panel setup">
      <h3 class="section-title">Friends</h3>
      <p class="dim small">Paste a code a friend sent you. A newer code from the same friend updates them.</p>
      <div class="line">
        <input class="field paste" placeholder="Paste a friend's code" aria-label="Friend's code" bind:value={pasted} />
        <button class="button dark" disabled={!pasted.trim()} onclick={add}>Add or update</button>
      </div>
      {#if profile?.friends.length}
        <div class="line">
          <select class="field" aria-label="Friend" bind:value={friend} onchange={load}>
            {#each profile.friends as f (f.name)}<option value={f.name}>{f.name}</option>{/each}
          </select>
          <button class="button dark" class:danger={confirmRemove} onclick={remove}>{confirmRemove ? `Remove ${friend}?` : "Remove"}</button>
        </div>
      {/if}
    </section>
  </div>
  {#if message}<p class="message small" role="status">{message}</p>{/if}
  <p class="dim small">
    Codes hold a name and quest progress, nothing else, and go only where you paste them. Between swaps,
    tick quests off below as you finish them: <strong>✓ Both</strong> for ones you did together, <strong>✓ Done</strong> for ones your friend did.
  </p>

  {#if plan}
    {#if plan.maps.length}
      <p class="small">Best map together: <strong>{mapName(plan.maps[0][0])}</strong> ({plan.maps[0][1]} of your shared quests)</p>
    {/if}
    <div class="columns">
      {@render column("Do together", "You can both do these now. Oldest first.", plan.together, "both")}
      {@render column(`${plan.friend} catches up`, "You've finished these. Help them get level with you.", plan.friend_catch_up, "friend")}
      {@render column("You catch up", `${plan.friend} has finished these. Do them to get level.`, plan.you_catch_up, null)}
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
  .setup-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr);
    gap: 10px;
  }
  .setup .section-title {
    margin: 0;
  }
  .entry {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }
  .entry.on {
    background: var(--panel-2);
  }
  .entry .row {
    border-bottom: 0;
  }
  .tick {
    padding: 2px 8px;
    margin-right: 4px;
    background: none;
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--teal);
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    white-space: nowrap;
  }
  .tick:hover {
    border-color: var(--teal);
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
  .small {
    font-size: 13px;
    margin: 0;
  }
  .message {
    color: var(--teal);
  }
  .entry:not(.on) .row:hover {
    background: var(--panel-2);
  }
</style>
