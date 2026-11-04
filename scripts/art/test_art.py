"""The generators still draw the committed sprites, pixel for pixel.

    python3 -m pytest scripts/art      (needs Pillow and pytest)
"""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

import pytest
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))

import characters  # noqa: E402
import emotes  # noqa: E402
import rooms  # noqa: E402
from body import HEIGHT, WIDTH, Frame  # noqa: E402
from looks import LOOKS  # noqa: E402
from pix import Canvas  # noqa: E402

ASSETS = Path(__file__).resolve().parents[2] / "assets"


def pixels(image: Image.Image) -> tuple[tuple[int, int], bytes]:
    image = image.convert("RGBA")
    return image.size, image.tobytes()


def committed(path: Path) -> tuple[tuple[int, int], bytes]:
    with Image.open(path) as image:
        return pixels(image)


@pytest.mark.parametrize("cid", sorted(LOOKS))
def test_characters_match_their_sprite(cid):
    assert pixels(characters.render(LOOKS[cid]).image) == committed(ASSETS / "characters" / f"{cid}.png")


@pytest.mark.parametrize("spec", rooms.ROOMS, ids=lambda spec: spec["id"])
def test_rooms_match_their_sprite(spec):
    assert pixels(rooms.render(spec).image) == committed(ASSETS / "rooms" / f"{spec['id']}.png")


def test_emotes_match_their_sheet(tmp_path, monkeypatch):
    (tmp_path / "assets" / "ui").mkdir(parents=True)
    monkeypatch.setattr(emotes, "ROOT", tmp_path)

    emotes.main()

    assert committed(tmp_path / "assets" / "ui" / "emotes.png") == committed(ASSETS / "ui" / "emotes.png")


def test_characters_main_rewrites_the_manifest_and_sprites(tmp_path, monkeypatch):
    shutil.copy(ASSETS / "characters" / "characters.json", tmp_path)
    monkeypatch.setattr(characters, "OUT", tmp_path)

    characters.main(["storm", "wolverine"])

    assert sorted(p.name for p in tmp_path.glob("*.png")) == ["storm.png", "wolverine.png"]
    assert committed(tmp_path / "storm.png") == committed(ASSETS / "characters" / "storm.png")
    manifest = (tmp_path / "characters.json").read_text()
    assert manifest == (ASSETS / "characters" / "characters.json").read_text()


def test_characters_main_needs_a_look_for_every_character(tmp_path, monkeypatch):
    manifest = json.loads((ASSETS / "characters" / "characters.json").read_text())
    manifest["characters"].append({"id": "nobody"})
    (tmp_path / "characters.json").write_text(json.dumps(manifest))
    monkeypatch.setattr(characters, "OUT", tmp_path)

    with pytest.raises(SystemExit, match="no look for: nobody"):
        characters.main([])


def test_rooms_main_writes_the_manifest(tmp_path, monkeypatch):
    monkeypatch.setattr(rooms, "ASSETS", tmp_path)
    monkeypatch.setattr(rooms, "render", lambda spec: Canvas(1, 1))

    rooms.main([])

    assert (tmp_path / "rooms.json").read_text() == (ASSETS / "rooms" / "rooms.json").read_text()
    assert len(list(tmp_path.glob("*.png"))) == len(rooms.ROOMS)


def test_rooms_main_renders_only_the_rooms_asked_for(tmp_path, monkeypatch):
    monkeypatch.setattr(rooms, "ASSETS", tmp_path)

    rooms.main(["office"])

    assert [p.name for p in tmp_path.iterdir()] == ["office.png"]
    assert committed(tmp_path / "office.png") == committed(ASSETS / "rooms" / "office.png")


def test_paint_needs_a_color_for_every_region():
    frame = Frame("front", 0)
    LOOKS["storm"].dress(frame)

    with pytest.raises(KeyError, match="no color for region"):
        characters.paint(frame, {}, None)


def test_sprite_sheets_have_one_frame_per_cell():
    sheet = characters.render(LOOKS["storm"])

    assert (sheet.width, sheet.height) == (WIDTH * 4, HEIGHT * 3)


def test_shift_moves_every_cell_and_drops_those_leaving_the_frame():
    frame = Frame("front", 0)
    before = {(x, y): c for x, y, c in frame.where(*{c.region for row in frame.cells for c in row if c})}

    frame.shift(9, -1)

    after = {(x, y): c for x, y, c in frame.where(*{c.region for row in frame.cells for c in row if c})}
    assert after == {(x + 9, y - 1): c for (x, y), c in before.items() if 0 <= x + 9 < WIDTH and 0 <= y - 1 < HEIGHT}
    assert len(after) < len(before)
