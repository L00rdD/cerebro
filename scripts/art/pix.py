"""Pixel-art primitives shared by the character and room generators.

Everything is drawn pixel by pixel on an RGBA canvas. Colors are given as hex strings;
`ramp` derives shades with a hue shift (shadows go cooler, lights go warmer), which is
what keeps the art from looking flat.
"""

from __future__ import annotations

import colorsys
import math
from pathlib import Path

from PIL import Image

RGBA = tuple[int, int, int, int]


def rgba(color: str | RGBA, alpha: int = 255) -> RGBA:
    if isinstance(color, tuple):
        return color
    c = color.lstrip("#")
    return (int(c[0:2], 16), int(c[2:4], 16), int(c[4:6], 16), alpha)


def hex_of(c: RGBA) -> str:
    return "#%02X%02X%02X" % c[:3]


def _towards(h: float, target: float, t: float) -> float:
    """Moves hue h towards target by fraction t, the short way round the wheel."""
    d = (target - h + 0.5) % 1.0 - 0.5
    return (h + d * t) % 1.0


def shade(color: str, amount: float) -> str:
    """amount < 0 darkens towards a cool shadow, > 0 lightens towards a warm light."""
    r, g, b, _ = rgba(color)
    h, l, s = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
    if amount < 0:
        # Shadows: darker, slightly more saturated, hue pulled towards blue/purple.
        target = 0.68
        h = _towards(h, target, min(1.0, -amount * 0.35)) if s > 0.08 else h
        l = max(0.0, l * (1 + amount))
        s = min(1.0, s * (1 - amount * 0.25))
    else:
        # Lights: lighter, hue pulled towards yellow.
        target = 0.14
        h = _towards(h, target, min(1.0, amount * 0.3)) if s > 0.08 else h
        l = min(1.0, l + (1 - l) * amount)
        s = s * (1 - amount * 0.2)
    r, g, b = colorsys.hls_to_rgb(h % 1.0, l, s)
    return hex_of((round(r * 255), round(g * 255), round(b * 255), 255))


def ramp(color: str) -> dict[str, str]:
    """Four tones: dark (outline side), shadow, base, light."""
    return {
        "dark": shade(color, -0.55),
        "shadow": shade(color, -0.25),
        "base": color,
        "light": shade(color, 0.3),
    }


def mix(a: str, b: str, t: float) -> str:
    ca, cb = rgba(a), rgba(b)
    return hex_of(tuple(round(ca[i] + (cb[i] - ca[i]) * t) for i in range(3)) + (255,))


