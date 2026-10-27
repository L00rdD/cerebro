<script lang="ts">
  import { onMount } from "svelte";
  import type { Catalog, RoomAsset } from "../lib/assets/catalog";
  import { drawScene, type SceneActor } from "../lib/scene/renderer";
  import { onFrame } from "../lib/scene/ticker";

  let {
    catalog,
    room,
    actors,
    scale = 1,
    fill = false,
  }: {
    catalog: Catalog;
    room: RoomAsset;
    actors: SceneActor[];
    scale?: number;
    /** Stretch to the parent's width (phone) instead of `scale` pixels per room pixel. */
    fill?: boolean;
  } = $props();

  let canvas: HTMLCanvasElement;
  let width = $derived(catalog.roomSize.width);
  let height = $derived(catalog.roomSize.height);

  onMount(() => {
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * scale * dpr;
    canvas.height = height * scale * dpr;
    const ctx = canvas.getContext("2d")!;
    ctx.setTransform(scale * dpr, 0, 0, scale * dpr, 0, 0);
    // Reads the latest props on every frame, so updates need no re-subscription.
    return onFrame((seconds) => drawScene(ctx, catalog, room, actors, seconds));
  });
</script>

<canvas
  bind:this={canvas}
  style:width={fill ? "100%" : `${width * scale}px`}
  style:height={fill ? "auto" : `${height * scale}px`}
></canvas>

<style>
  canvas { display: block; border-radius: 5px; image-rendering: pixelated; }
</style>
