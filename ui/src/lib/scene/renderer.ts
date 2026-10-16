// Draws one room and its actors on a canvas, in logical pixels.

import type { Catalog, RoomAsset } from "../assets/catalog";
import { poseAt, type Pose } from "./wander";

export type ActorState = "walking" | "asleep" | "alert";

export interface SceneActor {
  id: string;
  character: string;
  state: ActorState;
}

const EMOTE_SIZE = 5;

export function drawScene(
  ctx: CanvasRenderingContext2D,
  catalog: Catalog,
  room: RoomAsset,
  actors: SceneActor[],
  seconds: number,
): void {
  const { width, height } = catalog.roomSize;
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, width, height);

  const frame = room.frames > 1 ? Math.floor(seconds * room.fps) % room.frames : 0;
  ctx.drawImage(room.image, frame * width, 0, width, height, 0, 0, width, height);

  // Back to front so characters lower on screen overlap the ones behind them.
  actors
    .map((actor) => ({ actor, pose: poseAt(actor.id, room.walkArea, seconds, actor.state === "walking") }))
    .sort((a, b) => a.pose.y - b.pose.y)
    .forEach(({ actor, pose }) => drawActor(ctx, catalog, actor, pose, seconds));
}

function drawActor(ctx: CanvasRenderingContext2D, catalog: Catalog, actor: SceneActor, pose: Pose, seconds: number) {
  const asset = catalog.characters.get(actor.character);
  if (!asset) return;
  const size = catalog.characterSize;
  const left = pose.x - size / 2;
  const top = pose.y - size + 1;

  ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
  ctx.fillRect(left + 2, pose.y, 8, 1);

  if (asset.aura) {
    ctx.globalAlpha = 0.18 + 0.08 * Math.sin(seconds * 3 + pose.x);
    pixelDisc(ctx, pose.x, pose.y - 5, 6, asset.aura);
    ctx.globalAlpha = 1;
  }

  const row = pose.facing === "front" ? 0 : 1;
  ctx.save();
  ctx.globalAlpha = actor.state === "asleep" ? 0.75 : 1;
  if (pose.flip) {
    ctx.translate(left + size, top);
    ctx.scale(-1, 1);
    ctx.drawImage(asset.image, pose.frame * size, row * size, size, size, 0, 0, size, size);
  } else {
    ctx.drawImage(asset.image, pose.frame * size, row * size, size, size, left, top, size, size);
  }
  ctx.restore();

  if (actor.state === "asleep") {
    const rise = Math.floor(seconds * 2) % 4;
    ctx.globalAlpha = 1 - rise * 0.2;
    drawEmote(ctx, catalog, 0, left + 10, top - 3 - rise);
    ctx.globalAlpha = 1;
  } else if (actor.state === "alert" && Math.floor(seconds * 3) % 2 === 0) {
    drawEmote(ctx, catalog, 1, left + 3, top - 7);
  }
}

function drawEmote(ctx: CanvasRenderingContext2D, catalog: Catalog, index: number, x: number, y: number) {
  ctx.drawImage(catalog.emotes, index * EMOTE_SIZE, 0, EMOTE_SIZE, EMOTE_SIZE, x, y, EMOTE_SIZE, EMOTE_SIZE);
}

/** Filled disc made of 1px rows, to stay crisp like the sprites. */
function pixelDisc(ctx: CanvasRenderingContext2D, cx: number, cy: number, radius: number, color: string) {
  ctx.fillStyle = color;
  for (let dy = -radius; dy <= radius; dy++) {
    const half = Math.round(Math.sqrt(radius * radius - dy * dy));
    ctx.fillRect(cx - half, cy + dy, half * 2 + 1, 1);
  }
}
