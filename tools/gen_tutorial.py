"""Seihai's built-in tutorial: its tile sheet, its tile tables and its levels.

Called by tools/gen_default_assets.py (`write_tutorial(out, base)`), so the
same `--check` covers it. Everything is original: the art is drawn here from
shapes, shading rules and a hand-picked palette, with seeded randomness only.
No pixel of any Graal sheet was read.

What it writes (under assets/default/):

  tutorial/pics1.png     2048x512 tile sheet. The standard Graal ids that the
                         client gives a meaning (bush 2, sign 512, pot 684,
                         stones 34 and 990, tall grass 420, chests 1784 and
                         829, deep water 322, shallow water 398, ...) keep
                         that meaning and get new art; everything else the
                         tutorial draws (paths, shores, trees, the cottage,
                         its rooms, fences, cliffs, the bridge) sits in ids
                         2560 to 4095, which only this sheet and table use.
  tutorial/arrays.dat    the tile table and the cut, lift, ledge and block
                         tables of that sheet, in the arrays.dat layout
                         (big-endian u32 runs) that TileTypes::parse_arrays
                         and ObjectTables::parse read.
  levels/seihai_tutorial*.nw   the tutorial levels.

The client swaps this sheet and these tables in while the player is in a
level whose name starts with `seihai_tutorial`, and puts the player's own
back when they leave, so the tutorial looks and plays the same with or
without a Graal folder. See documentation/tutorial-levels.md.
"""
import math
import os
import random
import struct

from PIL import Image, ImageDraw

TUT_SEED = 20261031
PREFIX = "seihai_tutorial"

# ---------------------------------------------------------------------------
# Palette (RGB). Every colour drawn on the sheet comes from here or from the
# default sheet's palette, so the result fits an 8-bit palette PNG.
# ---------------------------------------------------------------------------

C = dict(
    # grass, dark to light
    g0=(30, 74, 58), g1=(46, 112, 66), g2=(70, 146, 74), g3=(102, 176, 86), g4=(150, 206, 108),
    # dirt and sand
    d0=(86, 58, 46), d1=(132, 92, 62), d2=(176, 130, 86), d3=(208, 168, 116), d4=(234, 206, 156),
    # water
    w0=(26, 48, 100), w1=(38, 82, 154), w2=(56, 120, 196), w3=(104, 172, 228), w4=(206, 238, 252),
    # stone
    s0=(46, 46, 64), s1=(82, 86, 106), s2=(122, 128, 146), s3=(166, 172, 186), s4=(212, 216, 226),
    # wood
    k0=(60, 38, 30), k1=(104, 66, 42), k2=(150, 100, 60), k3=(192, 140, 86), k4=(226, 186, 126),
    # roof
    r0=(80, 28, 40), r1=(132, 44, 50), r2=(180, 68, 60), r3=(218, 108, 84), r4=(240, 156, 120),
    # leaves
    l0=(20, 56, 48), l1=(32, 88, 60), l2=(52, 124, 68), l3=(90, 162, 80), l4=(146, 202, 102),
    # plaster
    p0=(136, 114, 100), p1=(196, 176, 150), p2=(230, 216, 190), p3=(246, 238, 220),
    # flowers and accents
    pink=(240, 128, 168), pink_dk=(186, 76, 120), yel=(250, 214, 84), yel_dk=(206, 156, 40),
    wht=(250, 250, 240), blu=(118, 156, 240), blu_dk=(70, 100, 190), red=(220, 62, 72), red_dk=(150, 36, 52),
    gold=(244, 204, 76), gold_dk=(178, 128, 40), gold_lt=(252, 238, 160),
    terra=(184, 94, 60), terra_dk=(120, 58, 42), terra_lt=(226, 144, 96),
    glass=(118, 176, 220), glass_lt=(204, 234, 250), glass_dk=(60, 96, 150),
    rug=(150, 58, 82), rug_lt=(196, 98, 108), rug_dk=(98, 36, 60),
    cloth=(84, 112, 190), cloth_lt=(140, 166, 226), cloth_dk=(54, 70, 136),
    void=(16, 14, 22), ol=(34, 28, 40),
    key=(255, 0, 255),  # chest art only: left out when a chest is drawn
    glow=(150, 236, 220), glow_lt=(226, 252, 246), glow_dk=(70, 170, 170),
    fire=(244, 140, 52), fire_lt=(252, 216, 96),
)

# Tile types (Seihai's numbers, as in gen_default_assets.py)
OPEN, CHAIR, BED_TOP, BED_BOTTOM = 0, 3, 4, 5
WATER, THROW_THROUGH, WALL = 11, 20, 22


