#!/usr/bin/env python3
"""Generate Seihai's default asset set (assets/default/).

Every file is original: drawn here, pixel by pixel, from primitives and hand
specified palettes, or synthesised from noise and sine tones. Nothing is read
from any Graal client, playerworld or other asset; the only numbers taken from
the measurements in documentation/asset-reference.md are sizes, grid layouts and
frame counts.

Usage (from the repository root, Python 3.9+, Pillow):

    python3 tools/gen_default_assets.py                 # write assets/default/
    python3 tools/gen_default_assets.py --check         # regenerate in a temp
                                                        # dir, compare (exit 1)
    python3 tools/gen_default_assets.py --preview P.png # contact sheet of OUR art

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


def save_png(im, pal, path, bits, size, transparent):
    assert len(pal.names) <= size, (path, len(pal.names), size)
    im.putpalette(pal.flat(size))
    kw = dict(optimize=True, bits=bits, compress_level=9)
    if transparent:
        kw["transparency"] = 0
    im.save(path, "PNG", **kw)


# ---------------------------------------------------------------------------
# Tile sheet (pics1.png): 2048 x 512, 128 x 32 cells of 16 x 16
# ---------------------------------------------------------------------------

SHEET_W, SHEET_H = 2048, 512

# Tile ids. All below 512, so they sit in block 0 (columns 0..15) and exist on
# every sheet at least 256 px wide. Cell: column = id % 16 + (id / 512) * 16,
# row = (id / 16) % 32.
T = dict(
    GRASS_A=0, GRASS_B=1, GRASS_C=2, PATH=3,
    BUSH=16, ROCK=17,
    WATER_A=32, WATER_B=33, SAND_A=34, SAND_B=35,
    BRICK=48, BRICK_CAP=49, ROOF=50, DOORWAY=51, WOOD_FLOOR=52, VOID=54,
    CHEST_G=64, CHEST_G_OPEN=66, CHEST_W=68, CHEST_W_OPEN=70,  # 2x2: id, id+1, id+16, id+17
    SIGN_G=96, SIGN_W=98,                                       # 2x1: id, id+1
    TREE=112,                                                   # 2x2
)

WALL = 22
WATER = 11
# id -> (collision type, name); type >= 20 blocks, 11 is water, else open
def type_table():
    t = {}

    def put(i, ty, name):
        t[i] = (ty, name)

    for i, n in ((T["GRASS_A"], "grass"), (T["GRASS_B"], "grass2"), (T["GRASS_C"], "flowers"),
                 (T["PATH"], "path"), (T["SAND_A"], "sand"), (T["SAND_B"], "sand2"),
                 (T["DOORWAY"], "doorway"), (T["WOOD_FLOOR"], "woodfloor")):
        put(i, 0, n)
    put(T["BUSH"], WALL, "bush")
    put(T["ROCK"], WALL, "rock")
    put(T["WATER_A"], WATER, "water")
    put(T["WATER_B"], WATER, "water2")
    put(T["BRICK"], WALL, "wall")
    put(T["BRICK_CAP"], WALL, "wallcap")
    put(T["ROOF"], WALL, "roof")
    put(T["VOID"], WALL, "void")
    for base, nm in ((T["CHEST_G"], "chest"), (T["CHEST_G_OPEN"], "chestopen"),
                     (T["CHEST_W"], "chestw"), (T["CHEST_W_OPEN"], "chestwopen"),
                     (T["TREE"], "tree")):
        for k, off in enumerate((0, 1, 16, 17)):
            put(base + off, WALL, f"{nm}{k}")
    for base, nm in ((T["SIGN_G"], "sign"), (T["SIGN_W"], "signw")):
        put(base, WALL, nm + "0")
        put(base + 1, WALL, nm + "1")
    return t


def tile_cell(tid):
    """The mapping Seihai's blit_tile uses."""
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
        ("sand", (232, 208, 148)), ("sand_dk", (200, 172, 112)), ("sand_lt", (248, 232, 184)),
        ("brick", (156, 124, 104)), ("brick_dk", (100, 78, 70)), ("brick_lt", (190, 160, 132)),
        ("roof", (176, 64, 52)), ("roof_dk", (120, 36, 36)), ("roof_lt", (216, 104, 76)),
        ("door", (38, 30, 36)), ("door_lt", (74, 56, 52)),
        ("wood", (192, 142, 92)), ("wood_dk", (146, 100, 62)), ("wood_lt", (222, 176, 120)),
        ("void", (16, 12, 20)),
        ("outline", (28, 24, 32)),
        ("trunk", (112, 74, 44)), ("trunk_dk", (72, 46, 30)),
        ("chest", (150, 86, 40)), ("chest_dk", (92, 50, 28)), ("chest_lt", (192, 122, 64)),
        ("gold", (244, 204, 64)), ("gold_dk", (184, 140, 40)), ("white", (248, 248, 248)),
        ("shadow", (30, 80, 34)), ("ink", (60, 40, 30)),
        ("miss_a", (160, 32, 160)), ("miss_b", (40, 16, 40)),
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


