import { describe, expect, it } from "vitest";
import { castOffset } from "./powers";
import { poseAt } from "./wander";

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
      expect(lift).toBeLessThanOrEqual(10);
      if (lift > 0) leaps++;
    }

    // Assert
    expect(leaps).toBeGreaterThan(0);
  });

  it("leaves other powers in place", () => {
    // Act & Assert
    for (let t = 0; t < 50; t += 0.1) expect(castOffset("beam", poseAt("agent:g", area, t, true), t)).toEqual({ dx: 0, lift: 0 });
  });
});
