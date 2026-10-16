import { describe, expect, it } from "vitest";
import { hash32, poseAt, unit } from "./wander";

const area = { x: 10, y: 20, width: 70, height: 30 };

describe("wander", () => {
  it("is deterministic", () => {
    expect(poseAt("agent:a", area, 12.3, true)).toEqual(poseAt("agent:a", area, 12.3, true));
    expect(hash32("x")).toBe(hash32("x"));
  });

  it("stays inside the walk area", () => {
    for (let t = 0; t < 200; t += 0.37) {
      const pose = poseAt("agent:b", area, t, true);
      expect(pose.x).toBeGreaterThanOrEqual(area.x);
      expect(pose.x).toBeLessThanOrEqual(area.x + area.width);
      expect(pose.y).toBeGreaterThanOrEqual(area.y);
      expect(pose.y).toBeLessThanOrEqual(area.y + area.height);
    }
  });

  it("stands still when not walking", () => {
    const a = poseAt("lead:s", area, 1, false);
    const b = poseAt("lead:s", area, 99, false);
    expect(a).toEqual(b);
    expect(a.frame).toBe(0);
  });

  it("spreads unit values over [0, 1)", () => {
    const values = Array.from({ length: 1000 }, (_, i) => unit(42, i));
    expect(Math.min(...values)).toBeGreaterThanOrEqual(0);
    expect(Math.max(...values)).toBeLessThan(1);
    expect(values.filter((v) => v < 0.5).length).toBeGreaterThan(400);
  });
});
