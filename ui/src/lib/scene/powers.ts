// Character powers: small pixel-art effects an actor plays while it pauses at a waypoint.
// Like wandering, every effect is a pure function of (actor, time).

import { pixelDisc, pixelLine, pixelRing } from "./pixels";
import { unit, type Pose } from "./wander";

export type PowerKind =
  | "beam"
  | "bolt"
  | "psychic"
  | "fire"
  | "frost"
  | "claws"
  | "throw"
  | "sparks"
  | "shield"
  | "magnet"
  | "sonic"
  | "quake"
  | "clone"
  | "leap"
  | "absorb"
  | "glow";

export interface PowerContext {
  ctx: CanvasRenderingContext2D;
  pose: Pose;
  /** Feet position of the caster, after `castOffset`. */
  x: number;
  lift: number;
  seed: number;
  seconds: number;
  color: string;
  /** Canvas size, so beams and projectiles stop at the walls. */
  width: number;
  height: number;
  /** Draws a translucent copy of the caster with its feet at (x, y). */
  ghost: (x: number, y: number, alpha: number) => void;
}

/** The effect plays in the middle of the pause, leaving a breath before and after. */
function effectProgress(cast: number): number | undefined {
  if (cast < 0.1 || cast >= 0.9) return undefined;
  return (cast - 0.1) / 0.8;
}

/** How a power moves its caster: leaps lift it, quakes shake it. */
export function castOffset(kind: PowerKind, pose: Pose, seconds: number): { dx: number; lift: number } {
  const e = pose.cast && effectProgress(pose.cast.progress);
  if (e === undefined) return { dx: 0, lift: 0 };
  if (kind === "leap") return { dx: 0, lift: Math.round(Math.sin(Math.PI * e) * 10) };
  if (kind === "quake") return { dx: Math.floor(seconds * 20) % 2, lift: 0 };
  return { dx: 0, lift: 0 };
}

export function drawPower(kind: PowerKind, p: PowerContext): void {
  const e = p.pose.cast && effectProgress(p.pose.cast.progress);
  if (e === undefined) return;
  const { ctx } = p;
  ctx.save();
  // Fade in and out so effects never pop.
  ctx.globalAlpha = Math.min(1, e * 6, (1 - e) * 6);
  DRAWERS[kind](p, e);
  ctx.restore();
}

const DRAWERS: Record<PowerKind, (p: PowerContext, e: number) => void> = {
  beam,
  bolt,
  psychic,
  fire,
  frost,
  claws,
  throw: projectile,
  sparks,
  shield,
  magnet,
  sonic,
  quake,
  clone,
  leap,
  absorb,
  glow,
};

/** Anchor points on a 12px sprite. */
function body(p: PowerContext) {
  const feet = p.pose.y - p.lift;
  return { dir: p.pose.flip ? -1 : 1, feet, chest: feet - 5, head: feet - 9 };
}

/** Distance from x to the wall in direction dir. */
function toWall(p: PowerContext, x: number, dir: number): number {
  return dir > 0 ? p.width - 1 - x : x;
}

function burst(ctx: CanvasRenderingContext2D, cx: number, cy: number, radius: number, color: string) {
  ctx.fillStyle = color;
  for (let i = 0; i < 8; i++) {
    const angle = (i / 8) * Math.PI * 2;
    ctx.fillRect(Math.round(cx + Math.cos(angle) * radius), Math.round(cy + Math.sin(angle) * radius), 1, 1);
  }
}

/** Optic blast or plasma: a beam from the eyes to the wall. */
function beam(p: PowerContext, e: number) {
  const { ctx } = p;
  const { dir, head } = body(p);
  const x0 = p.x + dir * 3;
  const y = head + 1;
  const length = Math.round(Math.min(1, e * 4) * toWall(p, x0, dir));
  const x1 = x0 + dir * length;
  ctx.fillStyle = p.color;
  ctx.fillRect(Math.min(x0, x1), y, length + 1, 2);
  if (Math.floor(p.seconds * 10) % 2 === 0) {
    ctx.fillStyle = "#FFFFFF";
    ctx.fillRect(Math.min(x0, x1), y, length + 1, 1);
  }
  if (e > 0.25) burst(ctx, x1, y, 1 + (Math.floor(p.seconds * 10) % 3), p.color);
}

