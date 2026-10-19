// Draws one room and its actors on a canvas, in logical pixels.

import type { Catalog, CharacterAsset, RoomAsset } from "../assets/catalog";
import { pixelDisc } from "./pixels";
import { castOffset, drawPower } from "./powers";
import { hash32, poseAt, type Pose } from "./wander";

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
    .flatMap((actor) => {
      const asset = catalog.characters.get(actor.character);
      if (!asset) return [];
      const pose = poseAt(actor.id, room.walkArea, seconds, actor.state === "walking", asset.move);
      return [{ actor, asset, pose }];
    })
    .sort((a, b) => a.pose.y - b.pose.y)
    .forEach(({ actor, asset, pose }) => drawActor(ctx, catalog, actor, asset, pose, seconds));
}

function drawActor(
  ctx: CanvasRenderingContext2D,
  catalog: Catalog,
  actor: SceneActor,
  asset: CharacterAsset,
  pose: Pose,
  seconds: number,
) {
  const size = catalog.characterSize;
  const offset = castOffset(asset.power, pose, seconds);
  const x = pose.x + offset.dx;
  const lift = pose.lift + offset.lift;
  const left = x - size / 2;
  const top = pose.y - lift - size + 1;
  const sprite = (feetX: number, feetY: number, alpha: number) =>
    drawSprite(ctx, asset, size, pose, feetX, feetY, alpha);

  // The shadow stays on the floor and shrinks as the character rises.
  ctx.globalAlpha = pose.alpha;
  ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
  const shadow = lift > 0 ? 6 : 8;
  ctx.fillRect(x - shadow / 2, pose.y, shadow, 1);

  if (asset.aura) {
    ctx.globalAlpha = pose.alpha * (0.18 + 0.08 * Math.sin(seconds * 3 + pose.x));
    pixelDisc(ctx, x, pose.y - lift - 5, 6, asset.aura);
  }
  ctx.globalAlpha = 1;

  if (pose.travel && asset.move === "dash") drawAfterimages(pose, sprite);

  sprite(x, pose.y - lift, pose.alpha * (actor.state === "asleep" ? 0.75 : 1));

  if (pose.travel && asset.move === "teleport") drawTeleportSmoke(ctx, pose, asset.powerColor);

  drawPower(asset.power, {
    ctx,
    pose,
    x,
    lift,
    seed: hash32(actor.id),
    seconds,
    color: asset.powerColor,
    width: catalog.roomSize.width,
    height: catalog.roomSize.height,
    ghost: (gx, gy, alpha) => sprite(gx, gy, alpha),
  });

  if (actor.state === "asleep") {
    const rise = Math.floor(seconds * 2) % 4;
    ctx.globalAlpha = 1 - rise * 0.2;
    drawEmote(ctx, catalog, 0, left + 10, top - 3 - rise);
    ctx.globalAlpha = 1;
  } else if (actor.state === "alert" && Math.floor(seconds * 3) % 2 === 0) {
    drawEmote(ctx, catalog, 1, left + 3, top - 7);
  }
}

/** Draws the character's current frame with its feet at (x, y). */
function drawSprite(
  ctx: CanvasRenderingContext2D,
  asset: CharacterAsset,
  size: number,
  pose: Pose,
  x: number,
  y: number,
  alpha: number,
) {
  if (alpha <= 0) return;
  const left = x - size / 2;
  const top = y - size + 1;
  const row = pose.facing === "front" ? 0 : 1;
  ctx.save();
  ctx.globalAlpha = alpha;
  if (pose.flip) {
    ctx.translate(left + size, top);
    ctx.scale(-1, 1);
    ctx.drawImage(asset.image, pose.frame * size, row * size, size, size, 0, 0, size, size);
  } else {
    ctx.drawImage(asset.image, pose.frame * size, row * size, size, size, left, top, size, size);
  }
  ctx.restore();
}

/** Super speed: fading copies left behind along the path. */
function drawAfterimages(pose: Pose, sprite: (x: number, y: number, alpha: number) => void) {
  const { from, to, progress } = pose.travel!;
  for (let i = 3; i >= 1; i--) {
    const u = progress - i * 0.08;
    if (u <= 0) continue;
    sprite(Math.floor(from.x + (to.x - from.x) * u), Math.floor(from.y + (to.y - from.y) * u), 0.4 / i);
  }
}

/** Teleport: a puff of smoke where the character leaves, then where it lands. */
function drawTeleportSmoke(ctx: CanvasRenderingContext2D, pose: Pose, color: string) {
  const { from, to, progress } = pose.travel!;
  const at = progress < 0.5 ? from : to;
  const t = progress < 0.5 ? progress / 0.5 : (progress - 0.5) / 0.5;
  // Leaving: the puff grows as the character fades; landing: it clears as the character appears.
  const spread = progress < 0.5 ? t : 1 - t;
  const cx = Math.floor(at.x);
  const cy = Math.floor(at.y) - 5;
  ctx.save();
  ctx.globalAlpha = 0.9 * (progress < 0.5 ? 1 - t * 0.6 : 0.4 + (1 - t) * 0.6);
  for (let i = 0; i < 10; i++) {
    const angle = (i / 10) * Math.PI * 2 + spread;
    const r = 2 + spread * 6;
    ctx.fillStyle = i % 3 === 0 ? "#FFFFFF" : color;
    ctx.fillRect(Math.round(cx + Math.cos(angle) * r), Math.round(cy + Math.sin(angle) * r * 0.8), 2, 2);
  }
  ctx.restore();
}

function drawEmote(ctx: CanvasRenderingContext2D, catalog: Catalog, index: number, x: number, y: number) {
  ctx.drawImage(catalog.emotes, index * EMOTE_SIZE, 0, EMOTE_SIZE, EMOTE_SIZE, x, y, EMOTE_SIZE, EMOTE_SIZE);
}
