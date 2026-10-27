"""Body templates for 24x24 character frames.

A frame is a grid of cells, each holding a (region, tone) pair. Templates are ASCII art,
16 columns wide, padded to the 24px frame; features (hair, masks, costume patterns) edit
the grid, then it is painted with each character's colors and outlined.
"""

from __future__ import annotations

from dataclasses import dataclass

WIDTH = 24
HEIGHT = 24
PAD = (WIDTH - 16) // 2

DIRECTIONS = ("front", "back", "side")
FRAMES = 4

# Template letters -> (region, tone).
LEGEND = {
    "S": ("skin", "base"),
    "s": ("skin", "shadow"),
    "k": ("skin", "light"),
    "e": ("eye", "base"),
    "w": ("eyewhite", "base"),
    "m": ("mouth", "base"),
    "H": ("hair", "base"),
    "h": ("hair", "shadow"),
    "I": ("hair", "light"),
    "T": ("torso", "base"),
    "t": ("torso", "shadow"),
    "U": ("torso", "light"),
    "A": ("arm", "base"),
    "a": ("arm", "shadow"),
    "Q": ("arm", "light"),
    "G": ("hand", "base"),
    "g": ("hand", "shadow"),
    "W": ("belt", "base"),
    "X": ("buckle", "base"),
    "L": ("legs", "base"),
    "l": ("legs", "shadow"),
    "P": ("legs", "light"),
    "B": ("boots", "base"),
    "b": ("boots", "shadow"),
    "N": ("neck", "base"),
}


def _grid(art: str) -> list[str]:
    rows = [r for r in art.strip("\n").split("\n")]
    rows = [r.split(" ", 1)[-1] if " " in r else r for r in rows]
    assert len(rows) == HEIGHT, f"template has {len(rows)} rows"
    for r in rows:
        assert len(r) == 16, f"row {r!r} is not 16 wide"
    return rows


# Rows 0-9 head, 10 neck, 11-16 torso and arms, 17-22 legs, 23 left for the outline.
HEAD_FRONT = """
00 ................
01 ................
02 .....SSSSSS.....
03 ....SSSSSSSS....
04 ...SSSSSSSSSS...
05 ...SSSSSSSSSs...
06 ...SSSweSweSs...
07 ...SSSweSweSs...
08 ...SSSSSSSSSs...
09 ....SSSSmSSs....
10 .....sssssss....
"""

HEAD_BACK = """
00 ................
01 ................
02 .....SSSSSS.....
03 ....SSSSSSSS....
04 ...SSSSSSSSSS...
05 ...SSSSSSSSSs...
06 ...SSSSSSSSSs...
07 ...SSSSSSSSSs...
08 ...SSSSSSSSSs...
09 ....SSSSSSSs....
10 .....sssssss....
"""

HEAD_SIDE = """
00 ................
01 ................
02 .....SSSSSS.....
03 ....SSSSSSSS....
04 ....SSSSSSSSS...
05 ....SSSSSSSSS...
06 ....SSSSSSSwe...
07 ....SSSSSSSweS..
08 ....SSSSSSSSS...
09 .....SSSSSSmS...
10 ......sssss.....
"""

TORSO_FRONT = {
    0: """
11 ...AATUTTTTtAa..
12 ...AATTTTTTtAa..
13 ...AATTTTTTtAa..
14 ...AaTTTTTTtaa..
15 ...GGTTTTTTtGg..
16 ...GgWWWXXWWgg..
""",
    # Steps: the hand on the side of the lifted foot swings up a pixel.
    1: """
11 ...AATUTTTTtAa..
12 ...AATTTTTTtAa..
13 ...AATTTTTTtAa..
14 ...GGTTTTTTtaa..
15 ...GgTTTTTTtGg..
16 .....WWWXXWWgg..
""",
    3: """
11 ...AATUTTTTtAa..
12 ...AATTTTTTtAa..
13 ...AATTTTTTtAa..
14 ...AaTTTTTTtGg..
15 ...GGTTTTTTtgg..
16 ...GgWWWXXWW....
""",
}

