"""Generates assets/ui/emotes.png: 9x9 emotes side by side, "z" (asleep) then "!" (waiting).

    python3 -I scripts/art/emotes.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from pix import Canvas  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
SIZE = 9

# Bubble outline (o), fill (#), glyph (x), glyph shadow (s).
EMOTES = [
    """
.ooooooo.
o#######o
o#xxxxx#o
o####xs#o
o###xs##o
o##xs###o
o#xxxxx#o
o#######o
.oooo.oo.
""",
    """
.ooooooo.
o###x###o
o###x###o
o###x###o
o###x###o
o###s###o
o###x###o
o#######o
.oo.oooo.
""",
]

COLORS = [
    {"o": "#3A3550", "#": "#F4F1EA", "x": "#5A6AC8", "s": "#A8B0E0"},
    {"o": "#5A1E1E", "#": "#FFE27A", "x": "#C8202A", "s": "#FFE27A"},
]


def main():
    sheet = Canvas(SIZE * len(EMOTES), SIZE)
    for i, (art, colors) in enumerate(zip(EMOTES, COLORS)):
        for y, row in enumerate(art.strip("\n").split("\n")):
            for x, ch in enumerate(row):
                if ch in colors:
                    sheet.set(i * SIZE + x, y, colors[ch])
    sheet.save(ROOT / "assets" / "ui" / "emotes.png")


if __name__ == "__main__":
    main()