/** Lightning: zigzag strikes from the top of the room next to the caster. */
function bolt(p: PowerContext, e: number) {
  const { ctx } = p;
  const { dir, head } = body(p);
  // Eyes glow white the whole time.
  ctx.fillStyle = p.color;
  ctx.fillRect(p.x - 2, head, 4, 1);
  const strike = Math.floor(e * 3);
  const local = e * 3 - strike;
  if (local > 0.6) return;
  const leg = p.pose.cast!.leg;
  const n = leg * 7 + strike;
  const target = Math.round(Math.max(2, Math.min(p.width - 3, p.x + dir * (6 + unit(p.seed, n) * 14))));
  const ground = p.pose.y;
  let x = target + Math.round((unit(p.seed, n + 100) - 0.5) * 10);
  for (let y = 0; y < ground; y += 4) {
    const next = y + 4 >= ground ? target : x + Math.round((unit(p.seed, n * 31 + y) - 0.5) * 6);
    pixelLine(ctx, x, y, next, Math.min(y + 4, ground), p.color);
    x = next;
  }
  burst(ctx, target, ground, 2, "#FFFFFF");
  // A faint flash over the whole room.
  ctx.globalAlpha *= 0.12;
  ctx.fillStyle = "#FFFFFF";
  ctx.fillRect(0, 0, p.width, p.height);
}

/** Telepathy: rings spreading from the head. */
function psychic(p: PowerContext, e: number) {
  const { ctx } = p;
  const { head } = body(p);
  const alpha = ctx.globalAlpha;
  for (let i = 0; i < 3; i++) {
    const r = ((e * 2.5 + i / 3) % 1) * 11;
    ctx.globalAlpha = alpha * (1 - r / 11);
    pixelRing(ctx, p.x, head, Math.max(2, Math.round(r)), p.color, 0.7);
  }
  ctx.globalAlpha = alpha;
  ctx.fillStyle = p.color;
  ctx.fillRect(p.x - 1, head - 1, 2, 1);
}

/** Flames rising around the body. */
function fire(p: PowerContext, e: number) {
  const { ctx } = p;
  const { feet } = body(p);
  const alpha = ctx.globalAlpha;
  for (let i = 0; i < 14; i++) {
    const life = (e * 4 + unit(p.seed, i)) % 1;
    const x = Math.round(p.x + (unit(p.seed, i + 50) - 0.5) * 12 + Math.sin(life * 6 + i) * 1.5);
    const y = Math.round(feet - life * 16);
    ctx.globalAlpha = alpha * (1 - life);
    ctx.fillStyle = life < 0.3 ? "#FFE27A" : p.color;
    const size = life < 0.4 ? 2 : 1;
    ctx.fillRect(x, y, size, size);
  }
}

/** Ice crystals spreading on the floor and a few snowflakes. */
function frost(p: PowerContext, e: number) {
  const { ctx } = p;
  const ground = p.pose.y;
  const spread = Math.min(1, e * 2) * 13;
  for (let i = 0; i < 10; i++) {
    const angle = (i / 10) * Math.PI * 2 + unit(p.seed, i) * 0.4;
    const cx = Math.round(p.x + Math.cos(angle) * spread);
    const cy = Math.round(ground + Math.sin(angle) * spread * 0.35);
    ctx.fillStyle = p.color;
    ctx.fillRect(cx - 1, cy, 3, 1);
    ctx.fillRect(cx, cy - 1, 1, 3);
    ctx.fillStyle = "#FFFFFF";
    ctx.fillRect(cx, cy, 1, 1);
  }
  ctx.fillStyle = "#FFFFFF";
  for (let i = 0; i < 6; i++) {
    const fall = (e * 2 + unit(p.seed, i + 20)) % 1;
    ctx.fillRect(Math.round(p.x + (unit(p.seed, i + 30) - 0.5) * 20), Math.round(ground - 18 + fall * 18), 1, 1);
  }
}

