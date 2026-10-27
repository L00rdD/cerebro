"""Each character's look: colors, hair, headgear and costume features."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable

from body import Frame

Feature = Callable[[Frame, "Look"], None]

SKIN = {
    "pale": "#F6D3B5",
    "light": "#EFC09A",
    "tan": "#D9A06E",
    "brown": "#A8673E",
    "dark": "#6E4127",
}


@dataclass
class Look:
    skin: str = SKIN["light"]
    hair: str = "#4A3426"
    torso: str = "#2F4FA8"
    legs: str | None = None
    arm: str | None = None
    hand: str | None = None
    boots: str = "#2A2A35"
    belt: str = "#E8C33A"
    buckle: str | None = None
    eye: str = "#1E1B2E"
    big: bool = False
    outline: str | None = None
    extra: dict[str, str] = field(default_factory=dict)
    features: list[Feature] = field(default_factory=list)

    def colors(self) -> dict[str, str]:
        return {
            "skin": self.skin,
            "neck": self.skin,
            "eye": self.eye,
            "eyewhite": "#F4F1EA",
            "mouth": "#B05A4E",
            "hair": self.hair,
            "torso": self.torso,
            "arm": self.arm or self.torso,
            "hand": self.hand or self.skin,
            "belt": self.belt,
            "buckle": self.buckle or self.belt,
            "legs": self.legs or self.torso,
            "boots": self.boots,
            **self.extra,
        }

    def dress(self, frame: Frame):
        for feature in self.features:
            feature(frame, self)


LOOKS: dict[str, Look] = {}


def look(cid: str, **kwargs) -> Look:
    LOOKS[cid] = Look(**kwargs)
    return LOOKS[cid]


from features import (  # noqa: E402
    band, beard, big_hands, cape, claws, cowl, visor, emblem, eyemask, glowing_eyes, hair, helmet, jacket,
    long_gloves, no_mouth, recolor, scarf, shades, sides, skin_marks, straps, streak, stripe, tail,
    tall_boots, trunks, wings, yoke, ears,
)

XGOLD = "#F2C230"
XBLUE = "#2D4FB5"
XNAVY = "#1F2A5C"

look("professor-x", skin=SKIN["light"], torso="#3B4C7A", legs="#2E3550", belt="#2E3550", boots="#2A2430",
     extra={"tie": "#8C2A3A", "shirt": "#E8E4D8"},
     features=[recolor("torso", "shirt", rows=range(11, 13), where=lambda f, x, y: f.direction == "front" and 9 <= x <= 13),
               stripe("tie", 1, range(11, 16))])
look("cyclops", torso=XBLUE, legs=XBLUE, hand=XGOLD, boots=XGOLD, belt=XGOLD, buckle="#C8202A", hair="#5A3A24",
     extra={"mask": XBLUE, "visor": "#E8282E", "strap": XGOLD},
     features=[hair("short"), visor(), straps(), tall_boots()])
look("wolverine", torso=XGOLD, legs=XNAVY, arm=XNAVY, hand="#2A3A80", boots=XNAVY, belt="#C8202A", buckle=XGOLD, hair="#2A1E18",
     extra={"mask": XGOLD, "visor": "#1E2347", "sides": "#2A3A80", "claw": "#D9DEE8", "trunks": XNAVY},
     features=[cowl(), sides("sides", legs=False), tall_boots(), claws()])
look("storm", skin=SKIN["brown"], hair="#F2F2F6", torso="#24242E", hand="#24242E", boots="#24242E", belt="#E8C33A",
     extra={"cape": "#2E2E3C", "band": "#E8C33A"},
     features=[hair("mane"), band(), cape(), tall_boots(18)])
look("jean-grey", hair="#D8462A", torso="#2E9A4A", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"yoke": XGOLD},
     features=[hair("long"), tall_boots(), long_gloves()])
look("rogue", hair="#7A3E24", torso="#3A9A3A", legs="#3A9A3A", arm="#E9C34A", hand="#E9C34A", boots="#E9C34A", belt="#E9C34A",
     extra={"streak": "#F4F2EE", "jacket": "#5A3A2A"},
     features=[hair("long"), streak(), jacket(length=16, sleeves=True), tall_boots()])
look("gambit", hair="#6A3A24", torso="#D04A8A", legs="#2A2A3A", hand="#3A2A2A", boots="#2A2A35", belt="#3A2A2A",
     extra={"jacket": "#8A5A34", "mask": "#2A2A3A", "eye": "#E8282E", "eyewhite": "#1A1A1A"},
     features=[hair("slick"), jacket(length=21), glowing_eyes("eye")])
look("beast", skin="#3A5AC8", hair="#2E46A8", torso="#2E2E3A", legs="#2E2E3A", hand="#3A5AC8", boots="#3A5AC8", belt="#E8C33A", big=True,
     features=[hair("curly"), recolor("torso", "skin", rows=range(11, 17)), recolor("arm", "skin"), shades(), big_hands()],
     extra={"visor": "#2A2A3A"})
look("nightcrawler", skin="#2A3E8A", hair="#1E2A64", torso="#2A2A38", hand="#F4F2EE", boots="#C8202A", belt="#C8202A",
     extra={"sides": "#C8202A", "glow": "#F2E64A"},
     features=[hair("short"), ears("skin"), glowing_eyes(), sides("sides", legs=False), tail("skin")])
look("angel", hair="#E8C33A", torso=XBLUE, legs=XBLUE, hand="#E8E8F0", boots="#E8E8F0", belt="#E8E8F0", buckle="#C8202A",
     extra={"wing": "#F4F2EE"},
     features=[wings(), hair("short"), tall_boots()])
look("colossus", skin="#A8B4C8", hair="#2A1E18", torso="#C8202A", legs="#C8202A", arm="#A8B4C8", hand="#A8B4C8", boots="#E8C33A", belt="#E8C33A", big=True,
     extra={"yoke": "#E8C33A"},
     features=[hair("short"), tall_boots(), stripe("yoke", 2, range(11, 12))])
look("magneto", torso="#B0242E", legs="#B0242E", hand="#6A2A8A", boots="#6A2A8A", belt="#6A2A8A", buckle="#E8C33A",
     extra={"mask": "#B0242E", "visor": "#6A2A8A", "cape": "#6A2A8A"},
     features=[helmet(crest=True), cape(), tall_boots()])

# --- the rest of the cast -----------------------------------------------------------------

def blob(art: dict[str, str], legend: dict[str, tuple[str, str]]):
    """Replaces the body with a custom shape (non-humanoid characters)."""

    def apply(f, look):
        for y in range(len(f.cells)):
            for x in range(len(f.cells[y])):
                f.cells[y][x] = None
        bob = 1 if f.frame in (1, 3) else 0
        lines = []
        for line in art[f.direction].strip("\n").split("\n"):
            row, _, px = line.partition(" ")
            lines.append(f"{int(row) - bob:02d} {px}")
        f.overlay("\n".join(lines), legend)

    return apply


BLOB = {"B": ("skin", "base"), "b": ("skin", "shadow"), "k": ("skin", "light"), "e": ("eye", "base"), "w": ("eyewhite", "base"),
        "m": ("mouth", "base"), "A": ("hand", "base"), "a": ("hand", "shadow"), "X": ("bone", "base"), "x": ("bone", "shadow")}

ICE = "#8FD3F4"

look("phoenix", hair="#E0401E", torso="#2E9A4A", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"emblem": XGOLD},
     features=[hair("mane"), emblem(), tall_boots(), long_gloves()])
look("iceman", skin=ICE, hair="#BFE9FF", torso=ICE, hand=ICE, boots="#6FBDE6", belt="#BFE9FF", eye="#2A6A9A",
     features=[hair("spiky"), recolor("arm", "skin")])
look("shadowcat", hair="#5A3420", torso="#2A2A38", legs="#2A2A38", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"yoke": XGOLD},
     features=[hair("bob"), yoke(bottom=11), tall_boots()])
look("jubilee", skin=SKIN["tan"], hair="#1E1A22", torso="#D84A8A", legs="#3A5AA8", hand="#E8C33A", boots="#E8E8F0", belt="#3A5AA8",
     extra={"jacket": "#F2C230", "visor": "#F06AAE"},
     features=[hair("bob"), shades(), jacket(length=18)])
look("psylocke", hair="#6A3AC8", torso="#2840A8", legs=SKIN["light"], hand="#2840A8", boots="#2840A8", belt="#D8262E",
     extra={"glow": "#FF4FD8"},
     features=[hair("long"), tall_boots(19), long_gloves(13), skin_marks("glow", {"front": [(7, 5)]})])
look("bishop", skin=SKIN["dark"], hair="#1A1418", torso="#2A3A6A", legs="#2A3A6A", hand="#2A2A35", boots="#2A2A35", belt="#B08A3A",
     extra={"yoke": "#B08A3A", "tattoo": "#C8202A"},
     features=[hair("ponytail"), yoke(bottom=11), skin_marks("tattoo", {"front": [(5, 4), (5, 5)], "side": [(10, 4), (10, 5)]})])
look("cable", hair="#E8E8EE", torso="#3A4A6A", legs="#2E3A52", arm="#6A7890", hand="#2A2A35", boots="#2A2A35", belt="#8A6A3A",
     extra={"glow": "#FFB23A", "strap": "#8A6A3A"},
     features=[hair("buzz"), straps(), skin_marks("glow", {"front": [(6, 6), (6, 7)], "side": [(10, 6)]}), tall_boots()])
look("emma-frost", hair="#F2E2A0", torso="#F4F2EE", hand="#F4F2EE", boots="#F4F2EE", belt="#D8D8E8",
     extra={"cape": "#E8E8F4"},
     features=[hair("long"), cape(), tall_boots(18), long_gloves()])
look("havok", hair="#E8C33A", torso="#24242E", hand="#24242E", boots="#24242E", belt="#24242E",
     extra={"emblem": "#F4F2EE", "ring": "#6FB7FF"},
     features=[hair("short"), recolor("torso", "ring", rows=range(12, 15),
               where=lambda f, x, y: f.direction == "front" and (abs(x - 11.5) + abs(y - 13)) in (2.5, 3.5)), emblem(row=12)])
look("polaris", hair="#3AC86A", torso="#2E8A3A", hand="#2E8A3A", boots="#2E8A3A", belt="#E8C33A",
     extra={"band": "#2E8A3A"},
     features=[hair("long"), band(row=3), tall_boots(18)])
look("banshee", hair="#D8642A", torso="#2E8A3A", legs="#2E8A3A", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"yoke": XGOLD},
     features=[hair("short"), yoke(bottom=11), tall_boots(), long_gloves()])
look("sunfire", hair="#1E1A22", torso="#E8C33A", legs="#D8262E", arm="#D8262E", hand="#D8262E", boots="#D8262E", belt="#D8262E",
     extra={"emblem": "#D8262E"},
     features=[hair("short"), emblem()])
look("thunderbird", skin=SKIN["brown"], hair="#1A1418", torso="#2A2A38", legs="#2A2A38", hand="#2A2A38", boots="#B8342A", belt="#B8342A",
     extra={"band": "#B8342A", "emblem": "#B8342A"},
     features=[hair("long"), band(), emblem(), tall_boots()])
look("forge", skin=SKIN["brown"], hair="#1A1418", torso="#4A5A72", legs="#3A4458", hand="#8A96A8", boots="#2A2A35", belt="#2A2A35",
     extra={"band": "#C8202A", "jacket": "#6A7890"},
     features=[hair("long"), band(), jacket(length=16)])
look("dazzler", hair="#F2E07A", torso="#E8E8F4", hand="#E8E8F4", boots="#E8E8F4", belt="#B8B8D8",
     extra={"mask": "#D8D8E8", "visor": "#B8B8D8", "emblem": "#FF7AD9"},
     features=[hair("mane"), eyemask(), emblem(), tall_boots(18)])
look("longshot", skin=SKIN["pale"], hair="#F2E07A", torso="#24242E", legs="#24242E", hand="#24242E", boots="#24242E", belt="#E8E8F0",
     extra={"stripe": "#E8E8F0"},
     features=[hair("slick"), stripe()])
look("cannonball", hair="#E8C33A", torso=XBLUE, legs=XBLUE, hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"yoke": XGOLD, "visor": "#6A4A2A"},
     features=[hair("short"), yoke(bottom=11), tall_boots()])
look("sunspot", skin=SKIN["brown"], hair="#1A1418", torso="#24242E", legs="#24242E", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"sides": XGOLD},
     features=[hair("curly"), sides("sides", legs=False), tall_boots()])
look("magik", skin=SKIN["pale"], hair="#F2E07A", torso="#24242E", legs="#24242E", arm="#A8B4C8", hand="#A8B4C8", boots="#24242E", belt="#C8202A",
     features=[hair("long"), tall_boots()])
look("wolfsbane", skin="#B8743A", hair="#C8502A", torso="#2E8A3A", legs="#2E8A3A", hand="#B8743A", boots="#B8743A", belt=XGOLD,
     extra={"yoke": XGOLD},
     features=[hair("bob"), ears("hair"), yoke(bottom=11)])
look("mirage", skin=SKIN["brown"], hair="#1A1418", torso="#C8302A", legs="#24242E", hand="#C8302A", boots="#24242E", belt=XGOLD,
     extra={"band": "#E8E8F0", "yoke": XGOLD},
     features=[hair("long"), band(), yoke(bottom=11)])
look("cypher", hair="#E8C33A", torso="#24242E", legs="#24242E", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"glow": "#7CFFB2"},
     features=[hair("short"), skin_marks("glow", {"front": [(3, 8), (4, 9), (11, 8)], "side": [(9, 8), (8, 9)]})])
look("warlock", skin="#2A2A33", hair="#2A2A33", torso="#2A2A33", hand="#2A2A33", boots="#2A2A33", belt="#E0C020",
     extra={"glow": "#E0C020"},
     features=[glowing_eyes(), recolor("arm", "skin"), no_mouth(),
               skin_marks("glow", {"front": [(5, 3), (6, 2), (9, 2), (10, 3), (7, 11), (8, 12), (7, 13), (8, 14)],
                                   "back": [(5, 3), (8, 2), (10, 4), (7, 12), (8, 13)], "side": [(6, 3), (9, 2), (8, 12), (8, 13)]})])
look("karma", skin=SKIN["tan"], hair="#1A1418", torso="#3A3A8A", legs="#3A3A8A", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     features=[hair("bob"), tall_boots()])
look("boom-boom", hair="#F2E07A", torso="#3A5AA8", legs="#3A5AA8", hand="#F06AAE", boots="#F06AAE", belt="#F2C230",
     extra={"jacket": "#F06AAE", "visor": "#2A2A35"},
     features=[hair("ponytail"), shades(), jacket(length=15)])
look("domino", skin="#F2F2F2", hair="#1A1418", torso="#24242E", legs="#24242E", hand="#24242E", boots="#24242E", belt="#5A5A6A",
     extra={"mask": "#1A1418"},
     features=[hair("short"), skin_marks("mask", {"front": [(5, 5), (5, 6), (6, 5), (5, 7), (7, 5)], "side": [(10, 5), (11, 5)]})])
look("x-23", hair="#1A1418", torso="#24242E", legs="#24242E", hand="#24242E", boots="#24242E", belt="#5A5A6A",
     extra={"sides": "#6A3AC8", "claw": "#D9DEE8"},
     features=[hair("long"), sides("sides"), claws()])
look("armor", skin=SKIN["tan"], hair="#1A1418", torso="#24242E", legs="#24242E", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"yoke": XGOLD},
     features=[hair("bob"), yoke(bottom=11), tall_boots()])
look("pixie", hair="#F06AAE", torso="#6A3AC8", legs="#6A3AC8", hand="#F06AAE", boots="#F06AAE", belt="#F06AAE",
     extra={"wing": "#FFB6E1"},
     features=[wings(), hair("bob")])
look("blink", skin="#D88AD8", hair="#8A3AC8", torso="#4A2A6A", legs="#4A2A6A", hand="#D88AD8", boots="#24242E", belt="#7CC85A",
     extra={"mark": "#4A9A3A"},
     features=[hair("long"), skin_marks("mark", {"front": [(4, 7), (11, 7), (4, 8)], "side": [(9, 7), (9, 8)]})])
look("multiple-man", hair="#5A3A24", torso="#2E7A4A", legs="#2E7A4A", hand="#E8E8F0", boots="#E8E8F0", belt="#E8E8F0",
     extra={"emblem": "#E8E8F0"},
     features=[hair("short"), emblem(), tall_boots()])
look("strong-guy", hair="#E8C33A", torso="#24242E", legs="#24242E", hand="#24242E", boots="#24242E", belt="#E8C33A", big=True,
     extra={"visor": "#2A2A35", "yoke": "#6A3AC8"},
     features=[hair("buzz"), shades(), yoke(), big_hands()])
look("siryn", hair="#E0401E", torso="#2E8A7A", legs="#2E8A7A", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"yoke": XGOLD},
     features=[hair("mane"), yoke(bottom=11), tall_boots()])
look("rachel-summers", hair="#E0401E", torso="#B8282E", legs="#B8282E", hand=XGOLD, boots=XGOLD, belt=XGOLD,
     extra={"mark": "#B8282E", "emblem": XGOLD},
     features=[hair("long"), emblem(), skin_marks("mark", {"front": [(4, 8), (5, 9), (11, 8), (10, 9)], "side": [(10, 9), (9, 8)]})])
look("northstar", hair="#1A1418", torso="#24242E", legs="#24242E", hand="#E8E8F0", boots="#24242E", belt="#E8E8F0",
     extra={"yoke": "#E8E8F0"},
     features=[hair("spiky"), yoke(bottom=11)])
look("aurora", hair="#1A1418", torso="#E8E8F0", legs="#E8E8F0", hand="#24242E", boots="#24242E", belt="#24242E",
     extra={"yoke": "#24242E"},
     features=[hair("mane"), yoke(bottom=11), tall_boots()])
look("marrow", skin=SKIN["pale"], hair="#C84AA8", torso="#3A4A2A", legs="#4A3A2A", hand=SKIN["pale"], boots="#2A2A35", belt="#4A3A2A",
     extra={"bone": "#F2EEDC"},
     features=[hair("spiky"), skin_marks("bone", {"front": [(3, 11), (12, 11), (2, 10), (13, 10)], "back": [(7, 12), (8, 14), (7, 16)], "side": [(5, 12), (5, 14)]})])
look("chamber", hair="#5A3420", torso="#2A2A38", legs="#2A2A38", hand="#2A2A38", boots="#2A2A35", belt="#2A2A38",
     extra={"scarf": "#4A4A5A", "emblem": "#FF8A00", "jacket": "#3A3A4A"},
     features=[hair("short"), jacket(length=19), scarf(), emblem()])
look("husk", hair="#F2E07A", torso="#B8282E", legs="#2A2A38", hand="#B8282E", boots="#2A2A38", belt=XGOLD,
     features=[hair("long")])
look("monet", skin=SKIN["tan"], hair="#1A1418", torso="#6A3AC8", legs="#6A3AC8", hand="#E8E8F0", boots="#E8E8F0", belt="#E8C33A",
     extra={"yoke": "#E8E8F0"},
     features=[hair("long"), yoke(bottom=11), tall_boots()])
look("synch", skin=SKIN["dark"], hair="#1A1418", torso="#2A3A6A", legs="#2A3A6A", hand="#2A3A6A", boots="#2A2A35", belt="#9BE15D",
     extra={"stripe": "#9BE15D"},
     features=[hair("buzz"), stripe(width=1)])
look("hope", hair="#E0401E", torso="#5A6A3A", legs="#4A3A2A", hand="#4A3A2A", boots="#3A2A20", belt="#8A6A3A",
     extra={"jacket": "#6A5A3A"},
     features=[hair("ponytail"), jacket(length=16)])
look("doop", skin="#7CC85A", eye="#1E1B2E", features=[blob({
    "front": """