class Canvas:
    def __init__(self, width: int, height: int, fill: str | None = None):
        self.width = width
        self.height = height
        self.image = Image.new("RGBA", (width, height), rgba(fill) if fill else (0, 0, 0, 0))
        self.px = self.image.load()

    # --- single pixels -------------------------------------------------------------------

    def inside(self, x: int, y: int) -> bool:
        return 0 <= x < self.width and 0 <= y < self.height

    def set(self, x: int, y: int, color: str | RGBA, alpha: float = 1.0):
        x, y = int(x), int(y)
        if not self.inside(x, y):
            return
        c = rgba(color)
        if alpha >= 1.0 and c[3] == 255:
            self.px[x, y] = c
            return
        a = alpha * c[3] / 255
        under = self.px[x, y]
        if under[3] == 0:
            self.px[x, y] = (c[0], c[1], c[2], round(a * 255))
            return
        self.px[x, y] = (
            round(under[0] + (c[0] - under[0]) * a),
            round(under[1] + (c[1] - under[1]) * a),
            round(under[2] + (c[2] - under[2]) * a),
            max(under[3], round(a * 255)),
        )

    def get(self, x: int, y: int) -> RGBA:
        if not self.inside(x, y):
            return (0, 0, 0, 0)
        return self.px[int(x), int(y)]

    def clear(self, x: int, y: int):
        if self.inside(x, y):
            self.px[int(x), int(y)] = (0, 0, 0, 0)

    # --- shapes --------------------------------------------------------------------------

    def rect(self, x: int, y: int, w: int, h: int, color: str, alpha: float = 1.0):
        for yy in range(y, y + h):
            for xx in range(x, x + w):
                self.set(xx, yy, color, alpha)

    def hline(self, x: int, y: int, w: int, color: str, alpha: float = 1.0):
        self.rect(x, y, w, 1, color, alpha)

    def vline(self, x: int, y: int, h: int, color: str, alpha: float = 1.0):
        self.rect(x, y, 1, h, color, alpha)

    def frame(self, x: int, y: int, w: int, h: int, color: str):
        self.hline(x, y, w, color)
        self.hline(x, y + h - 1, w, color)
        self.vline(x, y, h, color)
        self.vline(x + w - 1, y, h, color)

    def line(self, x0: int, y0: int, x1: int, y1: int, color: str, alpha: float = 1.0):
        steps = max(abs(x1 - x0), abs(y1 - y0), 1)
        for i in range(steps + 1):
            t = i / steps
            self.set(round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t), color, alpha)

    def ellipse(self, cx: float, cy: float, rx: float, ry: float, color: str, alpha: float = 1.0):
        for y in range(math.floor(cy - ry), math.ceil(cy + ry) + 1):
            for x in range(math.floor(cx - rx), math.ceil(cx + rx) + 1):
                if ((x - cx) / max(rx, 0.5)) ** 2 + ((y - cy) / max(ry, 0.5)) ** 2 <= 1.0:
                    self.set(x, y, color, alpha)

    def ellipse_ring(self, cx: float, cy: float, rx: float, ry: float, color: str, alpha: float = 1.0):
        steps = max(16, round(2 * math.pi * max(rx, ry)) * 2)
        seen = set()
        for i in range(steps):
            a = i / steps * 2 * math.pi
            p = (round(cx + math.cos(a) * rx), round(cy + math.sin(a) * ry))
            if p not in seen:
                seen.add(p)
                self.set(*p, color, alpha)

    def dither(self, x: int, y: int, w: int, h: int, color: str, phase: int = 0, alpha: float = 1.0):
        """Checkerboard fill: a 50% blend that stays crisp."""
        for yy in range(y, y + h):
            for xx in range(x, x + w):
                if (xx + yy + phase) % 2 == 0:
                    self.set(xx, yy, color, alpha)

    def gradient_v(self, x: int, y: int, w: int, h: int, top: str, bottom: str, bands: int = 4):
        """Vertical gradient in a few flat bands with dithered seams."""
        for yy in range(h):
            t = yy / max(h - 1, 1)
            band = min(bands - 1, int(t * bands))
            nxt = min(bands - 1, band + 1)
            c = mix(top, bottom, band / max(bands - 1, 1))
            c2 = mix(top, bottom, nxt / max(bands - 1, 1))
            frac = t * bands - band
            for xx in range(w):
                use_next = frac > 0.75 and (xx + yy) % 2 == 0
                self.set(x + xx, y + yy, c2 if use_next else c)

    def blit(self, other: "Canvas", x: int, y: int, alpha: float = 1.0):
        for yy in range(other.height):
            for xx in range(other.width):
                c = other.px[xx, yy]
                if c[3]:
                    self.set(x + xx, y + yy, c, alpha)

    def outline(self, color: str | None = None, darken: float = -0.6):
        """1px outline around opaque pixels. Without a color, each outline pixel is a dark
        version of the neighbor it surrounds (selective outline)."""
        src = self.image.copy().load()
        for y in range(self.height):
            for x in range(self.width):
                if src[x, y][3]:
                    continue
                for dx, dy in ((0, 1), (0, -1), (1, 0), (-1, 0)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < self.width and 0 <= ny < self.height and src[nx, ny][3] > 128:
                        self.px[x, y] = rgba(color) if color else rgba(shade(hex_of(src[nx, ny]), darken))
                        break

    def flipped(self) -> "Canvas":
        out = Canvas(self.width, self.height)
        out.image = self.image.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
        out.px = out.image.load()
        return out

    def save(self, path: Path):
        path.parent.mkdir(parents=True, exist_ok=True)
        self.image.save(path, optimize=True)


def strip(frames: list[Canvas]) -> Canvas:
    """Frames side by side, as the UI expects for animated rooms."""
    w, h = frames[0].width, frames[0].height
    out = Canvas(w * len(frames), h)
    for i, f in enumerate(frames):
        out.image.paste(f.image, (i * w, 0))
    out.px = out.image.load()
    return out


class Rng:
    """Small deterministic generator, so regenerating the art gives the same files."""

    def __init__(self, seed: int):
        self.state = seed & 0xFFFFFFFF or 1

    def next(self) -> float:
        x = self.state
        x ^= (x << 13) & 0xFFFFFFFF
        x ^= x >> 17
        x ^= (x << 5) & 0xFFFFFFFF
        self.state = x & 0xFFFFFFFF
        return self.state / 4294967296

    def int(self, lo: int, hi: int) -> int:
        return lo + int(self.next() * (hi - lo + 1))

    def pick(self, items):
        return items[int(self.next() * len(items))]
