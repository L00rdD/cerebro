<script lang="ts">
  import type { NetworkDto } from "../lib/api/types";
  import { app } from "../lib/state/app.svelte";

  /** `10.147.17.42:4243`: the link without its scheme and key. */
  const host = (network: NetworkDto) => (network.urls[0] ? new URL(network.urls[0]).host : null);

  /** What the user should know before turning a network on. */
  const HINTS: Record<string, string> = {
    zerotier: "Encrypted, works from anywhere: the phone needs ZeroTier on.",
    local: "Same Wi-Fi only, no VPN needed. Not encrypted: keep it for home.",
  };
</script>

<!-- One line per network: a switch, then its link or why there is none. A stopped network is a
     banner (it has an action). -->
{#if app.mobile}
  <div class="phone">
    {#each app.mobile.networks as network (network.id)}
      {@const link = host(network)}
      <div class="network" title={network.message ?? HINTS[network.id] ?? ""}>
        <label>
          <input
            type="checkbox"
            checked={network.state !== "off"}
            onchange={(e) => app.setNetworkOn(network, e.currentTarget.checked)}
          />
          {network.name}
        </label>
        {#if link}
          <span>{link}</span>
          <div class="actions">
            {#if app.canShare}<button onclick={() => app.sharePhoneLink(network)}>Share</button>{/if}
            <button onclick={() => app.copyPhoneLink(network)}>Copy link</button>
          </div>
        {:else if network.state === "off"}
          <span class="muted">{HINTS[network.id] ?? "off"}</span>
        {:else if network.state === "online"}
          <span class="muted">{network.message ?? `join a private ${network.name} network to use it`}</span>
        {:else if network.state === "offline"}
          <span class="muted">{network.id === "local" ? "not connected" : "offline"}</span>
        {:else if network.state === "notInstalled"}
          <span class="muted">install it to follow sessions from your phone</span>
        {:else if network.state === "stopped"}
          <span class="muted">not running</span>
        {:else}
          <span class="muted">{network.message}</span>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .phone { display: flex; flex-direction: column; gap: 4px; font-size: 11px; }
  .network { display: flex; align-items: center; gap: 8px; min-height: 22px; }
  label { display: flex; align-items: center; gap: 6px; cursor: pointer; flex-shrink: 0; }
  .network span { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .muted { color: var(--muted); }
  .actions { display: flex; gap: 6px; flex-shrink: 0; }
</style>
