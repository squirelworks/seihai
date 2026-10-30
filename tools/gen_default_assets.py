#!/usr/bin/env python3
"""Generate Seihai's default asset set (assets/default/).

Every file is original: drawn here, pixel by pixel, from primitives and hand
specified palettes, or synthesised from noise and sine tones. No image or sound
of any Graal client, playerworld or other asset is read. Numbers taken from
elsewhere: sizes, grid layouts and frame counts from
documentation/asset-reference.md, and the tile type lists of the standard
Graal tile layout (tools/graal_tile_lists.txt, from the 2.22 arrays.dat), so
that pics1.png and tiledefs.txt use the same tile ids as a Graal pics1.png.

Usage (from the repository root, Python 3.9+, Pillow):

    python3 tools/gen_default_assets.py                 # write assets/default/
    python3 tools/gen_default_assets.py --check         # regenerate in a temp
                                                        # dir, compare (exit 1)
    python3 tools/gen_default_assets.py --preview P.png # contact sheet of OUR art
    python3 tools/gen_default_assets.py --import-arrays arrays.dat
                                        # rewrite tools/graal_tile_lists.txt

No network access, no randomness other than `random.Random(SEED)`, so two runs
give the same files. PNG files are compared by decoded pixels in --check, not by
bytes, because zlib output can differ between zlib builds.
"""
import argparse
import math
import os
import random
import struct
import sys
import tempfile
import wave

from PIL import Image, ImageDraw

SEED = 20261025  # fixed; change it and the grass tufts and noise change
TILE = 16

# ---------------------------------------------------------------------------
# Palette helper
# ---------------------------------------------------------------------------


class Pal:
    """Named palette. Index 0 is reserved as 'transparent' when asked."""

    def __init__(self, transparent=False):
        self.names = []
        self.rgb = []
        if transparent:
            self.add("clear", (0, 0, 0))

    def add(self, name, rgb):
        self.names.append(name)
        self.rgb.append(tuple(rgb))
        return len(self.names) - 1

    def __getitem__(self, name):
        return self.names.index(name)

    def flat(self, size):
        out = []
        for c in self.rgb:
            out.extend(c)
        out.extend([0] * (size * 3 - len(out)))
        return out


def save_png(im, pal, path, bits, size, transparent, clear=0):
    assert len(pal.names) <= size, (path, len(pal.names), size)
    im.putpalette(pal.flat(size))
    kw = dict(optimize=True, bits=bits, compress_level=9)
    if transparent:
        kw["transparency"] = clear
    im.save(path, "PNG", **kw)


def save_gif_copy(png_path):
    """A .gif copy of a player-art PNG, same palette and transparent index.
    Builds up to 2.05 read heads, bodies, swords and shields as .gif only."""
    im = Image.open(png_path)
    kw = {}
    if "transparency" in im.info:
        kw["transparency"] = im.info["transparency"]
    im.save(os.path.splitext(png_path)[0] + ".gif", "GIF", optimize=False, **kw)


# ---------------------------------------------------------------------------
# Tile sheet (pics1.png): 2048 x 512, 128 x 32 cells of 16 x 16
# ---------------------------------------------------------------------------

SHEET_W, SHEET_H = 2048, 512

# The sheet follows the standard Graal tile layout: tile id N is the cell the
# real pics1.png has for N, and its type is the one the real arrays.dat gives
# N. So the same tiledefs.txt fits this sheet and a Graal pics1.png, and an
# ordinary Graal level walks the same on both. The type lists are numbers
# only, kept in tools/graal_tile_lists.txt (see there for the source); no
# pixel of a Graal sheet was used. Each id is drawn with the art of its type
# (wall, water, floor, ...), and the ids below get art of their own.

# Ids whose meaning is documented (Graal_analysis notes and Seihai's docs).
G = dict(
    GRASS=0,          # binary-level-tiles.md: tile 0 is grass on the 1.x and 2.x sheets
    GRASS2=2047,      # lab-304 note: labbush.nw is grass (2047) with bushes, stones, vases
    BUSH=2,           # arrays.dat 9 and 11: bush 2 3 / 18 19 (cut and lifted)
    STONE=34,         # arrays.dat 11: stone 34 35 / 50 51
    SIGN=512,         # arrays.dat 11: sign 512 513 / 528 529 (board-respawn.md)
    POT=684,          # arrays.dat 11: pot (vase) 684 685 / 700 701
    BLACK_STONE=990,  # arrays.dat 11: black stone 990 991 / 1006 1007
    TALL_GRASS=420,   # arrays.dat 9: tall grass 420 421 / 436 437 (cut)
    BUSH_LEFT=677,    # arrays.dat 10 and 12: what a cut or lifted bush leaves
    GRASS_LEFT=679,   # arrays.dat 10: what cut tall grass leaves
    SIGN_LEFT=1802,   # arrays.dat 12: what a lifted sign leaves
    POT_LEFT=1770,    # arrays.dat 12: what a lifted pot leaves
    STONE_LEFT=1834,  # arrays.dat 12: what a lifted stone or black stone leaves
    CHEST=1784,       # arrays.dat 13: closed chest 1784 1785 / 1800 1801
    CHEST_OPEN=829,   # arrays.dat 14: open chest 829 830 / 845 846
    DEEP_WATER=322,   # v5v6-lab-play.md: "walk into deep water, tile 322" (swim)
    SHALLOW=398,      # tile-types.md, new4.nw: shallow water, walked on
    NO_SWIM=3644,     # tile-types.md, new4.nw: deep water in no list, a wall
)
# 2x2 objects: id, id + 1, id + 16, id + 17
OBJECTS_2X2 = ("BUSH", "STONE", "SIGN", "POT", "BLACK_STONE", "TALL_GRASS", "BUSH_LEFT", "GRASS_LEFT",
               "SIGN_LEFT", "POT_LEFT", "STONE_LEFT", "CHEST", "CHEST_OPEN")

# Seihai's type numbers (client/src/assets/level.rs, documentation/tile-types.md)
OPEN, CHAIR, BED_TOP, BED_BOTTOM = 0, 3, 4, 5
WATER, LAVA, LAVA_FLOOR = 11, 12, 13
THROW_THROUGH, WALL = 20, 22

LISTS_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "graal_tile_lists.txt")
LIST_NAMES = ("passable", "liquid", "throw_through", "lava", "chair", "bed_top", "bed_bottom")


def read_lists(path=LISTS_FILE):
    """The seven arrays.dat lists, by name, from tools/graal_tile_lists.txt."""
    lists = {}
    for line in open(path).read().splitlines():
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        name, _, ids = line.partition(":")
        out = lists.setdefault(name.strip(), [])
        for part in ids.split():
            a, _, b = part.partition("-")
            out.extend(range(int(a), int(b or a) + 1))
    assert set(lists) == set(LIST_NAMES), sorted(lists)
    # the 2.22 file's list lengths (documentation/tile-types.md)
    assert [len(lists[n]) for n in LIST_NAMES] == [934, 116, 133, 76, 40, 4, 4], [len(lists[n]) for n in LIST_NAMES]
    return lists


def type_table():
    """One type per id, by the rules of TileTypes::parse_arrays."""
    li = read_lists()
    t = [WALL] * 4096
    for i in li["passable"]:
        t[i] = OPEN
    lava = set(li["lava"])
    for i in li["liquid"]:
        t[i] = LAVA if i in lava else WATER
    for i in li["throw_through"]:
        if t[i] == WALL:
            t[i] = THROW_THROUGH
    for name, ty in (("chair", CHAIR), ("bed_top", BED_TOP), ("bed_bottom", BED_BOTTOM)):
        for i in li[name]:
            if t[i] == OPEN:
                t[i] = ty
    for i in lava:
        if t[i] == OPEN:
            t[i] = LAVA_FLOOR
    return t


def import_arrays(path):
    """Write tools/graal_tile_lists.txt from a Graal arrays.dat (arrays 0 to 6,
    which are flat lists of big-endian u32 ids, each after a u32 count)."""
    import hashlib
    data = open(path, "rb").read()
    off, arrays = 0, []
    while len(arrays) < len(LIST_NAMES):
        n = struct.unpack(">I", data[off:off + 4])[0]
        assert n >> 16 == 0, "arrays 0 to 6 are flat"
        arrays.append(sorted(struct.unpack(">%dI" % n, data[off + 4:off + 4 + 4 * n])))
        off += 4 + 4 * n
    out = [
        "# Tile type lists of the standard Graal tile layout: arrays 0 to 6 of the",
        "# Graal 2.22 arrays.dat (11,500 bytes, sha256",
        "# " + hashlib.sha256(data).hexdigest() + ").",
        "# Numbers only; written by `gen_default_assets.py --import-arrays`. The 2.05",
        "# rev 2 file differs only in passable: 2.22 adds 2650-2653, 2666-2669, 2682-2685,",
        "# 2698-2701, 2714-2717. Meaning: documentation/tile-types.md.",
    ]
    for name, ids in zip(LIST_NAMES, arrays):
        runs, start = [], None
        for k, i in enumerate(ids):
            if start is None:
                start = i
            if k + 1 == len(ids) or ids[k + 1] != i + 1:
                runs.append(str(start) if start == i else f"{start}-{i}")
                start = None
        words, line = [], name + ":"
        for r in runs:
            if len(line) + len(r) + 1 > 100:
                words.append(line)
                line = name + ":"
            line += " " + r
        words.append(line)
        out.extend(words)
    open(LISTS_FILE, "w", newline="\n").write("\n".join(out) + "\n")