TORSO_BACK = {
    0: """
11 ...AATTTTTTtAa..
12 ...AATTTTTTtAa..
13 ...AATTTTTTtAa..
14 ...AaTTTTTTtaa..
15 ...GGTTTTTTtGg..
16 ...GgWWWWWWWgg..
""",
    1: """
11 ...AATTTTTTtAa..
12 ...AATTTTTTtAa..
13 ...AATTTTTTtAa..
14 ...AaTTTTTTtGg..
15 ...GGTTTTTTtgg..
16 ...GgWWWWWWW....
""",
    3: """
11 ...AATTTTTTtAa..
12 ...AATTTTTTtAa..
13 ...AATTTTTTtAa..
14 ...GGTTTTTTtaa..
15 ...GgTTTTTTtGg..
16 .....WWWWWWWgg..
""",
}

TORSO_SIDE = {
    0: """
11 ......TTTTTt....
12 ......TAAATt....
13 ......TAAATt....
14 ......TAaaTt....
15 ......TGGgTt....
16 ......WGgWWW....
""",
    # Arm swings forward with the back leg, backwards with the front one.
    1: """
11 ......TTTTTt....
12 ......TTAAAt....
13 ......TTTAAA....
14 ......TTTtAaG...
15 ......TTTTtGg...
16 ......WWWWWW....
""",
    3: """
11 ......TTTTTt....
12 ......AAATTt....
13 .....AAaTTTt....
14 ....GAaTTTTt....
15 ....gG.TTTTt....
16 ......WWWWWW....
""",
}

LEGS_FRONT = {
    0: """
17 .....LLLLLLl....
18 .....LLLLLLl....
19 .....LLlLLLl....
20 .....LLlLLLl....
21 ....BBBbBBBb....
22 ....BBBbBBBb....
""",
    1: """
17 .....LLLLLLl....
18 .....LLlLLLl....
19 .....LLlLLLl....
20 ....BBBbLLLl....
21 ....BBBbBBBb....
22 ........BBBb....
""",
    3: """
17 .....LLLLLLl....
18 .....LLlLLLl....
19 .....LLlLLLl....
20 .....LLlBBBb....
21 ....BBBbBBBb....
22 ....BBBb........
""",
}

LEGS_SIDE = {
    0: """
17 ......LLLLLl....
18 ......LLLLLl....
19 ......LLLLLl....
20 ......LLLLLl....
21 ......BBBBBBb...
22 ......BBBBBBb...
""",
    # Near leg in the base tone, far leg in shadow.
    1: """
17 ......LLLLLl....
18 .....llL.LLLl...
19 ....lll...LLl...
20 ....lll...LLLl..
21 ...bbbb...BBBBb.
22 ...bbb.....BBBb.
""",
    3: """
17 ......LLLLLl....
18 .....LLl.lllL...
19 ....LLl...lll...
20 ....LLl...llll..
21 ...BBBb...bbbbb.
22 ...BBb.....bbbb.
""",
}


@dataclass
class Cell:
    region: str
    tone: str


