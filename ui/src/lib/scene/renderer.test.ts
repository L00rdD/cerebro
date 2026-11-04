import { describe, expect, it } from "vitest";
import type { Catalog, CharacterAsset, RoomAsset } from "../assets/catalog";
import { drawScene, type SceneActor } from "./renderer";
import { poseAt, type Movement, type Pose } from "./wander";

type Call = [string, ...unknown[]];

/** A canvas context that only records what is asked of it. */
function recordingContext() {
  const calls: Call[] = [];
  const ctx = new Proxy({} as Record<string, unknown>, {
    get: (target, prop: string) => (prop in target ? target[prop] : (...args: unknown[]) => calls.push([prop, ...args])),
    set: (target, prop: string, value) => {
      target[prop] = value;
      calls.push(["=" + prop, value]);
      return true;
    },
  });
  return { ctx: ctx as unknown as CanvasRenderingContext2D, calls };
}

const image = (name: string) => ({ name }) as unknown as HTMLImageElement;

function character(id: string, extra: Partial<CharacterAsset> = {}): CharacterAsset {
  return { id, name: id, move: "walk", power: "glow", powerColor: "#00FF00", image: image(id), ...extra };
}

const room: RoomAsset = { id: "office", name: "Office", frames: 4, fps: 2, walkArea: { x: 20, y: 60, width: 200, height: 60 }, image: image("office") };

function catalogOf(...characters: CharacterAsset[]): Catalog {
  return {
    characterSize: { width: 24, height: 24 },
    roomSize: { width: 320, height: 180 },
    characters: new Map(characters.map((c) => [c.id, c])),
    rooms: new Map([[room.id, room]]),
    emotes: image("emotes"),
  };
}

/** First instant, in tenths of a second, when the actor's pose matches. */
function when(id: string, movement: Movement, test: (pose: Pose) => boolean): number {
  for (let t = 0; t < 600; t += 0.1) if (test(poseAt(id, room.walkArea, t, true, movement))) return t;
  throw new Error(`no matching pose for ${id}`);
}

const drawn = (calls: Call[], what: string) => calls.filter(([name, img]) => name === "drawImage" && (img as { name: string }).name === what);

describe("renderer", () => {
  it("draws the room's current frame, then the characters back to front", () => {
    // Arrange
    const { ctx, calls } = recordingContext();
    const actors: SceneActor[] = ["a", "b", "c"].map((id) => ({ id, character: "logan", state: "walking" }));

    // Act
    drawScene(ctx, catalogOf(character("logan")), room, actors, 1.6);

    // Assert
    expect(calls[0]).toEqual(["=imageSmoothingEnabled", false]);
    expect(calls[1]).toEqual(["clearRect", 0, 0, 320, 180]);
    expect(calls[2]).toEqual(["drawImage", room.image, 3 * 320, 0, 320, 180, 0, 0, 320, 180]);
    const feet = drawn(calls, "logan").map((call) => (call[6] as number) + 36 - 1);
    expect(feet).toHaveLength(3);
    expect([...feet].sort((a, b) => a - b)).toEqual(feet);
  });

  it("skips characters missing from the catalog", () => {
    // Arrange
    const { ctx, calls } = recordingContext();

    // Act
    drawScene(ctx, catalogOf(), { ...room, frames: 1 }, [{ id: "a", character: "nobody", state: "walking" }], 0);

    // Assert
    expect(calls).toHaveLength(3);
    expect(calls[2]).toEqual(["drawImage", room.image, 0, 0, 320, 180, 0, 0, 320, 180]);
  });

  it("dims sleeping characters and floats a snore above them", () => {
    // Arrange
    const { ctx, calls } = recordingContext();

    // Act
    drawScene(ctx, catalogOf(character("logan")), room, [{ id: "a", character: "logan", state: "asleep" }], 1);

    // Assert
    const home = poseAt("a", room.walkArea, 1, false);
    const snore = drawn(calls, "emotes");
    expect(snore).toEqual([["drawImage", image("emotes"), 0, 0, 9, 9, home.x + 3, home.y - 36 + 1 - 2 - 2, 9, 9]]);
    const sprite = calls.indexOf(drawn(calls, "logan")[0]!);
    expect(calls[sprite - 1]).toEqual(["=globalAlpha", 0.75]);
  });

  it("blinks an exclamation mark over alert characters", () => {
    // Arrange
    const alert = (seconds: number) => {
      const { ctx, calls } = recordingContext();
      drawScene(ctx, catalogOf(character("logan")), room, [{ id: "a", character: "logan", state: "alert" }], seconds);
      return drawn(calls, "emotes");
    };

    // Act & Assert
    expect(alert(0)).toHaveLength(1);
    expect(alert(0)[0]![2]).toBe(9);
    expect(alert(0.4)).toHaveLength(0);
  });

  it("draws an aura under characters that have one", () => {
    // Arrange
    const { ctx, calls } = recordingContext();

    // Act
    drawScene(ctx, catalogOf(character("storm", { aura: "#FFEEAA" })), room, [{ id: "a", character: "storm", state: "asleep" }], 0);

    // Assert
    expect(calls).toContainEqual(["=fillStyle", "#FFEEAA"]);
  });

  it("mirrors the side view of characters walking left", () => {
    // Arrange
    const t = when("a", "walk", (pose) => pose.facing === "side" && pose.flip && !!pose.travel);
    const { ctx, calls } = recordingContext();

    // Act
    drawScene(ctx, catalogOf(character("logan")), room, [{ id: "a", character: "logan", state: "walking" }], t);

    // Assert
    expect(calls).toContainEqual(["scale", -1, 1]);
    const [sprite] = drawn(calls, "logan");
    expect(sprite!.slice(6)).toEqual([0, 0, 36, 36]);
  });

  it("leaves afterimages behind dashing characters", () => {
    // Arrange
    const t = when("a", "dash", (pose) => !!pose.travel && pose.travel.progress > 0.3);
    const { ctx, calls } = recordingContext();

    // Act
    drawScene(ctx, catalogOf(character("quicksilver", { move: "dash" })), room, [{ id: "a", character: "quicksilver", state: "walking" }], t);

    // Assert
    expect(drawn(calls, "quicksilver")).toHaveLength(4);
  });

  it("puffs smoke where teleporters leave and land", () => {
    // Arrange
    const puff = (progress: (u: number) => boolean) => {
      const t = when("a", "teleport", (pose) => !!pose.travel && progress(pose.travel.progress));
      const pose = poseAt("a", room.walkArea, t, true, "teleport");
      const { ctx, calls } = recordingContext();
      drawScene(ctx, catalogOf(character("nightcrawler", { move: "teleport", powerColor: "#5522AA" })), room, [{ id: "a", character: "nightcrawler", state: "walking" }], t);
      return { pose, calls };
    };

    // Act
    const leaving = puff((u) => u > 0.1 && u < 0.4);
    const landing = puff((u) => u > 0.6 && u < 0.9);

    // Assert
    for (const { pose, calls } of [leaving, landing]) {
      const smoke = calls.filter(([name, , , w, h]) => name === "fillRect" && w === 2 && h === 2);
      expect(smoke).toHaveLength(14);
      for (const [, x, y] of smoke) {
        expect(Math.abs((x as number) - pose.x)).toBeLessThanOrEqual(15);
        expect(Math.abs((y as number) - (pose.y - 10))).toBeLessThanOrEqual(12);
      }
      expect(calls).toContainEqual(["=fillStyle", "#FFFFFF"]);
      expect(calls).toContainEqual(["=fillStyle", "#5522AA"]);
    }
  });
});
