<script lang="ts">
  import { app } from "../lib/state/app.svelte";
</script>

{#each app.snapshot?.warnings ?? [] as warning (warning.tool)}
  <div class="banner warn">{warning.message}</div>
{/each}

{#if app.phone && !app.connected && !app.error}
  <div class="banner warn">Lost the link to the Mac. Retrying… Is it awake, and are you on its network?</div>
{/if}

{#each app.mobile?.networks.filter((network) => network.state === "stopped") ?? [] as network (network.id)}
  <div class="banner">
    <span>{network.name} is not running: your phone can't reach Cerebro through it.</span>
    <button onclick={() => app.startNetwork(network)}>Start</button>
  </div>
{/each}

{#if app.hooks === "missing" || app.hooks === "outdated"}
  <div class="banner">
    <span>
      {app.hooks === "missing"
        ? "Subagent tracking is off: Cerebro's hooks are not installed."
        : "Cerebro's hooks are outdated or point to another copy of the app."}
    </span>
    <button onclick={() => app.installHooks()}>{app.hooks === "missing" ? "Install" : "Repair"}</button>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    padding: 8px;
    border-radius: 8px;
    background: rgba(111, 183, 255, 0.12);
    border: 1px solid rgba(111, 183, 255, 0.3);
  }
  .warn { background: rgba(255, 210, 63, 0.1); border-color: rgba(255, 210, 63, 0.35); }
  button { flex-shrink: 0; }
</style>