08 ........kBBBB...........
09 .......kBBBBBBb.........
10 ......kBBBwwwBBb........
11 ......BBBwweewBbb.......
12 ......BBBwweewBbb.......
13 ......BBBBwwwBBbb.......
14 .....ABBBBBBBBBbbA......
15 ......BBBBmmmBBbb.......
16 ......bBBBBBBBbbb.......
17 .......bBBBBBbbb........
18 ........bbbbbbb.........
""",
    "back": """
08 ........kBBBB...........
09 .......kBBBBBBb.........
10 ......kBBBBBBBBb........
11 ......BBBBBBBBBbb.......
12 ......BBBBBBBBBbb.......
13 ......BBBBBBBBBbb.......
14 .....ABBBBBBBBBbbA......
15 ......BBBBBBBBBbb.......
16 ......bBBBBBBBbbb.......
17 .......bBBBBBbbb........
18 ........bbbbbbb.........
""",
    "side": """
08 ........kBBBB...........
09 .......kBBBBBBb.........
10 ......kBBBBBBwwb........
11 ......BBBBBBBwewb.......
12 ......BBBBBBBwewb.......
13 ......BBBBBBBBwwb.......
14 ......BBBBBABBBbb.......
15 ......BBBBBBBBmmb.......
16 ......bBBBBBBBbbb.......
17 .......bBBBBBbbb........
18 ........bbbbbbb.........
"""}, BLOB)])
look("rockslide", skin="#8A7A6A", hair="#6A5A4A", torso="#8A7A6A", legs="#3A3A4A", hand="#8A7A6A", boots="#6A5A4A", belt="#3A3A4A", big=True,
     extra={"crack": "#5A4A3E"},
     features=[recolor("torso", "skin"), recolor("arm", "skin"), no_mouth(),
               skin_marks("crack", {"front": [(5, 3), (6, 4), (10, 4), (7, 12), (8, 13), (5, 14)], "back": [(6, 3), (9, 5), (7, 13)], "side": [(7, 3), (8, 13)]})])
look("anole", skin="#6AB84A", hair="#4A9A3A", torso="#B8282E", legs="#2A2A38", hand="#6AB84A", boots="#2A2A38", belt=XGOLD,
     features=[hair("mohawk"), ears("skin")])
look("elixir", skin="#E8C33A", hair="#F2E07A", torso="#B8282E", legs="#2A2A38", hand="#E8C33A", boots="#2A2A38", belt=XGOLD,
     features=[hair("short")])
look("mercury", skin="#C8D0DC", hair="#E8ECF2", torso="#24242E", legs="#24242E", hand="#C8D0DC", boots="#24242E", belt=XGOLD,
     extra={"claw": "#E8ECF2"},
     features=[hair("bob")])
look("hellion", hair="#3A2A20", torso="#24242E", legs="#24242E", hand="#B8282E", boots="#B8282E", belt="#B8282E",
     extra={"visor": "#D8462A", "jacket": "#B8282E"},
     features=[hair("spiky"), shades(), jacket(length=15)])
look("sage", hair="#1A1418", torso="#24242E", legs="#24242E", hand="#24242E", boots="#24242E", belt="#5A5A6A",
     extra={"visor": "#8AC8E8"},
     features=[hair("long"), shades()])
look("caliban", skin="#EEEAE4", torso="#4A4A3A", legs="#3A3A30", hand="#EEEAE4", boots="#2A2A25", belt="#3A3A30", eye="#101010",
     extra={"jacket": "#5A5A48", "eyewhite": "#101010"},
     features=[jacket(length=21), ears("skin", pointy=False)])
look("cecilia-reyes", skin=SKIN["tan"], hair="#3A2418", torso="#4A7AC8", legs="#2A3A5A", hand=SKIN["tan"], boots="#2A2A35", belt="#2A3A5A",
     extra={"jacket": "#F4F2EE"},
     features=[hair("ponytail"), jacket(length=19)])
look("juggernaut", torso="#8A4A2A", legs="#8A4A2A", hand="#5A3A2A", boots="#5A3A2A", belt="#3A2A20", big=True,
     extra={"mask": "#A8342A", "visor": "#5A2A20", "strap": "#5A3A2A"},
     features=[helmet(), straps(), big_hands()])
look("rictor", skin=SKIN["tan"], hair="#3A2418", torso="#2E5AA8", legs="#3A4A6A", hand=SKIN["tan"], boots="#3A2A20", belt="#3A2A20",
     extra={"jacket": "#6A4A2A"},
     features=[hair("short"), jacket(length=16)])
look("shatterstar", skin=SKIN["pale"], hair="#C8302A", torso="#E8E8F0", legs="#E8E8F0", arm="#3A5AA8", hand="#3A5AA8", boots="#3A5AA8", belt="#3A5AA8",
     extra={"mark": "#1A1418", "claw": "#D9DEE8"},
     features=[hair("long"), skin_marks("mark", {"front": [(5, 5), (5, 8)], "side": [(10, 5)]}), tall_boots()])
look("feral", skin="#E0882A", hair="#F4E8D8", torso="#2A2A38", legs="#2A2A38", hand="#E0882A", boots="#E0882A", belt="#C8202A",
     extra={"claw": "#F4F2EE"},
     features=[hair("long"), ears("skin"), claws()])
look("stepford", hair="#F2E07A", torso="#F4F2EE", legs="#5A5A6A", hand=SKIN["light"], boots="#2A2A35", belt="#5A5A6A",
     extra={"tie": "#B8282E"},
     features=[hair("bob"), stripe("tie", 1, range(11, 14))])
look("gentle", skin=SKIN["dark"], hair="#1A1418", torso="#24242E", legs="#24242E", hand=SKIN["dark"], boots="#24242E", belt="#6A3AC8", big=True,
     extra={"mark": "#6A3AC8"},
     features=[hair("buzz"), recolor("arm", "skin"), skin_marks("mark", {"front": [(4, 6), (11, 6), (4, 7)], "side": [(9, 7), (9, 8)]})])
look("glob-herman", skin="#F2A8C0", eye="#1E1B2E", features=[blob({
    "front": """
