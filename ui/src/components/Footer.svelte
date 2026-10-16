<script lang="ts">
  import { backend } from "../lib/api/backend";
  import { app } from "../lib/state/app.svelte";
</script>

<footer>
  <label>
    <input type="checkbox" checked={app.autostart} onchange={(e) => app.setAutostart(e.currentTarget.checked)} />
    Launch at login
  </label>
  <button onclick={() => backend.quit()}>Quit</button>
</footer>

<style>
  footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 11px;
  }
  label { display: flex; align-items: center; gap: 6px; cursor: pointer; }
</style>
