import { describe, expect, it } from "vitest";
import { hash32, poseAt, unit } from "./wander";

const area = { x: 10, y: 20, width: 70, height: 30 };

describe("wander", () => {
  it("is deterministic", () => {
    // Act & Assert
    expect(poseAt("agent:a", area, 12.3, true)).toEqual(poseAt("agent:a", area, 12.3, true));
    expect(hash32("x")).toBe(hash32("x"));
  });

  it("stays inside the walk area", () => {
    // Act & Assert
    for (let t = 0; t < 200; t += 0.37) {
      const pose = poseAt("agent:b", area, t, true);
      expect(pose.x).toBeGreaterThanOrEqual(area.x);
      expect(pose.x).toBeLessThanOrEqual(area.x + area.width);
      expect(pose.y).toBeGreaterThanOrEqual(area.y);
      expect(pose.y).toBeLessThanOrEqual(area.y + area.height);
    }
  });

  it("stands still when not walking", () => {
    // Act
    const a = poseAt("lead:s", area, 1, false);
    const b = poseAt("lead:s", area, 99, false);

    // Assert
    expect(a).toEqual(b);
    expect(a.frame).toBe(0);
  });

  it("casts powers only while pausing", () => {
    // Arrange
    let casts = 0;

    // Act & Assert
    for (let t = 0; t < 200; t += 0.1) {
      const pose = poseAt("agent:c", area, t, true);
      if (pose.cast) {
        casts++;
        expect(pose.travel).toBeUndefined();
        expect(pose.cast.progress).toBeGreaterThanOrEqual(0);
        expect(pose.cast.progress).toBeLessThan(1);
      }
    }

    // Assert
    expect(casts).toBeGreaterThan(0);

    // Act & Assert
    expect(poseAt("agent:c", area, 12, false).cast).toBeUndefined();
  });

  it("keeps flyers in the air and lands them when asleep", () => {
    // Act & Assert
    for (let t = 0; t < 50; t += 0.37) expect(poseAt("agent:d", area, t, true, "fly").lift).toBeGreaterThan(0);
    expect(poseAt("agent:d", area, 3, false, "fly").lift).toBe(0);
  });

  it("teleports without crossing the room", () => {
    // Act & Assert
    for (let t = 0; t < 200; t += 0.13) {
      const pose = poseAt("agent:e", area, t, true, "teleport");
      if (!pose.travel) continue;
      const { from, to } = pose.travel;
      const onEnd = [from, to].some((p) => Math.floor(p.x) === pose.x && Math.floor(p.y) === pose.y);
      expect(onEnd).toBe(true);
      if (pose.travel.progress > 0.3 && pose.travel.progress < 0.7) expect(pose.alpha).toBe(0);
    }
  });

  it("spreads unit values over [0, 1)", () => {
    // Act
    const values = Array.from({ length: 1000 }, (_, i) => unit(42, i));

    // Assert
    expect(Math.min(...values)).toBeGreaterThanOrEqual(0);
    expect(Math.max(...values)).toBeLessThan(1);
    expect(values.filter((v) => v < 0.5).length).toBeGreaterThan(400);
  });
});