03 .........kBBBb..........
04 ........kBBBBBb.........
05 .......kBBXXXBBb........
06 .......BBXeXeXBb........
07 .......BBXXXXXBb........
08 .......BBBxxxBBb........
09 ......kBBBBXBBBBb.......
10 .....kBBXXXXXXXBBb......
11 .....BBBBBBXBBBBBb......
12 .....BBBXXXXXXXBBb......
13 ....ABBBBBBXBBBBBbA.....
14 .....BBBXXXXXXXBBb......
15 .....BBBBBBXBBBBBb......
16 .....bBBBBXXXBBBbb......
17 ......bBBBXBXBBbb.......
18 ......BBBXBBBXBbb.......
19 ......BBXBBBBBXbb.......
20 ......BBBBb.BBBbb.......
21 ......bbbb...bbbb.......
""",
    "back": """
03 .........kBBBb..........
04 ........kBBBBBb.........
05 .......kBBXXXBBb........
06 .......BBXXXXXBb........
07 .......BBXXXXXBb........
08 .......BBBxxxBBb........
09 ......kBBBBXBBBBb.......
10 .....kBBBBBXBBBBBb......
11 .....BBBBBBXBBBBBb......
12 .....BBBBBBXBBBBBb......
13 ....ABBBBBBXBBBBBbA.....
14 .....BBBBBBXBBBBBb......
15 .....BBBBBBXBBBBBb......
16 .....bBBBBXXXBBBbb......
17 ......bBBBXBXBBbb.......
18 ......BBBXBBBXBbb.......
19 ......BBXBBBBBXbb.......
20 ......BBBBb.BBBbb.......
21 ......bbbb...bbbb.......
""",
    "side": """
