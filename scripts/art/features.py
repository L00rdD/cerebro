"""Features that dress a body Frame: hair, headgear, costume pieces, capes, wings, tails.

Each feature is a function `(frame, look) -> None` built by a factory, so a look reads
like a list: `[hair("long"), visor(), straps()]`. Art is ASCII, row-numbered, in the
16-column body space (see body.py).
"""

from __future__ import annotations

from body import PAD, WIDTH, Cell, Frame

HAIR = {"H": ("hair", "base"), "h": ("hair", "shadow"), "I": ("hair", "light"), "S": ("skin", "base"), "s": ("skin", "shadow")}

# --- hair -------------------------------------------------------------------------------

HAIRSTYLES: dict[str, dict[str, str]] = {
    "short": {
        "front": """
02 .....IHHHHH.....
03 ....IHHHHHHh....
04 ...HHHHHHHHhh...
05 ...HHhH.HHhhh...
06 ...H........h...
""",
        "back": """
02 .....IHHHHH.....
03 ....IHHHHHHh....
04 ...HHHHHHHHhh...
05 ...HHHHHHHHhh...
06 ...HHHHHHHHhh...
07 ...HHHHHHHHhh...
08 ...hHHHHHHHhh...
09 ....hhhhhhhh....
""",
        "side": """
02 .....IHHHHH.....
03 ....IHHHHHHH....
04 ....HHHHHHHHH...
05 ....HHHHHhHh....
06 ....HHHHh.......
07 ....HHHh........
08 ....hHh.........
09 .....h..........
""",
    },
    "spiky": {
        "front": """
00 .....H..H.......
01 ....HIHHIH.H....
02 ...HIHHHHHHHh...
03 ..HIHHHHHHHHhh..
04 ...HHHHHHHHHh...
05 ...HhHHhHHhHh...
06 ...H........h...
""",
        "back": """
00 .....H..H.......
01 ....HIHHIH.H....
02 ...HIHHHHHHHh...
03 ..HIHHHHHHHHhh..
04 ...HHHHHHHHHhh..
05 ...HHHHHHHHHh...
06 ...HHHHHHHHHh...
07 ...HHHHHHHHhh...
08 ...hHhHHhHhhh...
09 ....h.h..h.h....
""",
        "side": """
00 ....H..H........
01 ...HIHHIHH......
02 ..HIHHHHHHH.....
03 ..HHHHHHHHHH....
04 ...HHHHHHHHHH...
05 ...HHHHHHhHh....
06 ....HHHHh.......
07 ...HHHHh........
08 ....hHh.........
""",
    },
    "long": {
        "front": """
01 ......IHHH......
02 ....IIHHHHHh....
03 ...IHHHHHHHHh...
04 ..HHHHHHHHHHhh..
05 ..HHHHh..hHHhh..
06 ..HHH......Hhh..
07 ..HH........hh..
08 ..HH........hh..
09 ..HHh.......hh..
10 ..HHh......hhh..
11 ..hHh......hhh..
12 ..hh........hh..
""",
        "back": """
01 ......IHHH......
02 ....IIHHHHHh....
03 ...IHHHHHHHHh...
04 ..HHHHHHHHHHhh..
05 ..HHHHHHHHHHhh..
06 ..HHHHHHHHHHhh..
07 ..HHHHHHHHHHhh..
08 ..HHHHHHHHHHhh..
09 ..HHHHHHHHHHhh..
10 ..HHHHHHHHHHhh..
11 ..hHHHHHHHHHhh..
12 ...hHHHHHHHhh...
13 ....hhHHHHhh....
""",
        "side": """
01 ......IHHH......
02 ....IIHHHHH.....
03 ...IHHHHHHHH....
04 ...HHHHHHHHHH...
05 ...HHHHHHhHh....
06 ...HHHHHh.......
07 ...HHHHh........
08 ...HHHHh........
09 ...HHHHh........
10 ...HHHh.........
11 ...hHHh.........
12 ...hhh..........
13 ....h...........
""",
    },
    "mane": {
        "front": """
00 .....IIHHH......
01 ...IIHHHHHHh....
02 ..IHHHHHHHHHh...
03 .IHHHHHHHHHHhh..
04 .HHHHHHHHHHHhhh.
05 .HHHHHh..hHHhhh.
06 .HHHH......Hhhh.
07 .HHH........hhh.
08 .HHH........hhh.
09 .HHHh.......hhh.
10 .HHHh......hhhh.
11 .HHHh......hhhh.
12 .hHHh......hhhh.
13 .hHh........hhh.
14 ..hh........hh..
""",
        "back": """
00 .....IIHHH......
01 ...IIHHHHHHh....
02 ..IHHHHHHHHHh...
03 .IHHHHHHHHHHhh..
04 .HHHHHHHHHHHhhh.
05 .HHHHHHHHHHHhhh.
06 .HHHHHHHHHHHhhh.
07 .HHHHHHHHHHHhhh.
08 .HHHHHHHHHHHhhh.
09 .HHHHHHHHHHHhhh.
10 .HHHHHHHHHHHhhh.
11 .HHHHHHHHHHHhhh.
12 .hHHHHHHHHHHhhh.
13 .hHHHHHHHHHHhh..
14 ..hhHHHHHHHhh...
15 ...hhhHHHhhh....
""",
        "side": """
00 .....IIHHH......
01 ...IIHHHHHH.....
02 ..IHHHHHHHHH....
03 .IHHHHHHHHHHH...
04 .HHHHHHHHHHHH...
05 .HHHHHHHHhHh....
06 .HHHHHHHh.......
07 .HHHHHHh........
08 .HHHHHHh........
09 .HHHHHHh........
10 .HHHHHh.........
11 .HHHHHh.........
12 .hHHHh..........
13 .hHHh...........
14 ..hh............
""",
    },
    "bob": {
        "front": """
02 .....IHHHHH.....
03 ....IHHHHHHh....
04 ...HHHHHHHHhh...
05 ...HHHHHHHHhh...
06 ...HH......hh...
07 ..HH........hh..
08 ..HH........hh..
09 ..hh........hh..
""",
        "back": """
02 .....IHHHHH.....
03 ....IHHHHHHh....
04 ...HHHHHHHHhh...
05 ...HHHHHHHHhh...
06 ...HHHHHHHHhh...
07 ..HHHHHHHHHHhh..
08 ..HHHHHHHHHHhh..
09 ..hhHHHHHHHhhh..
10 ....hhhhhhhh....
""",
        "side": """
02 .....IHHHHH.....
03 ....IHHHHHHH....
04 ...HHHHHHHHHH...
05 ...HHHHHHHHHh...
06 ...HHHHHH.......
07 ..HHHHHHh.......
08 ..HHHHHh........
09 ..hhhhh.........
""",
    },
    "ponytail": {
        "front": """
02 .....IHHHHH.....
03 ....IHHHHHHh....
04 ...HHHHHHHHhh...
05 ...HHhH..HHhh...
06 ...H........h...
""",
        "back": """
02 .....IHHHHH.....
03 ....IHHHHHHh....
04 ...HHHHHHHHhh...
05 ...HHHHHHHHhh...
06 ...HHHHHHHHhh...
07 ...HHHHHHHHhh...
08 ...hHHHHHHHhh...
09 ....hhhHHhhh....
10 ......HHhh......
11 ......HHh.......
12 ......Hhh.......
13 .......h........
""",
        "side": """
02 .....IHHHHH.....
03 ....IHHHHHHH....
04 ...HHHHHHHHHH...
05 ..HHHHHHHhHh....
06 .HHHHHHh........
07 .HHhHHh.........
08 .Hh.hh..........
09 .h..............
10 .h..............
""",
    },
    "mohawk": {
        "front": """
00 ......IH........
01 ......IHh.......
02 ......IHh.......
03 ......HHh.......
""",
        "back": """
00 ......IH........
01 ......IHh.......
02 ......IHh.......
03 ......HHh.......
04 ......HHh.......
05 ......hhh.......
""",
        "side": """
00 ........IH......
01 ......IIHHh.....
02 .....IHHHh......
03 ....HHHh........
04 ....hh..........
""",
    },
    "buzz": {
        "front": """
02 .....hHHHHh.....
03 ....hHHHHHHh....
04 ...h.HHHHHH.h...
""",
        "back": """
02 .....hHHHHh.....
03 ....hHHHHHHh....
04 ...hHHHHHHHHh...
05 ...hHHHHHHHHh...
06 ...hhHHHHHHhh...
07 ....hhhhhhhh....
""",
        "side": """
02 .....hHHHHh.....
03 ....hHHHHHHh....
04 ....hHHHHHh.....
05 ....hHHh........
06 ....hh..........
""",
    },
    "slick": {
        "front": """
01 ......IHHH......
02 ....IIHHHHHh....
03 ...IHHHHHHHHh...
04 ...HHHHHHHHHh...
05 ...HHH.HHhHHh...
06 ...HH.......hh..
07 ...HH.......hh..
08 ...Hh.......hh..
09 ...hh.......h...
""",
        "back": """
01 ......IHHH......
02 ....IIHHHHHh....
03 ...IHHHHHHHHh...
04 ...HHHHHHHHHhh..
05 ...HHHHHHHHHhh..
06 ...HHHHHHHHHhh..
07 ...HHHHHHHHHhh..
08 ...HHHHHHHHHhh..
09 ...hHHHHHHHHh...
10 ....hhhHHhhh....
""",
        "side": """
01 ......IHHH......
02 ....IIHHHHH.....
03 ...IHHHHHHHH....
04 ...HHHHHHHHHH...
05 ...HHHHHHHHh....
06 ...HHHHHh.......
07 ...HHHHh........
08 ...HHHHh........
09 ...hHHh.........
10 ....hh..........
""",
    },
    "curly": {
        "front": """
01 .....IHIHH......
02 ...IHHHHHHHh....
03 ..IHHHHHHHHHh...
04 ..HHHHHHHHHHhh..
05 ..HHHhHHhHHhhh..
06 ..HH........hh..
07 ..Hh........hh..
""",
        "back": """
01 .....IHIHH......
02 ...IHHHHHHHh....
03 ..IHHHHHHHHHh...
04 ..HHHHHHHHHHhh..
05 ..HHHHHHHHHHhh..
06 ..HHHHHHHHHHhh..
07 ..HHHHHHHHHHhh..
08 ..hHHHHHHHHHhh..
09 ...hHhhHhhHhh...
""",
        "side": """
01 .....IHIHH......
02 ...IHHHHHHHh....
03 ..IHHHHHHHHHh...
04 ..HHHHHHHHHHH...
05 ..HHHHHHHhHh....
06 ..HHHHHh........
07 ..HHHHh.........
08 ..hHHh..........
09 ...hh...........
""",
    },
}


