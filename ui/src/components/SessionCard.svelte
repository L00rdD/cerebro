<script lang="ts">
  import { backend } from "../lib/api/backend";
  import type { SceneDto } from "../lib/api/types";
  import type { Catalog } from "../lib/assets/catalog";
  import { STATUS_LABELS, humanizeDuration, shortenHome } from "../lib/format";
  import type { ActorState, SceneActor } from "../lib/scene/renderer";
  import { app } from "../lib/state/app.svelte";
  import SceneCanvas from "./SceneCanvas.svelte";

  let { scene, catalog, now }: { scene: SceneDto; catalog: Catalog; now: number } = $props();

  let menuOpen = $state(false);

  let room = $derived(catalog.rooms.get(scene.room) ?? [...catalog.rooms.values()][0]!);
  let leadState = $derived<ActorState>(scene.status === "busy" ? "walking" : scene.status === "idle" ? "asleep" : "alert");
  let actors = $derived<SceneActor[]>(
    scene.actors.map((a, i) => ({ id: a.id, character: a.character, state: i === 0 ? leadState : "walking" })),
  );
  let names = $derived(scene.actors.map((a) => catalog.characters.get(a.character)?.name ?? a.character));
  let statusLabel = $derived(`${STATUS_LABELS[scene.status] ?? scene.status} ${humanizeDuration(now - scene.statusSince)}`);
  let tooltip = $derived(
    [
      `${scene.name} — ${room.name}`,
      scene.cwd,
      ...scene.actors.map((a, i) => `${names[i]}: ${a.agentKind ?? "main session"}`),
      scene.focusable ? "Click to jump to this session" : "Hook-only session: cannot jump to it",
    ].join("\n"),
  );

  async function focus() {
    if (!scene.focusable) return;
    try {
      const outcome = await backend.focus(scene.id);
      if (outcome.kind === "notFound") app.notify("Couldn't find the terminal hosting this session (tmux? ssh?).");
    } catch (e) {
      app.notify(`Couldn't focus the session: ${String(e)}`);
    }
  }

  async function copyPath() {
    menuOpen = false;
    await navigator.clipboard.writeText(scene.cwd);
    app.notify("Path copied.");
  }

  async function openFolder() {
    menuOpen = false;
    await backend.openFolder(scene.id).catch((e) => app.notify(String(e)));
  }
</script>

<div
  class="card {scene.status}"
  class:focusable={scene.focusable}
  title={tooltip}
  onclick={focus}
  oncontextmenu={(e) => { e.preventDefault(); menuOpen = !menuOpen; }}
  onmouseleave={() => (menuOpen = false)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && focus()}
>
  <div class="title"><i class="dot"></i><strong>{scene.name}</strong></div>
  <SceneCanvas {catalog} {room} {actors} />
  <div class="cast"><span class="muted">{room.name} —</span> {names.join(" · ")}</div>
  <div class="meta"><span class="cwd">{shortenHome(scene.cwd)}</span><span>{statusLabel}</span></div>

  {#if menuOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="menu" role="menu" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <button onclick={openFolder}>Open folder</button>
      <button onclick={copyPath}>Copy path</button>
    </div>
  {/if}
</div>

<style>
  .card {
    position: relative;
    width: 204px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 5px;
    background: var(--card);
    border: 1px solid var(--card-border);
    border-radius: 8px;
    outline: none;
  }
  .card.focusable { cursor: pointer; }
  .card.focusable:hover { border-color: rgba(255, 255, 255, 0.3); }
  .card.busy { border-color: color-mix(in srgb, var(--busy) 70%, transparent); }
  .card.waiting { border-color: color-mix(in srgb, var(--waiting) 70%, transparent); }
  .title { display: flex; align-items: center; gap: 6px; font-size: 11px; overflow: hidden; }
  .title strong { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--idle); flex-shrink: 0; }
  .busy .dot { background: var(--busy); }
  .waiting .dot { background: var(--waiting); }
  .cast { font-size: 9px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .muted { color: var(--muted); font-weight: 400; }
  .meta { display: flex; justify-content: space-between; gap: 4px; font-size: 9px; color: var(--muted); }
  .cwd { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .menu {
    position: absolute;
    right: 8px;
    top: 24px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px;
    background: #2a2a33;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    z-index: 2;
  }
  .menu button { text-align: left; border: none; font-size: 11px; }
</style>
