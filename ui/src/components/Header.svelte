<script lang="ts">
  import { app } from "../lib/state/app.svelte";

  const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;
  let summary = $derived.by(() => {
    const c = app.snapshot?.counts;
    return c ? `${plural(c.sessions, "session")} · ${c.busy} busy · ${plural(c.agents, "agent")}` : "";
  });
</script>

<header>
  <h1>CEREBRO</h1>
  <span>
    {#if app.phone}<i class="link" class:live={app.connected} title={app.connected ? "Live" : "Reconnecting…"}></i>{/if}
    {summary}
  </span>
</header>

<style>
  header { display: flex; flex-wrap: wrap; gap: 4px 12px; justify-content: space-between; align-items: baseline; }
  h1 { font-size: 15px; font-weight: 900; margin: 0; letter-spacing: 0.5px; }
  span { color: var(--muted); font-size: 11px; display: flex; align-items: center; gap: 6px; }
  .link { width: 7px; height: 7px; border-radius: 50%; background: #ff6b6b; }
  .link.live { background: var(--busy); }
  :global(.phone) h1 { font-size: 20px; }
  :global(.phone) span { font-size: 13px; }
</style>
