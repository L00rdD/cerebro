import { describe, expect, it } from "vitest";
import { castOffset, drawPower, type PowerContext, type PowerKind } from "./powers";
import { hash32, poseAt, type Pose } from "./wander";

const area = { x: 10, y: 20, width: 70, height: 30 };

describe("powers", () => {
  it("lifts leapers only while they cast", () => {
    // Arrange
    let leaps = 0;

    // Act & Assert
    for (let t = 0; t < 200; t += 0.1) {
      const pose = poseAt("agent:f", area, t, true);
      const { lift } = castOffset("leap", pose, t);
      if (!pose.cast) expect(lift).toBe(0);
      expect(lift).toBeGreaterThanOrEqual(0);
      expect(lift).toBeLessThanOrEqual(18);
      if (lift > 0) leaps++;
    }

    // Assert
    expect(leaps).toBeGreaterThan(0);
  });

  it("leaves other powers in place", () => {
    // Act & Assert
    for (let t = 0; t < 50; t += 0.1) expect(castOffset("beam", poseAt("agent:g", area, t, true), t)).toEqual({ dx: 0, lift: 0 });
  });

  it("shakes quakers sideways while they cast", () => {
    // Arrange
    const pose: Pose = { ...poseAt("agent:q", area, 0, false), cast: { leg: 1, progress: 0.5 } };

    // Act & Assert
    expect(castOffset("quake", pose, 0)).toEqual({ dx: 0, lift: 0 });
    expect(castOffset("quake", pose, 0.05)).toEqual({ dx: 1, lift: 0 });
    expect(castOffset("quake", { ...pose, cast: { leg: 1, progress: 0.95 } }, 0.05)).toEqual({ dx: 0, lift: 0 });
  });

  it("draws nothing outside the middle of a cast", () => {
    // Arrange
    const { p, rects } = context("sparks", 0.05);

    // Act
    drawPower("sparks", p);
    drawPower("sparks", { ...p, pose: { ...p.pose, cast: undefined } });

    // Assert
    expect(rects).toHaveLength(0);
  });

  it("cracks the floor on both sides of a quake, with dust", () => {
    // Arrange
    const { p, rects, styles } = context("quake", 0.6);

    // Act
    drawPower("quake", p);

    // Assert
    const cracks = rects.slice(0, 52);
    expect(cracks.filter(([x]) => x < p.x)).toHaveLength(26);
    expect(cracks.filter(([x]) => x > p.x)).toHaveLength(26);
    for (const [, y] of cracks) expect(Math.abs(y - (p.pose.y + 1))).toBeLessThanOrEqual(26);
    expect(rects.slice(52)).toHaveLength(10);
    expect(styles).toEqual(expect.arrayContaining(["#A89878", "#C8B89A"]));
  });

  it("bursts fireworks one after the other above the caster", () => {
    // Arrange
    const shots = (cast: number) => {
      const { p, rects, styles } = context("sparks", cast);
      drawPower("sparks", p);
      return { rects, styles, head: p.pose.y - 24 };
    };

    // Act
    const first = shots(0.2);
    const all = shots(0.45);
    const last = shots(0.85);

    // Assert
    expect(first.styles).toEqual(["#ABCDEF", "#FFFFFF"]);
    expect(all.styles).toEqual(["#ABCDEF", "#FFFFFF", "#FFFFFF", "#ABCDEF", "#ABCDEF", "#FFFFFF"]);
    expect(last.styles).toEqual(["#ABCDEF", "#FFFFFF"]);
    for (const { rects, head } of [first, all, last]) for (const [, y] of rects) expect(y).toBeLessThan(head);
  });
});

/** A power about to be drawn at `cast` progress, on a context recording its rects and colors. */
function context(kind: PowerKind, cast: number) {
  const rects: [number, number][] = [];
  const styles: string[] = [];
  const ctx = {
    save() {},
    restore() {},
    globalAlpha: 1,
    set fillStyle(style: string) {
      styles.push(style);
    },
    fillRect(x: number, y: number) {
      rects.push([x, y]);
    },
  } as unknown as CanvasRenderingContext2D;
  const pose: Pose = { x: 150, y: 100, facing: "front", flip: false, frame: 0, lift: 0, alpha: 1, cast: { leg: 3, progress: cast } };
  const p: PowerContext = {
    ctx,
    pose,
    x: pose.x,
    lift: 0,
    seed: hash32(`agent:${kind}`),
    seconds: 10,
    scale: 1.5,
    color: "#ABCDEF",
    width: 320,
    height: 180,
    ghost: () => {},
  };
  return { p, rects, styles };
}