/** Three slashes in front of the caster, twice. */
function claws(p: PowerContext, e: number) {
  const { ctx } = p;
  const { dir, chest } = body(p);
  const swipe = (e * 2) % 1;
  const reach = Math.round(Math.min(1, swipe * 3) * 6);
  const alpha = ctx.globalAlpha;
  ctx.globalAlpha = alpha * (1 - swipe);
  for (let i = 0; i < 3; i++) {
    const x0 = p.x + dir * (4 + i * 2);
    const y0 = chest - 4;
    pixelLine(ctx, x0, y0, x0 + dir * reach, y0 + reach, p.color);
  }
  ctx.globalAlpha = alpha;
}

/** A glowing projectile thrown forward, exploding at the end of its flight. */
function projectile(p: PowerContext, e: number) {
  const { ctx } = p;
  const { dir, chest } = body(p);
  const x0 = p.x + dir * 4;
  const range = Math.min(34, toWall(p, x0, dir));
  const flight = Math.min(1, e / 0.55);
  const x = Math.round(x0 + dir * range * flight);
  const y = Math.round(chest - Math.sin(Math.PI * flight) * 4);
  if (e < 0.55) {
    ctx.fillStyle = p.color;
    ctx.fillRect(x, y, 2, 2);
    ctx.fillStyle = "#FFFFFF";
    ctx.fillRect(x - dir * 2, y, 1, 1);
  } else {
    const blast = (e - 0.55) / 0.45;
    burst(ctx, x, y, 1 + Math.round(blast * 5), p.color);
    burst(ctx, x, y, Math.round(blast * 3), "#FFFFFF");
  }
}

/** Fireworks bursting above the caster. */
function sparks(p: PowerContext, e: number) {
  const { ctx } = p;
  const { head } = body(p);
  const leg = p.pose.cast!.leg;
  for (let b = 0; b < 3; b++) {
    const local = e * 1.6 - b * 0.3;
    if (local < 0 || local >= 1) continue;
    const cx = Math.round(p.x + (unit(p.seed, leg * 3 + b) - 0.5) * 18);
    const cy = Math.round(head - 4 - unit(p.seed, leg * 3 + b + 40) * 8);
    burst(ctx, cx, cy, 1 + Math.round(local * 5), b % 2 === 0 ? p.color : "#FFFFFF");
  }
}

/** A force field around the body. */
function shield(p: PowerContext, e: number) {
  const { ctx } = p;
  const { chest } = body(p);
  const radius = Math.round(Math.min(1, e * 5) * 8);
  const alpha = ctx.globalAlpha;
  ctx.globalAlpha = alpha * 0.25;
  pixelDisc(ctx, p.x, chest, radius, p.color);
  ctx.globalAlpha = alpha * (0.7 + 0.3 * Math.sin(p.seconds * 8));
  pixelRing(ctx, p.x, chest, radius, p.color);
  ctx.globalAlpha = alpha;
}

/** Debris orbiting the caster, held by a magnetic or telekinetic field. */
function magnet(p: PowerContext, e: number) {
  const { ctx } = p;
  const { chest } = body(p);
  const radius = 4 + Math.min(1, e * 4) * 6;
  pixelRing(ctx, p.x, chest, Math.round(radius), p.color, 0.4);
  for (let i = 0; i < 4; i++) {
    const angle = p.seconds * 4 + (i / 4) * Math.PI * 2;
    const x = Math.round(p.x + Math.cos(angle) * radius);
    const y = Math.round(chest + Math.sin(angle) * radius * 0.4 - 3);
    ctx.fillStyle = "#9AA0A6";
    ctx.fillRect(x, y, 2, 2);
    ctx.fillStyle = p.color;
    ctx.fillRect(x, y, 1, 1);
  }
}

/** Sonic scream: arcs travelling forward. */
function sonic(p: PowerContext, e: number) {
  const { ctx } = p;
  const { dir, head } = body(p);
  const alpha = ctx.globalAlpha;
  const range = Math.min(24, toWall(p, p.x, dir));
  for (let i = 0; i < 3; i++) {
    const d = ((e * 2 + i / 3) % 1) * range;
    const half = 1 + Math.round(d / 5);
    ctx.globalAlpha = alpha * (1 - d / Math.max(range, 1));
    ctx.fillStyle = p.color;
    for (let dy = -half; dy <= half; dy++) {
      const bend = Math.round((dy * dy) / (half * 1.5));
      ctx.fillRect(Math.round(p.x + dir * (4 + d - bend)), head + 2 + dy, 1, 1);
    }
  }
  ctx.globalAlpha = alpha;
}