def hair(style: str) -> "callable":
    def apply(f: Frame, look):
        f.overlay(HAIRSTYLES[style][f.direction], HAIR)

    return apply


def streak(region: str = "streak"):
    """A white lock at the front of the hair (Rogue)."""

    def apply(f: Frame, look):
        if f.direction == "front":
            for x, y in ((6, 2), (6, 3), (5, 4), (4, 5), (3, 6), (3, 7), (3, 8), (2, 9), (2, 10)):
                if f.get(x + PAD, y) and f.get(x + PAD, y).region == "hair":
                    f.put(x + PAD, y, region)
        elif f.direction == "side":
            for x, y in ((8, 2), (9, 3), (10, 4), (10, 5)):
                if f.get(x + PAD, y) and f.get(x + PAD, y).region == "hair":
                    f.put(x + PAD, y, region)

    return apply


# --- headgear ---------------------------------------------------------------------------

MASK = {"M": ("mask", "base"), "m": ("mask", "shadow"), "n": ("mask", "light"), "V": ("visor", "base"), "v": ("visor", "light"),
        "D": ("visor", "shadow"), "w": ("eyewhite", "base"), "e": ("eye", "base"), "S": ("skin", "base"), "s": ("skin", "shadow")}


def visor():
    """Cyclops' ruby quartz visor: a band across the eyes, wrapping around the head."""
    art = {
        "front": """
05 ...MMMMMMMMMMm..
06 ..MVvvvvvvvvVm..
07 ..MVVVVVVVVVVm..
08 ...mmmmmmmmmm...
""",
        "back": """
06 ...MMMMMMMMMMm..
07 ...mmmmmmmmmmm..
""",
        "side": """
05 ....MMMMMMMMM...
06 ....MMMMMMVvvv..
07 ....MMMMMMVVVV..
08 ....mmmmmmmmm...
""",
    }

    def apply(f: Frame, look):
        f.overlay(art[f.direction], MASK)

    return apply