def tile_cell(tid):
    """The mapping Seihai's blit_tile uses (and the real sheets)."""
    return tid % 16 + (tid // 512) * 16, (tid // 16) % 32


def sheet_palette():
    p = Pal()
    for n, c in [
        ("grass", (72, 160, 56)), ("grass_lt", (100, 188, 72)), ("grass_dk", (50, 124, 44)),
        ("flower_w", (244, 244, 232)), ("flower_y", (248, 216, 64)), ("flower_p", (232, 112, 160)),
        ("dirt", (178, 132, 82)), ("dirt_dk", (140, 98, 58)), ("dirt_lt", (204, 164, 108)),
        ("bush", (36, 112, 52)), ("bush_dk", (20, 72, 36)), ("bush_lt", (84, 168, 80)), ("berry", (214, 52, 60)),
        ("rock", (148, 148, 156)), ("rock_dk", (88, 88, 100)), ("rock_lt", (200, 200, 208)),
        ("water", (48, 112, 208)), ("water_lt", (120, 176, 244)), ("water_dk", (32, 80, 168)),
        ("shallow", (96, 168, 224)), ("deep", (24, 52, 128)),
        ("lava", (224, 72, 24)), ("lava_lt", (252, 176, 48)), ("lava_dk", (150, 32, 20)), ("crust", (70, 40, 36)),
        ("wall", (128, 120, 112)), ("wall_dk", (82, 76, 72)), ("wall_lt", (170, 162, 150)),
        ("low", (156, 136, 104)), ("low_dk", (104, 88, 66)), ("low_lt", (196, 178, 140)),
        ("wood", (192, 142, 92)), ("wood_dk", (146, 100, 62)), ("wood_lt", (222, 176, 120)),
        ("cloth", (88, 112, 196)), ("cloth_lt", (150, 170, 232)), ("white", (248, 248, 248)),
        ("pot", (196, 112, 64)), ("pot_dk", (128, 64, 40)), ("pot_lt", (232, 160, 104)),
        ("outline", (28, 24, 32)),
        ("trunk", (112, 74, 44)), ("trunk_dk", (72, 46, 30)),
        ("chest", (150, 86, 40)), ("chest_dk", (92, 50, 28)), ("chest_lt", (192, 122, 64)),
        ("gold", (244, 204, 64)), ("gold_dk", (184, 140, 40)),
        ("shadow", (30, 80, 34)), ("ink", (60, 40, 30)),
    ]:
        p.add(n, c)
    return p


def draw_grass(d, pal, rng, tufts, flowers=False):
    d.rectangle([0, 0, 15, 15], fill=pal["grass"])
    for _ in range(tufts):
        x, y = rng.randrange(1, 15), rng.randrange(2, 15)
        d.point([(x, y), (x - 1, y - 1), (x + 1, y - 1)], fill=pal["grass_dk"])
        d.point([(x, y - 1)], fill=pal["grass_lt"])
    if flowers:
        for _ in range(3):
            x, y = rng.randrange(2, 14), rng.randrange(2, 14)
            c = rng.choice(["flower_w", "flower_y", "flower_p"])
            d.point([(x, y), (x + 1, y), (x, y + 1), (x - 1, y), (x, y - 1)], fill=pal["flower_w"] if c == "flower_w" else pal[c])
            d.point([(x, y)], fill=pal["flower_y"] if c == "flower_w" else pal["flower_w"])


def t_grass(pal, rng, tufts, flowers=False):
    im = Image.new("P", (16, 16), pal["grass"])
    draw_grass(ImageDraw.Draw(im), pal, rng, tufts, flowers)
    return im


def t_water(pal, variant, base="water"):
    im = Image.new("P", (16, 16), pal[base])
    d = ImageDraw.Draw(im)
    spots = [(2, 3), (9, 7), (4, 12)] if variant == 0 else [(7, 2), (2, 8), (10, 12)]
    for (x, y) in spots:
        d.line([(x, y), (x + 3, y)], fill=pal["water_lt"])
        d.point([(x + 1, y + 1), (x + 2, y + 1)], fill=pal["water_dk"])
    return im


def t_lava(pal, variant):
    im = Image.new("P", (16, 16), pal["lava"])
    d = ImageDraw.Draw(im)
    spots = [(3, 2), (9, 8), (2, 12)] if variant == 0 else [(8, 3), (1, 7), (10, 13)]
    for (x, y) in spots:
        d.ellipse([x, y, x + 3, y + 2], fill=pal["lava_lt"])
        d.point([(x + 4, y + 1)], fill=pal["lava_dk"])
    return im


def t_lava_floor(pal):
    im = Image.new("P", (16, 16), pal["crust"])
    d = ImageDraw.Draw(im)
    d.line([(0, 5), (5, 7), (9, 4), (15, 6)], fill=pal["lava"])
    d.line([(3, 15), (6, 10), (12, 12), (15, 10)], fill=pal["lava"])
    d.point([(5, 7), (12, 12)], fill=pal["lava_lt"])
    return im


def t_block(pal, base, variant):
    """A solid block: stone (wall) or a low, lighter one (thrown objects fly
    over it)."""
    im = Image.new("P", (16, 16), pal[base])
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 15, 15], outline=pal[base + "_dk"])
    d.line([(1, 1), (14, 1)], fill=pal[base + "_lt"])
    d.line([(1, 1), (1, 14)], fill=pal[base + "_lt"])
    if variant:
        d.line([(1, 8), (14, 8)], fill=pal[base + "_dk"])
        d.line([(8, 1), (8, 7)], fill=pal[base + "_dk"])
        d.line([(4, 9), (4, 14)], fill=pal[base + "_dk"])
    else:
        d.line([(1, 8), (14, 8)], fill=pal[base + "_dk"])
        d.line([(5, 1), (5, 7)], fill=pal[base + "_dk"])
        d.line([(11, 9), (11, 14)], fill=pal[base + "_dk"])
    return im


def t_chair(pal):
    im = t_wood(pal)
    d = ImageDraw.Draw(im)
    d.rectangle([3, 2, 12, 5], fill=pal["trunk"], outline=pal["trunk_dk"])
    d.rectangle([3, 6, 12, 12], fill=pal["wood_lt"], outline=pal["trunk_dk"])
    d.line([(4, 13), (4, 15)], fill=pal["trunk_dk"])
    d.line([(11, 13), (11, 15)], fill=pal["trunk_dk"])
    return im


def t_bed(pal, top):
    im = Image.new("P", (16, 16), pal["cloth"])
    d = ImageDraw.Draw(im)
    if top:
        d.rectangle([0, 0, 15, 15], outline=pal["trunk_dk"])
        d.rectangle([2, 2, 13, 7], fill=pal["white"], outline=pal["cloth_lt"])
    else:
        d.rectangle([0, 0, 15, 15], outline=pal["trunk_dk"])
        d.line([(1, 4), (14, 4)], fill=pal["cloth_lt"])
        d.rectangle([0, 13, 15, 15], fill=pal["trunk"])
    return im


def t_wood(pal):
    im = Image.new("P", (16, 16), pal["wood"])
    d = ImageDraw.Draw(im)
    for y in (3, 7, 11, 15):
        d.line([(0, y), (15, y)], fill=pal["wood_dk"])
    for y0, x in ((0, 5), (4, 12), (8, 3), (12, 9)):
        d.line([(x, y0), (x, y0 + 2)], fill=pal["wood_dk"])
    for y in (0, 4, 8, 12):
        d.line([(0, y), (15, y)], fill=pal["wood_lt"])
    return im


def grass32(pal, rng):
    im = Image.new("P", (32, 32), pal["grass"])
    d = ImageDraw.Draw(im)
    for _ in range(10):
        x, y = rng.randrange(1, 31), rng.randrange(2, 31)
        d.point([(x, y), (x - 1, y - 1), (x + 1, y - 1)], fill=pal["grass_dk"])
    return im, d


def bush_img(pal, rng):
    im, d = grass32(pal, rng)
    d.ellipse([2, 4, 29, 30], fill=pal["bush"], outline=pal["bush_dk"])
    d.ellipse([6, 7, 18, 17], fill=pal["bush_lt"])
    for (x, y) in ((20, 18), (11, 22), (22, 10), (15, 26)):
        d.point([(x, y), (x + 1, y)], fill=pal["berry"])
    d.arc([4, 16, 27, 30], 20, 160, fill=pal["bush_dk"])
    return im


def stone_img(pal, rng, dark):
    im, d = grass32(pal, rng)
    body, edge, hi = ("rock_dk", "outline", "rock") if dark else ("rock", "rock_dk", "rock_lt")
    d.ellipse([3, 24, 28, 31], fill=pal["shadow"])
    d.polygon([(3, 24), (5, 11), (13, 4), (23, 5), (28, 14), (27, 26), (15, 29)], fill=pal[body], outline=pal[edge])
    d.polygon([(8, 13), (13, 8), (20, 8), (14, 15)], fill=pal[hi])
    d.line([(16, 18), (22, 24)], fill=pal[edge])
    return im


def pot_img(pal, rng):
    im, d = grass32(pal, rng)
    d.ellipse([5, 25, 26, 31], fill=pal["shadow"])
    d.ellipse([5, 9, 26, 29], fill=pal["pot"], outline=pal["pot_dk"])
    d.rectangle([10, 4, 21, 10], fill=pal["pot"], outline=pal["pot_dk"])
    d.ellipse([10, 2, 21, 7], fill=pal["pot_dk"], outline=pal["outline"])
    d.arc([8, 12, 20, 24], 180, 260, fill=pal["pot_lt"])
    return im


def sign_img(pal, rng):
    im, d = grass32(pal, rng)
    d.rectangle([14, 20, 17, 31], fill=pal["trunk"], outline=pal["trunk_dk"])
    d.rectangle([3, 5, 28, 21], fill=pal["wood_lt"], outline=pal["trunk_dk"])
    for y in (9, 12, 15):
        d.line([(7, y), (24 - (y % 3) * 2, y)], fill=pal["ink"])
    return im


def tall_grass_img(pal, rng):
    im, d = grass32(pal, rng)
    for x in range(1, 31, 3):
        h = rng.randrange(14, 26)
        d.line([(x, 31), (x + rng.choice((-1, 0, 1)), 31 - h)], fill=pal["bush"])
        d.line([(x + 1, 31), (x + 1, 34 - h)], fill=pal["grass_dk"])
        d.point([(x, 31 - h)], fill=pal["grass_lt"])
    return im


def patch_img(pal, rng, fill, rim):
    """Bare ground where an object stood."""
    im, d = grass32(pal, rng)
    d.ellipse([4, 6, 27, 27], fill=pal[fill], outline=pal[rim])
    for _ in range(8):
        d.point([(rng.randrange(9, 23), rng.randrange(10, 23))], fill=pal[rim])
    return im


def chest_img(pal, opened):
    im = Image.new("P", (32, 32), pal["grass"])
    d = ImageDraw.Draw(im)
    d.ellipse([3, 24, 28, 31], fill=pal["shadow"])
    d.rectangle([3, 12, 28, 27], fill=pal["chest"], outline=pal["outline"])
    d.line([(4, 24), (27, 24)], fill=pal["chest_dk"])
    d.line([(4, 13), (27, 13)], fill=pal["chest_lt"])
    d.rectangle([14, 15, 17, 22], fill=pal["gold"], outline=pal["gold_dk"])
    d.point([(15, 18), (16, 18)], fill=pal["outline"])
    if not opened:
        d.rectangle([3, 6, 28, 13], fill=pal["chest_lt"], outline=pal["outline"])
        d.line([(4, 7), (27, 7)], fill=pal["white"])
        d.rectangle([14, 11, 17, 14], fill=pal["gold"], outline=pal["gold_dk"])
    else:
        d.polygon([(4, 12), (27, 12), (26, 4), (5, 4)], fill=pal["chest_dk"], outline=pal["outline"])
        d.rectangle([5, 13, 26, 17], fill=pal["outline"])
        d.point([(10, 15), (16, 14), (21, 15)], fill=pal["gold"])
        d.point([(16, 15)], fill=pal["white"])
    return im


