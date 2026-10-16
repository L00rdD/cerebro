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
  <span>{summary}</span>
</header>

<style>
  header { display: flex; justify-content: space-between; align-items: baseline; }
  h1 { font-size: 15px; font-weight: 900; margin: 0; letter-spacing: 0.5px; }
  span { color: var(--muted); font-size: 11px; }
</style>
