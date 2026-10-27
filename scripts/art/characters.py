"""Generates assets/characters/<id>.png: 4 frames x 3 rows (front, back, side), 24x24 each.

Frames: 0 standing, 1 step, 2 passing (standing), 3 other step. The side row faces right;
the UI mirrors it for left.

    python3 -I scripts/art/characters.py
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from body import DIRECTIONS, FRAMES, HEIGHT, WIDTH, Frame  # noqa: E402
from looks import LOOKS, Look  # noqa: E402
from pix import Canvas, hex_of, mix, ramp, rgba, shade  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "characters"


def paint(frame: Frame, colors: dict[str, str], outline_color: str | None) -> Canvas:
    canvas = Canvas(WIDTH, HEIGHT)
    ramps = {region: ramp(color) for region, color in colors.items()}
    rim_light(frame)
    for y in range(HEIGHT):
        for x in range(WIDTH):
            cell = frame.cells[y][x]
            if not cell:
                continue
            r = ramps.get(cell.region)
            if r is None:
                raise KeyError(f"no color for region {cell.region!r}")
            canvas.set(x, y, r[cell.tone] if cell.tone in r else cell.tone)
    outline(canvas, outline_color)
    return canvas


INK = "#1B1626"

# Regions that never catch the rim light (small or already bright details).
NO_RIM = {"eye", "eyewhite", "mouth", "buckle", "claw", "glow"}


def rim_light(frame: Frame):
    """The light comes from the top left: base-tone cells on the left edge of the
    silhouette, or under its top edge, get the light tone."""
    for y in range(HEIGHT):
        for x in range(WIDTH):
            cell = frame.cells[y][x]
            if not cell or cell.tone != "base" or cell.region in NO_RIM:
                continue
            left = frame.get(x - 1, y)
            above = frame.get(x, y - 1)
            if left is None or (above is None and y < 11):
                cell.tone = "light"


def outline(canvas: Canvas, color: str | None):
    """Selective outline: each edge pixel is a deep, inky version of what it surrounds."""
    src = canvas.image.copy().load()
    for y in range(HEIGHT):
        for x in range(WIDTH):
            if src[x, y][3]:
                continue
            for dx, dy in ((0, 1), (0, -1), (1, 0), (-1, 0)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < WIDTH and 0 <= ny < HEIGHT and src[nx, ny][3] > 128:
                    ink = color or mix(shade(hex_of(src[nx, ny]), -0.55), INK, 0.55)
                    canvas.px[x, y] = rgba(ink)
                    break


def render(look: Look) -> Canvas:
    sheet = Canvas(WIDTH * FRAMES, HEIGHT * len(DIRECTIONS))
    for row, direction in enumerate(DIRECTIONS):
        for f in range(FRAMES):
            frame = Frame(direction, f, big=look.big)
            look.dress(frame)
            sheet.blit(paint(frame, look.colors(), look.outline), f * WIDTH, row * HEIGHT)
    return sheet


def main(only: list[str]):
    manifest = json.loads((OUT / "characters.json").read_text())
    ids = [c["id"] for c in manifest["characters"]]
    missing = [i for i in ids if i not in LOOKS]
    if missing:
        raise SystemExit(f"no look for: {', '.join(missing)}")
    for cid in ids:
        if only and cid not in only:
            continue
        render(LOOKS[cid]).save(OUT / f"{cid}.png")
    manifest["frameWidth"] = WIDTH
    manifest["frameHeight"] = HEIGHT
    manifest.pop("frameSize", None)
    (OUT / "characters.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main(sys.argv[1:])