def build_sheet(rng):
    pal = sheet_palette()
    types = type_table()
    # Art by type: every id gets the art of its type, so any Graal level
    # shows where its walls, water and floors are.
    grass = [t_grass(pal, rng, 5)]
    by_type = {
        OPEN: grass,
        CHAIR: [t_chair(pal)],
        BED_TOP: [t_bed(pal, True)],
        BED_BOTTOM: [t_bed(pal, False)],
        WATER: [t_water(pal, 0)],
        LAVA: [t_lava(pal, 0)],
        LAVA_FLOOR: [t_lava_floor(pal)],
        THROW_THROUGH: [t_block(pal, "low", 0)],
        WALL: [t_block(pal, "wall", 0)],
    }
    sheet = Image.new("P", (SHEET_W, SHEET_H), pal["grass"])

    def put(tid, im, w=1, h=1):
        # a w x h block of tiles whose ids are tid + dx + 16 * dy
        for dy in range(h):
            for dx in range(w):
                cx, cy = tile_cell(tid + dx + 16 * dy)
                sheet.paste(im.crop((dx * 16, dy * 16, dx * 16 + 16, dy * 16 + 16)), (cx * 16, cy * 16))

    for tid, ty in enumerate(types):
        put(tid, by_type[ty][0])
    put(G["GRASS"], grass[0])
    put(G["GRASS2"], grass[0])
    put(G["DEEP_WATER"], t_water(pal, 0, "deep"))
    put(G["SHALLOW"], t_water(pal, 1, "shallow"))
    put(G["NO_SWIM"], Image.new("P", (16, 16), pal["deep"]))
    put(G["BUSH"], bush_img(pal, rng), 2, 2)
    put(G["STONE"], stone_img(pal, rng, False), 2, 2)
    put(G["BLACK_STONE"], stone_img(pal, rng, True), 2, 2)
    put(G["POT"], pot_img(pal, rng), 2, 2)
    put(G["SIGN"], sign_img(pal, rng), 2, 2)
    put(G["TALL_GRASS"], tall_grass_img(pal, rng), 2, 2)
    put(G["BUSH_LEFT"], patch_img(pal, rng, "grass_dk", "bush_dk"), 2, 2)
    put(G["GRASS_LEFT"], patch_img(pal, rng, "grass_lt", "grass_dk"), 2, 2)
    put(G["SIGN_LEFT"], patch_img(pal, rng, "dirt", "dirt_dk"), 2, 2)
    put(G["POT_LEFT"], patch_img(pal, rng, "dirt_lt", "dirt"), 2, 2)
    put(G["STONE_LEFT"], patch_img(pal, rng, "dirt_dk", "trunk_dk"), 2, 2)
    put(G["CHEST"], chest_img(pal, False), 2, 2)
    put(G["CHEST_OPEN"], chest_img(pal, True), 2, 2)
    return sheet, pal


def tiledefs_text():
    """All 4096 types as a flat list, 16 per line: line n holds ids 16n to
    16n + 15 (one row of one 256 px block of the sheet)."""
    types = type_table()
    lines = [
        "# Tile types of the standard Graal tile layout, for Seihai's pics1.png and for",
        "# a Graal pics1.png alike. Flat list: 4096 values in id order, 16 per line.",
        "# From arrays 0 to 6 of the Graal 2.22 arrays.dat (tools/graal_tile_lists.txt):",
        "# 0 open, 3 chair, 4 bed top, 5 bed bottom, 11 water, 12 lava, 13 lava floor,",
        "# 20 wall that thrown objects fly over, 22 wall. documentation/tile-types.md.",
    ]
    for n in range(256):
        if n % 32 == 0:
            lines.append(f"# ids {n * 16} to {n * 16 + 511}: sheet columns {n // 32 * 16} to {n // 32 * 16 + 15}")
        lines.append(",".join(str(t) for t in types[n * 16:(n + 1) * 16]))
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Player sheets: body.png 128x720, head0.png 32x560, sword1.png, shield1.png
# ---------------------------------------------------------------------------

DIRS = 4  # 0 up, 1 left, 2 down, 3 right (Graal order)


def body_palette():
    p = Pal(transparent=True)
    for n, c in [
        ("outline", (32, 28, 40)), ("skin", (240, 200, 160)), ("skin_dk", (208, 160, 120)),
        ("tunic", (40, 150, 150)), ("tunic_dk", (24, 104, 110)), ("tunic_lt", (88, 196, 184)),
        ("belt", (130, 80, 40)), ("gold", (244, 204, 64)), ("pants", (74, 72, 130)),
        ("pants_dk", (52, 50, 98)), ("boot", (96, 60, 42)), ("boot_dk", (64, 38, 30)),
        ("white", (240, 240, 240)), ("hair", (120, 70, 36)), ("hair_dk", (84, 46, 26)),
    ]:
        p.add(n, c)
    return p


def body_cell(pal, direction, phase):
    """One 32x32 body frame. phase None is the idle pose, 0..7 the walk."""
    mirror = direction == 3
    d_eff = 1 if mirror else direction
    im = Image.new("P", (32, 32), 0)
    d = ImageDraw.Draw(im)
    if phase is None:
        s, sn = 0.0, 0.0
    else:
        a = 2 * math.pi * phase / 8
        s, sn = math.cos(a), math.sin(a)
    P = pal
    if d_eff in (0, 2):
        lift_a = int(round(max(0.0, sn) * 2))
        lift_b = int(round(max(0.0, -sn) * 2))
        arm = int(round(s))
        # legs
        for x0, lift, dark in ((11, lift_a, False), (17, lift_b, True)):
            bot = 30 - lift
            d.rectangle([x0, 19, x0 + 3, bot - 3], fill=P["pants_dk"] if dark else P["pants"], outline=P["outline"])
            d.rectangle([x0, bot - 3, x0 + 3, bot], fill=P["boot"], outline=P["outline"])
        # arms (behind the torso edge)
        for x0, sh in ((7, arm), (22, -arm)):
            d.rectangle([x0, 9 + sh, x0 + 2, 13 + sh], fill=P["tunic_dk"], outline=P["outline"])
            d.rectangle([x0, 14 + sh, x0 + 2, 17 + sh], fill=P["skin"], outline=P["outline"])
        # torso
        d.rectangle([10, 8, 21, 20], fill=P["tunic"], outline=P["outline"])
        d.rectangle([20, 9, 20, 19], fill=P["tunic_dk"])
        d.line([(11, 9), (14, 9)], fill=P["tunic_lt"])
        d.rectangle([11, 16, 20, 18], fill=P["belt"])
        if direction == 2:
            d.rectangle([15, 16, 16, 18], fill=P["gold"], outline=P["outline"])
            d.rectangle([14, 7, 17, 9], fill=P["skin"])
            d.point([(14, 8), (17, 8)], fill=P["outline"])
        else:
            d.rectangle([13, 7, 18, 9], fill=P["hair_dk"])
    else:
        off = int(round(3 * s))
        # far leg then near leg; forward is -x when facing left
        for sign, dark in ((1, True), (-1, False)):
            x0 = 14 + sign * off
            d.rectangle([x0, 19, x0 + 3, 26], fill=P["pants_dk"] if dark else P["pants"], outline=P["outline"])
            d.rectangle([x0 - 2, 27, x0 + 3, 30], fill=P["boot_dk"] if dark else P["boot"], outline=P["outline"])
        d.rectangle([12, 8, 19, 20], fill=P["tunic"], outline=P["outline"])
        d.rectangle([18, 9, 18, 19], fill=P["tunic_dk"])
        d.rectangle([12, 16, 19, 18], fill=P["belt"])
        d.rectangle([12, 16, 13, 18], fill=P["gold"])
        d.rectangle([13, 7, 17, 9], fill=P["skin"])
        # near arm swings opposite the near leg
        ax = 14 - int(round(2 * s))
        d.rectangle([ax, 9, ax + 3, 13], fill=P["tunic_dk"], outline=P["outline"])
        d.rectangle([ax, 14, ax + 3, 17], fill=P["skin"], outline=P["outline"])
    if mirror:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    return im


# Extra body poses (item 22), in the rows after the eight walk poses. Our own
# convention: the stock sheets are not read for this.
POSE_WINDUP, POSE_SWING, POSE_SIT, POSE_HURT = 9, 10, 11, 12


def body_pose_cell(pal, direction, kind):
    """One 32x32 body frame of the sword wind-up, sword swing, sit or hurt pose."""
    P = pal
    mirror = direction == 3
    d_eff = 1 if mirror else direction
    if kind in ("windup", "swing"):
        # the idle figure with its sword arm redrawn: raised for the wind-up,
        # stretched forward for the swing
        im = body_cell(pal, direction, None)
        if mirror:
            im = im.transpose(Image.FLIP_LEFT_RIGHT)
        d = ImageDraw.Draw(im)
        if d_eff in (0, 2):
            d.rectangle([21, 8, 25, 19], fill=0)  # clear the idle right arm
            d.rectangle([20, 8, 21, 14], fill=P["tunic"])
            if kind == "windup":
                d.rectangle([22, 5, 24, 9], fill=P["tunic_dk"], outline=P["outline"])
                d.rectangle([22, 2, 24, 4], fill=P["skin"], outline=P["outline"])
            elif d_eff == 0:  # forward is up: the arm reaches higher than in the wind-up
                d.rectangle([22, 3, 24, 8], fill=P["tunic_dk"], outline=P["outline"])
                d.rectangle([22, 0, 24, 2], fill=P["skin"], outline=P["outline"])
            else:
                d.rectangle([22, 13, 24, 17], fill=P["tunic_dk"], outline=P["outline"])
                d.rectangle([22, 18, 24, 22], fill=P["skin"], outline=P["outline"])
        else:
            d.rectangle([14, 8, 18, 18], fill=P["tunic"])
            d.rectangle([12, 8, 19, 20], fill=P["tunic"], outline=P["outline"])
            d.rectangle([18, 9, 18, 19], fill=P["tunic_dk"])
            d.rectangle([12, 16, 19, 18], fill=P["belt"])
            d.rectangle([12, 16, 13, 18], fill=P["gold"])
            if kind == "windup":
                d.rectangle([14, 5, 17, 9], fill=P["tunic_dk"], outline=P["outline"])
                d.rectangle([14, 2, 17, 4], fill=P["skin"], outline=P["outline"])
            else:
                d.rectangle([8, 11, 13, 14], fill=P["tunic_dk"], outline=P["outline"])
                d.rectangle([4, 11, 7, 14], fill=P["skin"], outline=P["outline"])
        if mirror:
            im = im.transpose(Image.FLIP_LEFT_RIGHT)
        return im
    if kind == "sit":
        base = body_cell(pal, d_eff, None)
        im = Image.new("P", (32, 32), 0)
        im.paste(base.crop((0, 0, 32, 19)), (0, 6))
        d = ImageDraw.Draw(im)
        if d_eff in (0, 2):
            for x0, dark in ((10, False), (17, True)):
                d.rectangle([x0, 24, x0 + 4, 27], fill=P["pants_dk"] if dark else P["pants"], outline=P["outline"])
                d.rectangle([x0, 27, x0 + 4, 30], fill=P["boot"], outline=P["outline"])
        else:
            d.rectangle([11, 25, 21, 28], fill=P["pants"], outline=P["outline"])
            d.rectangle([20, 25, 24, 30], fill=P["boot"], outline=P["outline"])
            if mirror:
                im = im.transpose(Image.FLIP_LEFT_RIGHT)
        return im
    assert kind == "hurt"
    base = body_cell(pal, direction, None)
    lean = {0: -10, 1: -12, 2: 10, 3: 12}[direction]  # away from the facing side
    im = base.rotate(lean, resample=Image.NEAREST, center=(16, 30), fillcolor=0)
    d = ImageDraw.Draw(im)
    for (x, y) in ((3, 6), (27, 8)):  # two small impact sparks
        d.line([(x - 2, y), (x + 2, y)], fill=P["white"])
        d.line([(x, y - 2), (x, y + 2)], fill=P["gold"])
    return im