def hsh(*v):
    """A small deterministic hash in [0, 1)."""
    h = 2166136261
    for x in v:
        h = ((h ^ (int(x) & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    h ^= h >> 13
    h = (h * 0x5BD1E995) & 0xFFFFFFFF
    h ^= h >> 15
    return h / 4294967296.0


def new(w, h, fill=None):
    return Image.new("RGB", (w, h), C[fill] if fill else C["g2"])


def px(im, x, y, c):
    if 0 <= x < im.width and 0 <= y < im.height:
        im.putpixel((x, y), C[c])


# ---------------------------------------------------------------------------
# Grass
# ---------------------------------------------------------------------------


def grass_tile(seed, tufts=3, flowers=0, pebbles=0):
    im = new(16, 16, "g2")
    r = random.Random(seed)
    # soft mottling, one or two shades
    for y in range(16):
        for x in range(16):
            n = hsh(x // 2, y // 2, seed)
            if n < 0.10:
                px(im, x, y, "g1")
            elif n > 0.93:
                px(im, x, y, "g3")
    for _ in range(tufts):
        x, y = r.randrange(2, 14), r.randrange(3, 14)
        for dx, dy, c in ((0, 0, "g1"), (-1, -1, "g1"), (1, -1, "g1"), (0, -1, "g3"), (-2, -2, "g3"), (2, -2, "g3"), (0, -2, "g3")):
            px(im, x + dx, y + dy, c)
    for _ in range(pebbles):
        x, y = r.randrange(2, 13), r.randrange(2, 13)
        for dx, dy, c in ((0, 0, "s2"), (1, 0, "s3"), (0, 1, "s1"), (1, 1, "s2"), (2, 1, "g1"), (0, 2, "g1"), (1, 2, "g1")):
            px(im, x + dx, y + dy, c)
    spots = [(4, 4), (11, 6), (6, 11), (12, 12)]
    r.shuffle(spots)
    for i in range(flowers):
        x, y = spots[i]
        x += r.randrange(-1, 2)
        y += r.randrange(-1, 2)
        petal, mid = r.choice((("pink", "yel"), ("wht", "yel"), ("yel", "terra"), ("blu", "wht")))
        px(im, x, y + 2, "g1")
        px(im, x + 1, y + 2, "g1")
        for dx, dy in ((0, -1), (-1, 0), (1, 0), (0, 1)):
            px(im, x + dx, y + dy, petal)
        px(im, x, y, mid)
    return im


# ---------------------------------------------------------------------------
# Corner-blended terrain (16 variants per pair: one bit per tile corner)
# ---------------------------------------------------------------------------

CORNERS = ((0, 0), (1, 0), (0, 1), (1, 1))  # TL, TR, BL, BR: bit 1, 2, 4, 8


def corner_value(mask, u, v):
    tl, tr, bl, br = [(mask >> i) & 1 for i in range(4)]
    top = tl + (tr - tl) * u
    bot = bl + (br - bl) * u
    return top + (bot - top) * v


def blend_mask(mask, wobble_seed, pad=4):
    """Which pixels (x, y in -pad .. 15 + pad) are the upper terrain. Rounded
    edges come from the bilinear corner value; a small wobble that is zero on
    the tile's edges makes them less regular without breaking seams."""
    out = {}
    for y in range(-pad, 16 + pad):
        for x in range(-pad, 16 + pad):
            u, v = (x + 0.5) / 16.0, (y + 0.5) / 16.0
            val = corner_value(mask, u, v)
            if 0 <= x < 16 and 0 <= y < 16:
                val += (hsh(x, y, wobble_seed) - 0.5) * 0.16 * math.sin(math.pi * u) * math.sin(math.pi * v)
            out[(x, y)] = val > 0.5
    return out


def distances(m, upper):
    """Chebyshev-ish distance (1 = touches the other terrain) inside the tile,
    up to 4, for pixels of the given side."""
    d = {}
    for y in range(16):
        for x in range(16):
            if m[(x, y)] != upper:
                continue
            best = 9
            for r in range(1, 5):
                hit = False
                for dy in range(-r, r + 1):
                    for dx in range(-r, r + 1):
                        if max(abs(dx), abs(dy)) != r or abs(dx) + abs(dy) > r + (r > 1):
                            continue
                        if m.get((x + dx, y + dy), upper) != upper:
                            hit = True
                if hit:
                    best = r
                    break
            d[(x, y)] = best
    return d


def dirt_full(seed):
    im = new(16, 16, "d3")
    r = random.Random(seed)
    for y in range(16):
        for x in range(16):
            n = hsh(x, y, seed)
            if n < 0.06:
                px(im, x, y, "d2")
            elif n > 0.96:
                px(im, x, y, "d4")
    for _ in range(2):
        x, y = r.randrange(2, 14), r.randrange(2, 14)
        px(im, x, y, "d4")
        px(im, x + 1, y, "d2")
        px(im, x, y + 1, "d2")
    return im


def water_full(seed, base="w1"):
    im = new(16, 16, base)
    r = random.Random(seed)
    for y in range(16):
        for x in range(16):
            if hsh(x // 3, y // 2, seed) < 0.18:
                px(im, x, y, "w2" if base == "w1" else "w3")
    for _ in range(2):
        x, y = r.randrange(1, 11), r.randrange(2, 14)
        for dx in range(4):
            px(im, x + dx, y, "w3")
        px(im, x + 1, y + 1, "w2")
        px(im, x + 2, y + 1, "w2")
    return im


def path_tile(mask, seed):
    if mask == 0:
        return grass_tile(seed, tufts=2)
    base_g = grass_tile(seed, tufts=1)
    base_d = dirt_full(seed + 7)
    m = blend_mask(mask, seed)
    dd = distances(m, True)
    dg = distances(m, False)
    im = new(16, 16)
    for y in range(16):
        for x in range(16):
            if m[(x, y)]:
                k = dd[(x, y)]
                c = base_d.getpixel((x, y))
                if k == 1:
                    c = C["d1"]
                elif k == 2:
                    c = C["d2"]
                im.putpixel((x, y), c)
            else:
                k = dg[(x, y)]
                c = base_g.getpixel((x, y))
                if k == 1:
                    c = C["g3"]
                im.putpixel((x, y), c)
    return im


def shore_tile(mask, seed):
    """Upper terrain = deep water; the lower side is grass with a sandy bank."""
    if mask == 0:
        return grass_tile(seed, tufts=2)
    if mask == 15:
        return water_full(seed)
    base_g = grass_tile(seed, tufts=1)
    base_w = water_full(seed + 3)
    m = blend_mask(mask, seed)
    dw = distances(m, True)
    dg = distances(m, False)
    im = new(16, 16)
    for y in range(16):
        for x in range(16):
            if m[(x, y)]:
                k = dw[(x, y)]
                c = {1: C["w4"], 2: C["w3"], 3: C["w2"]}.get(k, base_w.getpixel((x, y)))
                im.putpixel((x, y), c)
            else:
                k = dg[(x, y)]
                c = {1: C["d2"], 2: C["d3"], 3: C["g1"]}.get(k, base_g.getpixel((x, y)))
                im.putpixel((x, y), c)
    return im


# ---------------------------------------------------------------------------
# Shaded shapes
# ---------------------------------------------------------------------------


def sphere_shade(nx, ny, levels):
    """Pick a shade from a list (dark to light) for a point on a sphere with
    light from the upper left."""
    nz = math.sqrt(max(0.0, 1 - nx * nx - ny * ny))
    lum = 0.55 * (-nx * 0.55 - ny * 0.7 + nz * 0.45) + 0.5
    i = int(lum * len(levels))
    return levels[max(0, min(len(levels) - 1, i))]


def leafy_blob(im, cx, cy, rx, ry, seed, levels=("l1", "l2", "l3", "l4"), outline="l0", clumps=True):
    """A round leafy mass: sphere shading broken into small clumps, an outline
    and a darker underside."""
    inside = {}
    for y in range(int(cy - ry - 2), int(cy + ry + 3)):
        for x in range(int(cx - rx - 2), int(cx + rx + 3)):
            dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
            # scalloped edge
            ang = math.atan2(dy, dx)
            edge = 1.0 + 0.07 * math.sin(ang * 7 + seed) + 0.04 * math.sin(ang * 13 + seed * 2)
            if dx * dx + dy * dy <= edge * edge:
                inside[(x, y)] = (dx, dy)
    for (x, y), (dx, dy) in inside.items():
        # clump centres on a jittered grid: shade by the local clump normal
        if clumps:
            gx, gy = math.floor((x - cx) / 5.0), math.floor((y - cy) / 5.0)
            ccx = cx + (gx + 0.5) * 5 + (hsh(gx, gy, seed) - 0.5) * 2
            ccy = cy + (gy + 0.5) * 5 + (hsh(gy, gx, seed + 1) - 0.5) * 2
            lx, ly = (x + 0.5 - ccx) / 3.4, (y + 0.5 - ccy) / 3.4
            nx, ny = dx * 0.65 + lx * 0.35, dy * 0.65 + ly * 0.35
        else:
            nx, ny = dx, dy
        n = math.hypot(nx, ny)
        if n > 0.99:
            nx, ny = nx / n * 0.99, ny / n * 0.99
        c = sphere_shade(nx, ny, levels)
        px(im, x, y, c)
    for (x, y) in inside:
        if any((x + a, y + b) not in inside for a, b in ((1, 0), (-1, 0), (0, 1), (0, -1))):
            px(im, x, y, outline)
    return inside


def shadow_ellipse(im, cx, cy, rx, ry, c="g1"):
    for y in range(int(cy - ry), int(cy + ry) + 1):
        for x in range(int(cx - rx), int(cx + rx) + 1):
            if ((x + 0.5 - cx) / rx) ** 2 + ((y + 0.5 - cy) / ry) ** 2 <= 1:
                cur = im.getpixel((x, y)) if 0 <= x < im.width and 0 <= y < im.height else None
                if cur in (C["g2"], C["g3"], C["g4"], C["g1"]):
                    px(im, x, y, c)
                elif cur in (C["d3"], C["d4"], C["d2"]):
                    px(im, x, y, "d1")


def grass_canvas(w, h, seed):
    im = new(w * 16, h * 16)
    for ty in range(h):
        for tx in range(w):
            im.paste(grass_tile(seed + tx * 7 + ty * 13, tufts=1), (tx * 16, ty * 16))
    return im


def rock_shape(im, pts, seed, levels=("s1", "s2", "s3", "s4"), outline="s0"):
    """A faceted rock from a polygon: shade each pixel by a fake normal from
    the centroid, plus a crack or two."""
    mask = Image.new("L", im.size, 0)
    ImageDraw.Draw(mask).polygon(pts, fill=255)
    cx = sum(p[0] for p in pts) / len(pts)
    cy = sum(p[1] for p in pts) / len(pts)
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    rx, ry = (max(xs) - min(xs)) / 2, (max(ys) - min(ys)) / 2
    inside = set()
    for y in range(im.height):
        for x in range(im.width):
            if mask.getpixel((x, y)):
                inside.add((x, y))
    for (x, y) in inside:
        nx, ny = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
        # facets: quantise the normal's angle
        ang = math.atan2(ny, nx)
        q = round(ang / (math.pi / 3)) * (math.pi / 3)
        r = min(0.95, math.hypot(nx, ny))
        nx2, ny2 = math.cos(q) * r, math.sin(q) * r
        px(im, x, y, sphere_shade(nx2 * 0.8 + nx * 0.2, ny2 * 0.8 + ny * 0.2, levels))
    for (x, y) in inside:
        if any((x + a, y + b) not in inside for a, b in ((1, 0), (-1, 0), (0, 1), (0, -1))):
            px(im, x, y, outline)
    return inside


# ---------------------------------------------------------------------------
# Objects on the standard ids
# ---------------------------------------------------------------------------


def bush_art(seed):
    im = grass_canvas(2, 2, seed)
    shadow_ellipse(im, 16, 27, 14, 4)
    leafy_blob(im, 16, 16, 14.5, 13, seed % 97, levels=("l1", "l2", "l2", "l3"), outline="l0")
    r = random.Random(seed)
    for _ in range(3):
        x, y = r.randrange(8, 24), r.randrange(10, 23)
        px(im, x, y, "red")
        px(im, x + 1, y, "red_dk")
        px(im, x, y - 1, "pink")
    return im


def stone_art(seed, dark):
    im = grass_canvas(2, 2, seed)
    shadow_ellipse(im, 16, 26, 13, 5)
    lv = ("s0", "s1", "s1", "s2") if dark else ("s1", "s2", "s3", "s4")
    rock_shape(im, [(4, 24), (5, 13), (11, 6), (20, 5), (27, 11), (28, 22), (22, 28), (10, 28)], seed, lv, "ol" if dark else "s0")
    # a crack
    for i, (x, y) in enumerate(((17, 12), (18, 13), (18, 14), (19, 15), (20, 16))):
        px(im, x, y, "s0" if not dark else "ol")
    px(im, 9, 11, "s4" if not dark else "s2")
    px(im, 10, 10, "s4" if not dark else "s2")
    return im


def pot_art(seed):
    im = grass_canvas(2, 2, seed)
    shadow_ellipse(im, 16, 28, 11, 3)
    inside = set()
    for y in range(4, 30):
        for x in range(4, 28):
            t = (y - 4) / 25.0
            # vase profile: neck, shoulder, belly, foot
            w = 5 if t < 0.15 else 5 + (t - 0.15) / 0.25 * 7 if t < 0.4 else 12 - max(0, (t - 0.72)) * 14
            if abs(x + 0.5 - 16) <= w:
                inside.add((x, y))
    for (x, y) in inside:
        t = (y - 4) / 25.0
        nx = (x + 0.5 - 16) / 12.0
        c = sphere_shade(nx, (t - 0.55), ("terra_dk", "terra", "terra", "terra_lt"))
        px(im, x, y, c)
    for (x, y) in inside:
        if any((x + a, y + b) not in inside for a, b in ((1, 0), (-1, 0), (0, 1), (0, -1))):
            px(im, x, y, "ol")
    # rim and opening
    for x in range(11, 21):
        px(im, x, 4, "terra_lt")
        px(im, x, 5, "terra_dk")
    for x in range(12, 20):
        px(im, x, 5, "ol")
    # a painted band
    for x in range(6, 26):
        if (x + 0.5 - 16) ** 2 < 100 and (x, 17) in inside:
            px(im, x, 17, "yel_dk" if x % 3 else "wht")
    return im


def sign_art(seed):
    im = grass_canvas(2, 2, seed)
    shadow_ellipse(im, 16, 29, 7, 2)
    d = ImageDraw.Draw(im)
    d.rectangle([14, 18, 17, 30], fill=C["k1"])
    d.line([(14, 18), (14, 30)], fill=C["k0"])
    d.line([(17, 18), (17, 30)], fill=C["k0"])
    d.line([(15, 18), (15, 29)], fill=C["k2"])
    d.rectangle([2, 4, 29, 20], fill=C["k3"], outline=C["k0"])
    d.line([(3, 5), (28, 5)], fill=C["k4"])
    d.line([(3, 12), (28, 12)], fill=C["k2"])
    d.line([(3, 19), (28, 19)], fill=C["k2"])
    for (x0, x1, y) in ((6, 24, 8), (6, 20, 10), (6, 22, 15), (6, 17, 17)):
        for x in range(x0, x1):
            if hsh(x, y, 5) > 0.2:
                px(im, x, y, "k0")
    for x, y in ((4, 6), (27, 6), (4, 18), (27, 18)):
        px(im, x, y, "s2")
    return im


def tall_grass_art(seed):
    im = grass_canvas(2, 2, seed)
    r = random.Random(seed)
    blades = []
    for i in range(22):
        x = 2 + (i * 28) // 21 + r.randrange(-1, 2)
        h = r.randrange(14, 27)
        lean = r.choice((-2, -1, 0, 1, 2))
        blades.append((x, h, lean))
    for x, h, lean in sorted(blades, key=lambda b: b[1]):
        for k in range(h):
            yy = 30 - k
            xx = x + round(lean * (k / h) ** 2 * 2)
            c = "g1" if k < h * 0.3 else "g2" if k < h * 0.65 else "g3" if k < h - 2 else "g4"
            px(im, xx, yy, c)
            if k < h - 3:
                px(im, xx + 1, yy, "g1" if k < h * 0.5 else "g2")
    return im


def patch_art(seed, kind):
    """What an object leaves: a stump, bare earth, a dent."""
    im = grass_canvas(2, 2, seed)
    r = random.Random(seed)
    if kind == "bush":
        shadow_ellipse(im, 16, 18, 9, 6, "g1")
        for _ in range(9):
            x, y = r.randrange(9, 23), r.randrange(13, 23)
            px(im, x, y, "l1")
            px(im, x + 1, y, "g3")
        d = ImageDraw.Draw(im)
        d.ellipse([12, 13, 19, 19], fill=C["k2"], outline=C["k0"])
        d.ellipse([14, 15, 17, 17], fill=C["k3"])
    elif kind == "grass":
        for i in range(14):
            x, y = 3 + i * 2, 24 + r.randrange(-1, 3)
            px(im, x, y, "g1")
            px(im, x, y - 1, "g3")
            px(im, x + 1, y - 2, "g4")
    else:
        fill, rim = {"sign": ("d2", "d1"), "pot": ("d3", "d2"), "stone": ("d1", "d0")}[kind]
        for y in range(6, 28):
            for x in range(5, 28):
                v = ((x + 0.5 - 16) / 10.5) ** 2 + ((y + 0.5 - 17) / 9.0) ** 2
                v += (hsh(x, y, seed) - 0.5) * 0.25
                if v < 0.75:
                    px(im, x, y, fill)
                elif v < 1.0:
                    px(im, x, y, rim)
        for _ in range(6):
            px(im, r.randrange(10, 22), r.randrange(11, 22), rim)
    return im


def chest_art(opened, seed):
    """A chest on the key colour: the client draws chests over the board and
    leaves the key out, so the chest stands on grass outside and on the floor
    indoors."""
    im = new(32, 32, "key")
    d = ImageDraw.Draw(im)
    d.rectangle([4, 27, 27, 29], fill=C["ol"])
    # body
    d.rectangle([3, 13, 28, 28], fill=C["k2"], outline=C["k0"])
    for y in (17, 21, 25):
        d.line([(4, y), (27, y)], fill=C["k1"])
    d.line([(4, 14), (27, 14)], fill=C["k3"])
    for x in (6, 25):
        d.rectangle([x, 13, x + 1, 28], fill=C["gold_dk"])
        px(im, x, 14, "gold")
    if not opened:
        d.rectangle([3, 5, 28, 13], fill=C["k3"], outline=C["k0"])
        d.line([(4, 6), (27, 6)], fill=C["k4"])
        d.line([(4, 9), (27, 9)], fill=C["k2"])
        for x in (6, 25):
            d.rectangle([x, 5, x + 1, 13], fill=C["gold"])
        d.rectangle([13, 10, 18, 17], fill=C["gold"], outline=C["gold_dk"])
        px(im, 15, 13, "ol")
        px(im, 16, 13, "ol")
        px(im, 15, 14, "ol")
        px(im, 14, 11, "gold_lt")
    else:
        d.polygon([(4, 13), (27, 13), (25, 3), (6, 3)], fill=C["k1"], outline=C["k0"])
        d.line([(7, 4), (24, 4)], fill=C["k2"])
        d.rectangle([4, 14, 27, 17], fill=C["ol"])
        for x, c in ((9, "gold"), (12, "gold_lt"), (16, "gold"), (20, "gold_lt"), (23, "gold")):
            px(im, x, 15, c)
            px(im, x + 1, 16, "gold_dk")
    return im


# ---------------------------------------------------------------------------
# Decorations on the tutorial's own ids
# ---------------------------------------------------------------------------


def tree_art(seed):
    """A round broadleaf tree, 4 x 4 tiles (64 x 64)."""
    im = grass_canvas(4, 4, seed)
    shadow_ellipse(im, 33, 56, 22, 6)
    d = ImageDraw.Draw(im)
    # trunk with roots
    d.polygon([(26, 60), (28, 40), (36, 40), (38, 60), (41, 62), (23, 62)], fill=C["k1"], outline=C["k0"])
    d.line([(30, 42), (29, 59)], fill=C["k2"])
    d.line([(34, 44), (35, 58)], fill=C["k0"])
    leafy_blob(im, 32, 24, 28, 22, seed % 97)
    # a few highlight leaves
    r = random.Random(seed)
    for _ in range(10):
        x, y = r.randrange(14, 40), r.randrange(8, 26)
        if im.getpixel((x, y)) in (C["l3"], C["l4"]):
            px(im, x, y, "l4")
            px(im, x + 1, y + 1, "l3")
    return im


def pine_art(seed):
    """A pine, 2 x 3 tiles (32 x 48)."""
    im = grass_canvas(2, 3, seed)
    shadow_ellipse(im, 16, 44, 11, 3)
    d = ImageDraw.Draw(im)
    d.rectangle([14, 36, 17, 45], fill=C["k1"], outline=C["k0"])
    for i, (top, bot, half) in enumerate(((2, 18, 8), (9, 28, 12), (17, 39, 15))):
        pts = [(16, top), (16 + half, bot), (16 - half, bot)]
        mask = Image.new("L", im.size, 0)
        ImageDraw.Draw(mask).polygon(pts, fill=255)
        cells = [(x, y) for y in range(im.height) for x in range(im.width) if mask.getpixel((x, y))]
        cs = set(cells)
        for (x, y) in cells:
            t = (x + 0.5 - 16) / half
            fy = (y - top) / max(1, bot - top)
            c = "l3" if t < -0.35 else "l2" if t < 0.25 else "l1"
            if fy > 0.85:
                c = "l1" if t < 0.2 else "l0"
            if (x + y * 3) % 7 == 0 and c == "l2":
                c = "l3"
            px(im, x, y, c)
        for (x, y) in cells:
            if any((x + a, y + b) not in cs for a, b in ((1, 0), (-1, 0), (0, 1))):
                px(im, x, y, "l0")
    return im


def flowerbed_art(seed):
    im = grass_canvas(2, 2, seed)
    d = ImageDraw.Draw(im)
    d.rectangle([1, 3, 30, 29], fill=C["d1"], outline=C["k1"])
    d.rectangle([2, 4, 29, 28], fill=C["d2"])
    r = random.Random(seed)
    for row in range(3):
        for col in range(4):
            x, y = 5 + col * 7 + r.randrange(-1, 2), 8 + row * 8 + r.randrange(-1, 2)
            petal, mid = (("pink", "yel"), ("yel", "terra"), ("wht", "yel"), ("blu", "wht"))[(row + col + seed) % 4]
            for dx, dy in ((0, 2), (1, 2), (0, 3)):
                px(im, x + dx, y + dy, "g1")
            for dx, dy in ((0, -1), (-1, 0), (1, 0), (0, 1), (-1, -1), (1, 1)):
                px(im, x + dx, y + dy, petal)
            px(im, x, y, mid)
    return im


def fence_art(kind):
    """Wooden fence pieces, 1 tile: 'h' rails with posts at both edges, 'v' a
    vertical run, 'post' a lone post."""
    im = new(16, 16)
    im.paste(grass_tile(91 + len(kind), tufts=1), (0, 0))
    d = ImageDraw.Draw(im)
    if kind == "h":
        for y in (6, 11):
            d.rectangle([0, y, 15, y + 1], fill=C["k3"])
            d.line([(0, y + 2), (15, y + 2)], fill=C["k0"])
        for y in range(13, 16):
            for x in range(16):
                if im.getpixel((x, y)) in (C["g2"], C["g3"]):
                    px(im, x, y, "g1") if y == 13 else None
        for x0 in (0, 13):
            d.rectangle([x0, 2, x0 + 2, 14], fill=C["k2"], outline=C["k0"])
            d.line([(x0 + 1, 3), (x0 + 1, 13)], fill=C["k3"])
    elif kind == "v":
        d.rectangle([6, 0, 9, 15], fill=C["k2"], outline=C["k0"])
        d.line([(7, 0), (7, 15)], fill=C["k3"])
        for y in (0, 15):
            d.line([(6, y), (9, y)], fill=C["k2"])
        d.rectangle([5, 4, 10, 8], fill=C["k2"], outline=C["k0"])
        d.line([(6, 5), (9, 5)], fill=C["k4"])
        for y in range(1, 16):
            px(im, 10, y, "g1")
    else:
        d.rectangle([6, 2, 9, 14], fill=C["k2"], outline=C["k0"])
        d.line([(7, 3), (7, 13)], fill=C["k3"])
        for x in range(5, 12):
            px(im, x, 15, "g1")
    return im


def cottage_art(seed):
    """The cottage, 8 x 7 tiles: a red gable roof, plaster walls with timber,
    two windows and a door (columns 3 and 4, rows 5 and 6)."""
    W, H = 128, 112
    im = grass_canvas(8, 7, seed)
    d = ImageDraw.Draw(im)
    # wall block rows 4..6 (y 60..111), slightly inset
    d.rectangle([4, 58, 123, 108], fill=C["p2"], outline=C["ol"])
    for y in range(59, 108):
        px(im, 5, y, "p3")
    # timber frame
    for x in (4, 40, 86, 122):
        d.rectangle([x, 58, x + 2, 108], fill=C["k1"])
    d.rectangle([4, 58, 123, 61], fill=C["k1"])
    d.rectangle([4, 104, 123, 108], fill=C["s2"])
    d.line([(4, 104), (123, 104)], fill=C["s3"])
    d.line([(4, 108), (123, 108)], fill=C["s0"])
    # windows
    for wx in (14, 96):
        d.rectangle([wx, 68, wx + 18, 88], fill=C["glass"], outline=C["k0"])
        d.line([(wx + 9, 68), (wx + 9, 88)], fill=C["k1"])
        d.line([(wx, 78), (wx + 18, 78)], fill=C["k1"])
        d.line([(wx + 2, 70), (wx + 6, 70)], fill=C["glass_lt"])
        d.line([(wx + 2, 71), (wx + 3, 71)], fill=C["glass_lt"])
        d.rectangle([wx - 2, 89, wx + 20, 91], fill=C["k2"], outline=C["k0"])
        # flower box
        for i in range(5):
            fx = wx + 1 + i * 4
            px(im, fx, 88, ("pink", "yel", "red", "wht", "pink")[i])
            px(im, fx + 1, 88, "g1")
    # door: columns 3..4 (x 48..79), rows 5..6 (y 80..111)
    d.rectangle([50, 72, 77, 108], fill=C["k2"], outline=C["k0"])
    d.pieslice([50, 64, 77, 82], 180, 360, fill=C["k2"], outline=C["k0"])
    for x in (56, 63, 70):
        d.line([(x, 70), (x, 107)], fill=C["k1"])
    d.line([(52, 74), (52, 106)], fill=C["k3"])
    px(im, 72, 90, "gold")
    px(im, 72, 91, "gold_dk")
    d.rectangle([50, 106, 77, 111], fill=C["s3"], outline=C["s1"])
    # roof rows 0..3 (y 0..63) overhanging the wall
    roof = [(0, 60), (12, 6), (116, 6), (127, 60)]
    d.polygon(roof, fill=C["r2"])
    for row, y in enumerate(range(10, 60, 6)):
        off = 0 if row % 2 else 4
        frac = (y - 6) / 54.0
        x0, x1 = int(12 - 12 * frac), int(116 + 11 * frac)
        d.line([(x0, y), (x1, y)], fill=C["r1"])
        d.line([(x0, y + 1), (x1, y + 1)], fill=C["r3"])
        for x in range(x0 + off, x1, 8):
            d.line([(x, y - 5), (x, y - 1)], fill=C["r1"])
    d.polygon(roof, outline=C["ol"])
    d.line([(12, 7), (116, 7)], fill=C["r4"])
    d.rectangle([0, 58, 127, 61], fill=C["r0"])
    d.line([(1, 59), (126, 59)], fill=C["r1"])
    # chimney
    d.rectangle([92, 0, 103, 14], fill=C["s2"], outline=C["ol"])
    d.line([(93, 1), (102, 1)], fill=C["s4"])
    d.line([(93, 6), (102, 6)], fill=C["s1"])
    d.line([(93, 10), (102, 10)], fill=C["s1"])
    return im


def cliff_art(part, seed):
    """A raised terrace's south face, 1 x 3 tiles ('mid' or 'mid2'): grass
    ending in a lip, then a face of rounded rock columns lit from the left,
    then the shadow it casts on the grass below."""
    im = grass_canvas(1, 3, seed)
    r = random.Random(seed)
    # column boundaries across the 16 px (seamless: a boundary at 0 and 16)
    cuts = [0, 7 + r.randrange(0, 3), 16]
    top, bot = 7, 41
    joints = {(k, y) for k in range(2) for y in (top + r.randrange(8, 14), top + r.randrange(18, 26))}
    for x in range(16):
        k = max(i for i in range(2) if cuts[i] <= x)
        a, b = cuts[k], cuts[k + 1]
        t = (x + 0.5 - a) / (b - a)          # 0 at the column's left, 1 at its right
        for y in range(top, bot):
            v = (y - top) / (bot - top)
            shade = "s3" if t < 0.3 else "s2" if t < 0.75 else "s1"
            if v > 0.75 and shade != "s1":
                shade = {"s3": "s2", "s2": "s1"}[shade]
            if hsh(x, y // 3, seed) < 0.05:
                shade = "s1" if shade != "s1" else "s0"
            if (k, y) in joints and 0.1 < t < 0.95:
                shade = "s0"
            elif (k, y - 1) in joints and t < 0.8:
                shade = "s3"
            px(im, x, y, shade)
        px(im, b - 1 if b < 16 else 15, top, "s2")
        for y in range(top, bot):
            if x == a and a > 0:
                px(im, x, y, "s0")
        # chipped bottom edge of each column
        for y in range(bot, bot + 1 + (1 if t > 0.5 else 0)):
            px(im, x, y, "s0")
    # grass lip, hanging a little over the edge
    for x in range(16):
        lip = top + (1 if hsh(x, 1, seed) > 0.55 else 0)
        for y in range(0, lip):
            if y >= 3:
                px(im, x, y, "g2" if y < lip - 1 else "g3")
        px(im, x, lip, "g1")
        if hsh(x, 2, seed) > 0.65:
            px(im, x, lip + 1, "g1")
    # shadow on the grass below
    for x in range(16):
        for y in range(bot + 1, 47):
            c = im.getpixel((x, y))
            if c in (C["g2"], C["g3"], C["g4"]) and (y < bot + 4 or hsh(x, y, seed) < 0.4):
                px(im, x, y, "g1")
    return im


def stairs_art(seed):
    """Stone steps up the terrace face, 2 x 3 tiles (the face is 2 rows of
    rock and a row of shadow)."""
    im2 = Image.new("RGB", (32, 48))
    im2.paste(cliff_art("mid", seed), (0, 0))
    im2.paste(cliff_art("mid", seed + 1), (16, 0))
    d = ImageDraw.Draw(im2)
    d.rectangle([3, 4, 28, 44], fill=C["s1"], outline=C["s0"])
    for y in range(6, 43, 5):
        d.rectangle([4, y, 27, y + 3], fill=C["s3"])
        d.line([(4, y), (27, y)], fill=C["s4"])
        d.line([(4, y + 4), (27, y + 4)], fill=C["s0"])
    return im2


def block_art(kind):
    """A heavy carved stone, 2 x 2 tiles: 'push' shows four arrows, 'pull' an
    iron ring."""
    im = new(32, 32, "d2")
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 31, 31], fill=C["s2"], outline=C["s0"])
    d.rectangle([1, 1, 30, 30], outline=C["s3"])
    d.line([(1, 30), (30, 30)], fill=C["s1"])
    d.line([(30, 1), (30, 30)], fill=C["s1"])
    d.rectangle([5, 5, 26, 26], fill=C["s3"], outline=C["s1"])
    if kind == "push":
        for (ax, ay, dx, dy) in ((16, 8, 0, -1), (16, 23, 0, 1), (8, 16, -1, 0), (23, 16, 1, 0)):
            for k in range(4):
                for w in range(-k, k + 1):
                    x = ax + (w if dx == 0 else -dx * k + dx * 3)
                    y = ay + (w if dy == 0 else -dy * k + dy * 3)
                    px(im, x, y, "s0")
        d.ellipse([13, 13, 18, 18], fill=C["s1"])
    else:
        d.ellipse([9, 11, 22, 24], outline=C["s0"], width=2)
        d.ellipse([10, 12, 21, 23], outline=C["s4"])
        d.rectangle([14, 8, 17, 12], fill=C["s0"])
    return im


def shallow_art(seed):
    im = water_full(seed, "w2")
    r = random.Random(seed)
    for _ in range(3):
        x, y = r.randrange(2, 12), r.randrange(2, 14)
        px(im, x, y, "w4")
        px(im, x + 1, y, "w3")
    return im


def lily_art(seed):
    im = water_full(seed)
    d = ImageDraw.Draw(im)
    d.pieslice([2, 3, 13, 12], 30, 350, fill=C["l3"], outline=C["l1"])
    px(im, 6, 6, "l4")
    d.ellipse([9, 3, 12, 6], fill=C["pink"])
    px(im, 10, 4, "wht")
    return im


def reeds_art(seed):
    im = grass_tile(seed, tufts=0)
    r = random.Random(seed)
    for i in range(6):
        x = 2 + i * 2 + r.randrange(0, 2)
        h = r.randrange(8, 14)
        for k in range(h):
            px(im, x, 15 - k, "g1" if k < h - 3 else "l3")
        px(im, x, 15 - h, "k1")
        px(im, x, 14 - h, "k1")
    return im


def rock_small_art(seed):
    im = grass_tile(seed, tufts=0)
    shadow_ellipse(im, 8, 12, 6, 2)
    rock_shape(im, [(2, 12), (3, 6), (8, 3), (13, 5), (14, 11), (10, 14), (5, 14)], seed)
    return im


def mushroom_art(seed):
    im = grass_tile(seed, tufts=1)
    for (cx, cy, r) in ((5, 9, 3), (11, 11, 2)):
        for x in range(cx - 1, cx + 1):
            for y in range(cy, cy + 3):
                px(im, x, y, "p3")
        for y in range(cy - r, cy + 1):
            for x in range(cx - r - 1, cx + r + 1):
                if ((x + 0.5 - cx) / (r + 0.5)) ** 2 + ((y + 0.5 - cy) / (r + 0.2)) ** 2 <= 1:
                    px(im, x, y, "red")
        px(im, cx - 1, cy - r + 1, "wht")
        px(im, cx + 1, cy - 1, "wht")
    return im


def bridge_art(part, seed):
    """A north-south bridge, 1 tile: 'deck' planks, 'rail_w', 'rail_e'."""
    im = water_full(seed)
    d = ImageDraw.Draw(im)
    if part == "deck":
        d.rectangle([0, 0, 15, 15], fill=C["k3"])
        for y in (3, 7, 11, 15):
            d.line([(0, y), (15, y)], fill=C["k1"])
        for y in (0, 4, 8, 12):
            d.line([(0, y), (15, y)], fill=C["k4"])
        for y, x in ((1, 5), (5, 12), (9, 3), (13, 9)):
            px(im, x, y, "k2")
    else:
        west = part == "rail_w"
        x0, x1 = (6, 15) if west else (0, 9)
        d.rectangle([x0, 0, x1, 15], fill=C["k3"])
        for y in (3, 7, 11, 15):
            d.line([(x0, y), (x1, y)], fill=C["k1"])
        rx = 6 if west else 6
        d.rectangle([rx, 0, rx + 3, 15], fill=C["k2"], outline=C["k0"])
        d.line([(rx + 1, 0), (rx + 1, 15)], fill=C["k4"])
        d.rectangle([rx - 1, 5, rx + 4, 9], fill=C["k1"], outline=C["k0"])
        sx = 5 if west else 10
        for y in range(16):
            px(im, sx, y, "w0")
    return im


def warp_art(seed):
    """A glowing stone ring that takes the player home, 2 x 2 tiles."""
    im = grass_canvas(2, 2, seed)
    for y in range(32):
        for x in range(32):
            r = math.hypot(x + 0.5 - 16, (y + 0.5 - 16) * 1.15)
            if r <= 14:
                c = "s1" if r > 12.5 else "s3" if r > 11 else "glow_dk" if r > 8 else "glow" if r > 4 else "glow_lt"
                if 11 < r <= 12.5 and int(math.degrees(math.atan2(y - 16, x - 16)) // 30) % 2:
                    c = "s2"
                px(im, x, y, c)
    for (x, y) in ((16, 9), (10, 18), (21, 20), (14, 14), (19, 12)):
        px(im, x, y, "wht")
    return im


# ---------------------------------------------------------------------------
# Interior
# ---------------------------------------------------------------------------


def floor_art(seed):
    im = new(16, 16, "k3")
    d = ImageDraw.Draw(im)
    for y in (0, 8):
        d.line([(0, y), (15, y)], fill=C["k2"])
        d.line([(0, y + 1), (15, y + 1)], fill=C["k4"])
    off = (seed % 2) * 8
    for y0, x in ((1, 4 + off), (9, (12 + off) % 16)):
        d.line([(x % 16, y0), (x % 16, y0 + 6)], fill=C["k2"])
    for y in range(16):
        for x in range(16):
            if hsh(x, y, seed) < 0.05:
                px(im, x, y, "k2")
    return im


def wall_back_art(row, kind, seed):
    """The back wall, 1 tile wide, 2 rows: row 0 plaster, row 1 panelling.
    `kind` is 'plain', 'window' or 'shelf'."""
    im = new(16, 16, "p2" if row == 0 else "k2")
    d = ImageDraw.Draw(im)
    if row == 0:
        for y in range(16):
            for x in range(16):
                if hsh(x // 2, y // 2, seed) < 0.08:
                    px(im, x, y, "p1")
        d.line([(0, 0), (15, 0)], fill=C["k1"])
        d.line([(0, 1), (15, 1)], fill=C["k0"])
        if kind == "window":
            d.rectangle([1, 4, 14, 15], fill=C["glass"], outline=C["k0"])
            d.line([(8, 4), (8, 15)], fill=C["k1"])
            d.line([(3, 6), (5, 6)], fill=C["glass_lt"])
    else:
        d.line([(0, 0), (15, 0)], fill=C["k4"])
        d.line([(0, 1), (15, 1)], fill=C["k1"])
        for x in (0, 8):
            d.line([(x, 2), (x, 13)], fill=C["k1"])
            d.line([(x + 1, 2), (x + 1, 13)], fill=C["k3"])
        if kind == "window":
            d.rectangle([1, 0, 14, 3], fill=C["glass"], outline=C["k0"])
            d.rectangle([0, 3, 15, 5], fill=C["k3"], outline=C["k0"])
        d.line([(0, 14), (15, 14)], fill=C["k0"])
        d.line([(0, 15), (15, 15)], fill=C["k1"])
    return im


def wall_side_art(kind):
    """Room edges: 'w' / 'e' walls, 'top', 'bottom', and the four corners,
    as a thick dark rim."""
    im = new(16, 16, "void")
    d = ImageDraw.Draw(im)
    rim = {"w": [(12, 0, 15, 15)], "e": [(0, 0, 3, 15)], "top": [(0, 12, 15, 15)], "bottom": [(0, 0, 15, 3)],
           "nw": [(12, 12, 15, 15)], "ne": [(0, 12, 3, 15)], "sw": [(12, 0, 15, 3)], "se": [(0, 0, 3, 3)]}[kind]
    for r in rim:
        d.rectangle(r, fill=C["k1"])
    if kind == "w":
        d.line([(15, 0), (15, 15)], fill=C["k0"])
        d.line([(12, 0), (12, 15)], fill=C["k2"])
    if kind == "e":
        d.line([(0, 0), (0, 15)], fill=C["k0"])
        d.line([(3, 0), (3, 15)], fill=C["k2"])
    if kind == "bottom":
        d.line([(0, 0), (15, 0)], fill=C["k0"])
        d.line([(0, 3), (15, 3)], fill=C["k2"])
    if kind == "top":
        d.line([(0, 12), (15, 12)], fill=C["k2"])
    return im


def bed_art():
    """A bed, 2 x 3 tiles: pillow and blanket."""
    im = new(32, 48, "k3")
    im.paste(floor_art(1), (0, 0))
    im.paste(floor_art(2), (16, 0))
    for y in (16, 32):
        im.paste(floor_art(y), (0, y))
        im.paste(floor_art(y + 1), (16, y))
    d = ImageDraw.Draw(im)
    d.rectangle([1, 1, 30, 46], fill=C["k1"], outline=C["k0"])
    d.rectangle([3, 3, 28, 44], fill=C["wht"])
    d.rectangle([5, 4, 26, 13], fill=C["p3"], outline=C["p1"])
    d.rectangle([3, 16, 28, 44], fill=C["cloth"], outline=C["cloth_dk"])
    d.line([(4, 17), (27, 17)], fill=C["cloth_lt"])
    for y in range(20, 44, 6):
        for x in range(5, 27, 6):
            px(im, x + (y // 6) % 2 * 3, y, "cloth_lt")
    d.rectangle([1, 44, 30, 46], fill=C["k0"])
    return im


def table_art():
    """A round table, 2 x 2 tiles."""
    im = new(32, 32, "k3")
    for y in (0, 16):
        for x in (0, 16):
            im.paste(floor_art(x + y), (x, y))
    shadow = Image.new("L", (32, 32), 0)
    d = ImageDraw.Draw(im)
    d.ellipse([3, 19, 28, 30], fill=C["k1"])
    d.rectangle([14, 18, 17, 28], fill=C["k0"])
    d.ellipse([2, 4, 29, 22], fill=C["k3"], outline=C["k0"])
    d.ellipse([5, 6, 26, 19], outline=C["k4"])
    d.ellipse([12, 9, 19, 14], fill=C["wht"], outline=C["s2"])
    px(im, 15, 10, "red")
    px(im, 16, 11, "red")
    return im


def chair_art():
    im = floor_art(5)
    d = ImageDraw.Draw(im)
    d.rectangle([3, 1, 12, 5], fill=C["k2"], outline=C["k0"])
    d.rectangle([3, 6, 12, 12], fill=C["rug_lt"], outline=C["k0"])
    d.line([(4, 13), (4, 15)], fill=C["k0"])
    d.line([(11, 13), (11, 15)], fill=C["k0"])
    return im


def rug_art(w, h):
    im = new(w * 16, h * 16)
    for y in range(h):
        for x in range(w):
            im.paste(floor_art(x * 3 + y), (x * 16, y * 16))
    d = ImageDraw.Draw(im)
    d.rectangle([2, 2, w * 16 - 3, h * 16 - 3], fill=C["rug"], outline=C["rug_dk"])
    d.rectangle([5, 5, w * 16 - 6, h * 16 - 6], outline=C["gold"])
    d.rectangle([9, 9, w * 16 - 10, h * 16 - 10], outline=C["rug_lt"])
    cx, cy = w * 8, h * 8
    d.polygon([(cx, cy - 6), (cx + 8, cy), (cx, cy + 6), (cx - 8, cy)], fill=C["rug_lt"], outline=C["gold"])
    for x in range(3, w * 16 - 3, 3):
        px(im, x, 0, "rug_lt")
        px(im, x, h * 16 - 1, "rug_lt")
    return im


def shelf_art():
    """A bookshelf against the back wall, 2 x 2 tiles."""
    im = new(32, 32, "k1")
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 31, 31], fill=C["k1"], outline=C["k0"])
    for y0 in (3, 17):
        d.rectangle([2, y0, 29, y0 + 11], fill=C["k0"])
        x = 3
        r = random.Random(y0)
        while x < 28:
            w = r.randrange(2, 4)
            h = r.randrange(7, 11)
            c = r.choice(("red", "blu_dk", "cloth", "l2", "gold_dk", "rug"))
            d.rectangle([x, y0 + 11 - h, x + w - 1, y0 + 10], fill=C[c])
            px(im, x, y0 + 11 - h, "wht")
            x += w + (1 if r.random() < 0.3 else 0)
        d.line([(1, y0 + 12), (30, y0 + 12)], fill=C["k3"])
    return im


def hearth_art():
    """A stone fireplace, 2 x 2 tiles."""
    im = new(32, 32, "s2")
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 31, 31], fill=C["s2"], outline=C["s0"])
    for y in range(4, 32, 6):
        d.line([(1, y), (30, y)], fill=C["s1"])
        for x in range((y // 6 % 2) * 5 + 2, 31, 10):
            d.line([(x, y - 5), (x, y)], fill=C["s1"])
    d.rectangle([0, 0, 31, 3], fill=C["k2"], outline=C["k0"])
    d.rectangle([7, 13, 24, 31], fill=C["void"], outline=C["s0"])
    d.pieslice([7, 7, 24, 19], 180, 360, fill=C["void"])
    d.polygon([(10, 31), (13, 21), (16, 26), (19, 19), (22, 31)], fill=C["fire"])
    d.polygon([(13, 31), (15, 25), (17, 28), (19, 24), (20, 31)], fill=C["fire_lt"])
    d.rectangle([9, 29, 23, 31], fill=C["k1"])
    return im


def plant_art():
    im = floor_art(9)
    d = ImageDraw.Draw(im)
    d.rectangle([4, 10, 11, 15], fill=C["terra"], outline=C["terra_dk"])
    leafy_blob(im, 8, 6, 6, 5, 3)
    return im


def mat_art():
    """The way out: an opening in the front wall with a woven mat."""
    im = new(16, 16, "k1")
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, 15, 15], fill=C["d1"])
    d.rectangle([1, 0, 14, 12], fill=C["d2"], outline=C["d0"])
    for y in range(2, 11, 2):
        d.line([(3, y), (12, y)], fill=C["d3"])
    d.rectangle([0, 13, 15, 15], fill=C["void"])
    return im


def void_art():
    return new(16, 16, "void")


# ---------------------------------------------------------------------------
# The sheet: standard ids keep their place, the rest goes to ids 2560..4095
# ---------------------------------------------------------------------------

# Standard Graal ids with a meaning in the client (see gen_default_assets.G).
STD = dict(GRASS=0, GRASS2=2047, BUSH=2, STONE=34, SIGN=512, POT=684, BLACK_STONE=990, TALL_GRASS=420,
           BUSH_LEFT=677, GRASS_LEFT=679, SIGN_LEFT=1802, POT_LEFT=1770, STONE_LEFT=1834,
           CHEST=1784, CHEST_OPEN=829, DEEP_WATER=322, SHALLOW=398)

FIRST_BLOCK, LAST_BLOCK = 5, 7  # ids 2560 .. 4095: 16 x 32 cells each


class Sheet:
    def __init__(self, base_types):
        self.types = list(base_types)
        for tid in range(FIRST_BLOCK * 512, (LAST_BLOCK + 1) * 512):
            self.types[tid] = WALL
        self.art = {}       # id -> 16x16 RGB tile
        self.names = {}     # name -> (id, w, h)
        self.used = set()   # (block, col, row)
        self.ledges = [[], [], [], []]

    def _alloc(self, w, h):
        for blk in range(FIRST_BLOCK, LAST_BLOCK + 1):
            for row in range(32 - h + 1):
                for col in range(16 - w + 1):
                    if all((blk, col + dx, row + dy) not in self.used for dx in range(w) for dy in range(h)):
                        for dx in range(w):
                            for dy in range(h):
                                self.used.add((blk, col + dx, row + dy))
                        return blk * 512 + row * 16 + col
        raise RuntimeError("tutorial sheet is full")

    def add(self, name, im, ty=WALL, at=None):
        """Register an image of w x h tiles. `ty` is one type, or a list of
        rows of types. `at` puts it on a standard id instead."""
        w, h = im.width // 16, im.height // 16
        tid = at if at is not None else self._alloc(w, h)
        for dy in range(h):
            for dx in range(w):
                t = tid + dx + 16 * dy
                self.art[t] = im.crop((dx * 16, dy * 16, dx * 16 + 16, dy * 16 + 16))
                if ty is not None:
                    self.types[t] = ty[dy][dx] if isinstance(ty, list) else ty
        self.names[name] = (tid, w, h)
        return tid

    def id(self, name, dx=0, dy=0):
        tid, w, h = self.names[name]
        assert 0 <= dx < w and 0 <= dy < h, (name, dx, dy)
        return tid + dx + 16 * dy


def build_tutorial_sheet(base):
    s = Sheet(base.type_table())
    # standard ids: same types as the Graal table (None keeps them)
    s.add("grass", grass_tile(1, tufts=2), None, at=STD["GRASS"])
    s.add("grass_chest", grass_tile(2, tufts=1), None, at=STD["GRASS2"])
    s.add("bush", bush_art(11), None, at=STD["BUSH"])
    s.add("stone", stone_art(12, False), None, at=STD["STONE"])
    s.add("black_stone", stone_art(13, True), None, at=STD["BLACK_STONE"])
    s.add("pot", pot_art(14), None, at=STD["POT"])
    s.add("sign", sign_art(15), None, at=STD["SIGN"])
    s.add("tall_grass", tall_grass_art(16), None, at=STD["TALL_GRASS"])
    s.add("bush_left", patch_art(17, "bush"), None, at=STD["BUSH_LEFT"])
    s.add("grass_left", patch_art(18, "grass"), None, at=STD["GRASS_LEFT"])
    s.add("sign_left", patch_art(19, "sign"), None, at=STD["SIGN_LEFT"])
    s.add("pot_left", patch_art(20, "pot"), None, at=STD["POT_LEFT"])
    s.add("stone_left", patch_art(21, "stone"), None, at=STD["STONE_LEFT"])
    s.add("chest", chest_art(False, 22), None, at=STD["CHEST"])
    s.add("chest_open", chest_art(True, 23), None, at=STD["CHEST_OPEN"])
    s.add("deep", water_full(24), None, at=STD["DEEP_WATER"])
    s.add("shallow", shallow_art(25), None, at=STD["SHALLOW"])

    # grass variants
    for i, (tufts, flowers, pebbles) in enumerate(((1, 0, 0), (3, 0, 0), (2, 0, 0), (0, 2, 0), (1, 1, 0), (1, 0, 1), (0, 3, 0), (2, 0, 0))):
        s.add(f"grass{i}", grass_tile(100 + i, tufts, flowers, pebbles), OPEN)
    # blended terrain
    for m in range(16):
        s.add(f"path{m}", path_tile(m, 200 + m), OPEN)
    s.add("path15b", dirt_full(231), OPEN)
    s.add("path15c", dirt_full(232), OPEN)
    for m in range(16):
        s.add(f"shore{m}", shore_tile(m, 300 + m), WATER if m == 15 else OPEN)
    s.add("shore15b", water_full(331), WATER)
    s.add("lily", lily_art(332), WATER)
    s.add("lily2", lily_art(333), WATER)
    s.add("reeds", reeds_art(334), OPEN)
    s.add("reeds2", reeds_art(335), OPEN)
    s.add("rock_small", rock_small_art(336), OPEN)
    s.add("mushrooms", mushroom_art(337), OPEN)

    # scenery
    s.add("tree", tree_art(400), WALL)
    s.add("tree2", tree_art(401), WALL)
    s.add("pine", pine_art(402), WALL)
    s.add("flowerbed", flowerbed_art(403), THROW_THROUGH)
    s.add("fence_h", fence_art("h"), THROW_THROUGH)
    s.add("fence_v", fence_art("v"), THROW_THROUGH)
    s.add("fence_post", fence_art("post"), THROW_THROUGH)
    door = [[WALL] * 8 for _ in range(7)]
    for y in (5, 6):
        for x in (3, 4):
            door[y][x] = OPEN
    s.add("cottage", cottage_art(404), door)
    for name, seed in (("cliff_mid", 410), ("cliff_mid2", 411), ("cliff_mid3", 415)):
        tid = s.add(name, cliff_art("mid", seed), [[WALL], [WALL], [OPEN]])
        s.ledges[2] += [tid, tid + 16]
    s.add("stairs", stairs_art(414), OPEN)
    s.add("push_block", block_art("push"), WALL)
    s.add("pull_block", block_art("pull"), WALL)
    s.add("bridge_deck", bridge_art("deck", 420), OPEN)
    s.add("bridge_rail_w", bridge_art("rail_w", 421), THROW_THROUGH)
    s.add("bridge_rail_e", bridge_art("rail_e", 422), THROW_THROUGH)
    s.add("warp", warp_art(430), OPEN)

    # the cottage's room
    for i in range(3):
        s.add(f"floor{i}", floor_art(500 + i), OPEN)
    for kind in ("plain", "window"):
        s.add(f"wall_{kind}", _stack(wall_back_art(0, kind, 510), wall_back_art(1, kind, 511)), WALL)
    for kind in ("w", "e", "top", "bottom", "nw", "ne", "sw", "se"):
        s.add(f"rim_{kind}", wall_side_art(kind), WALL)
    s.add("void", void_art(), WALL)
    s.add("bed", bed_art(), [[BED_TOP, BED_TOP], [BED_BOTTOM, BED_BOTTOM], [BED_BOTTOM, BED_BOTTOM]])
    s.add("table", table_art(), THROW_THROUGH)
    s.add("chair", chair_art(), CHAIR)
    s.add("rug", rug_art(4, 3), OPEN)
    s.add("shelf", shelf_art(), WALL)
    s.add("hearth", hearth_art(), WALL)
    s.add("plant", plant_art(), WALL)
    s.add("mat", mat_art(), OPEN)
    s.add("floor_chest", floor_art(520), OPEN)
    return s


def _stack(a, b):
    im = Image.new("RGB", (a.width, a.height + b.height))
    im.paste(a, (0, 0))
    im.paste(b, (0, a.height))
    return im


def sheet_image(base, s, rng):
    """The 2048 x 512 sheet: the default sheet's art for every id, then ours.
    Returns a palette image and its palette (Pal of the base module)."""
    from_default, pal = base.build_sheet(rng)
    from_default.putpalette(pal.flat(256))
    rgb = from_default.convert("RGB")
    for tid, im in s.art.items():
        cx, cy = base.tile_cell(tid)
        rgb.paste(im, (cx * 16, cy * 16))
    colours = list(pal.rgb)
    for v in C.values():
        if v not in colours:
            colours.append(v)
    used = {c for _, c in rgb.getcolors(1 << 16)}
    assert used <= set(colours), sorted(used - set(colours))[:5]
    assert len(colours) <= 256, len(colours)
    palimg = Image.new("P", (1, 1))
    flat = [v for c in colours for v in c] + [0] * (768 - 3 * len(colours))
    palimg.putpalette(flat)
    out = rgb.quantize(palette=palimg, dither=Image.Dither.NONE)
    return out, colours


# ---------------------------------------------------------------------------
# arrays.dat of the tutorial sheet
# ---------------------------------------------------------------------------


def _flat(ids):
    ids = list(ids)
    return struct.pack(">I", len(ids)) + b"".join(struct.pack(">I", i) for i in ids)


def _nested(items):
    return struct.pack(">I", (1 << 16) | len(items)) + b"".join(items)


def _obj(tid):
    """A 2 x 2 object, [column][row], as arrays 9 to 12 and 16 hold it."""
    return _nested([_flat([tid, tid + 16]), _flat([tid + 1, tid + 17])])


def arrays_dat(s):
    t = s.types
    passable = [i for i, v in enumerate(t) if v in (OPEN, CHAIR, BED_TOP, BED_BOTTOM)]
    liquid = [i for i, v in enumerate(t) if v == WATER]
    through = [i for i, v in enumerate(t) if v == THROW_THROUGH]
    out = [
        _flat(passable), _flat(liquid), _flat(through), _flat([]),
        _flat([i for i, v in enumerate(t) if v == CHAIR]),
        _flat([i for i, v in enumerate(t) if v == BED_TOP]),
        _flat([i for i, v in enumerate(t) if v == BED_BOTTOM]),
        _flat([]),                                           # 7: not used here
        _nested([_flat(sorted(l)) for l in s.ledges]),         # 8: ledges up, left, down, right
        _nested([_obj(STD["BUSH"]), _obj(STD["TALL_GRASS"])]),  # 9: cut
        _nested([_obj(STD["BUSH_LEFT"]), _obj(STD["GRASS_LEFT"])]),  # 10: what a cut leaves
        _nested([_obj(STD["BUSH"]), _obj(STD["SIGN"]), _obj(STD["POT"]), _obj(STD["STONE"]), _obj(STD["BLACK_STONE"])]),  # 11: lift
        _nested([_obj(STD["BUSH_LEFT"]), _obj(STD["SIGN_LEFT"]), _obj(STD["POT_LEFT"]), _obj(STD["STONE_LEFT"]), _obj(STD["STONE_LEFT"])]),  # 12
        _flat([]), _flat([]), _flat([]),                        # 13 to 15: not used here
        _nested([_obj(s.id("push_block")), _obj(s.id("pull_block"))]),  # 16: push, pull
    ]
    return b"".join(out)


# ---------------------------------------------------------------------------
# Building levels
# ---------------------------------------------------------------------------

B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
GRASS_WEIGHTS = [("grass0", 40), ("grass1", 12), ("grass2", 12), ("grass3", 1), ("grass4", 2), ("grass5", 3), ("grass6", 0), ("grass7", 10)]


class Lvl:
    def __init__(self, s, seed, indoor=False):
        self.s = s
        self.r = random.Random(seed)
        self.indoor = indoor
        self.path = [[False] * 65 for _ in range(65)]
        self.water = [[False] * 65 for _ in range(65)]
        self.tiles = [None] * 4096   # explicit tile ids
        self.lines = []              # LINK, CHEST, SIGN records

    # terrain on tile corners -------------------------------------------
    def path_rect(self, x0, y0, x1, y1):
        """Dirt over tiles x0..x1, y0..y1 (the edges blend into the tiles
        around)."""
        for vy in range(y0, y1 + 2):
            for vx in range(x0, x1 + 2):
                if 0 <= vx <= 64 and 0 <= vy <= 64:
                    self.path[vy][vx] = True

    def water_rect(self, x0, y0, x1, y1, on=True):
        for vy in range(y0, y1 + 2):
            for vx in range(x0, x1 + 2):
                if 0 <= vx <= 64 and 0 <= vy <= 64:
                    self.water[vy][vx] = on

    def water_blob(self, cx, cy, rx, ry, on=True, jag=0.12):
        for vy in range(65):
            for vx in range(65):
                a = math.atan2(vy - cy, vx - cx)
                e = 1 + jag * math.sin(a * 3 + cx) + jag * 0.6 * math.sin(a * 5 + cy)
                if ((vx - cx) / rx) ** 2 + ((vy - cy) / ry) ** 2 <= e * e:
                    self.water[vy][vx] = on

    # objects ------------------------------------------------------------
    def put(self, name, x, y):
        tid, w, h = self.s.names[name]
        for dy in range(h):
            for dx in range(w):
                if 0 <= x + dx < 64 and 0 <= y + dy < 64:
                    self.tiles[(y + dy) * 64 + x + dx] = tid + dx + 16 * dy

    def tile(self, name, x, y):
        self.tiles[y * 64 + x] = self.s.id(name)

    def free(self, x, y, w=1, h=1):
        return all(0 <= x + dx < 64 and 0 <= y + dy < 64 and self.tiles[(y + dy) * 64 + x + dx] is None
                   and not self._terrain(x + dx, y + dy) for dx in range(w) for dy in range(h))

    def _mask(self, grid, x, y):
        return grid[y][x] | grid[y][x + 1] << 1 | grid[y + 1][x] << 2 | grid[y + 1][x + 1] << 3

    def _terrain(self, x, y):
        return self._mask(self.water, x, y) or self._mask(self.path, x, y)

    def link(self, dest, x, y, w, h, nx, ny):
        self.lines.append(f"LINK {dest} {x} {y} {w} {h} {nx} {ny}")

    def chest(self, x, y, item):
        if self.indoor:
            for dx in range(2):
                for dy in range(2):
                    self.tile("floor_chest", x + dx, y + dy)
        else:
            for dx in range(2):
                for dy in range(2):
                    self.tiles[(y + dy) * 64 + x + dx] = STD["GRASS2"]
        self.lines.append(f"CHEST {x} {y} {item} -1")

    def sign(self, x, y, text, art=True):
        """A sign post at (x, y) (2 x 2); its record is the lower row, read by
        walking up into it. With art=False only the record (a note on a wall)."""
        if art:
            self.put("sign", x, y)
            y += 1
        self.lines += [f"SIGN {x} {y}"] + text.strip().split("\n") + ["SIGNEND"]

    def trees_row(self, x0, x1, y, kinds=("tree", "tree2")):
        x = x0
        while x <= x1 - 3:
            self.put(kinds[self.r.randrange(len(kinds))], x, y)
            x += 4

    def scatter(self, name, n, area=(2, 2, 61, 61), tries=400):
        w, h = self.s.names[name][1:]
        x0, y0, x1, y1 = area
        done = 0
        for _ in range(tries):
            if done >= n:
                break
            x, y = self.r.randrange(x0, x1 - w + 2), self.r.randrange(y0, y1 - h + 2)
            if self.free(x - 1, y - 1, w + 2, h + 2):
                self.put(name, x, y)
                done += 1

    # output -------------------------------------------------------------
    def board(self):
        out = []
        names = [n for n, _ in GRASS_WEIGHTS]
        weights = [w for _, w in GRASS_WEIGHTS]
        rr = random.Random(self.r.random())
        for y in range(64):
            for x in range(64):
                t = self.tiles[y * 64 + x]
                if t is None:
                    if self.indoor:
                        t = self.s.id("void")
                    else:
                        wm = self._mask(self.water, x, y)
                        pm = self._mask(self.path, x, y)
                        if wm:
                            t = self.s.id(f"shore{wm}")
                            if wm == 15:
                                k = rr.random()
                                t = self.s.id("shore15b") if k < 0.3 else STD["DEEP_WATER"] if k < 0.5 else t
                        elif pm:
                            t = self.s.id(f"path{pm}")
                            if pm == 15:
                                k = rr.random()
                                t = self.s.id("path15b") if k < 0.25 else self.s.id("path15c") if k < 0.5 else t
                        else:
                            t = self.s.id(rr.choices(names, weights)[0])
                out.append(t)
        return out

    def text(self):
        b = self.board()
        rows = []
        for y in range(64):
            rows.append(f"BOARD 0 {y} 64 0 " + "".join(B64[(t >> 6) & 63] + B64[t & 63] for t in b[y * 64:(y + 1) * 64]))
        return "\n".join(["GLEVNW01"] + rows + self.lines) + "\n"


def lv(n):
    return f"{PREFIX}{n}.nw"


HOUSE = f"{PREFIX}_house.nw"


def border(L, gaps=()):
    """Trees round the edge. `gaps` are (side, a, b) openings: side 'n', 's'
    on columns a..b, 'w', 'e' on rows a..b."""
    def open_(side, a0, a1):
        return any(g[0] == side and not (a1 < g[1] or a0 > g[2]) for g in gaps)
    for x in range(0, 64, 4):
        if not open_("n", x, x + 3):
            L.put(L.r.choice(("tree", "tree2")), x, 0)
        if not open_("s", x, x + 3):
            L.put(L.r.choice(("tree", "tree2")), x, 60)
    for y in range(4, 60, 4):
        if not open_("w", y, y + 3):
            L.put(L.r.choice(("tree", "tree2")), 0, y)
        if not open_("e", y, y + 3):
            L.put(L.r.choice(("tree", "tree2")), 60, y)


def decorate(L, area=(4, 4, 59, 59), trees=6, pines=4, flowers=8, rocks=5, shrooms=3):
    L.scatter("tree", trees // 2, area)
    L.scatter("tree2", trees - trees // 2, area)
    L.scatter("pine", pines, area)
    for _ in range(flowers):
        # a little patch of flowers: a few flowery tiles together
        x, y = L.r.randrange(area[0], area[2] - 3), L.r.randrange(area[1], area[3] - 3)
        for dx, dy in ((0, 0), (1, 0), (0, 1), (2, 1), (1, 2)):
            if L.free(x + dx, y + dy) and L.r.random() < 0.8:
                L.tile(L.r.choice(("grass6", "grass3", "grass4")), x + dx, y + dy)
    L.scatter("rock_small", rocks, area)
    L.scatter("mushrooms", shrooms, area)


def level_meadow(s):
    """1: the meadow. Walking, reading signs, the cottage door, the way east."""
    L = Lvl(s, TUT_SEED + 1)
    border(L, gaps=[("e", 24, 39)])
    L.path_rect(10, 31, 63, 32)          # the main path, east to the garden
    L.path_rect(34, 13, 35, 30)          # from the cottage door
    L.put("cottage", 31, 6)
    L.link(HOUSE, 34, 11, 2, 2, 30.5, 30)
    L.link(lv(2), 62, 24, 2, 16, 3, "playery")
    L.sign(30, 26, """
Welcome to Seihai!
Walk with the arrow keys. You read a
sign by walking up into it, like this.
Follow the path up to your cottage:
walk into its door to go inside.
There is a chest in there for you.
Then follow the path east.
""")
    L.sign(29, 14, """
Your cottage. Doors are links: walk
into one and you are somewhere else.
""")
    L.sign(52, 28, """
East: the garden. Your sword (S) and
your hands (A) get some practice there.
""")
    # a fenced flower garden
    for x in range(15, 25):
        L.tile("fence_h", x, 15)
        if x not in (19, 20):
            L.tile("fence_h", x, 23)
    for y in range(16, 23):
        L.tile("fence_v", 14, y)
        L.tile("fence_v", 25, y)
    for (x, y) in ((14, 15), (25, 15), (14, 23), (25, 23)):
        L.tile("fence_post", x, y)
    for (x, y) in ((16, 17), (22, 17)):
        L.put("flowerbed", x, y)
    L.put("tall_grass", 19, 18)
    for (x, y) in ((40, 10), (42, 10), (40, 12)):
        L.put("pot", x, y)
    # a pond with lilies
    L.water_blob(15, 46, 7, 5)
    for (x, y) in ((12, 45), (16, 47), (18, 44)):
        L.tile("lily" if x % 2 else "lily2", x, y)
    for (x, y) in ((7, 40), (22, 43), (9, 51)):
        L.tile("reeds", x, y)
        L.tile("reeds2", x + 1, y)
    for (x, y) in ((44, 20), (8, 26), (48, 44), (24, 52)):
        L.put("bush", x, y)
    decorate(L, trees=7, pines=5)
    return L


def level_house(s):
    """The cottage's room: a chest with the glove, and a note."""
    L = Lvl(s, TUT_SEED + 2, indoor=True)
    x0, x1, y0, y1 = 22, 41, 20, 33
    for x in range(x0, x1 + 1):
        L.tile("rim_top", x, y0 - 1)
        L.tile("rim_bottom" if x not in (31, 32) else "mat", x, y1 + 1)
        L.put("wall_window" if x in (25, 38) else "wall_plain", x, y0)
    for y in range(y0, y1 + 1):
        L.tile("rim_w", x0 - 1, y)
        L.tile("rim_e", x1 + 1, y)
    for (k, x, y) in (("nw", x0 - 1, y0 - 1), ("ne", x1 + 1, y0 - 1), ("sw", x0 - 1, y1 + 1), ("se", x1 + 1, y1 + 1)):
        L.tile(f"rim_{k}", x, y)
    for y in range(y0 + 2, y1 + 1):
        for x in range(x0, x1 + 1):
            L.tile(("floor0", "floor1", "floor2")[(x * 7 + y * 3) % 3], x, y)
    L.put("shelf", 27, 20)
    L.put("hearth", 35, 20)
    L.put("wall_note", 31, 20)
    L.put("bed", 23, 22)
    L.put("rug", 30, 28)
    L.put("table", 30, 24)
    L.tile("chair", 29, 24)
    L.tile("chair", 32, 24)
    L.tile("plant", 41, 22)
    L.tile("plant", 22, 33)
    L.chest(24, 27, "glove1")
    L.sign(31, 21, """
Welcome home! The chest next to the
bed is yours. Stand below it, face it
and press A to open it.
The glove inside lets you lift grey
stones. You will need it in the garden.
Leave by the mat at the door.
""", art=False)
    L.link(lv(1), 31, 34, 2, 1, 33.5, 13.5)
    return L


def level_garden(s):
    """2: the garden. Cutting, lifting and throwing; the grey stone."""
    L = Lvl(s, TUT_SEED + 3)
    border(L, gaps=[("w", 24, 39), ("s", 28, 35)])
    L.link(lv(1), 0, 24, 2, 16, 58.5, "playery")
    L.link(lv(3), 28, 62, 8, 2, "playerx", 3)
    L.path_rect(0, 31, 31, 32)
    L.path_rect(30, 33, 31, 63)
    # the arrival yard, closed by fences and a hedge of bushes
    for x in range(4, 16):
        L.tile("fence_h", x, 23)
        L.tile("fence_h", x, 40)
    for y in range(24, 40, 2):
        L.put("bush", 14, y)
    L.sign(6, 26, """
Bushes block the way. Press S to swing
your sword and cut through them.
Or face a bush and press A to lift it,
then press A again to throw it.
""")
    # the field: tall grass to cut and pots to throw
    for (x, y) in ((20, 26), (22, 26), (24, 28), (20, 34), (22, 36), (36, 26), (38, 26), (38, 28), (40, 34)):
        L.put("tall_grass", x, y)
    for (x, y) in ((44, 30), (46, 30), (44, 32), (48, 28)):
        L.put("pot", x, y)
    L.sign(44, 26, """
Pots can be lifted too: face one and
press A. Press A or S to throw it. It
breaks where it lands.
""")
    # the wall of black stones with one grey stone in the path
    for x in range(4, 60, 2):
        L.put("stone" if x == 30 else "black_stone", x, 46)
    L.sign(26, 42, """
Grey stones are heavy. With the glove
from the cottage chest you can lift
them: face the stone and press A.
Black stones are too heavy for now.
""")
    L.sign(34, 54, """
South: the pond.
""")
    for (x, y) in ((50, 12), (12, 12), (52, 52), (10, 52)):
        L.put("flowerbed", x, y)
    decorate(L, trees=6, pines=4)
    return L


def level_pond(s):
    """3: the pond. Swimming, the island chest, the bridge, the ford."""
    L = Lvl(s, TUT_SEED + 4)
    border(L, gaps=[("n", 24, 39), ("e", 40, 55)])
    L.link(lv(2), 24, 0, 16, 2, "playerx", 57.5)
    L.link(lv(4), 62, 40, 2, 16, 3, 12)
    L.path_rect(32, 0, 33, 29)
    L.water_rect(0, 31, 63, 36)
    L.water_blob(14, 34, 12, 9)
    L.water_blob(14, 34.5, 3.2, 2.6, on=False, jag=0.05)  # the island
    L.chest(13, 33, "redrupee")
    # the bridge (x 30..35) and the ford (x 48..51) where there is water
    for y in range(64):
        if any(L._mask(L.water, x, y) for x in range(30, 36)):
            L.tile("bridge_rail_w", 30, y)
            for x in range(31, 35):
                L.tile("bridge_deck", x, y)
            L.tile("bridge_rail_e", 35, y)
        for x in range(48, 52):
            if L._mask(L.water, x, y):
                L.tile("shallow", x, y)
    L.path_rect(32, 39, 33, 47)
    L.path_rect(34, 46, 63, 47)
    L.path_rect(34, 24, 48, 25)      # to the ford
    L.path_rect(48, 24, 49, 29)
    L.path_rect(48, 39, 49, 45)
    L.sign(25, 22, """
Deep water: walk in and you swim.
Swim out to the island in the lake:
there is a chest on it.
The bridge keeps your feet dry, and so
does the shallow ford further east.
""")
    for (x, y) in ((10, 26), (20, 41), (4, 30)):
        L.tile("reeds", x, y)
    for (x, y) in ((8, 36), (20, 31), (40, 33), (56, 34)):
        L.tile("lily", x, y)
    L.sign(54, 42, """
East: the rock yard.
""")
    decorate(L, area=(4, 4, 59, 24), trees=5, pines=3)
    decorate(L, area=(4, 44, 59, 59), trees=4, pines=3)
    return L


def level_rocks(s):
    """4: the rock yard. The ledge jump, pushing, pulling, the way home."""
    L = Lvl(s, TUT_SEED + 5)
    for x in range(0, 64, 4):
        L.put(L.r.choice(("tree", "tree2")), x, 0)
        L.put(L.r.choice(("tree", "tree2")), x, 60)
    for y in range(24, 60, 4):
        L.put("tree", 0, y)
    for y in range(4, 60, 4):
        L.put("tree2", 60, y)
    L.link(lv(3), 0, 4, 2, 17, 58.5, 47)
    # the terrace edge, all the way across; stairs back up at x 54..55
    for x in range(0, 64):
        if x in (54, 55):
            continue
        L.put(("cliff_mid", "cliff_mid2", "cliff_mid3")[L.r.randrange(3)], x, 21)
    L.put("stairs", 54, 21)
    L.path_rect(0, 11, 20, 12)
    L.sign(14, 16, """
This edge is a drop. Keep walking down
against it and you jump off.
The stairs to the east lead back up.
""")
    # the wall with the push block (x 22..23) and the pull block (x 40..41)
    for x in range(4, 60, 2):
        if x == 22:
            L.put("push_block", x, 36)
        else:
            L.put("black_stone", x, 36)
    L.sign(16, 26, """
The block with arrows can be pushed:
walk against it and keep walking.
""")
    for x in range(4, 60, 2):
        if x == 40:
            L.put("pull_block", x, 48)
        else:
            L.put("black_stone", x, 48)
    L.sign(34, 40, """
The block with a ring can be pulled:
stand above it, face it, hold A and
press up to walk backwards with it.
""")
    L.put("warp", 30, 56)
    L.link(lv(1), 30, 56, 2, 2, 30, 33)
    L.sign(25, 54, """
Well done, that is the whole tutorial!
Step onto the glowing ring to go back
to the meadow.
""")
    L.chest(36, 54, "bluerupee")
    for (x, y) in ((8, 6), (26, 8), (44, 12)):
        L.put("pot", x, y)
    decorate(L, area=(4, 4, 59, 19), trees=3, pines=3, rocks=6)
    decorate(L, area=(4, 24, 59, 34), trees=2, pines=2, rocks=6)
    decorate(L, area=(4, 39, 59, 46), trees=1, pines=2, rocks=4)
    return L


LEVELS = (lambda s: (lv(1), level_meadow(s)), lambda s: (HOUSE, level_house(s)),
          lambda s: (lv(2), level_garden(s)), lambda s: (lv(3), level_pond(s)), lambda s: (lv(4), level_rocks(s)))


def wall_note_art():
    im = _stack(wall_back_art(0, "plain", 512), wall_back_art(1, "plain", 513))
    d = ImageDraw.Draw(im)
    d.rectangle([3, 6, 12, 18], fill=C["p3"], outline=C["p0"])
    for y in (9, 11, 13, 15):
        d.line([(5, y), (10 - (y % 3), y)], fill=C["s1"])
    px(im, 7, 7, "red")
    return im


def write_tutorial(out, base):
    """Write tutorial/pics1.png, tutorial/arrays.dat and the levels."""
    s = build_tutorial_sheet(base)
    s.add("wall_note", wall_note_art(), WALL)
    os.makedirs(os.path.join(out, "tutorial"), exist_ok=True)
    os.makedirs(os.path.join(out, "levels"), exist_ok=True)
    im, colours = sheet_image(base, s, random.Random(base.SEED))
    im.save(os.path.join(out, "tutorial", "pics1.png"), "PNG", optimize=True, bits=8, compress_level=9)
    open(os.path.join(out, "tutorial", "arrays.dat"), "wb").write(arrays_dat(s))
    for make in LEVELS:
        name, L = make(s)
        open(os.path.join(out, "levels", name), "w", newline="\n").write(L.text())
    return s
