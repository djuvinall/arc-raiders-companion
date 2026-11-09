<script lang="ts">
  // Right-click anywhere: Copy, and Report…, which files an issue on the data
  // repository through the relay. What was reported comes from the nearest
  // `data-report` tag ("file/id" or "file/id/field"), and its label from every
  // `data-report-label` above it, so "Anvil I · Sell value".
  import { tick } from "svelte";
  import { sendReport, type ReportSent } from "../api";

  type Kind = "incorrect" | "suggestion";
  interface Target {
    ref: string | null;
    label: string | null;
    selected: string | null;
    field: boolean;
  }

  let menu = $state<{ x: number; y: number } | null>(null);
  let target = $state<Target | null>(null);
  let dialog = $state(false);
  let kind = $state<Kind>("incorrect");
  let suggestion = $state("");
  let comment = $state("");
  let sending = $state(false);
  let sent = $state<ReportSent | null>(null);
  let error = $state<string | null>(null);
  let first = $state<HTMLElement>();

  function read(el: Element | null, selected: string | null): Target {
    const tagged = el?.closest("[data-report]") ?? null;
    const ref = tagged?.getAttribute("data-report") ?? null;
    const labels: string[] = [];
    for (let e = el?.closest("[data-report-label]"); e; e = e.parentElement?.closest("[data-report-label]")) {
      labels.unshift(e.getAttribute("data-report-label")!);
    }
    return { ref, label: labels.join(" · ") || null, selected, field: (ref?.split("/").length ?? 0) > 2 };
  }

  function onContext(event: MouseEvent) {
    // A page that uses right-click itself (the skill tree) has already handled it.
    if (event.defaultPrevented) return;
    const el = event.target instanceof Element ? event.target : null;
    // Text fields keep the system menu, for paste.
    if (!el || el.closest("input, textarea, select, [contenteditable]")) return;
    event.preventDefault();
    const sel = window.getSelection();
    const text = sel?.toString().trim().slice(0, 1000) || null;
    // With a selection, report what was selected; otherwise what was clicked.
    const node = text && sel?.anchorNode ? sel.anchorNode : el;
    const anchor = node instanceof Element ? node : node.parentElement;
    target = read(anchor, text);
    menu = { x: Math.min(event.clientX, innerWidth - 180), y: Math.min(event.clientY, innerHeight - 90) };
  }

  async function copy() {
    const text = target?.selected ?? "";
    menu = null;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      document.execCommand("copy");
    }
  }

  async function openDialog() {
    menu = null;
    kind = "incorrect";
    suggestion = "";
    comment = "";
    sent = null;
    error = null;
    dialog = true;
    await tick();
    first?.focus();
  }

  async function send() {
    if (!target) return;
    if (kind === "suggestion" && !suggestion.trim() && !comment.trim()) {
      error = "Write your suggestion first.";
      return;
    }
    sending = true;
    error = null;
    try {
      sent = await sendReport({
        kind,
        data_ref: target.ref,
        label: target.label,
        page: location.hash || "#/home",
        selected: target.selected,
        suggestion: suggestion.trim() || null,
        comment: comment.trim() || null,
      });
    } catch (e) {
      error = String(e);
    } finally {
      sending = false;
    }
  }

  // Capture phase, so Esc closes this before the app treats it as Back.
  function onKey(event: KeyboardEvent) {
    if (event.key !== "Escape" || !(menu || dialog)) return;
    event.preventDefault();
    event.stopPropagation();
    menu = null;
    if (!sending) dialog = false;
  }

  $effect(() => {
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });
</script>

<svelte:window oncontextmenu={onContext} onblur={() => (menu = null)} onresize={() => (menu = null)} />

{#if menu}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="catch" onclick={() => (menu = null)} oncontextmenu={(e) => { e.preventDefault(); menu = null; }}></div>
  <div class="menu" role="menu" style="left: {menu.x}px; top: {menu.y}px">
    {#if target?.selected}<button role="menuitem" onclick={copy}>Copy</button>{/if}
    <button role="menuitem" onclick={openDialog}>Report…</button>
  </div>
{/if}

{#if dialog && target}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="scrim" onclick={() => !sending && (dialog = false)}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="dialog panel" role="dialog" aria-modal="true" aria-label="Report" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      {#if sent}
        <h3 class="section-title">Thanks, report sent</h3>
        <p>It's report #{sent.issue}. It gets checked before anything in the app changes{sent.pr_url ? ", and your suggested value is ready to apply once it's confirmed" : ""}.</p>
        <div class="actions"><button class="button primary" onclick={() => (dialog = false)}>Close</button></div>
      {:else}
        <h3 class="section-title">Report</h3>
        <div class="about">
          {#if target.label}<strong>{target.label}</strong>{/if}
          {#if target.selected}<blockquote>{target.selected}</blockquote>{/if}
          {#if !target.label && !target.selected}<span class="dim">This part of the page</span>{/if}
        </div>

        <div class="kinds" role="radiogroup" aria-label="Report type">
          <button bind:this={first} role="radio" aria-checked={kind === "incorrect"} class:on={kind === "incorrect"} onclick={() => (kind = "incorrect")}>
            <strong>Incorrect data</strong><span class="dim small">Something here is wrong or out of date</span>
          </button>
          <button role="radio" aria-checked={kind === "suggestion"} class:on={kind === "suggestion"} onclick={() => (kind = "suggestion")}>
            <strong>Suggestion</strong><span class="dim small">A tip, missing info, or an idea</span>
          </button>
        </div>

        <label>
          <span>{kind === "incorrect" ? "What should it be?" : "Your suggestion"} <span class="dim small">{kind === "incorrect" ? "optional" : ""}</span></span>
          <input class="field" maxlength="500" bind:value={suggestion} placeholder={kind === "incorrect" ? (target.field ? "The correct value" : "What it should say") : "What should the app add or change?"} />
        </label>
        <label>
          <span>Comment <span class="dim small">optional</span></span>
          <textarea class="field" rows="3" maxlength="2000" bind:value={comment} placeholder="How you know, e.g. checked in game after patch 2.4.1"></textarea>
        </label>
        <p class="dim small">Reports are anonymous. Nothing about you or your profile is sent, only this form and the app's version.</p>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="actions">
          <button class="button" disabled={sending} onclick={() => (dialog = false)}>Cancel</button>
          <button class="button primary" disabled={sending} onclick={send}>{sending ? "Sending…" : "Send"}</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .catch {
    position: fixed;
    inset: 0;
    z-index: 20;
  }
  .menu {
    position: fixed;
    z-index: 21;
    display: flex;
    flex-direction: column;
    min-width: 160px;
    padding: 4px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  }
  .menu button {
    padding: 6px 10px;
    background: none;
    border: 0;
    border-radius: 4px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .menu button:hover,
  .menu button:focus-visible {
    background: var(--panel-2);
  }
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 22;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 10vh;
    background: rgba(0, 0, 0, 0.55);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: min(520px, 92vw);
    max-height: 80vh;
    overflow: auto;
  }
  .about {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  blockquote {
    margin: 0;
    padding: 4px 10px;
    border-left: 3px solid var(--yellow);
    color: var(--text-dim);
    max-height: 6em;
    overflow: auto;
    white-space: pre-wrap;
  }
  .kinds {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .kinds button {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .kinds button.on {
    border-color: var(--yellow);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  textarea {
    resize: vertical;
    font: inherit;
  }
  .small {
    font-size: 12px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  p {
    margin: 0;
  }
</style>
