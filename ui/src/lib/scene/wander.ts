// Deterministic wandering: a pure function of (actor id, time), so nothing is stored
// between frames and every redraw of the same instant gives the same picture.

import type { Rect } from "../assets/catalog";

export type Facing = "front" | "back";

export interface Pose {
  x: number;
  y: number;
  facing: Facing;
  /** Mirror the sprite (walking left). */
  flip: boolean;
  /** 0 = standing, 1 and 2 = walking steps. */
  frame: 0 | 1 | 2;
}

/** 32-bit FNV-1a with a murmur3 finalizer. */
export function hash32(text: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

/** Number in [0, 1) derived from (seed, n). */
export function unit(seed: number, n: number): number {
  let t = (seed + Math.imul(n, 0x9e3779b9)) >>> 0;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
}

/** Share of each leg spent walking; the rest is a pause at the waypoint. */
const WALK_SHARE = 0.6;

export function poseAt(actorId: string, area: Rect, seconds: number, walking: boolean): Pose {
  const seed = hash32(actorId);
  const point = (k: number) => ({
    x: area.x + unit(seed, k * 2) * area.width,
    y: area.y + unit(seed, k * 2 + 1) * area.height,
  });

  if (!walking) {
    const home = point(0);
    return { x: Math.floor(home.x), y: Math.floor(home.y), facing: "front", flip: false, frame: 0 };
  }

  const legSeconds = 3 + unit(seed, 999) * 2.5;
  const clock = seconds / legSeconds + unit(seed, 998) * 100;
  const k = Math.floor(clock);
  const progress = clock - k;
  const from = point(k);
  const to = point(k + 1);
  const moving = progress < WALK_SHARE;
  const u = moving ? progress / WALK_SHARE : 1;
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  return {
    x: Math.floor(from.x + dx * u),
    y: Math.floor(from.y + dy * u),
    facing: moving && dy < 0 && Math.abs(dy) > Math.abs(dx) ? "back" : "front",
    flip: moving && dx < 0,
    frame: moving ? ((1 + (Math.floor(seconds * 6) % 2)) as 1 | 2) : 0,
  };
}