03 .........kBBBb..........
04 ........kBBBBBb.........
05 .......kBBBXXXBb........
06 .......BBBXXXeBb........
07 .......BBBXXXXXb........
08 .......BBBBxxxBb........
09 ......kBBBBXBBBb........
10 ......BBBBXXXXBb........
11 ......BBBBBXBBBb........
12 ......BBBBXXXXBb........
13 ......BBBBBXABBb........
14 ......BBBBXXXXBb........
15 ......BBBBBXBBBb........
16 ......bBBBXXBBbb........
17 .......bBBXBXBb.........
18 .......BBXBBBXb.........
19 .......BXBBBBBX.........
20 .......BBBb.BBb.........
21 .......bbb...bb.........
"""}, BLOB)], extra={"bone": "#FFF6EA"})
look("beak", skin="#F4F2EE", hair="#E8E4DC", torso="#B8282E", legs="#2A2A38", hand="#F4F2EE", boots="#E8B83A", belt=XGOLD,
     extra={"beak": "#E8B83A"},
     features=[hair("spiky"), no_mouth(), skin_marks("beak", {"front": [(7, 8), (8, 8), (7, 9)], "side": [(13, 7), (13, 8), (12, 8), (14, 8)]})])
look("nocturne", skin="#3A4E9A", hair="#1E1A30", torso="#24242E", legs="#24242E", hand="#3A4E9A", boots="#24242E", belt="#C8202A",
     extra={"glow": "#F2E64A"},
     features=[hair("long"), glowing_eyes(), tail("skin")])
look("kid-omega", hair="#F06AAE", torso="#24242E", legs="#3A3A4A", hand=SKIN["light"], boots="#2A2A35", belt="#5A5A6A",
     extra={"visor": "#2A2A35", "jacket": "#4A4A5A"},
     features=[hair("mohawk"), shades(), jacket(length=16)])
look("archangel", skin="#5A7AD8", hair="#1E1A22", torso="#24242E", legs="#24242E", hand="#5A7AD8", boots="#24242E", belt="#24242E",
     extra={"wing": "#9AA8C0", "sides": "#3A3A5A"},
     features=[wings(metal=True), hair("short"), sides("sides", legs=False)])