# Carry poses: arms raised to hold an object over the head. Row 13 is the
# standing pose (carrystill.gani), rows 14 to 21 the eight carry-walk poses
# (carry.gani). The real client switches to its carry ganis while an object
# is held (2.05: lift, carrystill, carry); the art is our own.
POSE_CARRY_STILL = 13
POSE_CARRY_WALK = 14


def body_carry_cell(pal, direction, phase):
    """One 32x32 body frame with both arms raised (phase None: standing)."""
    P = pal
    mirror = direction == 3
    d_eff = 1 if mirror else direction
    im = body_cell(pal, d_eff if mirror else direction, phase)
    d = ImageDraw.Draw(im)
    if d_eff in (0, 2):
        # clear the hanging arms, then raise them beside the head
        for x0 in (6, 21):
            d.rectangle([x0, 8, x0 + 4, 19], fill=0)
        d.rectangle([10, 8, 21, 20], outline=P["outline"])
        for x0 in (5, 24):
            d.rectangle([x0, 3, x0 + 2, 9], fill=P["tunic_dk"], outline=P["outline"])
            d.rectangle([x0, 0, x0 + 2, 2], fill=P["skin"], outline=P["outline"])
    else:
        # side view: the near arm goes up in front of the face
        d.rectangle([11, 9, 19, 17], fill=P["tunic"])
        d.rectangle([12, 8, 19, 20], outline=P["outline"])
        d.rectangle([12, 16, 19, 18], fill=P["belt"])
        d.rectangle([12, 16, 13, 18], fill=P["gold"])
        d.rectangle([9, 3, 11, 9], fill=P["tunic_dk"], outline=P["outline"])
        d.rectangle([9, 0, 11, 2], fill=P["skin"], outline=P["outline"])
    if mirror:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    return im


# The lying body of dead.gani: 32x16 cells in the 16 spare rows at the foot
# of the sheet (y = 704), the same for each direction. Seen from above,
# shoulders to the left, feet to the right; the head goes left of it.
POSE_LYING = 22


def body_lying_cell(pal):
    P = pal
    im = Image.new("P", (32, 16), 0)
    d = ImageDraw.Draw(im)
    # legs and boots
    for y0, dark in ((3, True), (9, False)):
        d.rectangle([15, y0, 25, y0 + 3], fill=P["pants_dk"] if dark else P["pants"], outline=P["outline"])
        d.rectangle([25, y0 - 1, 29, y0 + 4], fill=P["boot"], outline=P["outline"])
    # arms along the sides: sleeves, then hands
    for y0 in (0, 12):
        d.rectangle([2, y0, 9, y0 + 3], fill=P["tunic_dk"], outline=P["outline"])
        d.rectangle([9, y0, 12, y0 + 3], fill=P["skin"], outline=P["outline"])
    # torso and belt
    d.rectangle([1, 3, 16, 12], fill=P["tunic"], outline=P["outline"])
    d.rectangle([13, 4, 15, 11], fill=P["belt"])
    d.rectangle([13, 7, 15, 8], fill=P["gold"])
    return im


def build_body():
    pal = body_palette()
    sheet = Image.new("P", (128, 720), 0)
    for dr in range(DIRS):
        sheet.paste(body_lying_cell(pal), (dr * 32, 32 * POSE_LYING))
        sheet.paste(body_cell(pal, dr, None), (dr * 32, 0))
        for k in range(8):
            sheet.paste(body_cell(pal, dr, k), (dr * 32, 32 * (1 + k)))
        for pose, kind in ((POSE_WINDUP, "windup"), (POSE_SWING, "swing"), (POSE_SIT, "sit"), (POSE_HURT, "hurt")):
            sheet.paste(body_pose_cell(pal, dr, kind), (dr * 32, 32 * pose))
        sheet.paste(body_carry_cell(pal, dr, None), (dr * 32, 32 * POSE_CARRY_STILL))
        for k in range(8):
            sheet.paste(body_carry_cell(pal, dr, k), (dr * 32, 32 * (POSE_CARRY_WALK + k)))
    return body_colour_layout(sheet, pal)


# The 2.05 client draws a body in the player's colours by replacing palette
# entries: 0 outline, 1 belt, 3 shoes, 4 sleeves, 5 skin, 6 coat (entry 2 is
# the clear colour in the stock bodies). See documentation/appearance.md.
# The shades of those parts are folded into them so a recoloured body has
# no stray tunic or skin tones left.
BODY_LAYOUT = ["outline", "belt", "clear", "boot", "tunic_dk", "skin", "tunic"]
BODY_FOLD = {"tunic_lt": "tunic", "skin_dk": "skin", "boot_dk": "boot"}
BODY_CLEAR = 2


def body_colour_layout(sheet, pal):
    order = BODY_LAYOUT + [n for n in pal.names if n not in BODY_LAYOUT and n not in BODY_FOLD]
    new = Pal()
    for n in order:
        new.add(n, pal.rgb[pal[n]])
    lut = [new[BODY_FOLD.get(n, n)] for n in pal.names]
    data = bytes(lut[b] for b in sheet.tobytes())
    return Image.frombytes("P", sheet.size, data), new


def head_palette():
    p = Pal(transparent=True)
    for n, c in [
        ("outline", (32, 28, 40)), ("skin", (240, 200, 160)), ("skin_dk", (208, 160, 120)),
        ("hair", (120, 70, 36)), ("hair_dk", (84, 46, 26)), ("hair_lt", (168, 112, 60)),
        ("white", (240, 240, 240)), ("eye", (30, 30, 60)), ("cheek", (232, 150, 140)),
    ]:
        p.add(n, c)
    return p


def head_cell(pal, direction):
    mirror = direction == 3
    de = 1 if mirror else direction
    im = Image.new("P", (32, 32), 0)
    d = ImageDraw.Draw(im)
    P = pal
    if de == 0:
        d.ellipse([5, 0, 26, 21], fill=P["hair"], outline=P["outline"])
        d.arc([8, 3, 23, 16], 200, 340, fill=P["hair_lt"])
        d.line([(15, 4), (15, 19)], fill=P["hair_dk"])
        d.point([(11, 18), (20, 18)], fill=P["hair_dk"])
    elif de == 2:
        d.ellipse([5, 0, 26, 21], fill=P["hair"], outline=P["outline"])
        d.ellipse([7, 7, 24, 20], fill=P["skin"])
        d.rectangle([4, 11, 5, 15], fill=P["skin_dk"], outline=P["outline"])
        d.rectangle([26, 11, 27, 15], fill=P["skin_dk"], outline=P["outline"])
        d.arc([6, 2, 25, 14], 190, 350, fill=P["hair_lt"])
        for x in (11, 19):
            d.rectangle([x, 11, x + 2, 14], fill=P["white"])
            d.rectangle([x + 1, 12, x + 2, 14], fill=P["eye"])
        d.line([(14, 17), (17, 17)], fill=P["skin_dk"])
        d.point([(9, 16), (22, 16)], fill=P["cheek"])
    else:
        d.ellipse([5, 0, 26, 21], fill=P["hair"], outline=P["outline"])
        d.ellipse([5, 6, 21, 20], fill=P["skin"])
        d.rectangle([17, 10, 21, 15], fill=P["skin_dk"], outline=P["outline"])
        d.arc([6, 2, 25, 14], 190, 350, fill=P["hair_lt"])
        d.rectangle([9, 11, 11, 14], fill=P["white"])
        d.rectangle([9, 12, 10, 14], fill=P["eye"])
        d.line([(7, 17), (10, 17)], fill=P["skin_dk"])
        d.point([(12, 16)], fill=P["cheek"])
    if mirror:
        im = im.transpose(Image.FLIP_LEFT_RIGHT)
    return im


def build_head():
    pal = head_palette()
    sheet = Image.new("P", (32, 560), 0)
    for dr in range(DIRS):
        sheet.paste(head_cell(pal, dr), (0, 32 * dr))
    return sheet, pal


def item_palette():
    p = Pal(transparent=True)
    for n, c in [
        ("outline", (32, 28, 40)), ("blade", (214, 220, 232)), ("blade_dk", (140, 148, 168)),
        ("hilt", (244, 204, 64)), ("grip", (130, 80, 40)), ("arc", (255, 255, 255)),
        ("shield", (60, 90, 190)), ("shield_dk", (36, 56, 130)), ("shield_lt", (130, 160, 236)),
        ("emblem", (244, 204, 64)),
    ]:
        p.add(n, c)
    return p


def build_sword():
    """128x96: three rows of four 32x32 cells (up, left, down, right).
    Row 0: sword held out; row 1: mid-swing with an arc; row 2: thrust."""
    pal = item_palette()
    sheet = Image.new("P", (128, 96), 0)
    for dr in range(DIRS):
        for row in range(3):
            cell = Image.new("P", (32, 32), 0)
            d = ImageDraw.Draw(cell)
            L = 12 if row == 0 else (16 if row == 1 else 20)
            # vertical sword pointing up from a hand at (16, 26), then rotated
            d.rectangle([15, 26 - L, 16, 25], fill=pal["blade"], outline=pal["blade_dk"])
            d.line([(15, 26 - L), (16, 26 - L)], fill=pal["arc"])
            d.rectangle([12, 25, 19, 26], fill=pal["hilt"], outline=pal["outline"])
            d.rectangle([15, 27, 16, 29], fill=pal["grip"], outline=pal["outline"])
            if row == 1:
                d.arc([4, 2, 28, 26], 200, 340, fill=pal["arc"])
            rot = {0: 0, 1: 90, 2: 180, 3: 270}[dr]
            cell = cell.rotate(rot, resample=Image.NEAREST)
            sheet.paste(cell, (dr * 32, row * 32))
    return sheet, pal


def build_shield():
    """38x20: four sprites at x = 0, 6, 14, 30 (up 6x18, left 8x20, down 16x20,
    right 8x20)."""
    pal = item_palette()
    sheet = Image.new("P", (38, 20), 0)
    d = ImageDraw.Draw(sheet)
    d.rectangle([0, 0, 5, 17], fill=pal["shield_dk"], outline=pal["outline"])
    d.line([(1, 1), (1, 16)], fill=pal["shield"])
    d.rectangle([6, 0, 13, 19], fill=pal["shield"], outline=pal["outline"])
    d.rectangle([7, 1, 8, 18], fill=pal["shield_lt"])
    d.rectangle([14, 0, 29, 19], fill=pal["shield"], outline=pal["outline"])
    d.rectangle([15, 1, 28, 2], fill=pal["shield_lt"])
    d.rectangle([20, 4, 23, 16], fill=pal["emblem"])
    d.rectangle([17, 8, 26, 11], fill=pal["emblem"])
    d.rectangle([30, 0, 37, 19], fill=pal["shield"], outline=pal["outline"])
    d.rectangle([35, 1, 36, 18], fill=pal["shield_dk"])
    return sheet, pal


