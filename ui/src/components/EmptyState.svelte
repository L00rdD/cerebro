<script lang="ts">
  import type { Catalog } from "../lib/assets/catalog";
  import SceneCanvas from "./SceneCanvas.svelte";

  let { catalog }: { catalog: Catalog } = $props();
  let room = $derived(catalog.rooms.get("cerebro") ?? [...catalog.rooms.values()][0]!);
</script>

<div class="empty">
  <SceneCanvas {catalog} {room} actors={[{ id: "empty", character: "professor-x", state: "asleep" }]} />
  <p>No mutants detected. Start a Claude Code session.</p>
</div>

<style>
  .empty { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 8px 0; }
  p { color: var(--muted); margin: 0; }
</style>