def t_path(pal, rng):
    im = Image.new("P", (16, 16), pal["dirt"])
    d = ImageDraw.Draw(im)
    for _ in range(14):
        x, y = rng.randrange(0, 15), rng.randrange(0, 15)
        d.point([(x, y)], fill=pal["dirt_dk"])
    for _ in range(6):
        x, y = rng.randrange(0, 14), rng.randrange(0, 14)
        d.rectangle([x, y, x + 1, y + 1], fill=pal["dirt_lt"])
    return im


def t_bush(pal, rng):
    im = Image.new("P", (16, 16), pal["grass"])
    d = ImageDraw.Draw(im)
    d.ellipse([1, 2, 14, 14], fill=pal["bush"], outline=pal["bush_dk"])
    d.ellipse([3, 3, 9, 8], fill=pal["bush_lt"])
    d.point([(5, 4), (6, 5)], fill=pal["grass_lt"])
    for (x, y) in ((10, 9), (6, 11), (11, 6)):
        d.point([(x, y)], fill=pal["berry"])
    d.arc([2, 8, 13, 15], 20, 160, fill=pal["bush_dk"])
    return im


def t_rock(pal, rng):
    im = Image.new("P", (16, 16), pal["grass"])
    d = ImageDraw.Draw(im)
    d.polygon([(2, 12), (3, 6), (7, 2), (12, 3), (14, 8), (13, 13), (7, 14)], fill=pal["rock"], outline=pal["rock_dk"])
    d.polygon([(4, 7), (7, 4), (10, 4), (7, 8)], fill=pal["rock_lt"])
    d.line([(8, 9), (11, 12)], fill=pal["rock_dk"])
    d.line([(5, 11), (7, 12)], fill=pal["rock_dk"])
    return im


def t_water(pal, variant):
    im = Image.new("P", (16, 16), pal["water"])
    d = ImageDraw.Draw(im)
    spots = [(2, 3), (9, 7), (4, 12)] if variant == 0 else [(7, 2), (2, 8), (10, 12)]
    for (x, y) in spots:
        d.line([(x, y), (x + 3, y)], fill=pal["water_lt"])
        d.point([(x + 1, y + 1), (x + 2, y + 1)], fill=pal["water_dk"])
    return im


def t_sand(pal, rng, variant):
    im = Image.new("P", (16, 16), pal["sand"])
    d = ImageDraw.Draw(im)
    for _ in range(10):
        d.point([(rng.randrange(0, 16), rng.randrange(0, 16))], fill=pal["sand_dk"])
    for _ in range(4):
        d.point([(rng.randrange(0, 16), rng.randrange(0, 16))], fill=pal["sand_lt"])
    if variant:
        d.ellipse([5, 6, 9, 9], fill=pal["sand_lt"], outline=pal["sand_dk"])
    return im


def t_brick(pal, cap):
    im = Image.new("P", (16, 16), pal["brick"])
    d = ImageDraw.Draw(im)
    for y in (3, 7, 11, 15):
        d.line([(0, y), (15, y)], fill=pal["brick_dk"])
    for y0, xs in ((0, (7, 15)), (4, (3, 11)), (8, (7, 15)), (12, (3, 11))):
        for x in xs:
            d.line([(x, y0), (x, y0 + 2)], fill=pal["brick_dk"])
    for y in (0, 4, 8, 12):
        d.line([(0, y), (15, y)], fill=pal["brick_lt"])
    if cap:
        d.rectangle([0, 0, 15, 5], fill=pal["brick_lt"])
        d.line([(0, 5), (15, 5)], fill=pal["brick_dk"])
        d.line([(0, 0), (15, 0)], fill=pal["white"])
    return im