class Frame:
    """A 24x24 grid of cells plus the direction/frame it represents."""

    def __init__(self, direction: str, frame: int, big: bool = False):
        self.direction = direction
        self.frame = frame
        self.big = big
        self.cells: list[list[Cell | None]] = [[None] * WIDTH for _ in range(HEIGHT)]
        self._build()

    # --- template ------------------------------------------------------------------------

    def _rows(self) -> list[str]:
        d, f = self.direction, self.frame
        head = {"front": HEAD_FRONT, "back": HEAD_BACK, "side": HEAD_SIDE}[d]
        pose = 0 if f in (0, 2) else f
        torso = {"front": TORSO_FRONT, "back": TORSO_BACK, "side": TORSO_SIDE}[d][pose]
        legs = (LEGS_SIDE if d == "side" else LEGS_FRONT)[pose]
        art = head.strip("\n") + "\n" + torso.strip("\n") + "\n" + legs.strip("\n") + "\n23 ................"
        return _grid(art)

    def _build(self):
        rows = self._rows()
        for y, row in enumerate(rows):
            for x, ch in enumerate(row):
                if ch in LEGEND:
                    region, tone = LEGEND[ch]
                    self.cells[y][x + PAD] = Cell(region, tone)
        if self.big:
            self._widen()

    def _widen(self):
        """Big build: torso, arms and legs one pixel wider on each side, shoulders raised."""
        center = WIDTH // 2
        new = [[None] * WIDTH for _ in range(HEIGHT)]
        for y in range(HEIGHT):
            for x in range(WIDTH):
                c = self.cells[y][x]
                if c is None:
                    continue
                if y < 11:
                    new[y][x] = c
                    continue
                shift = -1 if x < center else 1
                if y >= 17:
                    shift = -1 if x < center else 1
                nx = x + shift
                if 0 <= nx < WIDTH:
                    new[y][nx] = c
        # Fill the seam the shift opened in the middle.
        for y in range(11, HEIGHT):
            l, r = new[y][center - 2], new[y][center + 1]
            if l and r:
                for x in (center - 1, center):
                    if new[y][x] is None:
                        src = self.cells[y][x] or l
                        new[y][x] = Cell(src.region, src.tone)
        # Bulkier shoulders.
        for y in (11,):
            row = new[y]
            xs = [x for x in range(WIDTH) if row[x]]
            if xs and self.direction != "side":
                for x in (xs[0] - 1, xs[-1] + 1):
                    if 0 <= x < WIDTH:
                        new[y + 1][x] = new[y + 1][x] or Cell("arm", "base")
        self.cells = new

    # --- editing helpers -----------------------------------------------------------------

    def get(self, x: int, y: int) -> Cell | None:
        if 0 <= x < WIDTH and 0 <= y < HEIGHT:
            return self.cells[y][x]
        return None

    def put(self, x: int, y: int, region: str | None, tone: str = "base"):
        if 0 <= x < WIDTH and 0 <= y < HEIGHT:
            self.cells[y][x] = Cell(region, tone) if region else None

    def where(self, *regions: str):
        for y in range(HEIGHT):
            for x in range(WIDTH):
                c = self.cells[y][x]
                if c and c.region in regions:
                    yield x, y, c

    def span(self, y: int, *regions: str) -> tuple[int, int] | None:
        xs = [x for x in range(WIDTH) if self.cells[y][x] and (not regions or self.cells[y][x].region in regions)]
        return (xs[0], xs[-1]) if xs else None

    def overlay(self, art: str, legend: dict[str, tuple[str | None, str]], dx: int = PAD, under: bool = False):
        """Stamps ASCII art whose lines start with their row number ("05 ..HH.."), columns
        relative to the 16px body (dx pads them into the frame). '.' leaves the cell, '_'
        clears it. With `under`, only empty cells are filled (things behind the body)."""
        for line in art.strip("\n").split("\n"):
            row, _, pixels = line.partition(" ")
            ty = int(row)
            for x, ch in enumerate(pixels):
                tx = x + dx
                if ch == "." or not (0 <= tx < WIDTH and 0 <= ty < HEIGHT):
                    continue
                if ch == "_":
                    self.cells[ty][tx] = None
                    continue
                if ch not in legend:
                    raise KeyError(f"no legend for {ch!r}")
                if under and self.cells[ty][tx] is not None:
                    continue
                region, tone = legend[ch]
                self.cells[ty][tx] = Cell(region, tone) if region else None

    def shift(self, dx: int, dy: int):
        new = [[None] * WIDTH for _ in range(HEIGHT)]
        for y in range(HEIGHT):
            for x in range(WIDTH):
                c = self.cells[y][x]
                if c and 0 <= x + dx < WIDTH and 0 <= y + dy < HEIGHT:
                    new[y + dy][x + dx] = c
        self.cells = new
