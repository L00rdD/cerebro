"""Draws every room of the mansion and writes assets/rooms/*.png and rooms.json.

    python3 -I scripts/art/rooms.py [room-id ...]

Rooms are 192×120 frames in a 3/4 top-down view: the back wall takes the top of the frame,
the floor the rest. Light comes from the top left. Each room function draws the static
background once and may return an `animate(canvas, t)` callback, called on a copy of that
background for every frame with `t` in [0, 1); animations must loop, so they only use
periodic functions of `t`.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from pix import Canvas, Rng, mix, ramp, shade, strip  # noqa: E402

W, H = 192, 120
FPS = 8
ASSETS = Path(__file__).resolve().parents[2] / "assets" / "rooms"

SHADOW = "#140F22"
INK = "#1C1626"
TAU = math.pi * 2

ROOMS: list[dict] = []


def room(room_id: str, name: str, walk: tuple[int, int, int, int], frames: int = 1):
    """Registers a room; `walk` is (x, y, width, height) of the area where feet may land."""

    def register(draw):
        ROOMS.append({"id": room_id, "name": name, "walk": walk, "frames": frames, "draw": draw})
        return draw

    return register


# --- generic helpers --------------------------------------------------------------------


def copy(base: Canvas) -> Canvas:
    out = Canvas(base.width, base.height)
    out.image = base.image.copy()
    out.px = out.image.load()
    return out


def wave(t: float, k: int = 1, phase: float = 0.0) -> float:
    """sin over the loop: `k` full periods per loop, so it always comes back to the start."""
    return math.sin(TAU * (t * k + phase))


def soft_shadow(c: Canvas, x: int, y: int, w: int, h: int, alpha: float = 0.28):
    """Rounded cast shadow on the floor, with dithered edges."""
    for yy in range(h):
        for xx in range(w):
            edge = xx in (0, w - 1) or yy in (0, h - 1)
            corner = (xx in (0, w - 1)) and (yy in (0, h - 1))
            if corner:
                continue
            if edge and (x + xx + y + yy) % 2:
                continue
            c.set(x + xx, y + yy, SHADOW, alpha)


def oval_shadow(c: Canvas, cx: float, cy: float, rx: float, ry: float, alpha: float = 0.28):
    c.ellipse(cx, cy, rx, ry, SHADOW, alpha)


def rim(c: Canvas, x: int, y: int, w: int, h: int, color: str):
    """Outline of a rectangle with its corners left out, which reads rounder."""
    c.hline(x + 1, y, w - 2, color)
    c.hline(x + 1, y + h - 1, w - 2, color)
    c.vline(x, y + 1, h - 2, color)
    c.vline(x + w - 1, y + 1, h - 2, color)


def block(c: Canvas, x: int, y: int, w: int, h: int, d: int, color: str, top: str | None = None,
          shadow: bool = True, outline: bool = True):
    """A box seen in 3/4: a top face `d` pixels deep, then a front face `h` pixels tall.
    (x, y) is the top-left corner of the top face."""
    r = ramp(color)
    if shadow:
        soft_shadow(c, x + 2, y + d + h - 2, w + 2, 5)
        soft_shadow(c, x + w, y + d, 3, h, 0.22)
    t = top or r["light"]
    c.rect(x, y, w, d, t)
    c.hline(x, y, w, shade(t, 0.25))
    c.rect(x, y + d, w, h, r["base"])
    c.hline(x, y + d, w, r["light"] if not top else shade(r["base"], 0.15))
    c.hline(x, y + d + h - 1, w, r["shadow"])
    c.vline(x + w - 1, y + d, h, r["shadow"])
    if outline:
        rim(c, x - 1, y - 1, w + 2, d + h + 2, r["dark"])


def panel(c: Canvas, x: int, y: int, w: int, h: int, color: str, inset: bool = True):
    """Flat raised (or sunken) panel: light top-left edge, shadow bottom-right edge."""
    r = ramp(color)
    c.rect(x, y, w, h, r["base"])
    a, b = (r["shadow"], r["light"]) if inset else (r["light"], r["shadow"])
    c.hline(x, y, w, a)
    c.vline(x, y, h, a)
    c.hline(x, y + h - 1, w, b)
    c.vline(x + w - 1, y, h, b)


def noise(c: Canvas, x: int, y: int, w: int, h: int, colors: list[str], density: float, seed: int):
    rng = Rng(seed)
    for yy in range(y, y + h):
        for xx in range(x, x + w):
            if rng.next() < density:
                c.set(xx, yy, rng.pick(colors))


# --- walls ------------------------------------------------------------------------------


def wall(c: Canvas, height: int, color: str, trim: str, style: str = "plain", wainscot: int = 0,
         wainscot_color: str | None = None):
    """Back wall from y=0 to `height`, with a crown molding, an optional wainscot and a
    baseboard that casts a thin shadow on the floor."""
    r = ramp(color)
    c.rect(0, 0, W, height, r["base"])
    if style == "stripes":
        for x in range(0, W, 8):
            c.vline(x + 3, 4, height, r["shadow"])
            c.vline(x + 4, 4, height, mix(r["base"], r["shadow"], 0.5))
    elif style == "damask":
        for yy in range(8, height, 10):
            for xx in range(0, W, 12):
                ox = 6 if (yy // 10) % 2 else 0
                cx, cy = xx + ox, yy
                c.set(cx, cy - 2, r["shadow"])
                c.set(cx - 1, cy, r["shadow"])
                c.set(cx + 1, cy, r["shadow"])
                c.set(cx, cy + 2, r["shadow"])
                c.set(cx, cy, r["light"])
    elif style == "bricks":
        bricks(c, 0, 4, W, height - 4, color)
    elif style == "tiles":
        for yy in range(4, height, 6):
            off = 4 if (yy // 6) % 2 else 0
            c.hline(0, yy, W, r["shadow"])
            for xx in range(off, W, 8):
                c.vline(xx, yy, 6, r["shadow"])
                c.hline(xx + 1, yy + 1, 6, r["light"])
    elif style == "metal":
        for xx in range(0, W, 24):
            c.vline(xx, 0, height, r["dark"])
            c.vline(xx + 1, 0, height, r["light"])
            for yy in (6, height - 10):
                c.set(xx + 4, yy, r["light"])
                c.set(xx + 19, yy, r["light"])
                c.set(xx + 4, yy + 1, r["dark"])
                c.set(xx + 19, yy + 1, r["dark"])
    # Ambient occlusion under the ceiling.
    c.dither(0, 4, W, 2, r["shadow"])
    if wainscot:
        wr = ramp(wainscot_color or trim)
        top = height - wainscot
        c.rect(0, top, W, wainscot, wr["base"])
        c.hline(0, top, W, wr["light"])
        c.hline(0, top + 1, W, wr["base"])
        c.hline(0, top - 1, W, wr["dark"])
        for xx in range(4, W, 16):
            rim(c, xx, top + 3, 12, wainscot - 7, wr["shadow"])
            c.hline(xx + 1, top + 4, 10, wr["light"])
    tr = ramp(trim)
    # Crown molding.
    c.rect(0, 0, W, 4, tr["base"])
    c.hline(0, 0, W, tr["light"])
    c.hline(0, 3, W, tr["dark"])
    # Baseboard.
    c.rect(0, height - 3, W, 3, tr["base"])
    c.hline(0, height - 3, W, tr["light"])
    c.hline(0, height - 1, W, tr["dark"])
    c.dither(0, height, W, 1, SHADOW)


def bricks(c: Canvas, x: int, y: int, w: int, h: int, color: str, seed: int = 7):
    r = ramp(color)
    rng = Rng(seed)
    mortar = shade(color, 0.45)
    c.rect(x, y, w, h, mortar)
    for row, yy in enumerate(range(y, y + h, 5)):
        off = 0 if row % 2 else 5
        for xx in range(x - off, x + w, 10):
            tone = rng.pick([r["base"], r["base"], r["shadow"], mix(r["base"], r["light"], 0.4)])
            x0, x1 = max(x, xx), min(x + w, xx + 9)
            if x1 <= x0:
                continue
            hh = min(4, y + h - yy)
            c.rect(x0, yy, x1 - x0, hh, tone)
            c.hline(x0, yy, x1 - x0, mix(tone, r["light"], 0.5))


def window(c: Canvas, x: int, y: int, w: int, h: int, frame: str = "#E8DCC4", sky: tuple[str, str] = ("#7EC8F0", "#CDEBFA"),
           panes: tuple[int, int] = (2, 2), curtains: str | None = None, view=None):
    """Window on the back wall; `view(c, x, y, w, h)` paints what's outside."""
    fr = ramp(frame)
    c.rect(x - 2, y - 2, w + 4, h + 4, fr["base"])
    rim(c, x - 3, y - 3, w + 6, h + 6, fr["dark"])
    c.hline(x - 2, y - 2, w + 4, fr["light"])
    c.gradient_v(x, y, w, h, sky[0], sky[1], 4)
    if view:
        view(c, x, y, w, h)
    # Glass glints.
    for i in range(3):
        c.line(x + 2 + i * 6, y + h - 2, x + 2 + i * 6 + min(h - 3, 8), y + h - 3 - min(h - 3, 8), "#FFFFFF", 0.35)
    cols, rows = panes
    for i in range(1, cols):
        xx = x + i * w // cols
        c.vline(xx - 1, y, h, fr["base"])
        c.vline(xx, y, h, fr["shadow"])
    for j in range(1, rows):
        yy = y + j * h // rows
        c.hline(x, yy - 1, w, fr["base"])
        c.hline(x, yy, w, fr["shadow"])
    c.hline(x, y, w, fr["shadow"])
    # Sill.
    c.rect(x - 4, y + h + 1, w + 8, 3, fr["light"])
    c.hline(x - 4, y + h + 3, w + 8, fr["shadow"])
    rim(c, x - 5, y + h, w + 10, 5, fr["dark"])
    if curtains:
        cr = ramp(curtains)
        for side in (x - 6, x + w - 1):
            c.rect(side, y - 4, 7, h + 6, cr["base"])
            for k in range(1, 7, 3):
                c.vline(side + k, y - 4, h + 6, cr["shadow"])
            c.vline(side, y - 4, h + 6, cr["light"])
            rim(c, side - 1, y - 5, 9, h + 8, cr["dark"])
        c.rect(x - 8, y - 6, w + 16, 3, cr["shadow"])
        c.hline(x - 8, y - 6, w + 16, cr["light"])


def light_patch(c: Canvas, x: int, y: int, w: int, h: int, skew: int = 10, alpha: float = 0.12):
    """Sunlight from a window falling on the floor, as a skewed parallelogram."""
    for yy in range(h):
        off = round(skew * yy / max(h, 1))
        for xx in range(w):
            if yy in (0, h - 1) and (xx + yy) % 2:
                continue
            c.set(x + xx + off, y + yy, "#FFF6D8", alpha)


def frame_picture(c: Canvas, x: int, y: int, w: int, h: int, art, frame: str = "#B8862E"):
    fr = ramp(frame)
    c.rect(x - 2, y - 2, w + 4, h + 4, fr["base"])
    c.hline(x - 2, y - 2, w + 4, fr["light"])
    c.vline(x - 2, y - 2, h + 4, fr["light"])
    rim(c, x - 3, y - 3, w + 6, h + 6, fr["dark"])
    art(c, x, y, w, h)
    c.dither(x + w + 3, y, 1, h + 3, SHADOW)


def landscape(c: Canvas, x: int, y: int, w: int, h: int):
    c.gradient_v(x, y, w, h, "#E9B872", "#F7E3B0", 3)
    c.ellipse(x + w * 0.7, y + h * 0.35, 2, 2, "#FFF7D6")
    for xx in range(w):
        hh = round(h * 0.45 + 3 * math.sin(xx * 0.5) + 2 * math.sin(xx * 1.3))
        c.vline(x + xx, y + hh, h - hh, "#6E7F4A" if xx % 7 else "#5C6B3D")
    c.rect(x, y + h - 3, w, 3, "#4F5E33")