def cowl(fins: bool = True, color_face: bool = False):
    """Full head mask with the lower face open; Wolverine's fins when `fins`."""
    art = {
        "front": """
02 .....MMMMMM.....
03 ....nMMMMMMm....
04 ...nMMMMMMMMm...
05 ...MMDDMMDDMm...
06 ...MDwewDweMm...
07 ...MMDDMMDDmm...
""",
        "back": """
02 .....MMMMMM.....
03 ....nMMMMMMm....
04 ...nMMMMMMMMm...
05 ...MMMMMMMMMm...
06 ...MMMMMMMMMm...
07 ...MMMMMMMMMm...
08 ...mMMMMMMMMm...
09 ....mmmmmmmm....
""",
        "side": """
02 .....MMMMMM.....
03 ....nMMMMMMM....
04 ....MMMMMMMMM...
05 ....MMMMMMDDM...
06 ....MMMMMMwe....
07 ....MMMMMMDDS...
08 ....mMMMm.......
09 .....mm.........
""",
    }
    fin = {
        "front": """
00 ..M.........M...
01 ..MM.......mM...
02 ..nMM.....mMm...
03 ...nM.....Mm....
""",
        "back": """
00 ..M.........M...
01 ..MM.......mM...
02 ..nMM.....mMm...
03 ...nM.....Mm....
""",
        "side": """
00 ...M............
01 ...MM...........
02 ...nMM..........
03 ....nM..........
""",
    }

    def apply(f: Frame, look):
        f.overlay(art[f.direction], MASK)
        if fins:
            f.overlay(fin[f.direction], MASK)

    return apply


