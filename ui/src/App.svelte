<script lang="ts">
  import { onMount } from "svelte";
  import { desktop } from "./lib/api/backend";
  import { app } from "./lib/state/app.svelte";
  import Header from "./components/Header.svelte";
  import Banners from "./components/Banners.svelte";
  import SessionCard from "./components/SessionCard.svelte";
  import EmptyState from "./components/EmptyState.svelte";
  import Footer from "./components/Footer.svelte";
  import PhoneLink from "./components/PhoneLink.svelte";

  let panel: HTMLElement;

  onMount(() => {
    void app.start();
    if (!desktop) return;
    // The native window follows the content height.
    const api = desktop;
    const observer = new ResizeObserver(() => void api.resizePanel(Math.ceil(panel.getBoundingClientRect().height)));
    observer.observe(panel);
    return () => observer.disconnect();
  });
</script>

<main bind:this={panel} class:phone={app.phone}>
  <Header />
  <Banners />

  {#if app.error}
    <p class="error">{app.error}</p>
  {:else if !app.snapshot || !app.catalog}
    <p class="muted">Reaching out with Cerebro…</p>
  {:else if app.snapshot.scenes.length === 0}
    <EmptyState catalog={app.catalog} />
  {:else}
    <section class="grid" class:scroll={!app.phone && app.snapshot.scenes.length > 4}>
      {#each app.snapshot.scenes as scene (scene.id)}
        <SessionCard {scene} catalog={app.catalog} now={app.now} />
      {/each}
    </section>
  {/if}

  {#if app.toast}<div class="toast">{app.toast}</div>{/if}
  {#if !app.phone}
    <PhoneLink />
    <Footer />
  {/if}
</main>

<style>
  main {
    width: 446px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--bg);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(2, 204px);
    gap: 10px;
  }
  .scroll {
    max-height: 560px;
    overflow-y: auto;
  }
  /* Phone: the whole screen, clear of the notch and the home indicator. */
  main.phone {
    width: 100%;
    min-height: 100vh;
    padding: calc(14px + env(safe-area-inset-top)) calc(14px + env(safe-area-inset-right))
      calc(14px + env(safe-area-inset-bottom)) calc(14px + env(safe-area-inset-left));
    border: none;
    border-radius: 0;
  }
  /* minmax(0, …): long names shrink with an ellipsis instead of widening the page. */
  .phone .grid { grid-template-columns: minmax(0, 1fr); gap: 14px; }
  .muted { color: var(--muted); text-align: center; margin: 24px 0; }
  .error { color: #ff6b6b; }
  .toast {
    font-size: 11px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 8px;
  }
</style>