/** Cracks running along the floor, with dust. */
function quake(p: PowerContext, e: number) {
  const { ctx } = p;
  const ground = p.pose.y + 1;
  const length = Math.round(Math.min(1, e * 3) * 14);
  for (const dir of [-1, 1]) {
    let y = ground;
    for (let s = 0; s < length; s++) {
      if (unit(p.seed, s * 2 + (dir > 0 ? 1 : 0)) < 0.3) y += unit(p.seed, s + 77) < 0.5 ? -1 : 1;
      ctx.fillStyle = "#A89878";
      ctx.fillRect(p.x + dir * (3 + s), y, 1, 1);
    }
  }
  ctx.fillStyle = "#C8B89A";
  for (let i = 0; i < 6; i++) {
    const hop = (e * 3 + unit(p.seed, i)) % 1;
    const x = Math.round(p.x + (unit(p.seed, i + 10) - 0.5) * 16);
    ctx.fillRect(x, Math.round(ground - Math.sin(Math.PI * hop) * 4), 1, 1);
  }
}

/** Duplicates popping up around the caster. */
function clone(p: PowerContext, e: number) {
  const spots = [-11, 11, -6, 6];
  const shown = Math.min(spots.length, Math.floor(e * 6) + 1);
  for (let i = 0; i < shown; i++) {
    const x = Math.round(Math.max(6, Math.min(p.width - 6, p.x + spots[i]!)));
    const y = p.pose.y + (i < 2 ? 0 : -4);
    p.ghost(x, y, 0.85);
    if (e > 0.85) burst(p.ctx, x, y - 5, Math.round((e - 0.85) * 30), "#FFFFFF");
  }
}

/** Dust at take-off and landing; the jump itself is `castOffset`. */
function leap(p: PowerContext, e: number) {
  const ground = p.pose.y;
  if (e < 0.2) burst(p.ctx, p.x, ground, 2 + Math.round(e * 15), "#C8B89A");
  else if (e > 0.8) burst(p.ctx, p.x, ground, 2 + Math.round((e - 0.8) * 15), "#C8B89A");
}

/** Energy pulled in towards the caster. */
function absorb(p: PowerContext, e: number) {
  const { ctx } = p;
  const { chest } = body(p);
  const pull = (e * 2) % 1;
  const radius = (1 - pull) * 13;
  ctx.fillStyle = p.color;
  for (let i = 0; i < 8; i++) {
    const angle = (i / 8) * Math.PI * 2 + e * 3;
    ctx.fillRect(Math.round(p.x + Math.cos(angle) * radius), Math.round(chest + Math.sin(angle) * radius * 0.7), 1, 1);
  }
  if (pull > 0.8) pixelRing(ctx, p.x, chest, 2, "#FFFFFF");
}

/** A pulse of light from the body, for everyone else. */
function glow(p: PowerContext, e: number) {
  const { ctx } = p;
  const { chest } = body(p);
  const alpha = ctx.globalAlpha;
  ctx.globalAlpha = alpha * (0.3 + 0.2 * Math.sin(p.seconds * 6));
  pixelDisc(ctx, p.x, chest, 5 + Math.round(Math.sin(Math.PI * e) * 3), p.color);
  ctx.globalAlpha = alpha;
  const leg = p.pose.cast!.leg;
  ctx.fillStyle = "#FFFFFF";
  for (let i = 0; i < 4; i++) {
    const twinkle = (e * 3 + unit(p.seed, leg + i)) % 1;
    if (twinkle > 0.5) continue;
    ctx.fillRect(Math.round(p.x + (unit(p.seed, leg * 5 + i) - 0.5) * 16), Math.round(chest - 6 + unit(p.seed, i + 60) * 10), 1, 1);
  }
}