def helmet(crest: bool = False, front_open: bool = True):
    """Rounded helmet with the face showing (Magneto, Juggernaut)."""
    art = {
        "front": """
01 ......nMMM......
02 ....nnMMMMMm....
03 ...nMMMMMMMMm...
04 ..nMMMMMMMMMMm..
05 ..MMMDDDDDDMMm..
06 ..MMD......DMm..
07 ..MMD......DMm..
08 ..MMD......Dmm..
09 ..mMM......Mmm..
10 ...mm......mm...
""",
        "back": """
01 ......nMMM......
02 ....nnMMMMMm....
03 ...nMMMMMMMMm...
04 ..nMMMMMMMMMMm..
05 ..MMMMMMMMMMMm..
06 ..MMMMMMMMMMMm..
07 ..MMMMMMMMMMMm..
08 ..MMMMMMMMMMmm..
09 ..mMMMMMMMMMmm..
10 ...mmmmmmmmmm...
""",
        "side": """
01 ......nMMM......
02 ....nnMMMMMM....
03 ...nMMMMMMMMM...
04 ...MMMMMMMMMMM..
05 ...MMMMMMMDDD...
06 ...MMMMMMMD.....
07 ...MMMMMMMD.....
08 ...MMMMMMMD.....
09 ...mMMMMMMM.....
10 ....mmmmmm......
""",
    }
    crest_art = {
        "front": """
00 ......MMM.......
""",
        "back": """
00 ......MMM.......
""",
        "side": """
00 ......MMMM......
01 .....M..........
""",
    }

    def apply(f: Frame, look):
        f.overlay(art[f.direction], MASK)
        if crest:
            f.overlay(crest_art[f.direction], MASK)

    return apply


def eyemask():
    """A domino mask around the eyes."""
    art = {
        "front": """
06 ...MMMMMMMMMM...
07 ....MM....MM....
""",
        "back": "",
        "side": """
06 ....MMMMMMMMM...
07 .........MM.....
""",
    }

    def apply(f: Frame, look):
        a = art[f.direction]
        if a:
            f.overlay(a, MASK)
            # Eyes show through the mask.
            if f.direction == "front":
                f.overlay("06 ......we.we.....", MASK)
            elif f.direction == "side":
                f.overlay("06 ...........we..", MASK)

    return apply


def shades():
    """Sunglasses (visor region)."""
    art = {
        "front": """
06 ...VVvVDVvVVD...
07 ....VVD..VVD....
""",
        "back": "",
        "side": """
06 .......DDVvVV...
07 ..........VV....
""",
    }

    def apply(f: Frame, look):
        if art[f.direction]:
            f.overlay(art[f.direction], MASK)

    return apply


