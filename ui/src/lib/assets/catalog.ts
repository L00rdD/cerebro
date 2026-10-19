// Loads the asset manifests (assets/*/…json) and their decoded images.

import type { PowerKind } from "../scene/powers";
import type { Movement } from "../scene/wander";

export interface Rect { x: number; y: number; width: number; height: number }

export interface CharacterAsset {
  id: string;
  name: string;
  /** Hex color of the energy aura drawn under the character, if any. */
  aura?: string;
  /** How the character travels between waypoints. */
  move: Movement;
  /** The effect it plays while pausing, and its color. */
  power: PowerKind;
  powerColor: string;
  image: HTMLImageElement;
}

export interface RoomAsset {
  id: string;
  name: string;
  frames: number;
  fps: number;
  walkArea: Rect;
  image: HTMLImageElement;
}

export interface Catalog {
  characterSize: number;
  roomSize: { width: number; height: number };
  characters: Map<string, CharacterAsset>;
  rooms: Map<string, RoomAsset>;
  emotes: HTMLImageElement;
}

interface CharactersFile {
  frameSize: number;
  characters: {
    id: string;
    name: string;
    aura?: string;
    move?: Movement;
    power?: PowerKind;
    powerColor?: string;
    sprite: string;
  }[];
}
interface RoomsFile {
  frameWidth: number;
  frameHeight: number;
  rooms: { id: string; name: string; frames: number; fps: number; walkArea: Rect; sprite: string }[];
}

async function image(path: string): Promise<HTMLImageElement> {
  const img = new Image();
  img.src = `/${path}`;
  await img.decode();
  return img;
}

async function json<T>(path: string): Promise<T> {
  const response = await fetch(`/${path}`);
  if (!response.ok) throw new Error(`cannot load ${path}: ${response.status}`);
  return (await response.json()) as T;
}

let loading: Promise<Catalog> | undefined;

/** Loads everything once; later calls share the same promise. */
export function loadCatalog(): Promise<Catalog> {
  loading ??= (async () => {
    const [characters, rooms, emotes] = await Promise.all([
      json<CharactersFile>("characters/characters.json"),
      json<RoomsFile>("rooms/rooms.json"),
      image("ui/emotes.png"),
    ]);
    const characterAssets = await Promise.all(
      characters.characters.map(async (c) => ({
        id: c.id,
        name: c.name,
        aura: c.aura,
        move: c.move ?? "walk",
        power: c.power ?? "glow",
        powerColor: c.powerColor ?? c.aura ?? "#FFFFFF",
        image: await image(c.sprite),
      })),
    );
    const roomAssets = await Promise.all(
      rooms.rooms.map(async (r) => ({ ...r, image: await image(r.sprite) })),
    );
    return {
      characterSize: characters.frameSize,
      roomSize: { width: rooms.frameWidth, height: rooms.frameHeight },
      characters: new Map(characterAssets.map((c) => [c.id, c])),
      rooms: new Map(roomAssets.map((r) => [r.id, r])),
      emotes,
    };
  })();
  return loading;
}
