<script lang="ts">
  import { app } from "../lib/state/app.svelte";
</script>

{#if app.snapshot?.mode.kind === "hooksFallback"}
  <div class="banner warn">
    Fallback mode: {app.snapshot.mode.reason}. Sessions come from Cerebro's hooks only (no names, no click-to-focus).
  </div>
{/if}

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