# ---------------------------------------------------------------------------
# sprites.png (128 x 402): shadow at (0,0) 24x12, ripples at y=306 and y=322,
# lifted-object sprites at y=338/370 and the break-apart pieces (see the
# rect table in documentation/throwing.md)
# ---------------------------------------------------------------------------

WATER_RGB = (48, 112, 208)  # same as the tile sheet's 'water'


def build_sprites():
    p = Pal(transparent=True)
    p.add("shadow", (20, 48, 28))
    p.add("water", WATER_RGB)
    p.add("ring", (150, 196, 248))
    p.add("white", (248, 248, 248))
    for n, c in [("ink", (40, 32, 36)), ("leaf_dk", (30, 110, 50)), ("leaf", (60, 168, 70)), ("leaf_lt", (140, 220, 110)),
                 ("rock", (140, 140, 150)), ("rock_dk", (84, 84, 98)), ("rock_lt", (200, 200, 212)),
                 ("clay", (190, 112, 60)), ("clay_dk", (128, 68, 40)), ("clay_lt", (236, 168, 104))]:
        p.add(n, c)
    sheet = Image.new("P", (128, 402), 0)
    d = ImageDraw.Draw(sheet)
    # The renderer skips only fully transparent pixels, so the shadow is a
    # 50 % checker dither instead of a soft alpha.
    shadow = Image.new("P", (24, 12), 0)
    sd = ImageDraw.Draw(shadow)
    sd.ellipse([0, 0, 23, 11], fill=p["shadow"])
    for y in range(12):
        for x in range(24):
            if (x + y) % 2 and shadow.getpixel((x, y)) != 0:
                shadow.putpixel((x, y), 0)
    sheet.paste(shadow, (0, 0))
    for y0, grow in ((306, 0), (322, 2)):
        d.ellipse([2, y0 + 3, 29, y0 + 14], fill=p["water"])
        d.ellipse([4 - grow // 2, y0 + 5 - grow // 2, 27 + grow // 2, y0 + 12 + grow // 2], outline=p["ring"])
        d.point([(9, y0 + 8), (21, y0 + 9), (15, y0 + 6)], fill=p["white"])
    draw_throw_art(d, p)
    return sheet, p


def draw_throw_art(d, p):
    """Original art for the lifted objects (32x32, transparent around the
    object) and the pieces they break into. Positions are the sprite table's."""
    ink = p["ink"]

    def bush(x, y):
        d.ellipse([x + 2, y + 6, x + 29, y + 30], fill=p["leaf"], outline=ink)
        d.ellipse([x + 7, y + 2, x + 24, y + 16], fill=p["leaf"], outline=ink)
        d.ellipse([x + 5, y + 18, x + 16, y + 28], fill=p["leaf_dk"])
        d.ellipse([x + 18, y + 14, x + 27, y + 25], fill=p["leaf_dk"])
        d.line([(x + 11, y + 6), (x + 17, y + 5)], fill=p["leaf_lt"], width=2)
        d.point([(x + 8, y + 12), (x + 22, y + 11), (x + 14, y + 17)], fill=p["leaf_lt"])

    def pot(x, y):
        d.polygon([(x + 5, y + 8), (x + 26, y + 8), (x + 29, y + 26), (x + 22, y + 30), (x + 9, y + 30), (x + 2, y + 26)], fill=p["clay"], outline=ink)
        d.rectangle([x + 4, y + 4, x + 27, y + 9], fill=p["clay_lt"], outline=ink)
        d.line([(x + 7, y + 13), (x + 7, y + 24)], fill=p["clay_lt"], width=2)
        d.line([(x + 23, y + 14), (x + 25, y + 26)], fill=p["clay_dk"], width=2)

    def stone(x, y, fill, shade, light, big=False):
        r = 1 if big else 4
        d.polygon([(x + 6, y + 4 + r), (x + 14, y + 2 + r // 2), (x + 25, y + 5), (x + 30, y + 15), (x + 28, y + 27), (x + 16, y + 31), (x + 4, y + 27), (x + 2, y + 14)], fill=fill, outline=ink)
        d.polygon([(x + 16, y + 31), (x + 28, y + 27), (x + 30, y + 15), (x + 22, y + 20), (x + 18, y + 27)], fill=shade)
        d.line([(x + 7, y + 8 + r), (x + 15, y + 5 + r)], fill=light, width=2)
        d.point([(x + 9, y + 14), (x + 12, y + 17)], fill=light)
        if big:
            d.line([(x + 10, y + 22), (x + 17, y + 17), (x + 23, y + 21)], fill=ink)

    bush(0, 338)
    pot(32, 338)
    stone(64, 338, p["rock"], p["rock_dk"], p["rock_lt"])
    stone(96, 338, p["rock_dk"], p["ink"], p["rock"])
    stone(96, 370, p["rock"], p["rock_dk"], p["rock_lt"], big=True)

    # leaf bits (16x16) at (44, 0) and (60, 0)
    for x in (44, 60):
        d.polygon([(x + 3, 12), (x + 8, 2), (x + 13, 12), (x + 8, 14)], fill=p["leaf"], outline=p["leaf_dk"])
        d.line([(x + 8, 3), (x + 8, 13)], fill=p["leaf_lt"])
    # chunks: rock 16x16 at (24..40, 82..98), pot 16x14 at (56..72, 82..96)
    def chunk(x, y, w, h, fill, shade, light, k):
        pts = [(x + 2, y + h - 3), (x + 3 + k, y + 2), (x + w - 4, y + 3 + k), (x + w - 2, y + h - 4), (x + w // 2, y + h - 1)]
        d.polygon(pts, fill=fill, outline=ink)
        d.line([(x + 5, y + h - 4), (x + w - 5, y + h - 5)], fill=shade)
        d.point([(x + 5 + k, y + 5)], fill=light)
    for i, (x, y) in enumerate(((24, 82), (40, 82), (24, 98), (40, 98))):
        chunk(x, y, 16, 16, p["rock"], p["rock_dk"], p["rock_lt"], i % 3)
    for i, (x, y) in enumerate(((56, 82), (72, 82), (56, 96), (72, 96))):
        chunk(x, y, 16, 14, p["clay"], p["clay_dk"], p["clay_lt"], i % 3)
    # water splash: two 30x30 frames at (24, 114) and (54, 114)
    for x, grow in ((24, 0), (54, 3)):
        c = x + 15
        d.ellipse([c - 12 - grow, 114 + 15 - 6 - grow, c + 12 + grow, 114 + 15 + 6 + grow], outline=p["ring"])
        d.ellipse([c - 6, 114 + 15 - 3, c + 6, 114 + 15 + 3], outline=p["white"])
        d.line([(c - 3, 114 + 12 - grow), (c - 4, 114 + 5 - grow)], fill=p["white"], width=2)
        d.line([(c + 3, 114 + 12 - grow), (c + 5, 114 + 4 - grow)], fill=p["white"], width=2)
        d.point([(c, 114 + 3 - grow), (c - 8, 114 + 9), (c + 9, 114 + 8)], fill=p["ring"])


# ---------------------------------------------------------------------------
# HUD sheet (state.png, 224 x 145): our own layout, see documentation
# ---------------------------------------------------------------------------

DIGITS = [
    "01110 10001 10011 10101 11001 10001 01110",
    "00100 01100 00100 00100 00100 00100 01110",
    "01110 10001 00001 00110 01000 10000 11111",
    "11110 00001 00001 01110 00001 00001 11110",
    "00010 00110 01010 10010 11111 00010 00010",
    "11111 10000 11110 00001 00001 10001 01110",
    "00110 01000 10000 11110 10001 10001 01110",
    "11111 00001 00010 00100 01000 01000 01000",
    "01110 10001 10001 01110 10001 10001 01110",
    "01110 10001 10001 01111 00001 00010 01100",
]


def build_state():
    p = Pal(transparent=True)
    for n, c in [("outline", (32, 28, 40)), ("red", (220, 40, 56)), ("red_lt", (255, 130, 140)),
                 ("empty", (90, 40, 60)), ("green", (48, 190, 90)), ("white", (245, 245, 245)),
                 ("grey", (110, 110, 120)), ("brown", (140, 90, 50)), ("gold", (244, 204, 64))]:
        p.add(n, c)
    im = Image.new("P", (224, 145), 0)
    d = ImageDraw.Draw(im)

    def heart(x, fill, half=False):
        pts = [(x + 8, x * 0 + 14), (x + 1, 7), (x + 1, 3), (x + 3, 1), (x + 6, 1), (x + 8, 4), (x + 10, 1), (x + 13, 1), (x + 15, 3), (x + 15, 7)]
        d.polygon(pts, fill=p["empty"] if half else p[fill], outline=p["outline"])
        if half:
            d.polygon([(x + 8, 14), (x + 1, 7), (x + 1, 3), (x + 3, 1), (x + 6, 1), (x + 8, 4)], fill=p["red"])
            d.line([(x + 8, 4), (x + 8, 14)], fill=p["outline"])
        elif fill == "red":
            d.point([(x + 4, 3), (x + 3, 4)], fill=p["red_lt"])

    heart(0, "red")
    heart(16, "red", half=True)
    heart(32, "empty")
    d.polygon([(56, 2), (62, 8), (56, 14), (50, 8)], fill=p["green"], outline=p["outline"])      # rupee
    d.ellipse([68, 4, 78, 14], fill=p["grey"], outline=p["outline"])                              # bomb
    d.line([(76, 4), (79, 1)], fill=p["gold"])
    d.line([(84, 8), (95, 8)], fill=p["brown"])                                                   # arrow
    d.polygon([(95, 5), (98, 8), (95, 11)], fill=p["grey"], outline=p["outline"])
    d.line([(84, 8), (82, 5)], fill=p["white"])
    for i, rows in enumerate(DIGITS):
        for ry, row in enumerate(rows.split()):
            for rx, ch in enumerate(row):
                if ch == "1":
                    d.point([(i * 8 + 1 + rx, 18 + ry)], fill=p["white"])
    return im, p


# ---------------------------------------------------------------------------
# GANI files
# ---------------------------------------------------------------------------

# (shadow, body, head) offsets inside the 48x48 player box, per direction
# (up, left, down, right). The same numbers as render::draw_player_sprites.
OFFSETS = [((12, 34), (8, 16), (8, 2)), ((12, 36), (12, 16), (12, 2)),
           ((12, 34), (8, 16), (8, 2)), ((12, 36), (4, 16), (4, 2))]
SWIM_HEAD_DROP = 14  # draw_player_sprites lowers the head by this in water


# The hat (ATTR1) against the head, per head facing (up, left, down,
# right): a 48x48 cell of the 192x144 hat sheet at (48 * facing, 0), placed
# this far from the head and drawn after it. Measured on the stock 2.22
# player GANIs (idle, walk, sword, sit, swim, carry), where every row with
# a facing head has the same offsets (documentation/appearance.md).
HAT_FROM_HEAD = [(-8, -18), (-12, -18), (-8, -18), (-4, -18)]


def with_hats(text):
    """Add an ATTR1 sprite after every head placement of a GANI, so a
    player's hat (prop 37, `#P1`) draws on Seihai's own animations. No
    DEFAULTATTR1: Seihai ships no hat, so a player without one wears none."""
    out = []
    for line in text.split("\n"):
        if line.startswith("SPRITE 100 HEAD"):
            out += [f"SPRITE {40 + f} ATTR1 {48 * f} 0 48 48 hat {n}"
                    for f, n in enumerate(("up", "left", "down", "right"))]
        elif line and line[0].isdigit():
            parts = []
            for pl in line.split(", "):
                parts.append(pl)
                sid, x, y = (int(v) for v in pl.split())
                if 100 <= sid <= 103:
                    dx, dy = HAT_FROM_HEAD[sid - 100]
                    parts.append(f"{40 + sid - 100} {x + dx} {y + dy}")
            line = ", ".join(parts)
        out.append(line)
    return "\n".join(out)


# The shield (SHIELD, the 38x20 sheet: up 6x18 at x 0, left 8x20 at 6, down
# 16x20 at 14, right 8x20 at 30), placed against the head. Measured on the
# stock 2.22 player GANIs, step by step: (sprite, dx, dy) from the head per
# direction (up, left, down, right); None where no shield is drawn. Facing
# down the shield is drawn last (in front); otherwise right after the shadow.
# Sit, swim, hurt, carry and dead draw no shield, as the stock ones.
SHIELD_SPRITES = [f"SPRITE {10 + f} SHIELD {x} 0 {w} {h} shield {n}" for f, (x, w, h, n) in
                  enumerate(((0, 6, 18, "up"), (6, 8, 20, "left"), (14, 16, 20, "down"), (30, 8, 20, "right")))]
IDLE_SHIELD = [[(10, 28, 14), (11, -6, 14), (12, 0, 22), (13, 30, 14)]]
WALK_SHIELD = [[(10, 28, a), (11, b, c), (12, 0, d), (13, e, f)] for a, b, c, d, e, f in (
    (14, -6, 14, 24, 30, 14), (14, -6, 12, 22, 30, 12), (14, -6, 14, 24, 30, 14), (10, -6, 14, 24, 30, 14),
    (14, -6, 14, 24, 30, 14), (14, -4, 12, 22, 28, 12), (14, -6, 14, 24, 30, 14), (10, -6, 14, 24, 30, 14))]
# During the swing the shield is held to the side: the right-facing sprite
# when facing up, the left-facing one when facing down, none left or right.
SWORD_SHIELD = [[(13, 32, a), None, (11, -6, b), None] for a, b in ((18, 10), (16, 10), (18, 4), (22, 10), (16, 12))]
SHIELDS = {"idle": IDLE_SHIELD, "walk": WALK_SHIELD, "sword": SWORD_SHIELD}


def with_shields(text, table):
    """Add the SHIELD sprites and a shield placement to every direction row
    of a GANI, from `table[step][direction]` (see SHIELD_SPRITES)."""
    out, row = [], 0
    for line in text.split("\n"):
        if line.startswith("SPRITE 100 HEAD"):
            out += SHIELD_SPRITES
        elif line and line[0].isdigit():
            step, dr = divmod(row, 4)
            row += 1
            place = table[step][dr]
            if place:
                parts = line.split(", ")
                head = next(p for p in parts if 100 <= int(p.split()[0]) <= 103)
                hx, hy = (int(v) for v in head.split()[1:])
                sid, dx, dy = place
                pl = f"{sid} {hx + dx} {hy + dy}"
                if dr == 2 and sid == 12:
                    parts.append(pl)
                else:
                    parts.insert(1, pl)
                line = ", ".join(parts)
        out.append(line)
    return "\n".join(out)


def gani_header(title):
    return [
        f"// {title}",
        "// Original Seihai default animation; layout and numbers follow documentation/asset-reference.md.",
        "// A step is four direction rows (up, left, down, right) of 'sprite x y' placements.",
    ]


def gani_sprites(pal_poses, extra=()):
    out = ["SPRITE 0 SPRITES 0 0 24 12 shadow"]
    out += list(extra)
    out += [f"SPRITE {100 + dr} HEAD 0 {32 * dr} 32 32 head {n}" for dr, n in enumerate(("up", "left", "down", "right"))]
    for pose in pal_poses:
        for dr, n in enumerate(("up", "left", "down", "right")):
            out.append(f"SPRITE {200 + 4 * pose + dr} BODY {32 * dr} {32 * pose} 32 32 body pose {pose} {n}")
    return out


def idle_gani():
    L = gani_header("idle.gani: standing still (1 step)")
    L += ["DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites([0]) + ["", "ANI"]
    for dr in range(DIRS):
        (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
        L.append(f"0 {sx} {sy}, {200 + dr} {bx} {by}, {100 + dr} {hx} {hy}")
    L += ["ANIEND", ""]
    return "\n".join(L)


def walk_gani():
    L = gani_header("walk.gani: 8 steps; footsteps after steps 0 and 4")
    L += ["LOOP", "CONTINUOUS", "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites(range(1, 9)) + ["", "ANI"]
    for k in range(8):
        bob = int(round(1 + math.cos(2 * math.pi * 2 * k / 8)))  # 2 at the foot-falls, 0 between
        for dr in range(DIRS):
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            L.append(f"0 {sx} {sy}, {200 + 4 * (1 + k) + dr} {bx} {by - 1 + bob}, {100 + dr} {hx} {hy - 1 + bob}")
        if k in (0, 4):
            L.append("PLAYSOUND steps2.wav 1.5 2")
        L.append("")
    L += ["ANIEND", ""]
    return "\n".join(L)


def swim_gani():
    L = gani_header("swim.gani: 8 steps, head only, ripple rings; splash after step 0")
    L += ["LOOP", "CONTINUOUS", "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites([], ["SPRITE 20 SPRITES 0 306 32 16 ripple a", "SPRITE 21 SPRITES 0 322 32 16 ripple b"])
    L += ["", "ANI"]
    for k in range(8):
        bob = int(round(1 + math.cos(2 * math.pi * k / 4)))  # 0..2, gentle
        for dr in range(DIRS):
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            L.append(f"{100 + dr} {hx} {hy + SWIM_HEAD_DROP - 1 + bob}, {20 + k % 2} {hx} {hy + SWIM_HEAD_DROP + 12}")
        if k == 0:
            L.append("PLAYSOUND water.wav 1.5 2")
        L.append("WAIT 1")
        L.append("")
    L += ["ANIEND", ""]
    return "\n".join(L)


SWORD_STEPS = (  # (sword sheet row, body pose) of the five sword.gani steps
    (0, POSE_WINDUP), (1, POSE_SWING), (2, POSE_SWING), (2, POSE_SWING), (1, POSE_WINDUP))

# Where the sword hand is inside the 48x48 player box, per (pose, direction).
HAND = {
    POSE_WINDUP: [(31, 19), (27, 19), (31, 19), (20, 19)],
    POSE_SWING: [(31, 16), (17, 28), (31, 36), (30, 28)],
}


def sword_pivot(dr):
    """Where the grip of a (pre-rotation) sword cell lands inside the rotated
    32x32 cell: the cell is rotated about its centre, counter-clockwise, by
    90 degrees per direction step (up 0, left 90, down 180, right 270)."""
    px, py = 0, 10  # grip relative to the centre of the cell, before rotation
    for _ in range(dr):
        px, py = py, -px
    return 16 + px, 16 + py


def sword_gani():
    L = gani_header("sword.gani: 5 steps (wind-up, arc, thrust, thrust, recover); sword.wav after step 0")
    L += ["// Plays once, then goes back to idle (SETBACKTO).", "SETBACKTO idle",
          "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    sprites = gani_sprites([POSE_WINDUP, POSE_SWING])
    for row, rn in enumerate(("held", "arc", "thrust")):
        for dr, n in enumerate(("up", "left", "down", "right")):
            sprites.append(f"SPRITE {300 + 4 * row + dr} SWORD {32 * dr} {32 * row} 32 32 sword {rn} {n}")
    L += sprites + ["", "ANI"]
    for k, (row, pose) in enumerate(SWORD_STEPS):
        for dr in range(DIRS):
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            hand = HAND[pose][dr]
            pv = sword_pivot(dr)
            sw = (hand[0] - pv[0], hand[1] - pv[1])
            body = f"{200 + 4 * pose + dr} {bx} {by}"
            head = f"{100 + dr} {hx} {hy}"
            sword = f"{300 + 4 * row + dr} {sw[0]} {sw[1]}"
            order = [f"0 {sx} {sy}", sword, body, head] if dr == 0 else [f"0 {sx} {sy}", body, head, sword]
            L.append(", ".join(order))
        if k == 0:
            L.append("PLAYSOUND sword.wav 1.5 2")
        L.append("")
    L += ["ANIEND", ""]
    return "\n".join(L)


def sit_gani():
    L = gani_header("sit.gani: sitting (1 step)")
    L += ["DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites([POSE_SIT]) + ["", "ANI"]
    for dr in range(DIRS):
        (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
        L.append(f"0 {sx} {sy}, {200 + 4 * POSE_SIT + dr} {bx} {by}, {100 + dr} {hx} {hy + 6}")
    L += ["ANIEND", ""]
    return "\n".join(L)


def sleep_gani():
    # In bed the bed tile is the blanket: only the head is drawn, on the
    # pillow (the player's y is the bed top row). No shadow and no body.
    L = gani_header("sleep.gani: lying in bed, the head only (1 step)")
    L += ["DEFAULTHEAD head0.png", ""]
    L += [f"SPRITE {100 + dr} HEAD 0 {32 * dr} 32 32 head {n}" for dr, n in enumerate(("up", "left", "down", "right"))]
    L += ["", "ANI"]
    for dr in range(DIRS):
        _, _, (hx, hy) = OFFSETS[dr]
        L.append(f"{100 + dr} {hx} {hy}")
    L += ["ANIEND", ""]
    return "\n".join(L)


def carrystill_gani():
    L = gani_header("carrystill.gani: standing with an object held over the head (1 step)")
    L += ["DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites([POSE_CARRY_STILL]) + ["", "ANI"]
    for dr in range(DIRS):
        (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
        L.append(f"0 {sx} {sy}, {200 + 4 * POSE_CARRY_STILL + dr} {bx} {by}, {100 + dr} {hx} {hy}")
    L += ["ANIEND", ""]
    return "\n".join(L)


def carry_gani():
    L = gani_header("carry.gani: walking with an object held over the head; 8 steps, footsteps after steps 0 and 4")
    L += ["LOOP", "CONTINUOUS", "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites(range(POSE_CARRY_WALK, POSE_CARRY_WALK + 8)) + ["", "ANI"]
    for k in range(8):
        bob = int(round(1 + math.cos(2 * math.pi * 2 * k / 8)))
        for dr in range(DIRS):
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            L.append(f"0 {sx} {sy}, {200 + 4 * (POSE_CARRY_WALK + k) + dr} {bx} {by - 1 + bob}, {100 + dr} {hx} {hy - 1 + bob}")
        if k in (0, 4):
            L.append("PLAYSOUND steps2.wav 1.5 2")
        L.append("")
    L += ["ANIEND", ""]
    return "\n".join(L)


def dead_gani():
    """Our own falling sequence, one step per 50 ms tick of the death timer
    (pose.rs) and held on the last step: hurt for 6 ticks, kneeling for 8,
    then lying on the back. The same rows for each direction."""
    L = gani_header("dead.gani: falling down when the hearts run out; held on the last step")
    L += ["// The steps and their timing are Seihai's own; see documentation/health-death.md.",
          "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    lying = [f"SPRITE {200 + 4 * POSE_LYING + dr} BODY {32 * dr} {32 * POSE_LYING} 32 16 body lying" for dr in range(DIRS)]
    L += gani_sprites([POSE_HURT, POSE_SIT], lying) + ["", "ANI"]
    for _ in range(6):
        for dr in range(DIRS):
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            hdx, hdy = HURT_HEAD[dr]
            L.append(f"0 {sx} {sy}, {200 + 4 * POSE_HURT + dr} {bx} {by}, {100 + dr} {hx + hdx} {hy + hdy}")
        L.append("")
    for _ in range(8):
        for dr in range(DIRS):
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            L.append(f"0 {sx} {sy}, {200 + 4 * POSE_SIT + dr} {bx} {by + 2}, {100 + dr} {hx} {hy + 10}")
        L.append("")
    for dr in range(DIRS):
        L.append(f"0 14 34, {200 + 4 * POSE_LYING + dr} 18 28, 102 -6 17")
    L += ["", "ANIEND", ""]
    return "\n".join(L)


# Head shift of the hurt pose, away from the facing side (dx, dy per direction).
HURT_HEAD = [(1, 2), (2, 2), (-1, 2), (-2, 2)]


def hurt_gani():
    L = gani_header("hurt.gani: the pose Seihai shows for the first moments of hurt mode (1 step, 31 ticks)")
    L += ["// The name and the use are Seihai's convention, not a documented real-client one.",
          "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites([POSE_HURT]) + ["", "ANI"]
    for dr in range(DIRS):
        (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
        hdx, hdy = HURT_HEAD[dr]
        L.append(f"0 {sx} {sy}, {200 + 4 * POSE_HURT + dr} {bx} {by}, {100 + dr} {hx + hdx} {hy + hdy}")
    # Held 31 ticks, as long as the stock hurt.gani (Graal_analysis
    # player-mannerisms-live.md: the 2.05 client honours WAIT).
    L += ["WAIT 30", "ANIEND", ""]
    return "\n".join(L)


# ---------------------------------------------------------------------------
# Sounds: mono PCM WAV, synthesised
# ---------------------------------------------------------------------------


def write_wav(path, rate, bits, samples):
    """samples: floats in -1..1."""
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(bits // 8)
        w.setframerate(rate)
        if bits == 8:
            data = bytes(max(0, min(255, int(round(128 + s * 127)))) for s in samples)
        else:
            data = b"".join(struct.pack("<h", max(-32767, min(32767, int(round(s * 32767))))) for s in samples)
        w.writeframes(data)


def lowpass(xs, a):
    y, out = 0.0, []
    for x in xs:
        y += a * (x - y)
        out.append(y)
    return out


def step_sound(rng, rate, ms, f0, noise_amt):
    n = int(rate * ms / 1000)
    noise = lowpass([rng.uniform(-1, 1) for _ in range(n)], 0.35)
    out = []
    phase = 0.0
    for i in range(n):
        t = i / rate
        env = math.exp(-t * 38.0)
        f = f0 * (1.0 - 0.35 * min(1.0, t * 12))
        phase += 2 * math.pi * f / rate
        out.append((0.62 * math.sin(phase) + noise_amt * noise[i] * 2.0) * env * 0.85)
    return out


def splash_sound(rng, rate, ms):
    n = int(rate * ms / 1000)
    noise = [rng.uniform(-1, 1) for _ in range(n)]
    band = [a - b for a, b in zip(lowpass(noise, 0.55), lowpass(noise, 0.12))]
    out = []
    # bubbles: short rising sine chirps at fixed times
    bubbles = [(0.06, 380), (0.15, 520), (0.24, 450), (0.33, 640)]
    for i in range(n):
        t = i / rate
        env = (1 - math.exp(-t * 90)) * math.exp(-t * 7.0)
        s = band[i] * 3.2 * env
        for t0, f in bubbles:
            u = t - t0
            if 0 <= u < 0.07:
                s += 0.35 * math.sin(2 * math.pi * (f * u + 2200 * u * u)) * math.exp(-u * 45)
        out.append(max(-1, min(1, s)))
    return out


def sword_sound(rng, rate, ms):
    """A short whoosh: noise through a low-pass whose cut-off sweeps up, under a
    rising sine chirp, with a soft attack and a faster decay."""
    n = int(rate * ms / 1000)
    y, out = 0.0, []
    for i in range(n):
        t = i / rate
        u = i / n
        env = math.sin(math.pi * min(1.0, u * 1.15)) ** 2 * math.exp(-u * 1.5)
        a = 0.10 + 0.55 * u
        y += a * (rng.uniform(-1, 1) - y)
        chirp = math.sin(2 * math.pi * (700 * t + 2400 * t * t))
        out.append(max(-1, min(1, (y * 2.6 + 0.22 * chirp) * env)))
    return out


# ---------------------------------------------------------------------------
# Levels (GLEVNW01 text)
# ---------------------------------------------------------------------------

B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"


def code(tid):
    return B64[(tid >> 6) & 63] + B64[tid & 63]


def new_board(tid=None):
    return [G["GRASS"] if tid is None else tid] * (64 * 64)


def put_block(b, tid, x, y, w=2, h=2):
    for dy in range(h):
        for dx in range(w):
            b[(y + dy) * 64 + x + dx] = tid + dx + 16 * dy


def fill(b, tid, x0, y0, x1, y1):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            b[y * 64 + x] = tid


def board_text(b):
    lines = []
    for y in range(64):
        lines.append(f"BOARD 0 {y} 64 0 " + "".join(code(t) for t in b[y * 64:(y + 1) * 64]))
    return lines


# Both starter levels use only the documented ids in G, so they look the same
# on Seihai's sheet and on a Graal pics1.png.
def level_new1(rng):
    b = new_board()
    # border of bushes (2x2) on the north, west and east edges; the south edge is open
    for x in range(0, 64, 2):
        put_block(b, G["BUSH"], x, 0)
    for y in range(2, 64, 2):
        put_block(b, G["BUSH"], 0, y)
        put_block(b, G["BUSH"], 62, y)
    # pond: deep water (322, swum in) inside a ring of shallow water (398, walked on)
    cx, cy, rx, ry = 14, 42, 8, 6
    for y in range(cy - ry - 1, cy + ry + 2):
        for x in range(cx - rx - 1, cx + rx + 2):
            e = ((x - cx) / (rx + 1.5)) ** 2 + ((y - cy) / (ry + 1.5)) ** 2
            w = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2
            if w <= 1.0:
                b[y * 64 + x] = G["DEEP_WATER"]
            elif e <= 1.0:
                b[y * 64 + x] = G["SHALLOW"]
    # the way to the second level: a bare patch inside a ring of black stones
    for (x, y) in ((37, 16), (39, 16), (41, 16), (43, 16), (45, 16), (37, 18), (45, 18), (37, 20), (45, 20)):
        put_block(b, G["BLACK_STONE"], x, y)
    put_block(b, G["STONE_LEFT"], 41, 20)
    # scenery: bushes, stones, pots and tall grass
    for (x, y) in ((6, 28), (46, 30), (28, 12), (52, 44), (12, 56), (40, 52), (20, 20)):
        put_block(b, G["BUSH"], x, y)
    for (x, y) in ((30, 25), (48, 40), (24, 52), (56, 54), (4, 20)):
        put_block(b, G["STONE"], x, y)
    for (x, y) in ((50, 8), (52, 8), (54, 24)):
        put_block(b, G["POT"], x, y)
    for (x, y) in ((8, 10), (10, 10), (8, 12), (44, 44), (46, 46)):
        put_block(b, G["TALL_GRASS"], x, y)
    # sign (2x2; the record is its lower row) and the chest's place (the
    # client draws the chest from its record; servers leave 2047 under it)
    put_block(b, G["SIGN"], 26, 27)
    fill(b, G["GRASS2"], 34, 26, 35, 27)
    links = [
        "LINK new2.nw 41 20 2 2 31 47",
        "LINK new1.nw 2 63 60 1 playerx 3",   # south edge wraps to the north: a link to itself
    ]
    objs = [
        "CHEST 34 26 greenrupee -1",
        "SIGN 26 28",
        "Welcome to Seihai!",
        "Arrow keys walk. Walk up to the sign or the chest",
        "and press the action key to read it or open it.",
        "The pond to the west is deep enough to swim in.",
        "SIGNEND",
    ]
    return b, links, objs


def level_new2(rng):
    b = new_board()
    # a yard fenced with black stones (x 20..43, y 34..55); the gap in the
    # south fence (x 30..33) leads back
    for x in range(20, 44, 2):
        put_block(b, G["BLACK_STONE"], x, 34)
        if x not in (30, 32):
            put_block(b, G["BLACK_STONE"], x, 54)
    for y in range(36, 54, 2):
        put_block(b, G["BLACK_STONE"], 20, y)
        put_block(b, G["BLACK_STONE"], 42, y)
    # outside the yard: stones
    for y in range(0, 64, 2):
        for x in range(0, 64, 2):
            if not (18 <= x <= 44 and 32 <= y <= 56):
                put_block(b, G["STONE"], x, y)
    fill(b, G["GRASS2"], 24, 40, 25, 41)
    put_block(b, G["SIGN"], 36, 39)
    put_block(b, G["POT"], 22, 50)
    put_block(b, G["TALL_GRASS"], 38, 48)
    links = ["LINK new1.nw 30 55 4 1 40 23"]
    objs = [
        "CHEST 24 40 bluerupee -1",
        "SIGN 36 40",
        "This is the second starter level.",
        "The gap in the south fence leads back outside.",
        "SIGNEND",
    ]
    return b, links, objs


def level_text(b, links, objs):
    out = ["GLEVNW01"] + board_text(b) + links + objs
    return "\n".join(out) + "\n"


# ---------------------------------------------------------------------------
# Writing everything
# ---------------------------------------------------------------------------


def write_all(out):
    rng = random.Random(SEED)
    for sub in ("bodies", "heads", "swords", "shields", "gani", "sounds", "levels", os.path.join("levels", "tiledefs")):
        os.makedirs(os.path.join(out, sub), exist_ok=True)

    sheet, pal = build_sheet(rng)
    save_png(sheet, pal, os.path.join(out, "pics1.png"), 8, 256, False)
    open(os.path.join(out, "tiledefs.txt"), "w", newline="\n").write(tiledefs_text())

    im, p = build_body()
    save_png(im, p, os.path.join(out, "bodies", "body.png"), 4, 16, True, BODY_CLEAR)
    im, p = build_head()
    save_png(im, p, os.path.join(out, "heads", "head0.png"), 8, 16, True)
    im, p = build_sword()
    save_png(im, p, os.path.join(out, "swords", "sword1.png"), 4, 16, True)
    im, p = build_shield()
    save_png(im, p, os.path.join(out, "shields", "shield1.png"), 4, 16, True)
    for rel in (("bodies", "body.png"), ("heads", "head0.png"), ("swords", "sword1.png"), ("shields", "shield1.png")):
        save_gif_copy(os.path.join(out, *rel))
    im, p = build_sprites()
    save_png(im, p, os.path.join(out, "sprites.png"), 4, 16, True)
    im, p = build_state()
    save_png(im, p, os.path.join(out, "state.png"), 4, 16, True)

    for name, text in (("idle", idle_gani()), ("walk", walk_gani()), ("swim", swim_gani()),
                       ("sword", sword_gani()), ("sit", sit_gani()), ("sleep", sleep_gani()), ("hurt", hurt_gani()), ("dead", dead_gani()),
                       ("carrystill", carrystill_gani()), ("carry", carry_gani())):
        open(os.path.join(out, "gani", name + ".gani"), "w", newline="\n").write(with_shields(with_hats(text), SHIELDS[name]) if name in SHIELDS else with_hats(text))

    # the default sheet is the only tile sheet: one line, image,prefix,type,x,y
    open(os.path.join(out, "levels", "tiledefs", "tiledefsOffline.txt"), "w", newline="\n").write("pics1.png,,0,0,0\n")

    r = random.Random(SEED + 1)
    write_wav(os.path.join(out, "sounds", "steps.wav"), 22050, 8, step_sound(r, 22050, 150, 110, 0.35))
    write_wav(os.path.join(out, "sounds", "steps2.wav"), 22050, 8, step_sound(r, 22050, 150, 135, 0.45))
    write_wav(os.path.join(out, "sounds", "water.wav"), 8000, 16, splash_sound(r, 8000, 450))
    # a separate generator, so the older sounds keep their exact bytes
    write_wav(os.path.join(out, "sounds", "sword.wav"), 8000, 8, sword_sound(random.Random(SEED + 4), 8000, 260))

    for name, fn in (("new1", level_new1), ("new2", level_new2)):
        b, links, objs = fn(random.Random(SEED + 2 + (name == "new2")))
        open(os.path.join(out, "levels", name + ".nw"), "w", newline="\n").write(level_text(b, links, objs))


def listing(root):
    files = []
    for d, _, fs in os.walk(root):
        for f in fs:
            files.append(os.path.relpath(os.path.join(d, f), root))
    return sorted(files)


def same(a, b):
    if a.endswith(".png"):
        ia, ib = Image.open(a), Image.open(b)
        return ia.mode == ib.mode and ia.size == ib.size and ia.getpalette() == ib.getpalette() \
            and ia.info.get("transparency") == ib.info.get("transparency") and ia.tobytes() == ib.tobytes()
    return open(a, "rb").read() == open(b, "rb").read()


def check(out):
    with tempfile.TemporaryDirectory() as tmp:
        write_all(tmp)
        want, have = listing(tmp), listing(out)
        bad = [f"missing {f}" for f in want if f not in have] + [f"extra {f}" for f in have if f not in want]
        bad += [f"differs {f}" for f in want if f in have and not same(os.path.join(tmp, f), os.path.join(out, f))]
    for b in bad:
        print(b, file=sys.stderr)
    return not bad


def gani_frame_into(out, tile, name, step, dr):
    """Draw one step of a generated GANI onto an RGBA 48x48 tile (preview only);
    it also proves that the sprite ids a row names are defined."""
    sheets = {
        "BODY": Image.open(os.path.join(out, "bodies", "body.png")).convert("RGBA"),
        "HEAD": Image.open(os.path.join(out, "heads", "head0.png")).convert("RGBA"),
        "SPRITES": Image.open(os.path.join(out, "sprites.png")).convert("RGBA"),
        "SWORD": Image.open(os.path.join(out, "swords", "sword1.png")).convert("RGBA"),
        "SHIELD": Image.open(os.path.join(out, "shields", "shield1.png")).convert("RGBA"),
    }
    defs, rows, cur = {}, [], []
    in_ani = False
    for line in open(os.path.join(out, "gani", name + ".gani")).read().splitlines():
        line = line.strip()
        if line.startswith("SPRITE "):
            t = line.split()
            defs[int(t[1])] = (t[2], *map(int, t[3:7]))
        elif line == "ANI":
            in_ani = True
        elif line == "ANIEND":
            in_ani = False
        elif in_ani and "," in line:
            cur.append(line)
            if len(cur) == 4:
                rows.append(cur)
                cur = []
    for chunk in rows[step][dr].split(","):
        sid, x, y = map(int, chunk.split())
        sh, sx, sy, w, h = defs[sid]
        if sh not in sheets:  # ATTR1: Seihai ships no hat
            continue
        tile.alpha_composite(sheets[sh].crop((sx, sy, sx + w, sy + h)), (x, y))


def preview(out, path):
    """Contact sheet of our own art, for people to look at (not committed)."""
    S = 4
    tiles = list(G.items())
    sheet = Image.open(os.path.join(out, "pics1.png")).convert("RGB")
    body = Image.open(os.path.join(out, "bodies", "body.png")).convert("RGBA")
    head = Image.open(os.path.join(out, "heads", "head0.png")).convert("RGBA")
    spr = Image.open(os.path.join(out, "sprites.png")).convert("RGBA")
    W = 1100
    canvas = Image.new("RGB", (W, 1900), (40, 44, 52))

    def cell(tid):
        cx, cy = tile_cell(tid)
        return sheet.crop((cx * 16, cy * 16, cx * 16 + 16, cy * 16 + 16))

    d = ImageDraw.Draw(canvas)
    d.text((10, 6), "Seihai default assets (original, generated by tools/gen_default_assets.py)", fill=(255, 255, 255))
    d.text((10, 24), "tiles (x4), in id order: " + " ".join(f"{k.lower()}={v}" for k, v in sorted(tiles, key=lambda kv: kv[1])), fill=(200, 200, 200))
    x, ty, rowh = 10, 46, 0
    for k, v in sorted(tiles, key=lambda kv: kv[1]):
        w, h = (2, 2) if k in OBJECTS_2X2 else (1, 1)
        blk = Image.new("RGB", (16 * w, 16 * h))
        for dy in range(h):
            for dx in range(w):
                blk.paste(cell(v + dx + 16 * dy), (dx * 16, dy * 16))
        blk = blk.resize((blk.width * S, blk.height * S), Image.NEAREST)
        if x + blk.width > W - 10:
            x, ty, rowh = 10, ty + rowh + 8, 0
        canvas.paste(blk, (x, ty))
        rowh = max(rowh, blk.height)
        x += blk.width + 8
    y = ty + rowh + 30
    d.text((10, y - 14), "player (x4): idle + 8 walk poses, 4 directions (up, left, down, right); heads on top", fill=(200, 200, 200))
    for dr in range(4):
        for pose in range(9):
            tile = Image.new("RGBA", (48, 48), (72, 160, 56, 255))
            (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
            tile.alpha_composite(spr.crop((0, 0, 24, 12)), (sx, sy))
            tile.alpha_composite(body.crop((32 * dr, 32 * pose, 32 * dr + 32, 32 * pose + 32)), (bx, by))
            tile.alpha_composite(head.crop((0, 32 * dr, 32, 32 * dr + 32)), (hx, hy))
            t2 = tile.resize((48 * 2, 48 * 2), Image.NEAREST).convert("RGB")
            canvas.paste(t2, (10 + pose * 100, y + dr * 100))
    y += 4 * 100 + 10
    d.text((10, y - 14), "swimming (head + ripples over water), 2 ripple frames, 4 directions", fill=(200, 200, 200))
    for dr in range(4):
        for f in range(2):
            tile = Image.new("RGBA", (48, 48), WATER_RGB + (255,))
            (_, _), (_, _), (hx, hy) = OFFSETS[dr]
            tile.alpha_composite(head.crop((0, 32 * dr, 32, 32 * dr + 32)), (hx, hy + SWIM_HEAD_DROP))
            tile.alpha_composite(spr.crop((0, 306 + 16 * f, 32, 322 + 16 * f)), (hx, hy + SWIM_HEAD_DROP + 12))
            canvas.paste(tile.resize((96, 96), Image.NEAREST).convert("RGB"), (10 + (dr * 2 + f) * 100, y))
    y += 110
    d.text((10, y - 14), "item 22 poses from the GANIs (x2): sword.gani 5 steps, then sit.gani and hurt.gani, 4 directions", fill=(200, 200, 200))
    for dr in range(4):
        for i, (name, k) in enumerate([("sword", 0), ("sword", 1), ("sword", 2), ("sword", 3), ("sword", 4), ("sit", 0), ("hurt", 0)]):
            tile = Image.new("RGBA", (48, 48), (72, 160, 56, 255))
            gani_frame_into(out, tile, name, k, dr)
            canvas.paste(tile.resize((96, 96), Image.NEAREST).convert("RGB"), (10 + i * 100, y + dr * 100))
    y += 4 * 100 + 10
    # levels at 1:1 (64x64 tiles = 1024 px) scaled to 0.5
    for i, name in enumerate(("new1", "new2")):
        txt = open(os.path.join(out, "levels", name + ".nw")).read().splitlines()
        lv = Image.new("RGB", (1024, 1024))
        for line in txt:
            if line.startswith("BOARD "):
                parts = line.split()
                ry = int(parts[2])
                data = parts[5]
                for ix in range(64):
                    tid = (B64.index(data[ix * 2]) << 6) | B64.index(data[ix * 2 + 1])
                    lv.paste(cell(tid), (ix * 16, ry * 16))
        lv = lv.crop((0, 0, 1024, 1024)).resize((512, 512), Image.NEAREST)
        canvas.paste(lv, (10 + i * 522, y))
        d.text((10 + i * 522, y - 14), f"levels/{name}.nw (0.5x)", fill=(200, 200, 200))
    canvas.crop((0, 0, W, y + 520)).save(path)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ap.add_argument("--out", default=os.path.join(root, "assets", "default"))
    ap.add_argument("--check", action="store_true", help="regenerate to a temp dir and compare with --out")
    ap.add_argument("--preview", metavar="PNG", help="write a contact sheet of the generated art")
    ap.add_argument("--import-arrays", metavar="DAT", help="rewrite tools/graal_tile_lists.txt from a Graal arrays.dat, then stop")
    a = ap.parse_args()
    if a.import_arrays:
        import_arrays(a.import_arrays)
        print(f"wrote {LISTS_FILE}", file=sys.stderr)
        return 0
    if a.check:
        ok = check(a.out)
        print("assets/default is up to date" if ok else "assets/default differs from the generator", file=sys.stderr)
        return 0 if ok else 1
    write_all(a.out)
    if a.preview:
        preview(a.out, a.preview)
    total = sum(os.path.getsize(os.path.join(a.out, f)) for f in listing(a.out))
    print(f"wrote {len(listing(a.out))} files, {total} bytes, to {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