def portrait(c: Canvas, x: int, y: int, w: int, h: int):
    c.rect(x, y, w, h, "#3A2F3F")
    c.ellipse(x + w / 2, y + h * 0.42, w * 0.22, h * 0.24, "#E2B48A")
    c.rect(x + w // 2 - w // 3, y + int(h * 0.68), (w // 3) * 2, h - int(h * 0.68), "#2B4A7A")
    c.hline(x + w // 2 - w // 5, y + int(h * 0.2), (w // 5) * 2, "#5A3A22")


def plant(c: Canvas, x: int, y: int, size: int = 1, pot: str = "#C26A3D", leaf: str = "#4E9A3E"):
    """Potted plant; (x, y) is the bottom center of the pot."""
    pr = ramp(pot)
    lr = ramp(leaf)
    pw, ph = 6 + 2 * size, 5 + size
    oval_shadow(c, x + 2, y, pw * 0.7, 2)
    c.rect(x - pw // 2, y - ph, pw, ph, pr["base"])
    c.vline(x + pw // 2 - 1, y - ph, ph, pr["shadow"])
    c.vline(x - pw // 2, y - ph, ph, pr["light"])
    c.rect(x - pw // 2 - 1, y - ph - 2, pw + 2, 2, pr["light"])
    rim(c, x - pw // 2 - 2, y - ph - 3, pw + 4, ph + 4, pr["dark"])
    cy = y - ph - 4 - 3 * size
    blobs = [(-3, 2, 4), (3, 2, 4), (0, -2, 5), (-4, -3, 3), (4, -4, 3), (0, -6, 3)]
    for dx, dy, r_ in blobs:
        c.ellipse(x + dx * size, cy + dy * size, r_ * size + 1, (r_ * size + 1) * 0.8, lr["dark"])
    for dx, dy, r_ in blobs:
        c.ellipse(x + dx * size, cy + dy * size, r_ * size, r_ * size * 0.8, lr["base"])
    for dx, dy, r_ in blobs:
        c.ellipse(x + dx * size - 1, cy + dy * size - 1, max(1, r_ * size - 2), max(1, (r_ * size - 2) * 0.7), lr["light"])
    for dx, dy, r_ in blobs[:3]:
        c.set(x + dx * size + 1, cy + dy * size + 1, lr["shadow"])


def bookcase(c: Canvas, x: int, y: int, w: int, h: int, wood: str = "#7A4A2A", seed: int = 3, depth: int = 4):
    """Bookcase standing against the back wall; (x, y) is its top-left corner."""
    wr = ramp(wood)
    rng = Rng(seed)
    soft_shadow(c, x + 2, y + h - 2, w + 1, 4)
    c.rect(x, y, w, h, wr["base"])
    c.rect(x, y, w, depth, wr["light"])
    inner_top = y + depth + 1
    shelf_h = 11
    yy = inner_top
    book_colors = ["#8E2F2F", "#2F4F8E", "#3F7A3A", "#C9A34A", "#6A3A7A", "#B5653A", "#2E6E6E", "#DCD0B0", "#4A4A5A"]
    while yy + shelf_h <= y + h - 3:
        c.rect(x + 2, yy, w - 4, shelf_h - 2, wr["dark"])
        xx = x + 2
        while xx < x + w - 3:
            bw = rng.int(1, 3)
            bh = rng.int(6, shelf_h - 2)
            if xx + bw > x + w - 2:
                break
            if rng.next() < 0.08:
                xx += bw + 1
                continue
            col = rng.pick(book_colors)
            br = ramp(col)
            c.rect(xx, yy + shelf_h - 2 - bh, bw, bh, br["base"])
            c.vline(xx, yy + shelf_h - 2 - bh, bh, br["light"])
            if bw > 1:
                c.set(xx + bw - 1, yy + shelf_h - 3 - bh // 2, br["light"])
            c.hline(xx, yy + shelf_h - 2 - bh + 2, bw, br["shadow"])
            xx += bw
        c.hline(x + 1, yy + shelf_h - 2, w - 2, wr["light"])
        c.hline(x + 1, yy + shelf_h - 1, w - 2, wr["shadow"])
        yy += shelf_h
    c.vline(x, y, h, wr["light"])
    c.vline(x + w - 1, y, h, wr["shadow"])
    rim(c, x - 1, y - 1, w + 2, h + 2, wr["dark"])


def screen(c: Canvas, x: int, y: int, w: int, h: int, glow: str = "#3FD0FF", bezel: str = "#2A2D3A"):
    """Monitor frame with a dark glass; content is drawn by the caller (often animated)."""
    br = ramp(bezel)
    c.rect(x - 2, y - 2, w + 4, h + 4, br["base"])
    c.hline(x - 2, y - 2, w + 4, br["light"])
    rim(c, x - 3, y - 3, w + 6, h + 6, br["dark"])
    c.rect(x, y, w, h, mix("#06121A", glow, 0.12))


def scanlines(c: Canvas, x: int, y: int, w: int, h: int):
    for yy in range(y, y + h, 2):
        c.hline(x, yy, w, "#000000", 0.18)


def rug(c: Canvas, x: int, y: int, w: int, h: int, color: str, border: str, seed: int = 1):
    r = ramp(color)
    b = ramp(border)
    c.rect(x, y, w, h, b["base"])
    c.rect(x + 2, y + 2, w - 4, h - 4, r["base"])
    rim(c, x + 4, y + 4, w - 8, h - 8, b["light"])
    # Medallion and pattern.
    cx, cy = x + w / 2, y + h / 2
    c.ellipse(cx, cy, w * 0.16, h * 0.2, r["shadow"])
    c.ellipse(cx, cy, w * 0.1, h * 0.12, b["base"])
    c.ellipse(cx, cy, w * 0.04, h * 0.05, r["light"])
    for k in range(x + 8, x + w - 8, 6):
        c.set(k, y + 6, b["light"])
        c.set(k, y + h - 7, b["light"])
    for k in range(y + 8, y + h - 8, 4):
        c.set(x + 6, k, b["light"])
        c.set(x + w - 7, k, b["light"])
    # Fringes.
    for k in range(y + 1, y + h - 1, 2):
        c.set(x - 1, k, "#E8DCC0")
        c.set(x + w, k, "#E8DCC0")
    c.hline(x, y + h, w, SHADOW, 0.3)


# --- floors -----------------------------------------------------------------------------


def floor_wood(c: Canvas, y0: int, color: str, plank: int = 5, seed: int = 11):
    r = ramp(color)
    rng = Rng(seed)
    tones = [r["base"], mix(r["base"], r["light"], 0.25), mix(r["base"], r["shadow"], 0.3)]
    for row, y in enumerate(range(y0, H, plank)):
        x = -rng.int(0, 30)
        while x < W:
            length = rng.int(26, 48)
            tone = rng.pick(tones)
            c.rect(max(0, x), y, min(W, x + length) - max(0, x), plank, tone)
            c.hline(max(0, x), y, min(W, x + length) - max(0, x), mix(tone, r["light"], 0.35))
            # Grain.
            for _ in range(2):
                gx = x + rng.int(3, max(4, length - 8))
                gy = y + rng.int(1, plank - 2)
                c.hline(gx, gy, rng.int(3, 7), mix(tone, r["shadow"], 0.5))
            c.vline(x, y, plank, r["dark"])
            x += length
        c.hline(0, y + plank - 1, W, r["shadow"])


def floor_tiles(c: Canvas, y0: int, a: str, b: str | None = None, size: int = 12, grout: str | None = None):
    """Square tiles foreshortened to 3/4; a checkerboard if `b` is given."""
    th = round(size * 0.75)
    g = grout or shade(a, -0.3)
    for j, y in enumerate(range(y0, H, th)):
        for i, x in enumerate(range(0, W, size)):
            col = b if b and (i + j) % 2 else a
            r = ramp(col)
            c.rect(x, y, size, th, r["base"])
            c.hline(x + 1, y + 1, size - 2, r["light"])
            c.vline(x + 1, y + 1, th - 2, mix(r["base"], r["light"], 0.5))
            c.set(x + size - 3, y + th - 3, r["light"])
            c.hline(x, y + th - 1, size, g)
            c.vline(x + size - 1, y, th, g)


def floor_metal(c: Canvas, y0: int, color: str, size: int = 24):
    r = ramp(color)
    th = round(size * 0.6)
    c.rect(0, y0, W, H - y0, r["base"])
    for j, y in enumerate(range(y0, H, th)):
        off = size // 2 if j % 2 else 0
        c.hline(0, y, W, r["dark"])
        c.hline(0, y + 1, W, r["light"])
        for x in range(-off, W, size):
            c.vline(x, y, th, r["dark"])
            c.vline(x + 1, y + 1, th - 1, r["light"])
            for rx, ry in ((3, 3), (size - 4, 3), (3, th - 3), (size - 4, th - 3)):
                c.set(x + rx, y + ry, r["light"])
                c.set(x + rx + 1, y + ry + 1, r["dark"])
            # Diamond tread pattern.
            for ty in range(y + 4, y + th - 3, 3):
                for tx in range(x + 6 + (ty % 2) * 2, x + size - 6, 4):
                    c.set(tx, ty, r["shadow"])


def floor_concrete(c: Canvas, y0: int, color: str, seed: int = 5):
    r = ramp(color)
    c.rect(0, y0, W, H - y0, r["base"])
    noise(c, 0, y0, W, H - y0, [r["shadow"], mix(r["base"], r["light"], 0.4)], 0.08, seed)
    for x in range(0, W, 48):
        c.vline(x, y0, H - y0, r["shadow"])
    for y in range(y0 + 28, H, 28):
        c.hline(0, y, W, r["shadow"])


def grass(c: Canvas, x: int, y: int, w: int, h: int, color: str = "#5AA845", seed: int = 9, flowers: bool = True):
    r = ramp(color)
    c.rect(x, y, w, h, r["base"])
    rng = Rng(seed)
    for _ in range(w * h // 9):
        gx, gy = x + rng.int(0, w - 1), y + rng.int(0, h - 1)
        if rng.next() < 0.5:
            c.set(gx, gy, r["light"])
            c.set(gx, gy + 1, r["base"])
        else:
            c.set(gx, gy, r["shadow"])
    # Tufts.
    for _ in range(w * h // 70):
        gx, gy = x + rng.int(1, w - 2), y + rng.int(1, h - 2)
        c.set(gx, gy, r["dark"])
        c.set(gx - 1, gy - 1, r["light"])
        c.set(gx + 1, gy - 1, r["light"])
    if flowers:
        for _ in range(w * h // 260):
            gx, gy = x + rng.int(1, w - 2), y + rng.int(1, h - 2)
            col = rng.pick(["#FFF3F3", "#FFD84A", "#FF8FB0", "#B7A2FF"])
            c.set(gx, gy, col)
            c.set(gx, gy + 1, r["dark"])


def tree(c: Canvas, x: int, y: int, size: float = 1.0, leaf: str = "#3F8A3A", sway: float = 0.0):
    """Round tree; (x, y) is the base of the trunk. `sway` shifts the crown by a pixel or two."""
    lr = ramp(leaf)
    oval_shadow(c, x + 4, y, 12 * size, 4 * size, 0.3)
    tr = ramp("#6B4A2E")
    c.rect(x - 2, y - round(10 * size), 4, round(10 * size), tr["base"])
    c.vline(x + 1, y - round(10 * size), round(10 * size), tr["shadow"])
    c.vline(x - 2, y - round(10 * size), round(10 * size), tr["light"])
    cy = y - 18 * size
    s = round(sway)
    blobs = [(-7, 2, 7), (7, 2, 7), (0, -4, 9), (-5, -9, 6), (6, -8, 6), (0, 5, 7)]
    for dx, dy, r_ in blobs:
        c.ellipse(x + dx * size + s, cy + dy * size, r_ * size + 1, r_ * size + 1, lr["dark"])
    for dx, dy, r_ in blobs:
        c.ellipse(x + dx * size + s, cy + dy * size, r_ * size, r_ * size, lr["shadow"])
    for dx, dy, r_ in blobs:
        c.ellipse(x + dx * size - 1 + s, cy + dy * size - 1, r_ * size - 1.5, r_ * size - 2, lr["base"])
    for dx, dy, r_ in blobs[2:5]:
        c.ellipse(x + dx * size - 2 + s, cy + dy * size - 2, max(1, r_ * size - 5), max(1, r_ * size - 5.5), lr["light"])


def lamp_glow(c: Canvas, cx: int, cy: int, radius: int, color: str = "#FFE9A8", alpha: float = 0.12):
    for i in range(3):
        c.ellipse(cx, cy, radius - i * radius / 3, (radius - i * radius / 3) * 0.8, color, alpha)


def chair(c: Canvas, x: int, y: int, color: str, facing: str = "front"):
    """Small chair; (x, y) top-left of the seat."""
    r = ramp(color)
    soft_shadow(c, x + 1, y + 7, 9, 3)
    if facing == "back":
        c.rect(x, y - 7, 8, 8, r["base"])
        c.hline(x, y - 7, 8, r["light"])
        rim(c, x - 1, y - 8, 10, 10, r["dark"])
    c.rect(x, y, 8, 3, r["light"])
    c.rect(x, y + 3, 8, 2, r["shadow"])
    c.vline(x, y + 5, 3, r["dark"])
    c.vline(x + 7, y + 5, 3, r["dark"])
    if facing == "front":
        c.rect(x, y - 6, 8, 6, r["base"])
        c.hline(x, y - 6, 8, r["light"])
        c.hline(x + 1, y - 3, 6, r["shadow"])
    rim(c, x - 1, y - (7 if facing == "front" else 1), 10, (13 if facing == "front" else 7), r["dark"])


# --- rooms ------------------------------------------------------------------------------


@room("office", "Professor's Study", (18, 66, 156, 48), frames=16)
def office(c: Canvas):
    wall(c, 40, "#3E5B45", "#6B4226", style="damask", wainscot=14, wainscot_color="#5E3A22")
    floor_wood(c, 40, "#8A5A34", seed=21)
    rug(c, 44, 64, 104, 46, "#7A1F2B", "#C9A04A")
    light_patch(c, 140, 41, 30, 24, skew=-16, alpha=0.1)

    bookcase(c, 6, 4, 34, 46, seed=5)
    # Tall window with curtains.
    window(c, 148, 7, 26, 22, sky=("#2D3E73", "#7F8FC8"), panes=(2, 3), curtains="#7A1F2B",
           view=lambda c, x, y, w, h: [c.set(x + 4 + (i * 7) % w, y + 2 + (i * 5) % (h - 4), "#FFFFFF") for i in range(6)])

    # Fireplace: stone surround, mantel, dark hearth (the fire is animated).
    stone = ramp("#A89A88")
    fx, fy, fw = 78, 8, 36
    soft_shadow(c, fx, 46, fw + 4, 4)
    c.rect(fx, fy + 6, fw, 34, stone["base"])
    for yy in range(fy + 8, fy + 40, 5):
        off = 3 if (yy // 5) % 2 else 0
        c.hline(fx, yy, fw, stone["shadow"])
        for xx in range(fx + off, fx + fw, 7):
            c.vline(xx, yy, 5, stone["shadow"])
            c.set(xx + 1, yy + 1, stone["light"])
    c.rect(fx + 8, fy + 20, fw - 16, 20, "#1A1210")
    c.rect(fx + 9, fy + 21, fw - 18, 2, "#2A1C16")
    rim(c, fx + 7, fy + 19, fw - 14, 22, stone["dark"])
    # Logs and grate.
    c.rect(fx + 11, fy + 35, fw - 22, 3, "#5A3820")
    c.hline(fx + 11, fy + 35, fw - 22, "#7A5232")
    c.hline(fx + 10, fy + 38, fw - 20, "#2A2A30")
    # Mantel.
    block(c, fx - 4, fy + 2, fw + 8, 3, 3, "#6B4226", shadow=False)
    rim(c, fx - 5, fy + 5, fw + 10, 37, stone["dark"])
    # Candles on the mantel.
    for cx in (fx + 2, fx + fw - 4):
        c.rect(cx, fy - 1, 2, 4, "#F2E8D0")
        c.set(cx, fy - 2, "#FFB54A")

    # Painting and clock.
    frame_picture(c, 50, 8, 18, 13, landscape)
    frame_picture(c, 124, 8, 14, 16, portrait)

    # Desk with lamp, papers and globe.
    block(c, 22, 50, 42, 10, 6, "#5E3A22", top="#7A4C2C")
    c.rect(26, 51, 9, 4, "#EFE6D2")
    c.rect(28, 50, 9, 4, "#F8F1E0")
    c.hline(29, 51, 6, "#9A9080")
    c.hline(29, 52, 4, "#9A9080")
    c.rect(52, 47, 2, 6, "#C9A04A")
    c.ellipse(53, 46, 4, 2, "#2E6E3E")
    c.hline(50, 46, 6, "#4E9A5E")
    lamp_glow(c, 53, 50, 9, alpha=0.08)
    # Chess table by the window.
    block(c, 150, 46, 18, 7, 6, "#5E3A22", top="#7A4C2C")
    for i in range(4):
        for j in range(2):
            c.rect(152 + i * 3 + (j % 2), 47 + j * 2, 2, 2, "#F2E8D0" if (i + j) % 2 else "#2A1C16")
    c.set(155, 46, "#F2E8D0")
    c.set(160, 45, "#2A1C16")
    # Wingback chair.
    ch = ramp("#2F5A3A")
    soft_shadow(c, 128, 56, 16, 4)
    c.rect(127, 40, 14, 14, ch["base"])
    c.hline(127, 40, 14, ch["light"])
    c.rect(125, 46, 3, 10, ch["shadow"])
    c.rect(140, 46, 3, 10, ch["shadow"])
    c.rect(128, 50, 12, 5, ch["light"])
    rim(c, 124, 39, 20, 18, ch["dark"])
    plant(c, 10, 116)
    plant(c, 182, 116)

    def animate(c: Canvas, t: float):
        # Fire: three flickering tongues, red outside, yellow at the core.
        base_y = fy + 36
        for i, (dx, size) in enumerate(((-6, 0.8), (0, 1.0), (6, 0.75))):
            fx0 = fx + fw // 2 + dx
            h = (7 + 2.5 * wave(t, 2, i * 0.31) + 1.5 * wave(t, 3, i * 0.17)) * size
            sway = wave(t, 2, i * 0.4 + 0.1)
            for layer, (col, k) in enumerate((("#D8402A", 1.0), ("#FF9A2A", 0.7), ("#FFE27A", 0.4))):
                for yy in range(round(h * k)):
                    u = yy / max(h * k, 1)
                    half = (3.2 * size * k) * (1 - u) ** 0.8
                    off = sway * u * 1.5
                    c.hline(round(fx0 - half + off), base_y - yy, max(1, round(half * 2)), col)
        lamp_glow(c, fx + fw // 2, fy + 32, 14 + round(wave(t, 4)), "#FF9A3A", 0.07)
        for i in range(3):
            p = (t * 2 + i / 3) % 1
            c.set(fx + 14 + i * 4 + round(2 * wave(p, 1)), base_y - 8 - round(p * 10), "#FFD27A", 1 - p)
        # Candle flames.
        for cx in (fx + 2, fx + fw - 4):
            c.set(cx + (1 if wave(t, 3, cx) > 0.6 else 0), fy - 2, "#FFE07A")
            c.set(cx, fy - 3, "#FFF6D0", 0.6 + 0.4 * wave(t, 5))

    return animate


# Floor seams of the Danger Room, closer together towards the back.
DR_ROWS = [41, 46, 52, 59, 68, 79, 92, 107]


@room("danger-room", "Danger Room", (14, 64, 164, 50), frames=16)
def danger_room(c: Canvas):
    c.rect(0, 0, W, H, "#1B1F2E")
    wall(c, 38, "#2B3044", "#454C66", style="metal")
    # Observation window with the control booth.
    screen(c, 74, 6, 44, 14, glow="#FF5A4A", bezel="#3A3F55")
    c.gradient_v(74, 6, 44, 14, "#2A1A24", "#4A2A30", 3)
    for i, x in enumerate((80, 92, 104)):
        c.rect(x, 14, 6, 6, "#151821")
        c.ellipse(x + 3, 12, 2, 2, "#2A2D3A")
    # Hazard stripes along the top of the floor.
    for x in range(0, W):
        c.vline(x, 38, 3, "#F2C230" if (x // 4) % 2 else "#1E1E24")
    # Smooth dark floor plates; the glowing grid is animated.
    fr = ramp("#262B40")
    c.rect(0, 41, W, H - 41, fr["base"])
    for y, nxt in zip(DR_ROWS, DR_ROWS[1:] + [H]):
        c.hline(0, y, W, fr["dark"])
        c.hline(0, y + 1, W, mix(fr["base"], fr["light"], 0.3))
    for x in range(-96, W + 96, 16):
        c.line(96 + (x - 96) * 2 // 3, 41, x, H, fr["dark"])
    c.dither(0, 41, W, 3, SHADOW)
    # Wall turrets.
    for tx in (22, 162):
        block(c, tx, 12, 12, 8, 4, "#5A6178")
        c.rect(tx + 4, 18, 4, 8, "#3A3F55")
        c.rect(tx + 5, 25, 2, 4, "#22252F")
    # Exit doors.
    for dx in (44, 136):
        panel(c, dx, 14, 14, 24, "#3A4058", inset=False)
        c.vline(dx + 7, 14, 24, "#22252F")
        c.rect(dx + 2, 16, 10, 2, "#F2C230")

    def animate(c: Canvas, t: float):
        # Holographic grid pulsing over the floor, with a scan line sweeping down.
        glow = "#5AF0FF"
        pulse = 0.18 + 0.1 * wave(t, 2)
        for x in range(-96, W + 96, 16):
            c.line(96 + (x - 96) * 2 // 3, 41, x, H, glow, pulse)
        for y in DR_ROWS:
            c.hline(0, y, W, glow, pulse)
        # Wireframe obstacles flickering in and out.
        for i, (hx, hy, s_) in enumerate(((42, 60, 10), (152, 58, 9), (104, 52, 7))):
            a = 0.35 + 0.35 * wave(t, 1, i / 3)
            if wave(t, 4, i * 0.2) > 0.85:
                continue
            pts = [(hx - s_, hy), (hx, hy - s_ // 2), (hx + s_, hy), (hx, hy + s_ // 2)]
            top = [(px, py - s_ * 2) for px, py in pts]
            for (ax, ay), (bx, by) in zip(pts, pts[1:] + pts[:1]):
                c.line(ax, ay, bx, by, glow, a)
            for (ax, ay), (bx, by) in zip(top, top[1:] + top[:1]):
                c.line(ax, ay, bx, by, glow, a)
            for (ax, ay), (bx, by) in zip(pts, top):
                c.line(ax, ay, bx, by, glow, a)
        sy = 41 + round((H - 41) * t)
        c.hline(0, sy, W, "#B8FAFF", 0.5)
        c.hline(0, sy - 1, W, glow, 0.25)
        # Turret lights and tracking lasers.
        for i, tx in enumerate((22, 162)):
            on = wave(t, 4, i * 0.5) > 0
            c.set(tx + 5, 28, "#FF3A3A" if on else "#7A1A1A")
            c.set(tx + 6, 28, "#FF3A3A" if on else "#7A1A1A")
            aim = 96 + round(70 * wave(t, 1, i * 0.5))
            if wave(t, 2, i * 0.25) > 0.3:
                c.line(tx + 6, 29, aim, 100, "#FF4A4A", 0.55)
                c.ellipse(aim, 100, 2, 1, "#FF8A8A", 0.7)
        # Control booth silhouettes blinking consoles.
        for i in range(8):
            if wave(t, 2, i * 0.13) > 0:
                c.set(76 + i * 5, 19, "#7AF07A")
        # Light strips along the base of the walls.
        for x in range(0, W, 2):
            b = 0.5 + 0.5 * wave(t, 1, x / W)
            c.set(x, 33, glow, 0.3 + 0.6 * b)
        # Warning beacon.
        c.ellipse(96, 3, 3, 2, "#FF3A3A" if wave(t, 2) > 0 else "#5A1A1A")

    return animate


@room("hangar", "Blackbird Hangar", (10, 82, 172, 34), frames=16)
def hangar(c: Canvas):
    wall(c, 34, "#4A5160", "#2E333E", style="metal")
    # Big hangar door with horizontal slats.
    dr = ramp("#5E6676")
    c.rect(48, 4, 96, 30, dr["base"])
    for y in range(6, 34, 4):
        c.hline(48, y, 96, dr["shadow"])
        c.hline(48, y + 1, 96, dr["light"])
    rim(c, 47, 3, 98, 32, dr["dark"])
    for x in range(48, 144):
        c.vline(x, 30, 3, "#F2C230" if (x // 5) % 2 else "#1E1E24")
    floor_concrete(c, 34, "#6E7380")
    # Painted lines.
    for x in range(8, W - 8, 10):
        c.hline(x, 74, 6, "#F2C230")
    c.ellipse_ring(96, 56, 76, 20, "#C9A030")
    # Tool carts and fuel drums.
    for x in (8, 172):
        block(c, x, 40, 12, 10, 4, "#C23A2A")
        c.hline(x + 1, 47, 10, "#8A2A1E")
    block(c, 20, 44, 8, 8, 3, "#3A6A9A")
    block(c, 164, 44, 6, 8, 3, "#3A6A9A")

    # The Blackbird, top-down and foreshortened: long chined nose blending into a delta
    # wing, two engine nacelles with their inlet spikes and inward-canted fins.
    cx, cy, x0, x1 = 96, 56, 26, 170
    squash = 0.62
    nacelle = 17

    def half_width(x: int) -> float:
        u = (x - x0) / (x1 - x0)
        body = 1 + 6 * min(1.0, u / 0.5) ** 0.7
        if u > 0.85:
            body = max(3, body - (u - 0.85) * 30)
        wing = 0.0
        if 0.4 < u < 0.93:
            wing = 36 * min(1.0, (u - 0.4) / 0.32) ** 0.9
        return max(body, wing)

    oval_shadow(c, cx + 14, cy + 20, 64, 7, 0.32)
    jet = Canvas(W, H)
    upper, lower = "#2E313C", "#1F2129"
    for x in range(x0, x1):
        hw = round(half_width(x) * squash)
        jet.vline(x, cy - hw, hw, upper)
        jet.vline(x, cy, hw + 1, lower)
    for side in (-1, 1):
        ny = cy + round(side * nacelle * squash)
        for x in range(100, x1 + 3):
            hw = 2 if x < 106 or x >= x1 - 1 else 3
            jet.rect(x, ny - hw + 1, 1, hw * 2 - 1, upper if side < 0 else lower)
        jet.hline(95, ny, 5, "#3A3E4A")
    jet.outline("#0B0C10")
    c.blit(jet, 0, 0)
    # Spine highlight, cockpit, engine detail.
    c.hline(x0 + 8, cy - 1, 110, "#3E4352")
    c.hline(x0 + 14, cy - 2, 50, "#4A5060")
    c.rect(48, cy - 1, 9, 2, "#4A6A8A")
    c.hline(49, cy - 1, 6, "#A8D0F0")
    for side in (-1, 1):
        ny = cy + round(side * nacelle * squash)
        c.hline(104, ny - 1, x1 - 104, "#40444F")
        c.vline(x1 + 2, ny - 1, 3, "#5A4A3A")
        # Fins, canted towards the fuselage.
        for k in range(12):
            c.set(146 + k, ny - side * (k // 4), "#4A4F5E")
            c.set(146 + k, ny - side * (k // 4) + side, "#16171D")
    # White X roundels on both wings.
    for side in (-1, 1):
        wy = cy + round(side * 26 * squash) - 2
        for k in range(5):
            c.set(124 + k, wy + k, "#D0D4E0")
            c.set(128 - k, wy + k, "#D0D4E0")

    def animate(c: Canvas, t: float):
        # Navigation lights on the wing tips and the nose.
        on = int(t * 16) % 8 < 2
        tip = round(34 * squash)
        c.set(140, cy - tip, "#FF3A3A" if on else "#5A1A1A")
        c.set(140, cy + tip, "#3AFF6A" if on else "#1A5A2A")
        c.set(x0, cy, "#FFFFFF" if int(t * 16) % 4 == 0 else "#7A7A80")
        # Rotating amber beacons above the door.
        for i, bx in enumerate((40, 152)):
            c.rect(bx - 3, 12, 7, 3, "#2A2D3A")
            a = wave(t, 2, i * 0.25)
            c.ellipse(bx, 10, 3, 3, "#FFB43A" if a > 0 else "#7A4A1A")
            if a > 0:
                c.ellipse(bx, 10, 9, 6, "#FFB43A", 0.12)
        # Heat shimmer behind the engines.
        for side in (-1, 1):
            ny = cy + round(side * nacelle * squash)
            for k in range(3):
                c.set(x1 + 5 + k * 2, ny + round(wave(t, 4, k * 0.3)), "#9AA0B0", 0.5)

    return animate


@room("cerebro", "Cerebro", (58, 66, 76, 34), frames=16)
def cerebro(c: Canvas):
    # The inside of the sphere: concentric rings of panels fading into the dark.
    c.rect(0, 0, W, H, "#060814")
    cx, cy = 96, 66
    for i, r_ in enumerate(range(150, 30, -10)):
        col = mix("#1E2A52", "#0A0E22", i / 12)
        c.ellipse(cx, cy - 6, r_, r_ * 0.62, col)
        c.ellipse_ring(cx, cy - 6, r_, r_ * 0.62, mix("#3A5AA8", "#141A3A", i / 12))
    # Panel seams radiating from the center.
    for k in range(24):
        a = k / 24 * TAU
        c.line(cx + round(math.cos(a) * 34), cy - 6 + round(math.sin(a) * 21),
               cx + round(math.cos(a) * 150), cy - 6 + round(math.sin(a) * 93), "#2A3A72", 0.6)
    # The round platform and the walkway.
    pr = ramp("#5A6280")
    c.rect(78, 98, 36, 22, pr["shadow"])
    c.rect(80, 98, 32, 22, pr["base"])
    for y in range(100, 120, 4):
        c.hline(80, y, 32, pr["light"])
    c.vline(79, 98, 22, pr["dark"])
    c.vline(112, 98, 22, pr["dark"])
    oval_shadow(c, cx, 84, 66, 26, 0.45)
    c.ellipse(cx, 80, 64, 24, pr["dark"])
    c.ellipse(cx, 79, 63, 23, pr["shadow"])
    c.ellipse(cx, 78, 62, 22, pr["base"])
    c.ellipse_ring(cx, 78, 54, 18, pr["light"])
    c.ellipse_ring(cx, 78, 40, 13, pr["shadow"])
    c.ellipse(cx - 10, 72, 30, 9, pr["light"], 0.25)
    # Console and helmet.
    block(c, 80, 50, 32, 8, 6, "#3A4060", top="#4A5278")
    for i in range(6):
        c.rect(83 + i * 5, 51, 3, 2, ["#5AF0FF", "#FF5A8A", "#F2C230"][i % 3])
    hr = ramp("#9AA4C0")
    c.ellipse(96, 44, 9, 6, hr["dark"])
    c.ellipse(96, 44, 8, 5, hr["base"])
    c.ellipse(94, 42, 4, 2, hr["light"])
    c.rect(88, 46, 16, 2, hr["shadow"])
    for k in range(-3, 4):
        c.line(96 + k * 2, 38, 96 + k * 5, 26, "#5A6A9A", 0.7)

    stars = [(Rng(77).int(4, W - 4), Rng(78 + i).int(2, 44), i) for i in range(1)]
    rng = Rng(77)
    stars = []
    for i in range(70):
        x, y = rng.int(2, W - 3), rng.int(2, 60)
        if (x - cx) ** 2 / 64 ** 2 + (y - 78) ** 2 / 24 ** 2 < 1.2 or 40 < x < 152 and y > 24:
            continue
        stars.append((x, y, rng.next()))

    def animate(c: Canvas, t: float):
        # Minds lighting up all over the sphere.
        for x, y, p in stars:
            b = wave(t, 1, p)
            if b > 0.2:
                c.set(x, y, "#FFFFFF" if b > 0.8 else "#9AD8FF")
                if b > 0.85:
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        c.set(x + dx, y + dy, "#5AA8FF", 0.6)
        # A ring of light sweeping outwards from the helmet.
        r_ = 20 + t * 130
        c.ellipse_ring(cx, cy - 6, r_, r_ * 0.62, "#7AC8FF", 0.5 * (1 - t))
        c.ellipse_ring(cx, cy - 6, r_ + 2, (r_ + 2) * 0.62, "#3A7AFF", 0.3 * (1 - t))
        # Helmet glow.
        c.ellipse(96, 44, 12, 8, "#9AD8FF", 0.12 + 0.08 * wave(t, 2))
        for i in range(6):
            if wave(t, 4, i / 6) > 0:
                c.set(84 + i * 5, 55, "#5AF0FF")

    return animate


# --- more furniture ---------------------------------------------------------------------


def clouds(c: Canvas, x: int, y: int, w: int, h: int, t: float, seed: int = 1, count: int = 3,
           color: str = "#FFFFFF", alpha: float = 0.9):
    """Clouds drifting across a rect, one rect width per loop so they wrap seamlessly."""
    sub = Canvas(w, h)
    rng = Rng(seed)
    for _ in range(count):
        cx0, cy, s = rng.int(0, w - 1), rng.int(2, max(3, h // 2)), rng.int(3, 6)
        cx = (cx0 + t * w) % w
        for ox in (cx, cx - w):
            sub.ellipse(ox, cy + 1, s * 1.6, s * 0.55, shade(color, -0.12))
            sub.ellipse(ox - s * 0.6, cy, s * 0.8, s * 0.6, color)
            sub.ellipse(ox + s * 0.5, cy - 1, s * 0.9, s * 0.7, color)
    c.blit(sub, x, y, alpha)


def water(c: Canvas, x: int, y: int, w: int, h: int, t: float, color: str, seed: int = 4, density: int = 40):
    """Glints that sway and blink on still water; loops with `t`."""
    r = ramp(color)
    rng = Rng(seed)
    for _ in range(w * h // density):
        gx, gy, p, ln = rng.int(0, w - 5), rng.int(0, h - 1), rng.next(), rng.int(2, 5)
        b = wave(t, 1, p)
        if b < -0.2:
            continue
        ox = round(2 * wave(t, 1, p + 0.25))
        col = r["light"] if b > 0.5 else mix(r["base"], r["light"], 0.5)
        c.hline(x + min(w - ln, max(0, gx + ox)), y + gy, ln, col)


def bed(c: Canvas, x: int, y: int, w: int, h: int, blanket: str, frame: str = "#6B4A30", pillow: str = "#F4F0E6"):
    """Bed seen from above with its headboard against the wall; (x, y) is the headboard top."""
    fr = ramp(frame)
    br = ramp(blanket)
    soft_shadow(c, x + 3, y + h - 2, w, 5)
    soft_shadow(c, x + w, y + 8, 3, h - 8, 0.22)
    # Headboard.
    c.rect(x, y, w, 9, fr["base"])
    c.hline(x, y, w, fr["light"])
    rim(c, x + 2, y + 2, w - 4, 5, fr["shadow"])
    # Mattress, pillow and blanket.
    c.rect(x + 1, y + 9, w - 2, h - 12, "#EDE8DD")
    pr = ramp(pillow)
    c.rect(x + 4, y + 10, w - 8, 6, pr["base"])
    c.hline(x + 4, y + 15, w - 8, pr["shadow"])
    c.hline(x + 5, y + 10, w - 10, pr["light"])
    c.rect(x + 1, y + 18, w - 2, h - 21, br["base"])
    c.hline(x + 1, y + 18, w - 2, br["light"])
    c.hline(x + 1, y + 19, w - 2, br["light"])
    for k in range(y + 22, y + h - 4, 4):
        c.hline(x + 3, k, w - 6, br["shadow"])
    c.vline(x + w - 2, y + 18, h - 21, br["shadow"])
    # Footboard.
    c.rect(x, y + h - 3, w, 3, fr["shadow"])
    c.hline(x, y + h - 3, w, fr["base"])
    rim(c, x - 1, y - 1, w + 2, h + 2, fr["dark"])


def sofa(c: Canvas, x: int, y: int, w: int, color: str, cushions: int = 3):
    """Sofa facing the viewer, backrest against the wall; (x, y) is the top-left of the back."""
    r = ramp(color)
    soft_shadow(c, x + 2, y + 18, w + 2, 5)
    c.rect(x + 3, y, w - 6, 10, r["base"])
    c.hline(x + 3, y, w - 6, r["light"])
    c.rect(x + 3, y + 9, w - 6, 9, r["shadow"])
    cw = (w - 10) // cushions
    for i in range(cushions):
        cx = x + 5 + i * cw
        c.rect(cx, y + 8, cw - 1, 6, r["light"])
        c.hline(cx, y + 8, cw - 1, shade(r["light"], 0.2))
        c.vline(cx + cw - 2, y + 8, 6, r["base"])
    for ax in (x, x + w - 5):
        c.rect(ax, y + 4, 5, 14, r["base"])
        c.hline(ax, y + 4, 5, r["light"])
        c.vline(ax + 4, y + 5, 13, r["shadow"])
    rim(c, x - 1, y - 1, w + 2, 20, r["dark"])
    c.rect(x + 1, y + 18, 2, 2, "#2A1C16")
    c.rect(x + w - 3, y + 18, 2, 2, "#2A1C16")


def table(c: Canvas, x: int, y: int, w: int, d: int, h: int, color: str, top: str | None = None):
    """Table on legs; (x, y) top-left of the tabletop, `d` its depth, `h` the leg height."""
    r = ramp(color)
    soft_shadow(c, x + 3, y + d + h - 2, w, 4)
    c.rect(x, y, w, d, top or r["light"])
    c.hline(x, y, w, shade(top or r["light"], 0.2))
    c.rect(x, y + d, w, 2, r["base"])
    c.hline(x, y + d + 1, w, r["shadow"])
    for lx in (x + 1, x + w - 3):
        c.rect(lx, y + d + 2, 2, h, r["shadow"])
        c.vline(lx, y + d + 2, h, r["base"])
    rim(c, x - 1, y - 1, w + 2, d + 4, r["dark"])


def counter(c: Canvas, x: int, y: int, w: int, d: int, h: int, body: str, top: str, doors: bool = True):
    block(c, x, y, w, h, d, body, top=top)
    if doors:
        r = ramp(body)
        for dx in range(x + 2, x + w - 6, 12):
            rim(c, dx, y + d + 2, min(10, x + w - 2 - dx), h - 4, r["shadow"])
            c.rect(dx + 7 if dx % 24 else dx + 2, y + d + 5, 1, 3, "#D8D0C0")


def door(c: Canvas, x: int, y: int, w: int, h: int, color: str, frame: str):
    fr = ramp(frame)
    r = ramp(color)
    c.rect(x - 2, y - 2, w + 4, h + 2, fr["base"])
    c.hline(x - 2, y - 2, w + 4, fr["light"])
    c.rect(x, y, w, h, r["base"])
    rim(c, x + 2, y + 2, w - 4, h // 2 - 3, r["shadow"])
    rim(c, x + 2, y + h // 2 + 1, w - 4, h // 2 - 3, r["shadow"])
    c.set(x + w - 3, y + h // 2, "#E8C860")
    rim(c, x - 3, y - 3, w + 6, h + 3, fr["dark"])


def poster(c: Canvas, x: int, y: int, w: int, h: int, bg: str, fg: str, kind: str = "x"):
    c.rect(x, y, w, h, bg)
    c.hline(x, y, w, shade(bg, 0.3))
    c.dither(x + w, y + 1, 1, h, SHADOW)
    c.dither(x + 1, y + h, w, 1, SHADOW)
    if kind == "x":
        for k in range(min(w, h) - 4):
            c.set(x + 2 + k, y + 2 + k, fg)
            c.set(x + w - 3 - k, y + 2 + k, fg)
    elif kind == "band":
        c.ellipse(x + w / 2, y + h * 0.4, w * 0.3, h * 0.2, fg)
        c.hline(x + 2, y + h - 4, w - 4, fg)
        c.hline(x + 3, y + h - 2, w - 6, shade(fg, -0.3))
    elif kind == "stripes":
        for k in range(y + 2, y + h - 1, 3):
            c.hline(x + 2, k, w - 4, fg)


def wall_clock(c: Canvas, x: int, y: int, t: float | None = None):
    c.ellipse(x, y, 5, 5, "#3A2A1E")
    c.ellipse(x, y, 4, 4, "#F4EEDC")
    c.set(x, y - 3, "#3A2A1E")
    c.set(x + 3, y, "#3A2A1E")
    c.set(x, y + 3, "#3A2A1E")
    c.set(x - 3, y, "#3A2A1E")
    c.line(x, y, x, y - 2, "#2A1C16")
    c.line(x, y, x + 2, y, "#2A1C16")
    if t is not None:
        a = TAU * t - math.pi / 2
        c.line(x, y, x + round(math.cos(a) * 3), y + round(math.sin(a) * 3), "#C8302A")


def stool(c: Canvas, x: int, y: int, color: str):
    r = ramp(color)
    oval_shadow(c, x + 1, y + 8, 4, 1.5)
    c.vline(x, y + 2, 6, "#5A5A62")
    c.hline(x - 2, y + 7, 5, "#5A5A62")
    c.ellipse(x, y + 1, 3, 1.5, r["dark"])
    c.ellipse(x, y, 3, 1.5, r["base"])
    c.set(x - 1, y - 1, r["light"])


def car(c: Canvas, x: int, y: int, color: str):
    """Sports car seen from above, nose towards the viewer; (x, y) top-left, 30×58."""
    r = ramp(color)
    w, h = 30, 58
    soft_shadow(c, x + 3, y + 4, w + 2, h, 0.3)
    for wy in (y + 9, y + h - 15):
        for wx in (x - 2, x + w - 2):
            c.rect(wx, wy, 4, 10, "#1A1A20")
            c.vline(wx + (0 if wx < x else 3), wy + 1, 8, "#3A3A44")
    body = Canvas(W, H)
    for yy in range(h):
        u = yy / h
        inset = 3 if u < 0.06 or u > 0.95 else 1 if u < 0.12 or u > 0.9 else 0
        body.hline(x + inset, y + yy, w - inset * 2, r["base"])
    body.outline(r["dark"])
    c.blit(body, 0, 0)
    # Shading: left side lit, right side in shadow.
    c.vline(x + 1, y + 4, h - 8, r["light"])
    c.vline(x + w - 2, y + 4, h - 8, r["shadow"])
    # Trunk (back, far end), rear window, roof, windshield, hood.
    c.rect(x + 4, y + 2, w - 8, 6, r["light"])
    c.rect(x + 4, y + 10, w - 8, 6, "#26323E")
    c.hline(x + 5, y + 11, w - 10, "#4A6A84")
    c.rect(x + 4, y + 16, w - 8, 12, mix(r["base"], r["light"], 0.3))
    c.hline(x + 4, y + 16, w - 8, r["light"])
    c.rect(x + 3, y + 28, w - 6, 9, "#26323E")
    c.line(x + 6, y + 35, x + 12, y + 29, "#8AB0CC")
    c.line(x + 9, y + 35, x + 15, y + 29, "#5A7A94")
    c.rect(x + 3, y + 37, w - 6, 18, r["base"])
    c.vline(x + w // 2 - 4, y + 39, 14, r["light"])
    c.vline(x + w // 2 + 3, y + 39, 14, r["shadow"])
    # Headlights and mirrors.
    c.rect(x + 3, y + h - 3, 5, 2, "#FFF4C0")
    c.rect(x + w - 8, y + h - 3, 5, 2, "#FFF4C0")
    c.rect(x - 2, y + 29, 2, 2, r["shadow"])
    c.rect(x + w, y + 29, 2, 2, r["shadow"])


def ecg(c: Canvas, x: int, y: int, w: int, h: int, t: float, color: str = "#5AFF7A"):
    """Heart monitor trace scrolling to the left, two beats per loop."""
    period = w // 2
    for i in range(w):
        p = (i + t * w) % period / period
        if 0.40 < p < 0.44:
            off = -round(h * 0.4)
        elif 0.44 <= p < 0.48:
            off = round(h * 0.3)
        elif 0.6 < p < 0.68:
            off = -1
        else:
            off = 0
        c.set(x + i, y + h // 2 + off, color)
    c.vline(x + w - 1, y + 1, h - 2, color, 0.4)


# --- more rooms -------------------------------------------------------------------------


@room("garage", "Garage", (74, 66, 108, 48))
def garage(c: Canvas):
    wall(c, 36, "#8C8F98", "#5A5E68", style="tiles")
    floor_concrete(c, 36, "#7C7F88", seed=8)
    # Pegboard with tools.
    pb = ramp("#B8925E")
    c.rect(76, 6, 56, 22, pb["base"])
    rim(c, 75, 5, 58, 24, pb["dark"])
    c.hline(76, 6, 56, pb["light"])
    for yy in range(8, 27, 3):
        for xx in range(78, 131, 3):
            c.set(xx, yy, pb["shadow"])
    c.rect(80, 9, 2, 12, "#7A7F88")
    c.rect(79, 9, 4, 3, "#9AA0A8")
    c.line(88, 9, 94, 20, "#C23A2A")
    c.line(89, 9, 95, 20, "#8A2A1E")
    c.rect(100, 10, 10, 3, "#3A6A9A")
    c.rect(104, 13, 2, 8, "#5A3A22")
    c.ellipse(120, 16, 5, 5, "#2A2A30")
    c.ellipse(120, 16, 3, 3, pb["base"])
    c.rect(126, 9, 2, 14, "#9AA0A8")
    # Shelf with paint cans.
    block(c, 140, 8, 40, 2, 2, "#5A5E68", shadow=False)
    for i, col in enumerate(("#C23A2A", "#3A6A9A", "#E8C040", "#5A9A4A", "#DADADA")):
        c.rect(142 + i * 8, 3, 6, 6, col)
        c.hline(142 + i * 8, 3, 6, shade(col, 0.3))
        c.vline(147 + i * 8, 3, 6, shade(col, -0.25))
    # Workbench and red tool chest.
    counter(c, 136, 22, 32, 5, 12, "#7A5A3A", "#9A7650", doors=False)
    c.rect(140, 23, 8, 2, "#7A7F88")
    block(c, 172, 18, 14, 18, 4, "#C23A2A")
    for yy in (25, 30, 35):
        c.hline(174, yy, 10, "#8A2A1E")
        c.hline(177, yy - 2, 4, "#D8D0C0")
    # Oil stain and parking line.
    c.ellipse(46, 90, 9, 4, "#3A3A44", 0.45)
    c.ellipse(49, 91, 4, 2, "#2A2A30", 0.4)
    for y in range(40, H, 1):
        c.set(66, y, "#E8C040" if (y // 4) % 2 else "#C8A030")
    car(c, 22, 44, "#C8302A")
    # Wolverine's motorcycle against the wall, seen from the side.
    mx, my = 82, 56
    oval_shadow(c, mx + 16, my + 1, 20, 3)
    for wx in (mx + 4, mx + 28):
        c.ellipse(wx, my - 5, 6, 6, "#141418")
        c.ellipse(wx, my - 5, 4, 4, "#3A3A44")
        c.ellipse(wx, my - 5, 1.5, 1.5, "#B8BCC8")
    c.line(mx + 4, my - 5, mx + 12, my - 12, "#9AA0A8")
    c.line(mx + 28, my - 5, mx + 24, my - 16, "#B8BCC8")
    c.rect(mx + 10, my - 13, 16, 6, "#2A2A30")
    c.rect(mx + 14, my - 16, 10, 4, "#C8302A")
    c.hline(mx + 15, my - 16, 8, "#E85A4A")
    c.rect(mx + 6, my - 15, 9, 3, "#1A1A20")
    c.hline(mx + 7, my - 15, 7, "#3A3A44")
    c.rect(mx + 12, my - 8, 10, 4, "#6A6E78")
    c.hline(mx + 12, my - 8, 10, "#9AA0A8")
    c.hline(mx + 22, my - 19, 7, "#9AA0A8")
    c.rect(mx + 29, my - 15, 2, 2, "#FFF4C0")
    c.hline(mx + 4, my - 4, 10, "#B8BCC8")
    plant(c, 186, 116)


@room("courtyard", "Courtyard", (72, 60, 110, 54), frames=16)
def courtyard(c: Canvas):
    # Mansion facade: bricks, tall white windows, ivy, stone base.
    bricks(c, 0, 0, W, 30, "#9A5A44")
    for i, wx in enumerate(range(10, W, 30)):
        window(c, wx, 4, 12, 16, frame="#F0EADA", sky=("#5A7AA8", "#A8C8E8"), panes=(2, 2))
    rng = Rng(31)
    for vx in (4, 48, 140, 186):
        for k in range(40):
            c.set(vx + rng.int(-4, 4), rng.int(0, 28), rng.pick(["#3F7A3A", "#5A9A4A", "#2E5E2E"]))
    sr = ramp("#B8B0A0")
    c.rect(0, 28, W, 6, sr["base"])
    c.hline(0, 28, W, sr["light"])
    c.hline(0, 33, W, sr["dark"])
    grass(c, 0, 34, W, H - 34, seed=12)
    # Flower beds along the facade.
    for bx in range(2, W, 24):
        c.rect(bx, 34, 20, 4, "#6A4A30")
        for k in range(6):
            c.set(bx + 2 + k * 3, 34 + (k % 2), ["#FF6A8A", "#FFE27A", "#FFFFFF"][k % 3])
    # Stone paths in a cross.
    path = ramp("#D8CCB0")
    c.rect(86, 38, 20, H - 38, path["base"])
    c.rect(0, 76, W, 14, path["base"])
    rng = Rng(5)
    for yy in range(38, H, 5):
        for xx in range(86, 106, 7):
            c.hline(xx + (yy // 5) % 2 * 3, yy, 5, path["light"])
            c.set(xx + 6, yy + 2, path["shadow"])
    for xx in range(0, W, 7):
        for yy in range(77, 90, 5):
            c.hline(xx + (yy // 5) % 2 * 3, yy, 5, path["light"])
    c.hline(0, 75, W, path["shadow"])
    c.hline(0, 90, W, path["dark"])
    c.vline(85, 38, 37, path["shadow"])
    c.vline(106, 38, 37, path["dark"])
    # Benches.
    for bx in (114, 142):
        block(c, bx, 42, 20, 3, 3, "#8A5A34")
        c.rect(bx + 1, 48, 2, 3, "#3A3A44")
        c.rect(bx + 17, 48, 2, 3, "#3A3A44")
    # Fountain basin (the water is animated).
    fx, fy = 40, 80
    oval_shadow(c, fx + 4, fy + 8, 28, 8, 0.3)
    st = ramp("#C8C0B0")
    c.ellipse(fx, fy + 2, 27, 11, st["dark"])
    c.ellipse(fx, fy, 27, 11, st["base"])
    c.ellipse(fx, fy - 1, 26, 10, st["light"])
    c.ellipse(fx, fy, 22, 8, "#2E6EA8")
    c.ellipse(fx, fy + 1, 22, 7, "#3A8AC8")

    def pillar(c: Canvas):
        c.rect(fx - 3, fy - 10, 6, 10, st["base"])
        c.vline(fx - 3, fy - 10, 10, st["light"])
        c.vline(fx + 2, fy - 10, 10, st["shadow"])
        c.ellipse(fx, fy - 10, 7, 3, st["dark"])
        c.ellipse(fx, fy - 11, 7, 3, st["light"])
        c.ellipse(fx, fy - 11, 5, 2, "#3A8AC8")

    def animate(c: Canvas, t: float):
        water(c, fx - 18, fy - 4, 36, 10, t, "#3A8AC8", seed=3, density=18)
        # Rings spreading in the basin.
        for k in range(2):
            p = (t * 2 + k / 2) % 1
            c.ellipse_ring(fx, fy + 1, 6 + p * 14, 2 + p * 5, "#BFE6FF", 0.7 * (1 - p))
        # Water jet falling back on all sides.
        for k in range(10):
            p = (t * 2 + k / 10) % 1
            a = k / 10 * TAU
            dx = math.cos(a) * 12 * p
            yy = fy - 12 - 9 * math.sin(math.pi * p) + 10 * p
            c.set(round(fx + dx), round(yy), "#DFF4FF", 0.9)
        pillar(c)
        c.vline(fx, fy - 20, 8, "#DFF4FF", 0.8)
        # Trees swaying in the wind.
        tree(c, 178, 54, 1.0, sway=wave(t, 1))
        tree(c, 14, 50, 0.8, leaf="#4E8A3A", sway=wave(t, 1, 0.3))

    return animate


@room("basketball", "Basketball Court", (28, 48, 136, 64))
def basketball(c: Canvas):
    c.gradient_v(0, 0, W, 14, "#8FC8F0", "#CFEAF8", 3)
    rng = Rng(14)
    for x in range(-6, W + 8, 9):
        tree(c, x, 26 + rng.int(-2, 2), 0.7, leaf=rng.pick(["#3F7A3A", "#4E8A3A", "#356A30"]))
    # Chain-link fence.
    c.rect(0, 26, W, 2, "#7A7F88")
    for x in range(0, W, 4):
        for y in range(8, 28):
            if (x + y) % 4 == 0 or (x - y) % 4 == 0:
                c.set(x + (y % 2), y, "#9AA0A8", 0.7)
    for x in range(0, W, 32):
        c.rect(x, 6, 2, 24, "#5A5E68")
        c.set(x, 6, "#9AA0A8")
    c.hline(0, 7, W, "#6A6E78")
    grass(c, 0, 30, W, H - 30, color="#4E9A42", seed=40, flowers=False)
    # Court surface with white lines.
    court = ramp("#C8603A")
    cx0, cy0, cw, ch = 10, 34, 172, 82
    c.rect(cx0, cy0, cw, ch, court["base"])
    noise(c, cx0, cy0, cw, ch, [court["shadow"], mix(court["base"], court["light"], 0.4)], 0.06, 3)
    line = "#F4F0E6"
    c.frame(cx0 + 3, cy0 + 3, cw - 6, ch - 6, line)
    c.vline(96, cy0 + 3, ch - 6, line)
    c.ellipse_ring(96, cy0 + ch / 2, 14, 10, line)
    for side in (-1, 1):
        kx = cx0 + 3 if side < 0 else cx0 + cw - 3 - 30
        key = ramp("#2E5E9A")
        c.rect(kx + 1, cy0 + ch // 2 - 12, 29, 24, key["base"])
        c.frame(kx, cy0 + ch // 2 - 13, 31, 26, line)
        ax = cx0 + 3 if side < 0 else cx0 + cw - 4
        # Three-point arc.
        for k in range(160):
            a = -math.pi / 2 + k / 159 * math.pi
            px = ax - side * round(math.cos(a) * 48)
            py = round(cy0 + ch / 2 + math.sin(a) * 34)
            c.set(px, py, line)
        # Hoop: pole, backboard seen edge on, rim and net.
        hx = cx0 + 2 if side < 0 else cx0 + cw - 3
        bx = hx - side * 4
        oval_shadow(c, hx + 4, cy0 + ch // 2 + 4, 6, 2)
        c.rect(hx - 1, cy0 + ch // 2 - 22, 3, 26, "#5A5E68")
        c.rect(bx - 1, cy0 + ch // 2 - 30, 3, 16, "#F4F4F8")
        c.vline(bx + side, cy0 + ch // 2 - 30, 16, "#B8BCC8")
        rimx = bx - side * 5
        c.ellipse_ring(rimx, cy0 + ch // 2 - 20, 4, 1.5, "#E8602A")
        for k in range(-3, 4, 2):
            c.line(rimx + k, cy0 + ch // 2 - 19, rimx + k // 2, cy0 + ch // 2 - 14, "#F4F4F8")
    # A ball and a bench with towels.
    c.ellipse(170, 108, 3, 3, "#D8602A")
    c.line(167, 108, 173, 108, "#5A2A1A")
    block(c, 70, 32, 52, 3, 3, "#8A5A34")
    c.rect(80, 32, 6, 3, "#F4F4F8")
    c.rect(104, 32, 6, 3, "#E8C040")


@room("classroom", "Classroom", (10, 62, 86, 52), frames=16)
def classroom(c: Canvas):
    wall(c, 38, "#D8C49A", "#8A5A34", style="plain", wainscot=12, wainscot_color="#8A5A34")
    floor_wood(c, 38, "#B8864E", seed=31)
    # Chalkboard with lessons and the X logo.
    bx, by, bw, bh = 34, 6, 80, 22
    fr = ramp("#8A5A34")
    c.rect(bx - 3, by - 3, bw + 6, bh + 6, fr["base"])
    c.hline(bx - 3, by - 3, bw + 6, fr["light"])
    rim(c, bx - 4, by - 4, bw + 8, bh + 8, fr["dark"])
    c.rect(bx, by, bw, bh, "#2E4A3A")
    noise(c, bx, by, bw, bh, ["#365442", "#2A4234"], 0.15, 2)
    chalk = "#E8ECE0"
    rng = Rng(8)
    for row in range(4):
        x = bx + 4
        while x < bx + 46:
            ln = rng.int(2, 6)
            c.hline(x, by + 4 + row * 5, ln, chalk)
            if rng.next() < 0.4:
                c.set(x + 1, by + 3 + row * 5, chalk)
            x += ln + 2
    for k in range(12):
        c.set(bx + 56 + k, by + 4 + k, chalk)
        c.set(bx + 67 - k, by + 4 + k, chalk)
    c.ellipse_ring(bx + 61.5, by + 9.5, 9, 9, chalk)
    c.rect(bx + 4, by + bh, 20, 2, fr["light"])
    c.rect(bx + 8, by + bh - 1, 4, 1, "#FFFFFF")
    c.rect(bx + 14, by + bh - 1, 6, 2, "#C8B89A")
    # Windows with drifting clouds (animated).
    for wx in (132, 160):
        window(c, wx, 7, 20, 18, sky=("#6AB0E8", "#BFE2F8"), panes=(2, 2))
    light_patch(c, 120, 39, 60, 22, skew=-18, alpha=0.1)
    poster(c, 8, 8, 14, 18, "#2E5E9A", "#F2C230")
    # Teacher's desk.
    block(c, 12, 46, 36, 10, 6, "#7A4A2A", top="#9A6A40")
    c.rect(16, 47, 8, 3, "#F4F0E6")
    c.ellipse(40, 47, 3, 2, "#C8302A")
    c.set(41, 45, "#4E9A3E")
    # Student desks facing the board, each with its chair.
    for row in range(3):
        for col in range(3):
            dx, dy = 106 + col * 27, 52 + row * 22
            table(c, dx, dy, 20, 6, 5, "#9A9AA8", top="#C8A070")
            c.rect(dx + 3, dy + 1, 6, 3, "#F4F0E6")
            chair(c, dx + 6, dy + 16, "#3A6A9A", facing="back")

    def animate(c: Canvas, t: float):
        for wx in (132, 160):
            clouds(c, wx, 7, 20, 10, t, seed=wx, count=2)
            c.vline(wx + 9, 7, 18, "#D8CCB4")
            c.vline(wx + 10, 7, 18, "#B0A48C")
        wall_clock(c, 124, 14, t)

    return animate


@room("library", "Library", (12, 66, 168, 48))
def library(c: Canvas):
    wall(c, 42, "#4A3428", "#6B4226", style="plain")
    floor_wood(c, 42, "#6E4428", plank=4, seed=41)
    for i, bx in enumerate((4, 42, 116, 154)):
        bookcase(c, bx, 4, 34, 46, wood="#5E3A22", seed=10 + i)
    # Arched window between the bookcases.
    window(c, 84, 8, 24, 26, frame="#C8B898", sky=("#2D3E73", "#8A8FC8"), panes=(2, 3))
    c.ellipse(96, 8, 12, 6, "#2D3E73")
    c.ellipse_ring(96, 8, 14, 7, "#C8B898")
    light_patch(c, 76, 43, 34, 24, skew=-4, alpha=0.08)
    # Rolling ladder.
    lr = ramp("#9A6A40")
    c.line(130, 4, 126, 50, lr["base"])
    c.line(140, 4, 136, 50, lr["base"])
    for y in range(8, 50, 6):
        c.hline(130 - (y - 4) // 11, y, 10, lr["light"])
    # Reading table with banker lamps and open books.
    table(c, 62, 48, 68, 9, 5, "#5E3A22", top="#7A4C2C")
    for lx in (72, 118):
        c.rect(lx, 46, 1, 4, "#C9A04A")
        c.rect(lx - 3, 44, 7, 2, "#2E7A4E")
        c.hline(lx - 3, 44, 7, "#5ABA7E")
        lamp_glow(c, lx, 50, 10, alpha=0.08)
    c.rect(86, 50, 12, 5, "#F4EEDC")
    c.vline(92, 50, 5, "#C8BCA0")
    c.rect(102, 51, 6, 4, "#8E2F2F")
    for ax in (44, 140):
        ch = ramp("#7A2A2A")
        soft_shadow(c, ax + 1, 62, 16, 4)
        c.rect(ax, 46, 14, 14, ch["base"])
        c.hline(ax, 46, 14, ch["light"])
        c.rect(ax - 2, 51, 3, 10, ch["shadow"])
        c.rect(ax + 13, 51, 3, 10, ch["shadow"])
        c.rect(ax + 1, 55, 12, 5, ch["light"])
        rim(c, ax - 3, 45, 20, 17, ch["dark"])
    # Globe.
    c.vline(182, 54, 8, "#6B4226")
    c.hline(179, 61, 7, "#6B4226")
    c.ellipse(182, 50, 5, 5, "#2E6EA8")
    c.ellipse(181, 49, 2, 3, "#5A9A4A")
    c.set(180, 48, "#BFE6FF")
    rug(c, 40, 72, 112, 40, "#2E3E6A", "#B8902E")


@room("kitchen", "Kitchen", (10, 64, 106, 50), frames=16)
def kitchen(c: Canvas):
    wall(c, 36, "#ECE8DE", "#9AA0A8", style="tiles")
    floor_tiles(c, 36, "#E8E4DA", "#3A3A44", size=12)
    # Upper cabinets.
    for x in range(4, 112, 18):
        panel(c, x, 5, 16, 11, "#6A8AA8", inset=False)
        c.rect(x + 7, 12, 2, 2, "#D8D0C0")
    rim(c, 3, 4, 110, 13, ramp("#6A8AA8")["dark"])
    # Window over the sink.
    window(c, 46, 6, 24, 12, sky=("#7EC8F0", "#CDEBFA"), panes=(2, 1))
    # Counter with sink and stove.
    counter(c, 2, 26, 112, 6, 14, "#6A8AA8", "#D8D4CC")
    c.rect(48, 27, 20, 4, "#9AA0A8")
    c.rect(49, 28, 18, 3, "#6A7078")
    c.vline(58, 23, 5, "#C8CCD8")
    c.hline(58, 23, 3, "#C8CCD8")
    sr = ramp("#3A3A44")
    c.rect(82, 26, 22, 6, sr["light"])
    for bx in (86, 98):
        c.ellipse_ring(bx, 28, 3, 1.5, "#1A1A20")
    c.rect(84, 34, 18, 9, "#2A2A30")
    c.rect(86, 36, 14, 5, "#4A3A30")
    # Pot on the stove (the steam is animated).
    c.rect(94, 22, 9, 6, "#B8BCC8")
    c.hline(94, 22, 9, "#E8ECF4")
    c.vline(102, 22, 6, "#8A8E98")
    c.hline(92, 24, 2, "#2A2A30")
    # Cutting board, fruit bowl, knife block.
    c.rect(14, 27, 12, 4, "#C8945A")
    c.ellipse(34, 28, 5, 2, "#F4F0E6")
    c.set(32, 27, "#E83A3A")
    c.set(35, 27, "#FFD23A")
    c.set(34, 26, "#7AC83A")
    c.rect(74, 24, 4, 5, "#5A3A22")
    # Fridge.
    block(c, 118, 4, 20, 34, 3, "#E4E8EE")
    c.hline(119, 18, 18, "#A8B0BC")
    c.vline(135, 10, 6, "#8A929E")
    c.vline(135, 21, 8, "#8A929E")
    c.rect(122, 9, 4, 4, "#E8C040")
    c.rect(128, 11, 3, 3, "#E85A8A")
    # Kitchen island with stools.
    counter(c, 128, 68, 56, 10, 12, "#6A8AA8", "#D8D4CC")
    c.ellipse(146, 72, 6, 3, "#F4F0E6")
    c.ellipse(146, 71, 4, 1.5, "#E8C070")
    c.rect(168, 70, 3, 6, "#7AB8E8")
    for sx in (138, 156, 174):
        stool(c, sx, 98, "#C8302A")
    plant(c, 152, 50)
    plant(c, 184, 54)

    def animate(c: Canvas, t: float):
        for k in range(5):
            p = (t + k / 5) % 1
            x = 98 + round(2 * wave(p, 1, k * 0.2))
            c.ellipse(x, 20 - p * 16, 1 + p * 2, 1 + p * 1.5, "#FFFFFF", 0.45 * (1 - p))
        # Dripping tap.
        p = (t * 2) % 1
        c.set(58, 26 + round(p * 3), "#9AD8FF" if p < 0.8 else "#DFF4FF")

    return animate


@room("beast-lab", "Beast's Lab", (12, 66, 168, 48), frames=16)
def beast_lab(c: Canvas):
    wall(c, 38, "#4A5A6A", "#2E3A48", style="metal")
    floor_tiles(c, 38, "#7E8A9A", "#727E8E", size=16)
    # Chalkboard full of formulas.
    c.rect(42, 6, 44, 20, "#2A3A30")
    rim(c, 41, 5, 46, 22, "#6B4A30")
    rng = Rng(66)
    for row in range(4):
        x = 45
        while x < 82:
            ln = rng.int(2, 5)
            c.hline(x, 9 + row * 4, ln, "#DCE4D8")
            x += ln + rng.int(1, 3)
    c.ellipse_ring(76, 18, 4, 4, "#DCE4D8")
    # Monitors on the wall (the DNA helix is animated).
    for mx in (132, 160):
        screen(c, mx, 6, 22, 16, glow="#5AF0A0")
    # Specimen tank.
    tx, ty = 18, 6
    c.rect(tx - 8, ty + 40, 18, 6, "#3A4250")
    c.hline(tx - 8, ty + 40, 18, "#5A6478")
    c.rect(tx - 7, ty, 16, 40, "#1E5A3A")
    c.rect(tx - 9, ty - 3, 20, 4, "#5A6478")
    c.hline(tx - 9, ty - 3, 20, "#8A94A8")
    c.vline(tx - 6, ty + 1, 38, "#7AE8A8")
    c.vline(tx + 6, ty + 1, 38, "#0E3A24")
    c.ellipse(tx, ty + 22, 3, 5, "#3A8A5A")
    rim(c, tx - 8, ty - 1, 18, 42, "#1A2A2A")
    # Lab bench with glassware and a microscope.
    counter(c, 34, 32, 90, 7, 13, "#5A6478", "#D8DEE6")
    for i, (bx, col) in enumerate(((44, "#5AF0A0"), (54, "#FF5A8A"), (62, "#5AB8FF"), (100, "#F2C230"), (110, "#B46BFF"))):
        c.rect(bx, 30, 5, 6, mix(col, "#FFFFFF", 0.6))
        c.rect(bx, 32, 5, 4, col)
        c.rect(bx + 1, 27, 3, 3, mix(col, "#FFFFFF", 0.7))
        c.vline(bx, 30, 6, "#FFFFFF")
    c.rect(78, 28, 3, 7, "#2A2A30")
    c.rect(76, 34, 8, 2, "#3A3A44")
    c.rect(79, 26, 5, 3, "#5A5A62")
    c.rect(88, 33, 9, 3, "#F4F0E6")
    c.rect(90, 32, 9, 3, "#F8F4EA")
    # Book piles on the floor and a stool.
    for px, py in ((150, 54), (160, 56)):
        for k in range(4):
            col = ["#8E2F2F", "#2F4F8E", "#C9A34A", "#3F7A3A"][(k + px) % 4]
            c.rect(px + (k % 2), py - k * 3, 10, 3, col)
            c.hline(px + (k % 2), py - k * 3, 10, shade(col, 0.3))
    stool(c, 70, 52, "#3A6A9A")
    plant(c, 184, 60, pot="#5A6478")

    rng = Rng(6)
    bubbles = [(rng.int(-5, 5), rng.next()) for _ in range(7)]

    def animate(c: Canvas, t: float):
        for bx_, p in bubbles:
            q = (t + p) % 1
            c.set(tx + bx_ + round(wave(q, 2) * 0.6), ty + 38 - round(q * 36), "#BFFFD8", 0.8)
        for i, bx in enumerate((44, 54, 62, 100, 110)):
            q = (t * 2 + i * 0.27) % 1
            c.set(bx + 2, 31 - round(q * 4), "#FFFFFF", 1 - q)
        for k, mx in enumerate((132, 160)):
            for y in range(16):
                a = TAU * (y / 16 + t) + k
                x1 = mx + 11 + round(6 * math.sin(a))
                x2 = mx + 11 - round(6 * math.sin(a))
                front = math.cos(a) > 0
                c.set(x1, 6 + y, "#5AF0A0" if front else "#2A7A50")
                c.set(x2, 6 + y, "#2A7A50" if front else "#5AF0A0")
                if y % 3 == 0:
                    c.line(min(x1, x2), 6 + y, max(x1, x2), 6 + y, "#3AB878", 0.5)
            scanlines(c, mx, 6, 22, 16)

    return animate


@room("infirmary", "Infirmary", (10, 76, 172, 40), frames=16)
def infirmary(c: Canvas):
    wall(c, 34, "#CFE8DE", "#8ABAA8", style="tiles")
    floor_tiles(c, 34, "#E4EEEA", "#D4E2DC", size=16)
    for i, bx in enumerate((14, 78, 142)):
        # Curtain rail and half-drawn curtain.
        c.hline(bx - 6, 4, 46, "#9AA0A8")
        cr = ramp("#8AC8D8")
        for k in range(0, 8, 2):
            c.vline(bx - 6 + k, 5, 22, cr["base"] if k % 4 else cr["shadow"])
        bed(c, bx, 22, 34, 44, "#4A7AB8", frame="#B8C0CC")
    # Heart monitor above the middle bed (the trace is animated), IV stand.
    screen(c, 118, 6, 18, 12, glow="#5AFF7A")
    c.vline(64, 10, 50, "#B8C0CC")
    c.hline(60, 10, 9, "#B8C0CC")
    c.rect(60, 11, 5, 8, "#DDEEFF")
    c.rect(61, 15, 3, 4, "#BFE0FF")
    c.hline(61, 59, 7, "#8A929E")
    # Medicine cabinet with a red cross.
    block(c, 176, 6, 14, 18, 3, "#F4F8FA")
    c.rect(181, 12, 4, 10, "#D83A3A")
    c.rect(178, 15, 10, 4, "#D83A3A")
    plant(c, 186, 70)

    def animate(c: Canvas, t: float):
        ecg(c, 118, 6, 18, 12, t)
        scanlines(c, 118, 6, 18, 12)
        c.set(64, 20 + round((t * 3 % 1) * 6), "#BFE0FF", 0.9)

    return animate


@room("pool", "Pool", (10, 88, 172, 28), frames=16)
def pool(c: Canvas):
    wall(c, 30, "#E8F0F4", "#7AA8C8", style="tiles")
    window(c, 20, 6, 40, 14, sky=("#7EC8F0", "#CDEBFA"), panes=(4, 1), frame="#F4F8FA")
    window(c, 132, 6, 40, 14, sky=("#7EC8F0", "#CDEBFA"), panes=(4, 1), frame="#F4F8FA")
    poster(c, 86, 6, 20, 16, "#2E5E9A", "#F2C230")
    floor_tiles(c, 30, "#E8E0D0", "#DCD2C0", size=10)
    # The pool: coping, tiled walls seen at the far edge, then the water.
    px, py, pw, ph = 14, 40, 164, 40
    soft_shadow(c, px - 2, py - 2, pw + 4, ph + 4, 0.15)
    c.rect(px - 3, py - 3, pw + 6, ph + 6, "#F4F0E8")
    rim(c, px - 4, py - 4, pw + 8, ph + 8, "#B8B0A0")
    c.rect(px, py, pw, ph, "#2E8AC8")
    c.rect(px, py, pw, 5, "#1E6AA8")
    for x in range(px, px + pw, 4):
        c.vline(x, py, 5, "#3A9AD8")
    c.gradient_v(px, py + 5, pw, ph - 5, "#3AA0E0", "#2A7AC0", 4)
    # Lane ropes.
    for ly in (py + 15, py + 28):
        for x in range(px, px + pw):
            c.set(x, ly, "#F4F4F8" if (x // 3) % 2 else "#E83A3A")
    def fixtures(c: Canvas, bounce: int = 0):
        """Ladder and diving board, drawn over the water."""
        for lx in (px + pw - 18, px + pw - 10):
            c.vline(lx, py - 6, 12, "#C8CCD8")
            c.set(lx, py - 7, "#E8ECF4")
        for ry in (py + 1, py + 4):
            c.hline(px + pw - 18, ry, 9, "#C8CCD8")
        c.rect(px + 2, py + 2, 6, 8, "#8A929E")
        block(c, px + 6, py + 6 + bounce, 26, 2, 2, "#3A8AC8", top="#F4F4F8", shadow=False)

    fixtures(c)
    # Towels and a lounge chair on the deck.
    c.rect(150, 31, 12, 6, "#E83A6A")
    c.hline(150, 33, 12, "#F4F4F8")
    c.rect(166, 31, 12, 6, "#F2C230")
    plant(c, 8, 38)
    plant(c, 184, 38)

    def animate(c: Canvas, t: float):
        water(c, px, py + 6, pw, ph - 7, t, "#3AA0E0", seed=9, density=24)
        # Caustic squiggles drifting on the pool floor.
        for k in range(14):
            bx = px + 6 + (k * 23) % (pw - 12)
            by = py + 10 + (k * 7) % (ph - 14)
            off = round(3 * wave(t, 1, k * 0.13))
            c.line(bx + off, by, bx + off + 3, by + 1, "#7ACCF4", 0.6)
            c.line(bx + off + 3, by + 1, bx + off + 6, by, "#7ACCF4", 0.6)
        fixtures(c, 1 if wave(t, 2) > 0.7 else 0)

    return animate


@room("greenhouse", "Storm's Greenhouse", (12, 66, 168, 48), frames=16)
def greenhouse(c: Canvas):
    # Glass wall: sky and garden behind white mullions.
    c.gradient_v(0, 0, W, 38, "#7EC0EA", "#D4F0FA", 4)
    rng = Rng(21)
    for x in range(-8, W + 8, 14):
        tree(c, x + rng.int(-3, 3), 40, 0.75, leaf="#5A9A4A")
    c.rect(0, 0, W, 38, "#E8F6FF", 0.18)
    for x in range(0, W, 24):
        c.vline(x, 0, 38, "#F4F8FA")
        c.vline(x + 1, 0, 38, "#C8D4DC")
    for y in (0, 12, 25):
        c.hline(0, y, W, "#F4F8FA")
        c.hline(0, y + 1, W, "#C8D4DC")
    c.rect(0, 34, W, 4, "#D8DEE4")
    c.hline(0, 37, W, "#8A949E")
    # Terracotta floor.
    floor_tiles(c, 38, "#C88A5A", "#BA7C4E", size=14)
    # Raised planters along the glass.
    for i, bx in enumerate(range(4, W - 20, 47)):
        block(c, bx, 40, 42, 8, 5, "#8A5A34", top="#5A3A22")
        noise(c, bx + 1, 41, 40, 3, ["#4A2E1A", "#6A4A2E"], 0.4, i)
    rng = Rng(23)
    for x in range(10, W - 10, 9):
        kind = rng.pick(["leafy", "tomato", "flower", "sunflower", "leafy"])
        leaf = ramp(rng.pick(["#4E9A3E", "#3F8A5A", "#6AAA3E"]))
        h = rng.int(8, 14) if kind != "sunflower" else 20
        c.vline(x, 44 - h, h, leaf["shadow"])
        for k in range(0, h - 2, 3):
            side = 1 if k % 6 else -2
            c.rect(x + side, 42 - k, 2, 2, leaf["base"])
            c.set(x + (side + 1 if side > 0 else side), 42 - k, leaf["light"])
        if kind == "tomato":
            for k in range(3):
                c.set(x + (k % 2) * 3 - 1, 40 - k * 3, "#E83A3A")
        elif kind == "flower":
            c.ellipse(x, 44 - h, 2, 2, rng.pick(["#FF6A8A", "#B7A2FF", "#FFFFFF"]))
            c.set(x, 44 - h, "#FFE27A")
        elif kind == "sunflower":
            c.ellipse(x, 44 - h, 3, 3, "#FFD23A")
            c.ellipse(x, 44 - h, 1.5, 1.5, "#6B4226")
    # Potted ferns on the sides.
    plant(c, 8, 116, leaf="#3F8A5A")
    plant(c, 184, 116, leaf="#4E9A3E")
    light_patch(c, 30, 56, 40, 50, skew=-20, alpha=0.08)
    light_patch(c, 110, 56, 40, 50, skew=-20, alpha=0.08)
    # Watering can.
    c.rect(150, 58, 8, 6, "#5A9A6A")
    c.line(158, 60, 162, 57, "#5A9A6A")
    c.hline(150, 58, 8, "#7ACA8A")

    def animate(c: Canvas, t: float):
        # Flowers nodding and two butterflies on looping paths.
        for k, x in enumerate(range(16, W - 10, 24)):
            dy = 1 if wave(t, 1, k * 0.2) > 0.5 else 0
            col = ["#FF6A8A", "#FFE27A", "#B7A2FF", "#FFFFFF"][k % 4]
            c.set(x, 30 - dy, col)
            c.set(x + 1, 31 - dy, col)
        # Storm keeps a little rain cloud over the second planter.
        cx_, cy_ = 72 + round(2 * wave(t, 1)), 12
        for dx, dy, r_ in ((-6, 1, 4), (0, -1, 5), (6, 1, 4)):
            c.ellipse(cx_ + dx, cy_ + dy + 1, r_, r_ * 0.7, "#8A94A8")
            c.ellipse(cx_ + dx, cy_ + dy, r_, r_ * 0.7, "#C8D0DC")
        for k in range(8):
            p = (t * 2 + k / 8) % 1
            x = cx_ - 8 + (k * 5) % 17
            c.vline(x, cy_ + 4 + round(p * 22), 2, "#7AB8FF", 0.8)
        for k, (ox, oy, col) in enumerate(((60, 60, "#FFB43A"), (140, 52, "#7AB8FF"))):
            bx = ox + round(24 * math.sin(TAU * t + k))
            by = oy + round(8 * math.sin(TAU * 2 * t + k))
            flap = int(t * 16) % 2
            c.set(bx, by, "#2A2A30")
            c.set(bx - 1, by - flap, col)
            c.set(bx + 1, by - flap, col)
            if not flap:
                c.set(bx - 1, by - 1, col)
                c.set(bx + 1, by - 1, col)

    return animate


@room("gym", "Gym", (68, 64, 114, 50), frames=16)
def gym(c: Canvas):
    wall(c, 36, "#5A6A8A", "#3A4258", style="plain", wainscot=10, wainscot_color="#3A4258")
    # Mirrors.
    for mx in range(70, 150, 26):
        c.rect(mx, 6, 22, 18, "#A8C0D8")
        c.line(mx + 3, 22, mx + 12, 8, "#E0ECF8")
        c.line(mx + 8, 22, mx + 17, 8, "#C8DCEC")
        rim(c, mx - 1, 5, 24, 20, "#2A3040")
    # X logo.
    c.ellipse(170, 15, 9, 9, "#2A3040")
    c.ellipse(170, 15, 8, 8, "#F2C230")
    for k in range(-5, 6):
        c.set(170 + k, 15 + k, "#2A3040")
        c.set(170 + k, 15 - k, "#2A3040")
        c.set(171 + k, 15 + k, "#2A3040")
        c.set(171 + k, 15 - k, "#2A3040")
    # Rubber floor and a blue mat.
    fr = ramp("#3E3E4A")
    c.rect(0, 36, W, H - 36, fr["base"])
    noise(c, 0, 36, W, H - 36, [fr["shadow"], fr["light"]], 0.05, 7)
    for y in range(36, H, 20):
        c.hline(0, y, W, fr["shadow"])
    mr = ramp("#2E5E9A")
    c.rect(76, 72, 100, 40, mr["base"])
    c.hline(76, 72, 100, mr["light"])
    rim(c, 75, 71, 102, 42, mr["dark"])
    c.ellipse_ring(126, 92, 14, 9, "#F4F0E6")
    # Weight rack with dumbbells.
    block(c, 8, 22, 44, 10, 4, "#2A2A30")
    for i in range(6):
        col = ["#C8302A", "#3A6A9A", "#E8C040"][i % 3]
        c.rect(11 + i * 7, 23, 5, 3, col)
        c.hline(11 + i * 7, 23, 5, shade(col, 0.3))
    # Treadmill.
    block(c, 150, 40, 30, 8, 3, "#3A3A44", top="#2A2A30")
    c.rect(174, 32, 4, 10, "#5A5E68")
    c.rect(170, 30, 12, 4, "#2A2A30")
    c.set(175, 31, "#5AF0A0")
    # Bench press.
    block(c, 16, 84, 30, 6, 4, "#C8302A", top="#2A2A30")
    c.vline(12, 72, 14, "#9AA0A8")
    c.vline(50, 72, 14, "#9AA0A8")
    c.hline(6, 74, 52, "#C8CCD8")
    for wx in (6, 52):
        c.rect(wx, 69, 4, 10, "#2A2A30")
        c.vline(wx, 69, 10, "#5A5A62")

    def animate(c: Canvas, t: float):
        # Punching bag swinging on its chain.
        swing = wave(t, 1)
        topx, topy = 36, 2
        bx = topx + round(swing * 4)
        c.line(topx, topy, bx, 30, "#9AA0A8")
        oval_shadow(c, 36 + round(swing * 6), 64, 8, 2.5)
        br = ramp("#B83A2A")
        c.rect(bx - 6, 30, 12, 26, br["base"])
        c.vline(bx - 6, 31, 24, br["light"])
        c.vline(bx + 5, 31, 24, br["shadow"])
        c.hline(bx - 6, 34, 12, br["dark"])
        c.hline(bx - 6, 51, 12, br["dark"])
        rim(c, bx - 7, 29, 14, 28, br["dark"])
        # Treadmill belt.
        for k in range(4):
            c.vline(152 + (k * 7 + int(t * 16)) % 26, 41, 6, "#4A4A54")

    return animate


@room("lounge", "Lounge", (10, 70, 172, 44), frames=16)
def lounge(c: Canvas):
    wall(c, 40, "#C89A6A", "#6B4226", style="stripes", wainscot=13, wainscot_color="#6B4226")
    floor_wood(c, 40, "#9A6A40", seed=51)
    rug(c, 36, 74, 120, 38, "#3A5A7A", "#D8B860")
    window(c, 14, 7, 28, 20, sky=("#F0A870", "#FCE0B0"), panes=(2, 2), curtains="#5A7A4A",
           view=lambda c, x, y, w, h: c.ellipse(x + w * 0.7, y + h * 0.6, 3, 3, "#FFF4C8"))
    light_patch(c, 10, 42, 36, 24, skew=10, alpha=0.1)
    frame_picture(c, 80, 8, 30, 14, landscape)
    # Sofa against the wall and a coffee table.
    sofa(c, 62, 32, 66, "#8A3A3A")
    table(c, 76, 58, 38, 6, 3, "#5E3A22", top="#7A4C2C")
    c.rect(82, 59, 6, 3, "#F4EEDC")
    c.ellipse(104, 60, 2, 1.5, "#F4F4F8")
    c.set(104, 59, "#7A4A2A")
    # TV on a low cabinet.
    block(c, 140, 40, 40, 10, 5, "#4A3A30")
    screen(c, 146, 18, 28, 18, glow="#5AB8FF")
    c.rect(158, 37, 4, 3, "#2A2D3A")
    # Floor lamp and bookshelf.
    c.vline(52, 22, 26, "#3A3A44")
    c.hline(49, 48, 7, "#3A3A44")
    c.rect(48, 18, 9, 5, "#F4E4B8")
    c.hline(48, 18, 9, "#FFF4D0")
    lamp_glow(c, 52, 30, 16, alpha=0.08)
    plant(c, 186, 66, 1)
    plant(c, 8, 60, 1)

    def animate(c: Canvas, t: float):
        # A cartoon on the TV: bouncing ball over a cycling sky.
        sx, sy, sw, sh = 146, 18, 28, 18
        hue = ["#5AB8FF", "#7AC8FF", "#5AB8FF", "#4AA8F0"][int(t * 4) % 4]
        c.rect(sx, sy, sw, sh, hue)
        c.rect(sx, sy + 12, sw, 6, "#5AAA4A")
        bx = sx + 4 + round((sw - 8) * (0.5 + 0.5 * wave(t, 1)))
        by = sy + 10 - round(7 * abs(wave(t, 2)))
        c.ellipse(bx, by, 2, 2, "#E83A3A")
        scanlines(c, sx, sy, sw, sh)
        lamp_glow(c, 160, 44, 16, hue, 0.06)

    return animate


@room("dorm", "Dormitory", (12, 72, 168, 42))
def dorm(c: Canvas):
    wall(c, 40, "#9FB8D0", "#F0EADA", style="plain")
    floor_wood(c, 40, "#B8925E", seed=61)
    window(c, 80, 6, 32, 20, sky=("#7EC8F0", "#CDEBFA"), panes=(2, 2), curtains="#E8C040")
    light_patch(c, 78, 41, 36, 26, skew=-8, alpha=0.1)
    poster(c, 40, 8, 14, 18, "#2A2A30", "#E83A3A", "band")
    poster(c, 58, 10, 12, 14, "#F2C230", "#2E5E9A")
    poster(c, 124, 8, 16, 12, "#2E5E9A", "#F4F4F8", "stripes")
    poster(c, 144, 10, 10, 14, "#E85A8A", "#FFFFFF")
    bed(c, 8, 28, 30, 44, "#3A6AB8")
    bed(c, 154, 28, 30, 44, "#C84A6A")
    # Desk with a laptop and a chair.
    table(c, 80, 44, 32, 8, 5, "#8A5A34", top="#A87848")
    c.rect(90, 42, 12, 6, "#3A3A44")
    c.rect(91, 43, 10, 4, "#5AB8FF")
    c.rect(89, 48, 14, 2, "#9AA0A8")
    c.rect(104, 45, 3, 3, "#F4F4F8")
    chair(c, 92, 60, "#3A6A9A", facing="back")
    # Nightstands, lamp, guitar, backpack.
    block(c, 42, 44, 10, 8, 4, "#8A5A34")
    c.rect(45, 40, 4, 4, "#F4E4B8")
    block(c, 140, 44, 10, 8, 4, "#8A5A34")
    c.ellipse(146, 42, 3, 2, "#E8C040")
    c.ellipse(60, 54, 5, 6, "#A86A2E")
    c.ellipse(60, 46, 3, 3, "#A86A2E")
    c.vline(60, 34, 12, "#5A3A22")
    c.ellipse(60, 52, 1.5, 1.5, "#2A1C16")
    c.rect(126, 52, 9, 10, "#3A7A4A")
    c.hline(126, 52, 9, "#5A9A6A")
    c.rect(128, 56, 5, 3, "#2E5E3A")
    rug(c, 44, 76, 104, 34, "#E8C040", "#C84A6A")


@room("rec-room", "Rec Room", (10, 72, 172, 42), frames=16)
def rec_room(c: Canvas):
    wall(c, 40, "#3A2E4A", "#221A2E", style="stripes", wainscot=12, wainscot_color="#2A2236")
    # Patterned carpet.
    cr = ramp("#5A2A3A")
    c.rect(0, 40, W, H - 40, cr["base"])
    for y in range(44, H, 8):
        for x in range((y // 8 % 2) * 8, W, 16):
            c.set(x, y, cr["light"])
            c.set(x + 1, y + 1, "#C89A3A")
            c.set(x - 1, y + 1, "#C89A3A")
            c.set(x, y + 2, cr["light"])
    c.dither(0, 40, W, 2, SHADOW)
    # Dartboard.
    c.ellipse(130, 16, 7, 7, "#1A1A20")
    c.ellipse(130, 16, 6, 6, "#E8DCC0")
    c.ellipse(130, 16, 4, 4, "#C8302A")
    c.ellipse(130, 16, 2, 2, "#2E7A4E")
    c.set(130, 16, "#C8302A")
    # Arcade cabinets (screens animated).
    for i, ax in enumerate((8, 30)):
        col = ["#C8302A", "#2E5E9A"][i]
        block(c, ax, 8, 18, 36, 4, col)
        c.rect(ax + 2, 13, 14, 11, "#0A0A14")
        c.rect(ax + 2, 26, 14, 4, "#2A2A30")
        c.set(ax + 5, 27, "#E83A3A")
        c.set(ax + 9, 28, "#F2C230")
        c.set(ax + 12, 28, "#5AF0A0")
    # Pool table.
    px, py = 64, 44
    table(c, px, py, 64, 22, 4, "#5E3A22", top="#5E3A22")
    c.rect(px + 3, py + 3, 58, 16, "#2E7A4E")
    c.hline(px + 3, py + 3, 58, "#1E5A3A")
    for hx, hy in ((px + 2, py + 2), (px + 31, py + 1), (px + 60, py + 2), (px + 2, py + 18), (px + 31, py + 19), (px + 60, py + 18)):
        c.rect(hx, hy, 2, 2, "#0E0E12")
    for k, (bx, by) in enumerate(((px + 44, py + 10), (px + 47, py + 8), (px + 47, py + 12), (px + 50, py + 6), (px + 50, py + 10), (px + 50, py + 14))):
        col = ["#F2C230", "#2E5E9A", "#C8302A", "#6A3A7A", "#E8602A", "#1A1A20"][k]
        c.rect(bx, by, 2, 2, col)
        c.set(bx, by, shade(col, 0.4))
    c.rect(px + 16, py + 10, 2, 2, "#F4F4F8")
    c.line(px + 2, py + 15, px + 15, py + 11, "#C8A070")
    # Jukebox.
    block(c, 164, 10, 22, 32, 3, "#6A3A2A")
    c.ellipse(175, 16, 9, 6, "#C89A3A")
    c.rect(168, 18, 14, 8, "#2A1A14")
    c.rect(168, 28, 14, 10, "#8A5A34")
    for k in range(169, 182, 2):
        c.vline(k, 29, 8, "#5A3A22")
    plant(c, 156, 44)

    def animate(c: Canvas, t: float):
        # Neon X flickering above the pool table.
        on = not (0.40 < t < 0.45 or 0.50 < t < 0.53)
        col = "#FF4FD8" if on else "#5A2A50"
        for k in range(14):
            c.set(89 + k, 8 + k, col)
            c.set(102 - k, 8 + k, col)
        if on:
            for k in range(14):
                c.set(89 + k, 9 + k, col, 0.35)
                c.set(102 - k, 9 + k, col, 0.35)
            lamp_glow(c, 96, 15, 18, "#FF4FD8", 0.07)
        # Arcade attract modes.
        for i, ax in enumerate((8, 30)):
            sx, sy = ax + 2, 13
            if i == 0:
                for k in range(3):
                    x = sx + (k * 5 + int(t * 16)) % 14
                    c.rect(x, sy + 2 + k * 3, 2, 2, ["#F2C230", "#5AF0A0", "#FF5A8A"][k])
            else:
                c.rect(sx + 1, sy + 8, 3, 2, "#5AF0FF")
                for k in range(4):
                    c.set(sx + 2 + k * 3, sy + 1 + (k + int(t * 16)) % 6, "#F4F4F8")
                c.set(sx + 2, sy + 8 - int(t * 16) % 8, "#FF5A3A")
            scanlines(c, sx, sy, 14, 11)
        # Jukebox lights cycling.
        for k in range(7):
            on = (k + int(t * 16)) % 4 == 0
            c.set(167 + k * 2, 13 + abs(3 - k) // 2, "#FFF4C8" if on else "#E8602A")

    return animate


@room("refectory", "Dining Hall", (10, 72, 172, 42))
def refectory(c: Canvas):
    wall(c, 40, "#B8A88A", "#6B4226", style="plain", wainscot=14, wainscot_color="#5E3A22")
    floor_tiles(c, 40, "#9A8A74", "#8A7A64", size=16)
    for wx in (20, 84, 148):
        window(c, wx, 6, 24, 18, frame="#E8DCC4", sky=("#7EC8F0", "#CDEBFA"), panes=(2, 3))
        light_patch(c, wx - 4, 41, 30, 30, skew=-10, alpha=0.08)
    for bx in (60, 124):
        c.rect(bx, 4, 10, 18, "#2E4A8A")
        c.rect(bx + 3, 8, 4, 4, "#F2C230")
        for k in range(10):
            c.set(bx + k, 22 + (k % 2), "#2E4A8A")
    # Long table with benches.
    block(c, 14, 44, 164, 4, 3, "#6B4226", shadow=False)
    table(c, 14, 50, 164, 10, 6, "#6B4226", top="#8A5A34")
    rng = Rng(9)
    for x in range(20, 172, 14):
        c.ellipse(x + 3, 54, 4, 2, "#F4F4F8")
        c.ellipse(x + 3, 54, 2, 1, rng.pick(["#E8C070", "#C8603A", "#7AC83A", "#E8E0C0"]))
        c.rect(x + 9, 52, 1, 3, "#C8CCD8")
    for cx in (60, 96, 132):
        c.rect(cx - 3, 50, 7, 2, "#C9A04A")
        c.vline(cx, 45, 6, "#C9A04A")
        for k in (-3, 0, 3):
            c.rect(cx + k, 44, 1, 3, "#F4EEDC")
            c.set(cx + k, 43, "#FFB43A")
    block(c, 14, 64, 164, 2, 3, "#6B4226")
    plant(c, 8, 44)
    plant(c, 184, 44)


@room("war-room", "War Room", (10, 66, 172, 48), frames=16)
def war_room(c: Canvas):
    wall(c, 38, "#1E2433", "#2E3648", style="metal")
    floor_metal(c, 38, "#262C3C")
    # Big world map screen.
    mx, my, mw, mh = 42, 4, 108, 26
    screen(c, mx, my, mw, mh, glow="#3FD0FF")
    rng = Rng(3)
    continents = [(0.12, 0.3, 0.1, 0.2), (0.22, 0.65, 0.06, 0.2), (0.48, 0.3, 0.06, 0.12),
                  (0.52, 0.62, 0.07, 0.2), (0.7, 0.35, 0.16, 0.18), (0.84, 0.72, 0.06, 0.1)]
    for u, v, rw, rh in continents:
        c.ellipse(mx + u * mw, my + v * mh, rw * mw, rh * mh, "#1E6A8A")
    for x in range(mx, mx + mw, 6):
        c.vline(x, my, mh, "#3FD0FF", 0.12)
    for y in range(my, my + mh, 5):
        c.hline(mx, y, mw, "#3FD0FF", 0.12)
    for sx in (14, 162):
        screen(c, sx, 8, 18, 14, glow="#F2C230")
    # Consoles along the wall.
    for i, cx in enumerate(range(12, W - 20, 44)):
        counter(c, cx, 40, 36, 7, 10, "#3A4258", "#4A5470", doors=False)
        c.rect(cx + 3, 41, 30, 4, "#141824")
    for cx in range(20, W - 20, 44):
        chair(c, cx + 8, 60, "#2E3648", facing="back")
    blips = [(rng.next(), rng.next(), rng.next()) for _ in range(8)]

    def animate(c: Canvas, t: float):
        for u, v, p in blips:
            b = wave(t, 1, p)
            if b > 0:
                x, y = round(mx + 6 + u * (mw - 12)), round(my + 3 + v * (mh - 6))
                c.set(x, y, "#FF5A4A")
                if b > 0.7:
                    c.ellipse_ring(x, y, 2 + (b - 0.7) * 6, 1.5 + (b - 0.7) * 4, "#FF5A4A", 0.6)
        # Radar sweep across the map.
        sx = mx + round(t * mw)
        c.vline(sx, my, mh, "#7AE8FF", 0.6)
        c.vline(sx - 1, my, mh, "#3FD0FF", 0.3)
        c.vline(sx - 2, my, mh, "#3FD0FF", 0.15)
        scanlines(c, mx, my, mw, mh)
        for k, sx2 in enumerate((14, 162)):
            for j in range(6):
                hgt = 2 + round(4 * (0.5 + 0.5 * wave(t, 2, j * 0.15 + k * 0.5)))
                c.rect(sx2 + 2 + j * 3, 8 + 14 - hgt, 2, hgt, "#F2C230")
        for i, cx in enumerate(range(12, W - 20, 44)):
            for j in range(6):
                on = wave(t, 2, (i * 6 + j) * 0.11) > 0.2
                c.set(cx + 5 + j * 5, 43, ["#5AF0A0", "#FF5A4A", "#3FD0FF"][j % 3] if on else "#2A3040")

    return animate


@room("lake", "Breakstone Lake", (8, 80, 176, 34), frames=16)
def lake(c: Canvas):
    c.gradient_v(0, 0, W, 26, "#6AA8E0", "#D8EEF8", 4)
    # Distant hills and the mansion on the far shore.
    for x in range(W):
        h = round(8 + 4 * math.sin(x * 0.04) + 2 * math.sin(x * 0.13))
        c.vline(x, 26 - h, h, "#8AA8C0")
    c.rect(124, 13, 22, 10, "#B8A898")
    c.rect(130, 9, 8, 4, "#8A4A3A")
    c.rect(122, 11, 26, 3, "#7A3A2E")
    for wx in range(126, 146, 4):
        c.set(wx, 16, "#4A5A7A")
    rng = Rng(17)
    # Pine treeline.
    for x in range(-4, W + 4, 5):
        h = rng.int(10, 16)
        base = 34
        for k in range(h):
            half = round(k * 0.35)
            col = "#2E5A3A" if k % 3 else "#3A6A44"
            c.hline(x - half, base - h + k, half * 2 + 1, col)
        c.vline(x, base - h, h, "#3A6A44")
    # Lake.
    c.gradient_v(0, 34, W, 36, "#3A7AB0", "#2A5A90", 4)
    # Reflection of the trees.
    for x in range(0, W, 5):
        h = rng.int(3, 6)
        c.rect(x - 1, 34, 3, h, "#2A5A70", 0.5)
    # Shore: sand band and grass.
    c.rect(0, 68, W, 4, "#D8C89A")
    c.hline(0, 68, W, "#E8DCB0")
    for x in range(0, W, 3):
        c.set(x, 67 + (x * 7) % 2, "#E8DCB0")
    grass(c, 0, 72, W, H - 72, seed=71)
    # Dirt trail down to the dock, bushes along the shore.
    for y in range(72, H):
        x = 128 - round((y - 72) * 0.9)
        c.rect(x - 7, y, 16, 1, "#B89A6A")
        c.set(x - 7 + (y * 5) % 16, y, "#9A7E52")
        c.set(x - 8, y, "#7A9A4A")
        c.set(x + 9, y, "#7A9A4A")
    for bx, by in ((30, 76), (76, 74), (170, 78)):
        oval_shadow(c, bx + 2, by + 1, 9, 2.5)
        for dx, dy, r_ in ((-4, -2, 4), (3, -2, 4), (0, -5, 4)):
            c.ellipse(bx + dx, by + dy, r_ + 1, r_, "#2E5E2E")
        for dx, dy, r_ in ((-4, -3, 3), (3, -3, 3), (0, -6, 3)):
            c.ellipse(bx + dx, by + dy, r_, r_ - 1, "#4E8A3A")
        c.set(bx - 2, by - 8, "#7AB85A")
        c.set(bx + 1, by - 4, "#FF8FB0")
    def dock(c: Canvas, bob: int = 0):
        """Wooden dock reaching into the water and the rowboat tied to it."""
        dr = ramp("#9A6A40")
        c.rect(120, 38, 20, 34, dr["base"])
        for y in range(38, 72, 4):
            c.hline(120, y, 20, dr["shadow"])
            c.hline(120, y + 1, 20, dr["light"])
        c.vline(120, 38, 34, dr["light"])
        c.vline(139, 38, 34, dr["dark"])
        for py in (40, 54):
            c.rect(118, py, 2, 8, dr["dark"])
            c.rect(140, py, 2, 8, dr["dark"])
        c.rect(141, 44, 4, 1, "#2A5A70", 0.5)
        c.ellipse(158, 50 + bob, 12, 4, "#6B3A22")
        c.ellipse(158, 49 + bob, 10, 3, "#A86A3A")
        c.hline(150, 49 + bob, 16, "#C8884A")
        c.line(141, 46, 148, 49 + bob, "#C8B89A")
        c.hline(147, 53 + bob, 22, "#BFDDF0", 0.6)

    soft_shadow(c, 122, 40, 22, 30, 0.25)
    dock(c)
    for rx, ry in ((20, 70), (60, 69), (176, 71)):
        c.ellipse(rx, ry, 5, 3, "#7A7F88")
        c.ellipse(rx - 1, ry - 1, 3, 2, "#A8ACB4")
    for k, cx in enumerate((8, 12, 34, 90, 96, 184)):
        c.vline(cx, 60, 10, "#4E7A3A")
        c.rect(cx, 59, 1, 3, "#6B4226")

    def animate(c: Canvas, t: float):
        water(c, 0, 36, W, 30, t, "#3A7AB0", seed=21, density=30)
        # Waves lapping on the sand.
        lap = round(1.5 + 1.5 * wave(t, 1))
        for x in range(W):
            c.set(x, 67 + lap, "#E8F4FF" if (x + lap) % 3 else "#BFDDF0", 0.85)
        dock(c, round(wave(t, 1, 0.25)))
        clouds(c, 0, 0, W, 14, t, seed=4, count=4)

    return animate


@room("rooftop", "Rooftop Helipad", (14, 54, 164, 60), frames=16)
def rooftop(c: Canvas):
    c.gradient_v(0, 0, W, 24, "#3A3A7A", "#F0A06A", 5)
    # Forest silhouette on the horizon.
    for x in range(W):
        h = round(5 + 2 * math.sin(x * 0.3) + 2 * math.sin(x * 0.11))
        c.vline(x, 22 - h, h + 2, "#2A2A4A")
    # Parapet.
    pr = ramp("#9A8A7A")
    c.rect(0, 22, W, 10, pr["base"])
    c.hline(0, 22, W, pr["light"])
    c.hline(0, 23, W, pr["light"])
    for x in range(0, W, 12):
        c.vline(x, 24, 8, pr["shadow"])
    c.hline(0, 31, W, pr["dark"])
    # Gravel roof.
    rr = ramp("#7A7A84")
    c.rect(0, 32, W, H - 32, rr["base"])
    noise(c, 0, 32, W, H - 32, [rr["shadow"], mix(rr["base"], rr["light"], 0.5)], 0.1, 77)
    c.dither(0, 32, W, 2, SHADOW)
    # Helipad: dark disc, yellow ring, white H.
    cx, cy = 96, 84
    c.ellipse(cx, cy, 70, 30, "#3A3A44")
    c.ellipse_ring(cx, cy, 70, 30, "#2A2A30")
    c.ellipse_ring(cx, cy, 64, 27, "#F2C230")
    c.ellipse_ring(cx, cy, 63, 26, "#F2C230")
    hw = ramp("#F4F4F8")
    c.rect(cx - 16, cy - 12, 6, 24, hw["base"])
    c.rect(cx + 10, cy - 12, 6, 24, hw["base"])
    c.rect(cx - 10, cy - 3, 20, 6, hw["base"])
    # AC units and a vent.
    block(c, 156, 34, 26, 10, 6, "#A8ACB4")
    c.ellipse(169, 37, 7, 2, "#5A5E68")
    block(c, 12, 34, 14, 8, 5, "#A8ACB4")
    # Antenna mast.
    c.vline(140, 4, 34, "#5A5E68")
    c.line(136, 38, 140, 18, "#5A5E68")
    c.line(144, 38, 140, 18, "#5A5E68")

    def animate(c: Canvas, t: float):
        # Pad lights chasing around the ring.
        for k in range(12):
            a = k / 12 * TAU
            x, y = round(cx + math.cos(a) * 70), round(cy + math.sin(a) * 30)
            on = (k - int(t * 12)) % 12 < 2
            c.rect(x, y, 2, 2, "#7AF07A" if on else "#2A5A2A")
        # Antenna beacon.
        if wave(t, 2) > 0.3:
            c.ellipse(140, 4, 2, 2, "#FF3A3A")
            c.ellipse(140, 4, 5, 4, "#FF3A3A", 0.2)
        else:
            c.set(140, 4, "#7A1A1A")
        # Windsock fluttering.
        c.vline(40, 6, 30, "#C8CCD8")
        for k in range(5):
            y = 7 + round(wave(t, 2, k * 0.15) * (k * 0.4))
            col = "#E8602A" if k % 2 == 0 else "#F4F4F8"
            c.rect(41 + k * 3, y + k // 2, 3, 3 - k // 3, col)
        clouds(c, 0, 0, W, 16, t, seed=9, count=3, color="#F8C8A8", alpha=0.7)
        # AC fan spinning.
        a = TAU * t * 2
        c.line(169 - round(math.cos(a) * 5), 37 - round(math.sin(a) * 1.5), 169 + round(math.cos(a) * 5), 37 + round(math.sin(a) * 1.5), "#2A2A30")

    return animate


ORDER = ["office", "danger-room", "hangar", "garage", "courtyard", "basketball", "classroom", "library",
         "kitchen", "beast-lab", "infirmary", "cerebro", "pool", "greenhouse", "gym", "lounge", "dorm",
         "rec-room", "refectory", "war-room", "lake", "rooftop"]


# --- output -----------------------------------------------------------------------------


def render(spec: dict) -> Canvas:
    base = Canvas(W, H)
    animate = spec["draw"](base)
    if spec["frames"] == 1 or animate is None:
        if animate:
            animate(base, 0.0)
        return base
    frames = []
    for i in range(spec["frames"]):
        f = copy(base)
        animate(f, i / spec["frames"])
        frames.append(f)
    return strip(frames)


def main(only: list[str]):
    manifest = []
    for spec in sorted(ROOMS, key=lambda r: ORDER.index(r["id"])):
        if not only or spec["id"] in only:
            render(spec).save(ASSETS / f"{spec['id']}.png")
        x, y, w, h = spec["walk"]
        manifest.append({
            "fps": FPS,
            "frames": spec["frames"],
            "id": spec["id"],
            "name": spec["name"],
            "sprite": f"rooms/{spec['id']}.png",
            "walkArea": {"height": h, "width": w, "x": x, "y": y},
        })
    if not only:
        data = {"frameHeight": H, "frameWidth": W, "rooms": manifest}
        (ASSETS / "rooms.json").write_text(json.dumps(data, indent=2) + "\n")


if __name__ == "__main__":
    main(sys.argv[1:])
