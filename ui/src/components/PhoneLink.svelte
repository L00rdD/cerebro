<script lang="ts">
  import { app } from "../lib/state/app.svelte";

  let mobile = $derived(app.mobile);
  /** `10.147.17.42:4243`: the link without its scheme and key. */
  let host = $derived(mobile?.urls[0] ? new URL(mobile.urls[0]).host : null);
</script>

<!-- A stopped network is a banner (it has an action); every other state is one quiet line. -->
{#if mobile && mobile.state !== "stopped"}
  <div class="phone" title={mobile.message ?? ""}>
    {#if host}
      <span>Phone: {host}</span>
      <div class="actions">
        {#if app.canShare}<button onclick={() => app.sharePhoneLink()}>Share</button>{/if}
        <button onclick={() => app.copyPhoneLink()}>Copy link</button>
      </div>
    {:else if mobile.state === "online"}
      <span class="muted">Phone: {mobile.message ?? `join a private ${mobile.network} network to use it`}</span>
    {:else if mobile.state === "offline"}
      <span class="muted">Phone: {mobile.network} is offline</span>
    {:else if mobile.state === "notInstalled"}
      <span class="muted">Phone: install {mobile.network} to follow sessions from it</span>
    {:else}
      <span class="muted">Phone: {mobile.message}</span>
    {/if}
  </div>
{/if}

<style>
  .phone { display: flex; justify-content: space-between; align-items: center; gap: 8px; font-size: 11px; }
  .phone span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .muted { color: var(--muted); }
  .actions { display: flex; gap: 6px; flex-shrink: 0; }
</style>