def t_roof(pal):
    im = Image.new("P", (16, 16), pal["roof"])
    d = ImageDraw.Draw(im)
    for row in range(4):
        y = row * 4
        off = 0 if row % 2 == 0 else 4
        d.line([(0, y + 3), (15, y + 3)], fill=pal["roof_dk"])
        for x in range(off, 16, 8):
            d.line([(x, y), (x, y + 2)], fill=pal["roof_dk"])
        d.line([(0, y), (15, y)], fill=pal["roof_lt"])
    return im


def t_doorway(pal):
    im = Image.new("P", (16, 16), pal["door"])
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 1, 15], fill=pal["brick"])
    d.rectangle([14, 0, 15, 15], fill=pal["brick"])
    d.line([(1, 0), (1, 15)], fill=pal["brick_dk"])
    d.line([(14, 0), (14, 15)], fill=pal["brick_dk"])
    d.rectangle([2, 0, 13, 1], fill=pal["brick_lt"])
    d.rectangle([4, 10, 11, 15], fill=pal["door_lt"])
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


def chest_img(pal, bg, opened):
    im = Image.new("P", (32, 32), pal[bg])
    d = ImageDraw.Draw(im)
    d.ellipse([3, 24, 28, 31], fill=pal["shadow"] if bg == "grass" else pal["wood_dk"])
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


def sign_img(pal, bg):
    im = Image.new("P", (32, 16), pal[bg])
    d = ImageDraw.Draw(im)
    d.rectangle([14, 9, 17, 15], fill=pal["trunk"], outline=pal["trunk_dk"])
    d.rectangle([3, 1, 28, 10], fill=pal["wood_lt"], outline=pal["trunk_dk"])
    for y in (3, 5, 7):
        d.line([(6, y), (25 - (y % 3) * 2, y)], fill=pal["ink"])
    return im


def tree_img(pal, rng):
    im = Image.new("P", (32, 32), pal["grass"])
    d = ImageDraw.Draw(im)
    d.rectangle([13, 20, 18, 31], fill=pal["trunk"], outline=pal["trunk_dk"])
    d.ellipse([1, 1, 30, 24], fill=pal["bush"], outline=pal["bush_dk"])
    d.ellipse([5, 4, 17, 13], fill=pal["bush_lt"])
    for _ in range(10):
        x, y = rng.randrange(6, 26), rng.randrange(6, 20)
        d.point([(x, y)], fill=pal["bush_dk"])
    return im


