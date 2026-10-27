// Deterministic wandering: a pure function of (actor id, time), so nothing is stored
// between frames and every redraw of the same instant gives the same picture.

import type { Rect } from "../assets/catalog";

export type Facing = "front" | "back" | "side";

/** How a character gets from one waypoint to the next. */
export type Movement = "walk" | "fly" | "teleport" | "phase" | "dash";

export type Frame = 0 | 1 | 2 | 3;

/** Walk cycle: step, pass, other step, pass. */
const WALK_CYCLE: Frame[] = [1, 2, 3, 2];

export interface Point {
  x: number;
  y: number;
}

export interface Pose {
  x: number;
  y: number;
  facing: Facing;
  /** Facing left: the side sprite is mirrored, and directional powers aim left. */
  flip: boolean;
  /** 0 = standing, 1 and 3 = steps, 2 = passing between them. */
  frame: Frame;
  /** Height above the ground in pixels: flyers hover while their shadow stays on the floor. */
  lift: number;
  /** Sprite opacity: phasing and teleporting characters fade out. */
  alpha: number;
  /** Set while moving between two waypoints; `progress` goes from 0 to 1. */
  travel?: { from: Point; to: Point; progress: number };
  /** Set while using a power during the pause at a waypoint; `leg` tells casts apart. */
  cast?: { leg: number; progress: number };
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

/**
 * Per movement: `pace` scales the leg duration, `share` is the part of each leg spent
 * moving; the rest is a pause at the waypoint, where the character may use a power.
 */
const MOVEMENTS: Record<Movement, { pace: number; share: number }> = {
  walk: { pace: 1, share: 0.6 },
  phase: { pace: 1, share: 0.6 },
  fly: { pace: 1.1, share: 0.55 },
  teleport: { pace: 1, share: 0.3 },
  dash: { pace: 0.8, share: 0.2 },
};

/** Share of pauses where the character uses its power. */
const CAST_CHANCE = 0.75;

/** Hovering height of flyers, with a slow bob. */
function hover(seed: number, seconds: number): number {
  return 8 + Math.round(Math.sin(seconds * 3 + (seed % 7)) * 2);
}

export function poseAt(actorId: string, area: Rect, seconds: number, walking: boolean, movement: Movement = "walk"): Pose {
  const seed = hash32(actorId);
  const point = (k: number) => ({
    x: area.x + unit(seed, k * 2) * area.width,
    y: area.y + unit(seed, k * 2 + 1) * area.height,
  });

  if (!walking) {
    const home = point(0);
    return { x: Math.floor(home.x), y: Math.floor(home.y), facing: "front", flip: false, frame: 0, lift: 0, alpha: 1 };
  }

  const { pace, share } = MOVEMENTS[movement];
  const legSeconds = (3 + unit(seed, 999) * 2.5) * pace;
  const clock = seconds / legSeconds + unit(seed, 998) * 100;
  const k = Math.floor(clock);
  const progress = clock - k;
  const from = point(k);
  const to = point(k + 1);
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const flying = movement === "fly";

  if (progress >= share) {
    return {
      x: Math.floor(to.x),
      y: Math.floor(to.y),
      facing: "front",
      flip: dx < 0,
      frame: 0,
      lift: flying ? hover(seed, seconds) : 0,
      alpha: 1,
      cast: unit(seed, 5000 + k) < CAST_CHANCE ? { leg: k, progress: (progress - share) / (1 - share) } : undefined,
    };
  }

  const u = progress / share;
  const travel = { from, to, progress: u };

  if (movement === "teleport") {
    // Fade out where it stands, vanish, then fade in at the destination.
    const at = u < 0.5 ? from : to;
    const alpha = u < 0.3 ? 1 - u / 0.3 : u > 0.7 ? (u - 0.7) / 0.3 : 0;
    return { x: Math.floor(at.x), y: Math.floor(at.y), facing: "front", flip: dx < 0, frame: 0, lift: 0, alpha, travel };
  }

  const stepsPerSecond = movement === "dash" ? 16 : 8;
  // The floor is seen at an angle: vertical moves count double when picking a facing.
  const vertical = Math.abs(dy) * 2 > Math.abs(dx);
  return {
    x: Math.floor(from.x + dx * u),
    y: Math.floor(from.y + dy * u),
    facing: vertical ? (dy < 0 ? "back" : "front") : "side",
    flip: dx < 0,
    frame: flying ? 0 : WALK_CYCLE[Math.floor(seconds * stepsPerSecond) % WALK_CYCLE.length]!,
    lift: flying ? hover(seed, seconds) + 3 : 0,
    alpha: movement === "phase" ? 0.45 : 1,
    travel,
  };
}