def band(region: str = "band", row: int = 4):
    """Headband or tiara across the forehead."""

    def apply(f: Frame, look):
        span = f.span(row, "skin", "hair")
        if not span:
            return
        x0, x1 = span
        for x in range(x0, x1 + 1):
            f.put(x, row, region, "shadow" if x == x1 else "base")
        if f.direction == "front":
            f.put((x0 + x1) // 2, row - 1, region, "light")

    return apply


def glowing_eyes(region: str = "glow"):
    def apply(f: Frame, look):
        for x, y, c in list(f.where("eye", "eyewhite")):
            f.put(x, y, region, "light" if c.region == "eye" else "base")

    return apply


def beard(region: str = "hair"):
    def apply(f: Frame, look):
        if f.direction == "front":
            f.overlay("""
08 ...H........h...
09 ....HHHmmHhh....
10 .....hhhhhh.....
""".replace("m", "H"), HAIR)
        elif f.direction == "side":
            f.overlay("""
08 ........H.H.....
09 .....HHHHHHH....
10 ......hhhhh.....
""", HAIR)

    return apply


# --- costume ----------------------------------------------------------------------------

def recolor(src: str, dst: str, rows: range | None = None, where=None):
    """Moves cells of region `src` (optionally only on some rows) to region `dst`."""

    def apply(f: Frame, look):
        for x, y, c in list(f.where(src)):
            if rows is not None and y not in rows:
                continue
            if where is not None and not where(f, x, y):
                continue
            c.region = dst

    return apply


def trunks(region: str = "trunks"):
    return recolor("legs", region, rows=range(17, 19))


def tall_boots(top: int = 19):
    return recolor("legs", "boots", rows=range(top, 21))


def long_gloves(top: int = 14):
    return recolor("arm", "hand", rows=range(top, 17))


def yoke(region: str = "yoke", bottom: int = 12):
    """Shoulders and upper chest in a second color."""

    def apply(f: Frame, look):
        for x, y, c in list(f.where("torso", "arm")):
            if y <= bottom:
                c.region = region

    return apply


def sides(region: str = "sides", legs: bool = True):
    """Side panels on the torso (and legs): a lighter center, darker flanks."""

    def apply(f: Frame, look):
        for y in range(11, 21):
            regions = ("torso",) + (("legs",) if legs else ())
            span = f.span(y, *regions)
            if not span:
                continue
            x0, x1 = span
            if f.direction == "side":
                targets = [x1] if y < 17 else []
            else:
                targets = [x0, x1] if x1 - x0 >= 4 else []
            for x in targets:
                c = f.get(x, y)
                if c and c.region in regions:
                    c.region = region

    return apply


def stripe(region: str = "stripe", width: int = 2, rows: range = range(11, 17)):
    """Vertical stripe down the middle of the chest."""

    def apply(f: Frame, look):
        if f.direction == "side":
            return
        for y in rows:
            span = f.span(y, "torso")
            if not span:
                continue
            mid = (span[0] + span[1] + 1) // 2
            for x in range(mid - width // 2, mid - width // 2 + width):
                c = f.get(x, y)
                if c and c.region == "torso":
                    c.region = region

    return apply


def straps(region: str = "strap"):
    """Cyclops' X harness across the chest (front) and back."""

    def apply(f: Frame, look):
        if f.direction == "side":
            for y in range(11, 16):
                span = f.span(y, "torso")
                if span:
                    f.put(span[0] + 1, y, region)
            return
        span = f.span(13, "torso")
        if not span:
            return
        x0, x1 = span
        for i, y in enumerate(range(11, 16)):
            for x in (x0 + i, x1 - i):
                c = f.get(x, y)
                if c and c.region == "torso":
                    c.region = region

    return apply


def emblem(region: str = "emblem", row: int = 12):
    """A small X or round logo in the middle of the chest."""

    def apply(f: Frame, look):
        if f.direction != "front":
            return
        span = f.span(row, "torso", "yoke", "stripe", "sides")
        if not span:
            return
        mid = (span[0] + span[1]) // 2
        for dx, dy in ((0, 0), (1, 0), (0, 1), (1, 1)):
            f.put(mid + dx, row + dy, region, "light" if (dx, dy) == (0, 0) else "base")

    return apply


def jacket(region: str = "jacket", length: int = 18, open_front: bool = True, sleeves: bool = True):
    """A jacket or coat over the torso; `length` is the last row it covers."""

    def apply(f: Frame, look):
        if sleeves:
            for x, y, c in list(f.where("arm")):
                c.region = region
        for y in range(11, length + 1):
            span = f.span(y, "torso", "belt", "buckle", "legs", "sides", "yoke", "stripe", "strap", "trunks")
            if not span:
                continue
            x0, x1 = span
            if f.direction == "back":
                cols = range(x0, x1 + 1)
            elif f.direction == "side":
                cols = range(x0, x1 + 1) if y < 17 else range(x0 - (y - 16), x1 + 1)
            else:
                cols = [x for x in range(x0, x1 + 1) if not open_front or x < x0 + 2 or x > x1 - 2]
                if y >= 17:
                    cols = [x0 - 1, x0, x1, x1 + 1]
            for x in cols:
                c = f.get(x, y)
                if y >= 17 or c is None:
                    if c is None and y >= 17 and f.direction != "back" and f.direction != "side" and x not in (x0 - 1, x1 + 1):
                        continue
                tone = "shadow" if (f.direction != "side" and x >= (x0 + x1) // 2 + 2) or (c and c.tone == "shadow") else "base"
                f.put(x, y, region, tone)
        if f.direction == "front":
            # Collar.
            for x in range(PAD + 4, PAD + 6):
                f.put(x, 11, region, "light")
            for x in range(PAD + 10, PAD + 12):
                f.put(x, 11, region, "base")

    return apply


def cape(region: str = "cape", collar: bool = True):
    """A cape hanging behind; seen whole from the back, at the shoulders from the front."""

    def apply(f: Frame, look):
        if f.direction == "back":
            for y in range(11, 23):
                spread = min(3, (y - 10) // 3)
                x0, x1 = PAD + 3 - spread, PAD + 13 + spread
                for x in range(x0, x1 + 1):
                    if y == 22 and (x + f.frame) % 3 == 0:
                        continue
                    tone = "shadow" if x >= x1 - 1 or (x - x0) % 4 == 3 else ("light" if x == x0 else "base")
                    f.put(x, y, region, tone)
        elif f.direction == "front":
            for y in range(11, 22):
                spread = min(3, (y - 10) // 3)
                for x in (PAD + 2 - spread + 1, PAD + 14 + spread - 1):
                    f.put(x, y, region, "shadow") if f.get(x, y) is None else None
                for x in (PAD + 2 - spread, PAD + 14 + spread):
                    if f.get(x, y) is None:
                        f.put(x, y, region, "shadow")
            if collar:
                f.put(PAD + 3, 10, region, "base")
                f.put(PAD + 12, 10, region, "shadow")
        else:
            sway = 1 if f.frame in (1, 3) else 0
            for y in range(11, 22):
                back = min(4, (y - 10) // 2) + (sway if y > 15 else 0)
                for x in range(PAD + 5 - back, PAD + 6):
                    if f.get(x, y) is None:
                        f.put(x, y, region, "shadow" if x == PAD + 5 - back else "base")

    return apply


def wings(region: str = "wing", metal: bool = False):
    """Feathered (or metal) wings folded behind the shoulders."""
    front = """
02 ..IWW..............WWI..
03 .IWWWW............WWWWw.
04 .WWWWWw..........wWWWWw.
05 IWWWwww..........wwwWWw.
06 WWWw..............wwWWw.
07 WWw.................wWw.
08 WWw.................wWw.
09 Ww...................ww.
10 Ww...................ww.
11 W.....................w.
12 W.....................w.
13 w.....................w.
"""
    back = """
03 ..WW................WW..
04 .WWWW..............WWWW.
05 WWWWWw............wWWWWW
06 WWWWWWw..........wWWWWWW
07 WWWwWWWw........wWWWwWWW
08 WWWwWWWWw......wWWWWwWWW
09 WWwWWWWWw......wWWWWWwWW
10 WWwWWWWw........wWWWWwWW
11 WwWWWWw..........wWWWWwW
12 WwWWWw............wWWWwW
13 W.wWw..............wWw.W
14 w..w................w..w
"""
    side = """
02 ......WW................
03 .....WWWW...............
04 ....WWWWww..............
05 ...WWWWWw...............
06 ..WWWWWw................
07 .WWWWWw.................
08 WWWWWw..................
09 WWWWw...................
10 WWWw....................
11 WWw.....................
12 Ww......................
13 W.......................
"""
    legend = {"W": (region, "base"), "w": (region, "shadow"), "I": (region, "light")}

    def apply(f: Frame, look):
        art = {"front": front, "back": back, "side": side}[f.direction]
        # Wings beat slowly with the step frames.
        dy = -1 if f.frame in (1, 3) else 0
        lines = []
        for line in art.strip("\n").split("\n"):
            row, _, px = line.partition(" ")
            lines.append(f"{int(row) + dy:02d} {px}")
        f.overlay("\n".join(lines), legend, dx=0 if f.direction != "side" else 1, under=f.direction != "back")
        if metal:
            for x, y, c in list(f.where(region)):
                if (x + y) % 3 == 0:
                    c.tone = "light"

    return apply


def tail(region: str = "skin", tip: bool = True):
    """A thin tail curling behind (Nightcrawler's ends in a spade)."""
    legend = {"T": (region, "base"), "t": (region, "shadow")}
    art = {
        "front": """
14 ..............T.
15 ..............T.
16 .............tT.
17 ..............T.
18 ...............T
19 ..............TT
""",
        "back": """
16 ........t.......
17 ........tT......
18 .........TT.....
19 ..........TT....
20 ...........TT...
21 ...........tTT..
""",
        "side": """
16 ....t...........
17 ...tT...........
18 ..tT............
19 .tT.............
20 tT..............
21 T...............
""",
    }

    def apply(f: Frame, look):
        f.overlay(art[f.direction], legend, under=True)

    return apply


def claws(region: str = "claw"):
    """Three blades from each fist."""

    def apply(f: Frame, look):
        hands = list(f.where("hand"))
        if not hands:
            return
        bottom = max(y for _, y, _ in hands)
        cols = sorted({x for x, y, _ in hands if y == bottom})
        if f.direction == "side":
            x = max(cols)
            for i in range(3):
                f.put(x + 1 + i, bottom - 1 - i, region, "light" if i == 2 else "base")
                f.put(x + i, bottom - 1 - i, region, "base") if i == 0 else None
            return
        left = [x for x in cols if x < WIDTH // 2]
        right = [x for x in cols if x >= WIDTH // 2]
        for group in (left, right):
            if not group:
                continue
            for x in group[:2]:
                for dy in range(1, 4):
                    if f.get(x, bottom + dy) is None:
                        f.put(x, bottom + dy, region, "light" if dy == 3 else "base")

    return apply


def big_hands(region: str = "hand"):
    """Oversized fists (Beast, Strong Guy)."""

    def apply(f: Frame, look):
        for x, y, c in list(f.where("hand")):
            if f.direction != "side":
                nx = x - 1 if x < WIDTH // 2 else x + 1
                if f.get(nx, y) is None:
                    f.put(nx, y, region, c.tone)

    return apply


def scarf(region: str = "scarf"):
    def apply(f: Frame, look):
        if f.direction == "side":
            f.overlay("10 ......SSSSs.....\n11 ......SSSSS.....\n12 .....Ss.........\n13 ....Ss..........", {"S": (region, "base"), "s": (region, "shadow")})
        else:
            f.overlay("10 .....SSSSSSs....\n11 ....SSSSSSSSs...", {"S": (region, "base"), "s": (region, "shadow")})

    return apply


def skin_marks(region: str, points: dict[str, list[tuple[int, int]]]):
    """Fixed pixels on the head or body, per direction, in body-space columns."""

    def apply(f: Frame, look):
        for x, y in points.get(f.direction, []):
            if f.get(x + PAD, y):
                f.put(x + PAD, y, region)

    return apply


def no_mouth():
    def apply(f: Frame, look):
        for x, y, c in list(f.where("mouth")):
            c.region = "skin"

    return apply


def ears(region: str = "skin", pointy: bool = True):
    def apply(f: Frame, look):
        if f.direction in ("front", "back"):
            f.put(PAD + 2, 6, region, "base")
            f.put(PAD + 13, 6, region, "shadow")
            if pointy:
                f.put(PAD + 1, 5, region, "base")
                f.put(PAD + 14, 5, region, "shadow")

    return apply