def build_sheet(rng):
    pal = sheet_palette()
    sheet = Image.new("P", (SHEET_W, SHEET_H), pal["miss_b"])
    d = ImageDraw.Draw(sheet)
    # Unassigned cells: 8 px purple/black checker, so a wrong id is obvious.
    for cy in range(SHEET_H // 8):
        for cx in range(SHEET_W // 8):
            if (cx + cy) % 2 == 0:
                d.rectangle([cx * 8, cy * 8, cx * 8 + 7, cy * 8 + 7], fill=pal["miss_a"])

    def put(tid, im, w=1, h=1):
        # a w x h block of tiles whose ids are tid + dx + 16 * dy
        for dy in range(h):
            for dx in range(w):
                cx, cy = tile_cell(tid + dx + 16 * dy)
                sheet.paste(im.crop((dx * 16, dy * 16, dx * 16 + 16, dy * 16 + 16)), (cx * 16, cy * 16))

    for tid, tufts, fl in ((T["GRASS_A"], 5, False), (T["GRASS_B"], 9, False), (T["GRASS_C"], 4, True)):
        im = Image.new("P", (16, 16), pal["grass"])
        draw_grass(ImageDraw.Draw(im), pal, rng, tufts, fl)
        put(tid, im)
    put(T["PATH"], t_path(pal, rng))
    put(T["BUSH"], t_bush(pal, rng))
    put(T["ROCK"], t_rock(pal, rng))
    put(T["WATER_A"], t_water(pal, 0))
    put(T["WATER_B"], t_water(pal, 1))
    put(T["SAND_A"], t_sand(pal, rng, 0))
    put(T["SAND_B"], t_sand(pal, rng, 1))
    put(T["BRICK"], t_brick(pal, False))
    put(T["BRICK_CAP"], t_brick(pal, True))
    put(T["ROOF"], t_roof(pal))
    put(T["DOORWAY"], t_doorway(pal))
    put(T["WOOD_FLOOR"], t_wood(pal))
    put(T["VOID"], Image.new("P", (16, 16), pal["void"]))
    put(T["CHEST_G"], chest_img(pal, "grass", False), 2, 2)
    put(T["CHEST_G_OPEN"], chest_img(pal, "grass", True), 2, 2)
    put(T["CHEST_W"], chest_img(pal, "wood", False), 2, 2)
    put(T["CHEST_W_OPEN"], chest_img(pal, "wood", True), 2, 2)
    put(T["SIGN_G"], sign_img(pal, "grass"), 2, 1)
    put(T["SIGN_W"], sign_img(pal, "wood"), 2, 1)
    put(T["TREE"], tree_img(pal, rng), 2, 2)
    return sheet, pal


def tiledefs_text():
    lines = [
        "# Seihai default tile types (original). index,x,y,type,name",
        "# x,y: pixel position of the tile's cell on pics1.png (2048x512, 16 px cells).",
        "# type: 11 water, 20 and above blocks, anything else is open floor.",
        "# Ids not listed are type 0 (open) and draw the purple/black 'unassigned' checker.",
    ]
    for tid, (ty, name) in sorted(type_table().items()):
        cx, cy = tile_cell(tid)
        lines.append(f"{tid},{cx * 16},{cy * 16},{ty},{name}")
    # TileTypes::parse needs a value for every id up to 4095 (it counts the
    # highest index), so the last id is listed explicitly.
    cx, cy = tile_cell(4095)
    lines.append(f"4095,{cx * 16},{cy * 16},0,unassigned")
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


def build_body():
    pal = body_palette()
    sheet = Image.new("P", (128, 720), 0)
    for dr in range(DIRS):
        sheet.paste(body_cell(pal, dr, None), (dr * 32, 0))
        for k in range(8):
            sheet.paste(body_cell(pal, dr, k), (dr * 32, 32 * (1 + k)))
        for pose, kind in ((POSE_WINDUP, "windup"), (POSE_SWING, "swing"), (POSE_SIT, "sit"), (POSE_HURT, "hurt")):
            sheet.paste(body_pose_cell(pal, dr, kind), (dr * 32, 32 * pose))
    return sheet, pal


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


# Head shift of the hurt pose, away from the facing side (dx, dy per direction).
HURT_HEAD = [(1, 2), (2, 2), (-1, 2), (-2, 2)]


def hurt_gani():
    L = gani_header("hurt.gani: the pose Seihai shows for the first moments of hurt mode (1 step)")
    L += ["// The name and the use are Seihai's convention, not a documented real-client one.",
          "DEFAULTHEAD head0.png", "DEFAULTBODY body.png", ""]
    L += gani_sprites([POSE_HURT]) + ["", "ANI"]
    for dr in range(DIRS):
        (sx, sy), (bx, by), (hx, hy) = OFFSETS[dr]
        hdx, hdy = HURT_HEAD[dr]
        L.append(f"0 {sx} {sy}, {200 + 4 * POSE_HURT + dr} {bx} {by}, {100 + dr} {hx + hdx} {hy + hdy}")
    L += ["ANIEND", ""]
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


def new_board(rng):
    g = []
    for _ in range(64 * 64):
        r = rng.random()
        g.append(T["GRASS_A"] if r < 0.68 else (T["GRASS_B"] if r < 0.9 else T["GRASS_C"]))
    return g


def put_block(b, tid, x, y, w, h):
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


def level_new1(rng):
    b = new_board(rng)
    # border of bushes on the north, west and east edges; the south edge is open
    fill(b, T["BUSH"], 0, 0, 63, 1)
    fill(b, T["BUSH"], 0, 0, 1, 63)
    fill(b, T["BUSH"], 62, 0, 63, 63)
    # pond with a sand ring (centre 14,42; the water is deep enough to swim in)
    cx, cy, rx, ry = 14, 42, 8, 6
    for y in range(cy - ry - 1, cy + ry + 2):
        for x in range(cx - rx - 1, cx + rx + 2):
            e = ((x - cx) / (rx + 1.5)) ** 2 + ((y - cy) / (ry + 1.5)) ** 2
            w = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2
            if w <= 1.0:
                b[y * 64 + x] = T["WATER_A"] if (x + y) % 3 else T["WATER_B"]
            elif e <= 1.0:
                b[y * 64 + x] = T["SAND_A"] if (x * 7 + y * 3) % 5 else T["SAND_B"]
    # house: roof rows 14..16, wall rows 17..21, doorway at 41..42 on row 21
    fill(b, T["ROOF"], 36, 14, 47, 16)
    fill(b, T["BRICK_CAP"], 36, 17, 47, 17)
    fill(b, T["BRICK"], 36, 18, 47, 21)
    fill(b, T["DOORWAY"], 41, 21, 42, 21)
    # dirt path from the door to the spawn point and on to the pond
    fill(b, T["PATH"], 41, 22, 42, 30)
    fill(b, T["PATH"], 24, 30, 42, 31)
    # scenery
    put_block(b, T["TREE"], 50, 8, 2, 2)
    put_block(b, T["TREE"], 54, 24, 2, 2)
    put_block(b, T["TREE"], 8, 10, 2, 2)
    put_block(b, T["TREE"], 20, 20, 2, 2)
    for (x, y) in ((6, 28), (7, 28), (7, 29), (46, 30), (47, 30), (28, 12), (29, 12), (52, 44), (12, 58), (13, 58), (40, 52)):
        b[y * 64 + x] = T["BUSH"]
    for (x, y) in ((30, 26), (46, 40), (47, 41), (24, 52), (56, 54), (4, 20)):
        b[y * 64 + x] = T["ROCK"]
    # sign (2x1) and chest (2x2), both on grass
    put_block(b, T["SIGN_G"], 26, 28, 2, 1)
    put_block(b, T["CHEST_G"], 34, 26, 2, 2)
    links = [
        "LINK new2.nw 41 21 2 1 30 47",
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
    b = [T["VOID"]] * (64 * 64)
    # room: floor x 22..41, y 38..52; walls around it, top row 37 is the wall cap
    fill(b, T["BRICK_CAP"], 21, 37, 42, 37)
    fill(b, T["BRICK"], 21, 38, 21, 53)
    fill(b, T["BRICK"], 42, 38, 42, 53)
    fill(b, T["BRICK"], 21, 53, 42, 53)
    fill(b, T["WOOD_FLOOR"], 22, 38, 41, 52)
    fill(b, T["DOORWAY"], 31, 53, 32, 53)
    put_block(b, T["CHEST_W"], 24, 40, 2, 2)
    put_block(b, T["SIGN_W"], 36, 40, 2, 1)
    links = ["LINK new1.nw 31 53 2 1 40 22"]
    objs = [
        "CHEST 24 40 bluerupee -1",
        "SIGN 36 40",
        "This is the second starter level.",
        "The doorway to the south leads back outside.",
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
    save_png(im, p, os.path.join(out, "bodies", "body.png"), 4, 16, True)
    im, p = build_head()
    save_png(im, p, os.path.join(out, "heads", "head0.png"), 8, 16, True)
    im, p = build_sword()
    save_png(im, p, os.path.join(out, "swords", "sword1.png"), 4, 16, True)
    im, p = build_shield()
    save_png(im, p, os.path.join(out, "shields", "shield1.png"), 4, 16, True)
    im, p = build_sprites()
    save_png(im, p, os.path.join(out, "sprites.png"), 4, 16, True)
    im, p = build_state()
    save_png(im, p, os.path.join(out, "state.png"), 4, 16, True)

    for name, text in (("idle", idle_gani()), ("walk", walk_gani()), ("swim", swim_gani()),
                       ("sword", sword_gani()), ("sit", sit_gani()), ("hurt", hurt_gani())):
        open(os.path.join(out, "gani", name + ".gani"), "w", newline="\n").write(text)

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
        tile.alpha_composite(sheets[sh].crop((sx, sy, sx + w, sy + h)), (x, y))


def preview(out, path):
    """Contact sheet of our own art, for people to look at (not committed)."""
    S = 4
    tiles = [(k, v) for k, v in T.items() if k not in ("CHEST_G_OPEN", "CHEST_W_OPEN", "CHEST_W", "SIGN_W")]
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
        w, h = 1, 1
        if k.startswith(("CHEST", "TREE")):
            w, h = 2, 2
        if k.startswith("SIGN"):
            w, h = 2, 1
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
    a = ap.parse_args()
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
