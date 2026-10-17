"""Render concept art for the README. Pure Pillow, no external assets.

Run:  python tools/render_concept.py
Output: docs/img/*.png
These are mock-ups of the intended look, not screenshots of the game.
"""
import math
import os
import random

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

OUT = os.path.join(os.path.dirname(__file__), "..", "docs", "img")
os.makedirs(OUT, exist_ok=True)

T, W, H = 16, 26, 15  # tile size, map size in tiles
PW, PH = W * T, H * T  # 416 x 240 base pixels
S = 3  # upscale factor

FONT_DIR = r"C:\Windows\Fonts"


def font(size, bold=False):
    for name in (("consolab.ttf" if bold else "consola.ttf"), "arial.ttf"):
        try:
            return ImageFont.truetype(os.path.join(FONT_DIR, name), size)
        except OSError:
            pass
    return ImageFont.load_default()


# ---------- palette ----------
GRASS = (84, 128, 66)
PATH = (150, 120, 84)
COBBLE = (128, 128, 134)
OUTLINE = (28, 22, 30)
TIER_COL = {0: (150, 150, 150), 1: (90, 170, 255), 2: (255, 200, 60)}


def rect(d, x0, y0, x1, y1, fill, outline=None):
    d.rectangle([x0, y0, x1, y1], fill=fill, outline=outline)


# ---------- ground ----------
def ground(rng):
    im = Image.new("RGBA", (PW, PH), GRASS + (255,))
    d = ImageDraw.Draw(im)
    for _ in range(1400):
        x, y = rng.randrange(PW), rng.randrange(PH)
        c = rng.choice([(72, 114, 58), (98, 142, 76), (78, 120, 62)])
        d.point((x, y), fill=c)
    # roads
    rect(d, 0, 7 * T, PW, 8 * T + 15, PATH)
    rect(d, 13 * T, 4 * T, 14 * T + 15, PH, PATH)
    # plaza cobble
    for ty in range(6, 10):
        for tx in range(11, 17):
            c = rng.choice([(122, 122, 128), (134, 134, 140), (118, 118, 124)])
            rect(d, tx * T, ty * T, tx * T + 15, ty * T + 15, c, (100, 100, 106))
    for _ in range(500):
        x, y = rng.randrange(PW), rng.randrange(PH)
        px = im.getpixel((x, y))
        if px[:3] == PATH:
            d.point((x, y), fill=rng.choice([(140, 110, 76), (160, 130, 92)]))
    return im, d


# ---------- structures ----------
def house(d, x, y, w, h, wall, roof):
    rh = int(h * 0.45)
    d.polygon([(x - 2, y + rh), (x + 4, y), (x + w - 4, y), (x + w + 2, y + rh)], fill=roof, outline=OUTLINE)
    for yy in range(y + 3, y + rh, 3):
        d.line([(x + 1, yy), (x + w - 1, yy)], fill=tuple(max(0, c - 28) for c in roof))
    rect(d, x, y + rh, x + w, y + h, wall, OUTLINE)
    rect(d, x + w // 2 - 3, y + h - 9, x + w // 2 + 3, y + h, (84, 52, 30), OUTLINE)
    wins = []
    for fx in (0.18, 0.70):
        wx = x + int(w * fx)
        wy = y + rh + 4
        rect(d, wx, wy, wx + 4, wy + 4, (110, 140, 160), OUTLINE)
        wins.append((wx, wy, wx + 4, wy + 4))
    rect(d, x + w - 9, y - 4, x + w - 5, y + 4, (90, 80, 80), OUTLINE)
    return wins


def church(d, x, y):
    w, h = 80, 74
    wall = (150, 148, 156)
    rect(d, x, y + 26, x + w, y + h, wall, OUTLINE)
    d.polygon([(x - 3, y + 26), (x + 12, y + 8), (x + w - 12, y + 8), (x + w + 3, y + 26)], fill=(78, 70, 98), outline=OUTLINE)
    tx = x + w // 2 - 11
    rect(d, tx, y - 6, tx + 22, y + 30, (158, 156, 164), OUTLINE)
    d.polygon([(tx - 2, y - 6), (tx + 11, y - 34), (tx + 24, y - 6)], fill=(70, 62, 92), outline=OUTLINE)
    d.line([(tx + 11, y - 46), (tx + 11, y - 34)], fill=(220, 210, 120), width=2)
    d.line([(tx + 7, y - 42), (tx + 15, y - 42)], fill=(220, 210, 120), width=2)
    rect(d, tx + 7, y + 2, tx + 15, y + 14, (30, 24, 40), OUTLINE)  # bell opening
    rect(d, x + w // 2 - 6, y + h - 18, x + w // 2 + 6, y + h, (70, 44, 28), OUTLINE)
    wins = []
    for wx in (x + 10, x + 24, x + w - 18, x + w - 32):
        rect(d, wx, y + 40, wx + 6, y + 54, (140, 120, 200), OUTLINE)
        wins.append((wx, y + 40, wx + 6, y + 54))
    return wins


def tavern(d, x, y):
    w = house(d, x, y, 76, 56, (146, 106, 70), (112, 52, 44))
    rect(d, x + 58, y + 28, x + 70, y + 32, (210, 170, 90), OUTLINE)  # sign
    d.line([(x + 64, y + 22), (x + 64, y + 28)], fill=OUTLINE)
    return w


def tree(d, x, y, rng):
    rect(d, x - 1, y - 4, x + 1, y, (80, 54, 34))
    for dx, dy, r in ((0, -12, 8), (-6, -7, 6), (6, -7, 6)):
        d.ellipse([x + dx - r, y + dy - r, x + dx + r, y + dy + r], fill=(46, 92, 52), outline=(30, 64, 38))
    d.ellipse([x - 4, y - 18, x + 1, y - 13], fill=(70, 120, 70))


def tombstone(d, x, y):
    rect(d, x - 3, y - 8, x + 3, y, (150, 150, 158), OUTLINE)
    d.pieslice([x - 3, y - 12, x + 3, y - 4], 180, 360, fill=(150, 150, 158), outline=OUTLINE)
    d.line([(x, y - 8), (x, y - 4)], fill=(90, 90, 98))


def well(d, x, y):
    d.ellipse([x - 9, y - 6, x + 9, y + 6], fill=(120, 120, 128), outline=OUTLINE)
    d.ellipse([x - 6, y - 4, x + 6, y + 3], fill=(24, 40, 70))
    rect(d, x - 9, y - 16, x - 7, y - 3, (84, 52, 30))
    rect(d, x + 7, y - 16, x + 9, y - 3, (84, 52, 30))
    d.polygon([(x - 11, y - 16), (x, y - 24), (x + 11, y - 16)], fill=(112, 52, 44), outline=OUTLINE)


def shrine(d, cx, cy, doom=0.0):
    """Stone circle + obelisk with a glyph that glows redder as doom approaches."""
    d.ellipse([cx - 34, cy - 18, cx + 34, cy + 18], fill=(70, 50, 86))
    d.ellipse([cx - 28, cy - 14, cx + 28, cy + 14], outline=(150, 100, 190))
    for i in range(8):
        a = i * math.tau / 8
        sx, sy = cx + int(math.cos(a) * 31), cy + int(math.sin(a) * 15)
        rect(d, sx - 2, sy - 9, sx + 2, sy, (112, 110, 120), OUTLINE)
    rect(d, cx - 5, cy - 34, cx + 5, cy + 2, (36, 30, 46), OUTLINE)
    d.polygon([(cx - 5, cy - 34), (cx, cy - 44), (cx + 5, cy - 34)], fill=(36, 30, 46), outline=OUTLINE)
    g = (int(160 + 95 * doom), int(60 - 40 * doom), int(200 - 150 * doom))
    d.line([(cx, cy - 30), (cx, cy - 8)], fill=g, width=2)
    d.line([(cx - 3, cy - 22), (cx + 3, cy - 22)], fill=g, width=1)
    return (cx, cy - 20), g


# ---------- characters ----------
def agent(d, cx, fy, robe, hair=(70, 50, 36), hood=False, tier=None, kneel=False, skin=(236, 200, 160)):
    if tier is not None:
        d.ellipse([cx - 3, fy - 1, cx + 3, fy + 1], fill=TIER_COL[tier])
    else:
        d.ellipse([cx - 3, fy - 1, cx + 3, fy + 1], fill=(30, 50, 30))
    top = fy - (6 if kneel else 8)
    rect(d, cx - 2, fy - 2, cx - 1, fy, (60, 40, 30))
    rect(d, cx + 1, fy - 2, cx + 2, fy, (60, 40, 30))
    rect(d, cx - 2, top + 3, cx + 2, fy - 2, robe, OUTLINE if False else None)
    if hood:
        rect(d, cx - 2, top - 3, cx + 2, top + 3, robe)
        rect(d, cx - 1, top - 1, cx + 1, top + 1, (24, 18, 28))
    else:
        rect(d, cx - 1, top - 1, cx + 1, top + 2, skin)
        rect(d, cx - 2, top - 3, cx + 2, top - 1, hair)


def ghost(layer, cx, fy, rng, alpha=150, big=False):
    d = ImageDraw.Draw(layer)
    r = 6 if big else 4
    h = 14 if big else 9
    col = (205, 230, 255, alpha)
    d.ellipse([cx - r, fy - h - r, cx + r, fy - h + r], fill=col)
    d.rectangle([cx - r, fy - h, cx + r, fy], fill=col)
    for i in range(-r, r + 1, 2):  # wavy hem
        d.rectangle([cx + i, fy, cx + i + 1, fy + (2 if (i // 2) % 2 else 0)], fill=col)
    ey = fy - h - 1
    d.rectangle([cx - 2, ey, cx - 1, ey + 1], fill=(20, 30, 60, 230))
    d.rectangle([cx + 1, ey, cx + 2, ey + 1], fill=(20, 30, 60, 230))


# ---------- scene ----------
ROBES = [(150, 80, 70), (80, 110, 150), (120, 140, 80), (160, 130, 70), (110, 90, 130), (140, 140, 150)]


def scene(seed=7, mode="calm", doom=0.0, tint=(255, 255, 255), fog=0.25, ghosts=3, windows_on=0.0,
          n_agents=46, with_tiers=False):
    """Return (base_image_RGB at PWxPH, dict of notable positions)."""
    rng = random.Random(seed)
    im, d = ground(rng)
    glow_pts = []
    # background trees
    for x, y in ((20, 70), (4, 130), (396, 120), (408, 70), (180, 222), (250, 226), (40, 228), (380, 228), (96, 100), (330, 104)):
        tree(d, x, y, rng)
    # graveyard
    rect(d, 12, 152, 118, 232, (74, 112, 62))
    for x in range(12, 119, 6):
        d.line([(x, 152), (x, 160)], fill=(60, 44, 34))
    d.line([(12, 156), (118, 156)], fill=(60, 44, 34))
    stones = [(26, 176), (46, 172), (66, 180), (88, 174), (36, 200), (60, 204), (84, 200), (104, 196), (28, 224), (74, 226)]
    for sx, sy in stones:
        tombstone(d, sx, sy)
    # buildings (drawn top to bottom for overlap sanity)
    wins = []
    wins += house(d, 10, 12, 62, 50, (172, 140, 100), (120, 56, 48))
    wins += house(d, 92, 14, 62, 48, (160, 130, 96), (96, 70, 56))
    wins += church(d, 168, 30)
    wins += house(d, 272, 14, 62, 50, (172, 140, 100), (96, 70, 56))
    wins += house(d, 348, 18, 54, 44, (156, 124, 90), (120, 56, 48))
    wins += tavern(d, 296, 152)
    wins += house(d, 384, 170, 30, 40, (160, 128, 92), (96, 70, 56))
    well(d, 13 * T + 16, 7 * T + 4)
    shrine_pt, g = shrine(d, 13 * T + 16, 13 * T, doom)
    glow_pts.append((shrine_pt, 26, g))
    # banners on church
    rect(d, 174, 74, 178, 92, (110, 30, 50), OUTLINE)
    rect(d, 238, 74, 242, 92, (110, 30, 50), OUTLINE)

    # agents
    pos = {"church_door": (208, 108), "shrine": (13 * T + 16, 13 * T), "well": (13 * T + 16, 7 * T + 4)}
    people = []
    prophet = (200, 112)
    plan = []
    if mode == "calm":
        for i in range(n_agents):
            r = rng.random()
            if r < 0.5:
                p = (rng.randrange(8, PW - 8), rng.choice([7 * T + 10, 8 * T + 6]))
            elif r < 0.85:
                p = (rng.choice([13 * T + 4, 14 * T + 6]), rng.randrange(5 * T, PH - 8))
            else:
                p = (rng.randrange(11 * T, 17 * T), rng.randrange(6 * T + 8, 9 * T))
            plan.append(p)
    elif mode == "panic":
        for i in range(n_agents):
            if i < n_agents * 0.45:  # crowd the church door
                a = rng.random() * math.tau
                p = (208 + int(math.cos(a) * rng.randrange(8, 38)), 112 + int(abs(math.sin(a)) * rng.randrange(6, 28)))
            elif i < n_agents * 0.75:  # shrine devotees
                a = rng.random() * math.tau
                p = (13 * T + 16 + int(math.cos(a) * rng.randrange(24, 40)), 13 * T + int(math.sin(a) * 14))
            else:
                p = (rng.randrange(10, PW - 10), rng.choice([7 * T + 10, 8 * T + 6]))
            plan.append(p)
    else:  # doom
        for i in range(n_agents):
            if i < n_agents * 0.55:
                a = i / (n_agents * 0.55) * math.tau
                p = (13 * T + 16 + int(math.cos(a) * 36), 13 * T + int(math.sin(a) * 17))
            else:
                p = (rng.choice([rng.randrange(6, 60), rng.randrange(PW - 70, PW - 6)]), rng.randrange(7 * T, PH - 10))
            plan.append(p)
    for i, (x, y) in enumerate(plan):
        tier = 2 if i < 4 else (1 if i < 16 else 0)
        cultist = (mode != "calm" and i % 3 == 0) or i % 7 == 0
        robe = (96, 40, 110) if cultist else rng.choice(ROBES)
        people.append((y, x, robe, cultist, tier, mode == "doom" and cultist))
    px, py = prophet
    people.append((py, px, (150, 24, 40), True, 2, False))
    people.sort()
    for y, x, robe, cultist, tier, kneel in people:
        agent(d, x, y, robe, hood=cultist, tier=tier if with_tiers else None, kneel=kneel)
    pos["prophet"] = (px, py)
    pos["people"] = [(x, y, t) for y, x, _, _, t, _ in people]

    # ghosts on a separate alpha layer
    layer = Image.new("RGBA", (PW, PH), (0, 0, 0, 0))
    gpos = [(40, 190), (92, 206), (210, 70), (100, 170), (300, 130), (150, 120), (350, 110), (60, 150)]
    for i in range(ghosts):
        gx, gy = gpos[i % len(gpos)]
        ghost(layer, gx, gy, rng, alpha=130 + 20 * (i % 3), big=(i == 0))
    pos["ghosts"] = gpos[:ghosts]
    im = Image.alpha_composite(im, layer).convert("RGB")

    # tint
    im = ImageChops.multiply(im, Image.new("RGB", im.size, tint))

    # glow: windows, shrine, plus lit lanterns
    glow = Image.new("RGB", im.size, (0, 0, 0))
    gd = ImageDraw.Draw(glow)
    if windows_on > 0:
        wc = (int(255 * windows_on), int(200 * windows_on), int(90 * windows_on))
        for (a, b, c2, e) in wins:
            gd.rectangle([a - 1, b - 1, c2 + 1, e + 1], fill=wc)
    for (pt, r, col) in glow_pts:
        gd.ellipse([pt[0] - r, pt[1] - r, pt[0] + r, pt[1] + r], fill=tuple(int(c * (0.35 + 0.5 * doom)) for c in col))
    glow = glow.filter(ImageFilter.GaussianBlur(7))
    im = ImageChops.screen(im, glow)
    # crisp window lights on top
    if windows_on > 0:
        dd = ImageDraw.Draw(im)
        for (a, b, c2, e) in wins:
            dd.rectangle([a, b, c2, e], fill=(255, 214, 120))

    # fog
    nrng = random.Random(seed + 99)
    n = Image.new("L", (W // 2 + 2, H // 2 + 2))
    n.putdata([nrng.randrange(256) for _ in range((W // 2 + 2) * (H // 2 + 2))])
    n = n.resize((PW, PH), Image.BICUBIC).filter(ImageFilter.GaussianBlur(10))
    n = n.point(lambda v: int(max(0, v - 90) * 1.6 * fog))
    fog_col = Image.new("RGB", im.size, (190, 200, 225) if doom < 0.6 else (230, 150, 140))
    im = Image.composite(fog_col, im, n)
    # vignette
    vig = Image.new("L", im.size, 0)
    vd = ImageDraw.Draw(vig)
    vd.ellipse([-PW * 0.2, -PH * 0.35, PW * 1.2, PH * 1.35], fill=255)
    vig = vig.filter(ImageFilter.GaussianBlur(36))
    dark = Image.new("RGB", im.size, (0, 0, 0))
    im = Image.composite(im, dark, vig.point(lambda v: 150 + v * 105 // 255))
    return im, pos


def upscale(im, s=S):
    return im.resize((im.width * s, im.height * s), Image.NEAREST)


# ---------- UI helpers ----------
def panel(d, box, fill=(14, 10, 22, 220), outline=(120, 90, 160)):
    d.rectangle(box, fill=fill, outline=outline, width=2)


def bubble(d, x, y, text, f, maxw=300, tail=True):
    words = text.split()
    lines, cur = [], ""
    for w in words:
        t = (cur + " " + w).strip()
        if d.textlength(t, font=f) > maxw:
            lines.append(cur)
            cur = w
        else:
            cur = t
    lines.append(cur)
    lh = f.size + 4
    bw = int(max(d.textlength(l, font=f) for l in lines)) + 16
    bh = lh * len(lines) + 12
    d.rounded_rectangle([x, y - bh, x + bw, y], 6, fill=(250, 246, 230), outline=(30, 24, 36), width=2)
    if tail:
        d.polygon([(x + 12, y), (x + 24, y), (x + 14, y + 10)], fill=(250, 246, 230), outline=(30, 24, 36))
        d.line([(x + 13, y - 1), (x + 23, y - 1)], fill=(250, 246, 230), width=2)
    for i, l in enumerate(lines):
        d.text((x + 8, y - bh + 6 + i * lh), l, font=f, fill=(30, 24, 36))


def hud(img, days_txt, role, doom_ratio, rumor_lines=None, tier_legend=True, title=True):
    img = img.convert("RGBA")
    ov = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    w, h = img.size
    fl, fb, fs = font(15, True), font(30, True), font(13)
    if title:
        panel(d, (14, 12, 250, 52))
        d.text((26, 22), "LAST VIGIL", font=fl, fill=(230, 210, 255))
        d.text((148, 24), "seed 0x2F7A", font=fs, fill=(150, 130, 180))
    # countdown
    cx = w // 2
    panel(d, (cx - 190, 12, cx + 190, 72), outline=(190, 60, 70))
    d.text((cx - 176, 18), "DOOMSDAY IN", font=fs, fill=(220, 150, 150))
    col = (255, int(220 - 160 * doom_ratio), int(200 - 160 * doom_ratio))
    d.text((cx - 176, 34), days_txt, font=fb, fill=col)
    # progress bar
    d.rectangle([cx + 60, 24, cx + 176, 34], outline=(190, 60, 70))
    d.rectangle([cx + 62, 26, cx + 62 + int(112 * doom_ratio), 32], fill=(210, 60, 70))
    # role
    rw = int(d.textlength(role, font=fl)) + 28
    panel(d, (w - rw - 14, 12, w - 14, 46))
    d.text((w - rw, 20), role, font=fl, fill=(255, 220, 140))
    if tier_legend:
        panel(d, (w - 220, h - 100, w - 14, h - 14))
        d.text((w - 208, h - 94), "AGENT TIERS", font=fs, fill=(200, 190, 220))
        for i, (t, label) in enumerate(((2, "T2 key figure memory"), (1, "T1 townsfolk  events"), (0, "T0 crowd      rules"))):
            y = h - 74 + i * 20
            d.ellipse([w - 206, y + 2, w - 194, y + 12], fill=TIER_COL[t])
            d.text((w - 186, y), label, font=fs, fill=(230, 230, 240))
    if rumor_lines:
        bh = 28 + 22 * len(rumor_lines)
        panel(d, (14, h - bh - 14, 470, h - 14))
        d.text((26, h - bh - 6), "RUMOR TRACE", font=fs, fill=(200, 190, 220))
        for i, (txt, c) in enumerate(rumor_lines):
            d.text((26, h - bh + 18 + i * 22), txt, font=fs, fill=c)
    return Image.alpha_composite(img, ov).convert("RGB")


def save(im, name):
    p = os.path.join(OUT, name)
    im.save(p, optimize=True)
    print("wrote", os.path.relpath(p))


# ---------- images ----------
def make_hero():
    base, pos = scene(seed=7, mode="calm", doom=0.25, tint=(205, 205, 235), fog=0.30, ghosts=3, windows_on=0.9,
                      with_tiers=True)
    im = upscale(base)
    d = ImageDraw.Draw(im)
    f = font(14)
    px, py = pos["prophet"]
    bubble(d, px * S + 14, py * S - 46, "The end is near. Stay inside the walls.", f, 250)
    bubble(d, 40 * S, 120 * S, "I saw him leave at midnight...", f, 230)
    gx, gy = pos["ghosts"][0]
    bubble(d, gx * S + 18, gy * S - 66, "...the lamp burned blue...", font(14), 220)
    im = hud(im, "02d 14:07:33", "ROLE: OBSERVER", 0.35,
             rumor_lines=[("#031  \"The prophet left the compound at night\"   conf 0.78", (255, 230, 150)),
                          ("#031  hop 2  Marta -> Odo -> Brother Aldric", (190, 190, 210))])
    save(im, "hero.png")


def make_doom_triptych():
    specs = [
        ("30 days left   CALM", dict(seed=7, mode="calm", doom=0.05, tint=(235, 235, 245), fog=0.12, ghosts=1, windows_on=0.0), "29d 22:11:04", 0.05),
        ("3 days left   PANIC", dict(seed=7, mode="panic", doom=0.6, tint=(215, 180, 190), fog=0.35, ghosts=4, windows_on=0.8), "02d 23:58:40", 0.6),
        ("00:00   THE END", dict(seed=7, mode="doom", doom=1.0, tint=(255, 140, 120), fog=0.45, ghosts=8, windows_on=1.0), "00d 00:00:03", 1.0),
    ]
    tiles = []
    for label, kw, tm, ratio in specs:
        base, _ = scene(**kw)
        im = upscale(base, 2)
        im = hud(im, tm, "OBSERVER", ratio, tier_legend=False, title=False)
        d = ImageDraw.Draw(im)
        d.rectangle([0, im.height - 38, im.width, im.height], fill=(10, 8, 16))
        d.text((14, im.height - 30), label, font=font(17, True), fill=(235, 220, 255))
        tiles.append(im)
    w = sum(t.width for t in tiles) + 12 * 2
    out = Image.new("RGB", (w, tiles[0].height), (10, 8, 16))
    x = 0
    for t in tiles:
        out.paste(t, (x, 0))
        x += t.width + 12
    save(out, "doomsday-progression.png")


def make_rumor():
    base, pos = scene(seed=7, mode="calm", doom=0.3, tint=(190, 195, 225), fog=0.22, ghosts=2, windows_on=0.8)
    # darken for overlay legibility
    im = upscale(base)
    ov = Image.new("RGBA", im.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    chain = [(40, 190), (96, 112), (150, 118), (214, 112), (13 * T + 16, 7 * T + 14)]
    confs = [0.90, 0.78, 0.61, 0.44, 0.29]
    for i in range(len(chain) - 1):
        a = (chain[i][0] * S, chain[i][1] * S - 20)
        b = (chain[i + 1][0] * S, chain[i + 1][1] * S - 20)
        c = (int(255 * (1 - i / 5) + 120 * (i / 5)), int(210 - 90 * i / 5), int(90 + 60 * i / 5), 255)
        d.line([a, b], fill=c, width=5)
        ang = math.atan2(b[1] - a[1], b[0] - a[0])
        hx, hy = b
        d.polygon([(hx, hy), (hx - 18 * math.cos(ang - 0.4), hy - 18 * math.sin(ang - 0.4)),
                   (hx - 18 * math.cos(ang + 0.4), hy - 18 * math.sin(ang + 0.4))], fill=c)
    for i, (x, y) in enumerate(chain):
        cx, cy = x * S, y * S - 20
        d.ellipse([cx - 16, cy - 16, cx + 16, cy + 16], fill=(20, 14, 30, 235), outline=(255, 230, 150, 255), width=3)
        d.text((cx - 5, cy - 9), str(i), font=font(18, True), fill=(255, 230, 150, 255))
    im = Image.alpha_composite(im.convert("RGBA"), ov).convert("RGB")
    # side panel
    pw = 470
    out = Image.new("RGB", (im.width + pw, im.height), (14, 10, 22))
    out.paste(im, (0, 0))
    d = ImageDraw.Draw(out)
    x0 = im.width + 18
    d.text((x0, 18), "RUMOR #031 ANCESTRY", font=font(18, True), fill=(230, 210, 255))
    d.text((x0, 46), "truth: prophet slips out the east gate", font=font(13), fill=(150, 200, 150))
    hops = [
        ("0  GHOST OMEN (truth fragment)", "A lamp burns blue; a figure at the east gate.", 0.90),
        ("1  Marta (T1)", "Marta saw the prophet leave the compound at night.", 0.78),
        ("2  Odo (T1)", "They say the prophet meets someone beyond the walls.", 0.61),
        ("3  Brother Aldric (T2)", "The prophet sold us to the thing beyond the walls!", 0.44),
        ("4  Crowd (T0, canned)", "The prophet is the demon!", 0.29),
    ]
    y = 90
    for i, (who, txt, c) in enumerate(hops):
        d.rounded_rectangle([x0, y, x0 + pw - 36, y + 118], 8, fill=(26, 20, 40), outline=(100, 80, 140), width=2)
        d.text((x0 + 12, y + 8), who, font=font(14, True), fill=(255, 230, 150))
        # wrap text
        f = font(14)
        words, lines, cur = txt.split(), [], ""
        for w_ in words:
            t = (cur + " " + w_).strip()
            if d.textlength(t, font=f) > pw - 70:
                lines.append(cur)
                cur = w_
            else:
                cur = t
        lines.append(cur)
        for j, l in enumerate(lines):
            d.text((x0 + 12, y + 34 + j * 20), l, font=f, fill=(235, 235, 245))
        d.text((x0 + 12, y + 90), "confidence", font=font(12), fill=(160, 150, 190))
        d.rectangle([x0 + 100, y + 92, x0 + 300, y + 102], outline=(120, 100, 160))
        d.rectangle([x0 + 102, y + 94, x0 + 102 + int(196 * c), y + 100], fill=(int(255 * (1 - c * 0.4)), int(120 + 100 * c), 90))
        d.text((x0 + 312, y + 88), f"{c:.2f}", font=font(14, True), fill=(255, 255, 255))
        y += 126
    d.text((x0, y + 4), "wording drifts each hop; structure stays traceable", font=font(13), fill=(150, 130, 180))
    save(out, "rumor-trace.png")


def sprite_big(drawer, scale=10, w=14, h=18):
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    drawer(d)
    return im.resize((w * scale, h * scale), Image.NEAREST)


# ---------- hue-shifted shading (shadows cooler, highlights warmer) ----------
def shade(c, k=0.7):
    return (int(c[0] * k * 0.9), int(c[1] * k * 0.95), min(255, int(c[2] * k * 1.1 + 10)))


def lit(c, k=1.2):
    return (min(255, int(c[0] * k + 14)), min(255, int(c[1] * k + 8)), min(255, int(c[2] * k)))


def person(d, cx, fy, robe, hair=(74, 50, 36), skin=(232, 190, 150), hood=False, ring=None):
    """17px-tall villager: self-coloured outline, light from the upper left."""
    d.ellipse([cx - 5, fy - 1, cx + 5, fy + 2], fill=(40, 36, 46))  # drop shadow
    if ring:
        d.ellipse([cx - 6, fy - 2, cx + 6, fy + 2], outline=ring)
    ol = shade(robe, 0.4)
    rect(d, cx - 4, fy - 10, cx + 4, fy - 1, robe, ol)  # robe
    d.line([(cx - 3, fy - 9), (cx - 3, fy - 2)], fill=lit(robe))  # lit left edge
    rect(d, cx + 2, fy - 9, cx + 3, fy - 2, shade(robe, 0.75))  # shaded right side
    d.line([(cx - 3, fy - 6), (cx + 3, fy - 6)], fill=(96, 62, 36))  # belt
    d.point((cx - 5, fy - 4), fill=skin)  # hands
    d.point((cx + 5, fy - 4), fill=shade(skin, 0.8))
    rect(d, cx - 2, fy, cx - 1, fy, (50, 36, 30))  # feet
    rect(d, cx + 1, fy, cx + 2, fy, (50, 36, 30))
    if hood:
        rect(d, cx - 3, fy - 17, cx + 3, fy - 10, robe, ol)
        d.line([(cx - 2, fy - 16), (cx - 2, fy - 12)], fill=lit(robe))
        rect(d, cx - 1, fy - 14, cx + 1, fy - 11, (26, 18, 30))  # face in shadow
        d.point((cx, fy - 13), fill=(220, 60, 60))
    else:
        rect(d, cx - 3, fy - 16, cx + 3, fy - 10, skin, shade(skin, 0.45))
        rect(d, cx - 3, fy - 17, cx + 3, fy - 15, hair, shade(hair, 0.5))
        d.point((cx - 1, fy - 13), fill=(30, 22, 30))
        d.point((cx + 1, fy - 13), fill=(30, 22, 30))
        d.point((cx + 2, fy - 11), fill=shade(skin, 0.85))


def cobble(d, w, h, rng, y0=0, stones=((128, 120, 124), (118, 112, 120), (136, 128, 128))):
    rect(d, 0, y0, w, h, (70, 64, 74))
    for y in range(y0, h, 5):
        off = 3 if (y // 5) % 2 else 0
        for x in range(-off, w, 7):
            c = rng.choice(stones)
            rect(d, x, y, x + 5, y + 3, c)
            d.line([(x, y), (x + 5, y)], fill=lit(c, 1.1))
            d.line([(x, y + 3), (x + 5, y + 3)], fill=shade(c, 0.8))


def stall(d, x, y):
    for px in (x, x + 23):
        rect(d, px, y + 3, px + 1, y + 18, (96, 62, 36))
    for i in range(0, 24, 4):
        c = (176, 52, 48) if (i // 4) % 2 == 0 else (232, 214, 180)
        rect(d, x + i, y, x + i + 3, y + 5, c)
        rect(d, x + i + 1, y + 6, x + i + 2, y + 6, c)  # scalloped edge
    d.line([(x, y + 7), (x + 24, y + 7)], fill=(60, 50, 60))  # awning shadow
    rect(d, x - 1, y + 13, x + 25, y + 17, (128, 84, 48), (70, 44, 28))
    for bx in (x + 3, x + 9, x + 15):
        d.ellipse([bx, y + 10, bx + 5, y + 13], fill=(200, 146, 74), outline=(120, 76, 40))


def lantern(d, x, y):
    rect(d, x, y - 14, x + 1, y, (50, 40, 46))
    rect(d, x - 1, y - 19, x + 2, y - 15, (255, 210, 110), (60, 44, 40))
    return (x, y - 17)


def glow_pass(base, pts, tint):
    base = ImageChops.multiply(base, Image.new("RGB", base.size, tint))
    g = Image.new("RGB", base.size, (0, 0, 0))
    gd = ImageDraw.Draw(g)
    for (x, y), r, col in pts:
        gd.ellipse([x - r, y - r, x + r, y + r], fill=col)
    return ImageChops.screen(base, g.filter(ImageFilter.GaussianBlur(4)))


def vignette_t0(rng):
    im = Image.new("RGB", (120, 70))
    d = ImageDraw.Draw(im)
    cobble(d, 120, 70, rng)
    stall(d, 8, 8)
    crowd = [(14, 38), (40, 30), (58, 46), (78, 34), (96, 52), (30, 60), (68, 64), (104, 28), (88, 66)]
    people = sorted((y, x, i) for i, (x, y) in enumerate(crowd))
    for y, x, i in people:
        cult = i == 4
        person(d, x, y, (96, 40, 110) if cult else ROBES[i % 6], hood=cult, ring=TIER_COL[0])
    return glow_pass(im, [], (235, 228, 240))


def vignette_t1(rng):
    im = Image.new("RGB", (120, 70))
    d = ImageDraw.Draw(im)
    cobble(d, 120, 70, rng)
    rect(d, 0, 0, 120, 14, (60, 92, 54))  # verge
    for _ in range(60):
        d.point((rng.randrange(120), rng.randrange(14)), fill=rng.choice([(74, 112, 62), (50, 80, 46)]))
    well(d, 104, 32)
    lp = lantern(d, 10, 44)
    person(d, 24, 66, (140, 140, 150), ring=TIER_COL[0])
    person(d, 46, 58, (160, 130, 70), hair=(150, 90, 40), ring=TIER_COL[1])
    person(d, 74, 60, (80, 110, 150), ring=TIER_COL[1])
    return glow_pass(im, [(lp, 10, (150, 110, 50))], (190, 186, 222))


def vignette_t2(rng):
    im = Image.new("RGBA", (120, 70))
    d = ImageDraw.Draw(im)
    cobble(d, 120, 70, rng, stones=((104, 98, 110), (96, 92, 104), (112, 104, 114)))
    for y in range(0, 42, 4):  # church wall, brick bond
        off = 4 if (y // 4) % 2 else 0
        for x in range(-off, 120, 8):
            c = rng.choice([(138, 136, 150), (128, 126, 142), (146, 142, 154)])
            rect(d, x, y, x + 7, y + 3, c, shade(c, 0.7))
    rect(d, 0, 42, 120, 45, (90, 86, 100), (60, 56, 70))  # step
    d.pieslice([16, 16, 36, 36], 180, 360, fill=(84, 52, 30), outline=OUTLINE)  # arched door
    rect(d, 16, 26, 36, 42, (84, 52, 30), OUTLINE)
    d.line([(26, 18), (26, 42)], fill=(60, 36, 22))
    d.point((30, 32), fill=(220, 190, 90))
    d.pieslice([72, 4, 88, 20], 180, 360, fill=(30, 24, 44), outline=OUTLINE)  # stained glass
    rect(d, 72, 12, 88, 30, (30, 24, 44), OUTLINE)
    for i, c in enumerate([(150, 70, 200), (220, 70, 70), (240, 190, 80), (90, 120, 220)]):
        rect(d, 74 + (i % 2) * 7, 13 + (i // 2) * 8, 79 + (i % 2) * 7, 19 + (i // 2) * 8, c)
    cands = [(48, 56), (70, 56)]
    for cx, cy in cands:
        rect(d, cx, cy - 6, cx + 1, cy, (230, 224, 200))
        d.point((cx, cy - 7), fill=(255, 220, 120))
    person(d, 58, 62, (150, 24, 40), hood=True, ring=TIER_COL[2])
    person(d, 96, 64, (96, 40, 110), hood=True, ring=TIER_COL[0])
    layer = Image.new("RGBA", im.size, (0, 0, 0, 0))
    ghost(layer, 104, 30, None, alpha=110)
    im = Image.alpha_composite(im, layer).convert("RGB")
    pts = [((c[0], c[1] - 7), 8, (170, 120, 50)) for c in cands] + [((80, 18), 12, (90, 50, 120))]
    return glow_pass(im, pts, (160, 150, 196))


def make_tiers():
    W_, H_ = 1240, 740
    s = 3
    im = Image.new("RGB", (W_, H_), (14, 10, 22))
    d = ImageDraw.Draw(im)
    d.text((24, 16), "AGENT TIERS - cheap crowd, expensive stars", font=font(22, True), fill=(230, 210, 255))
    d.text((24, 48), "One model on one GPU. Tiers differ in how often an agent thinks and how much it remembers.",
           font=font(14), fill=(170, 160, 200))
    rng = random.Random(11)
    cols = [
        (0, "TIER 0  CROWD", "30-50 agents  ·  no model", vignette_t0(rng),
         ["Rust utility-AI, no model calls", "beliefs are just numbers", "repeats canned barks", "whole crowd in ~2 ms / tick"]),
        (1, "TIER 1  TOWNSFOLK", "10-15 agents  ·  short prompts", vignette_t1(rng),
         ["thinks only when something happens", "hears a rumor: believe? repeat?", "rewords the claim in its own voice", "a few calls per in-game day"]),
        (2, "TIER 2  KEY FIGURES", "3-5 agents  ·  full memory", vignette_t2(rng),
         ["memory retrieval + reflection", "prophet, elder, medium", "plans, doubts, panics", "first in line for the GPU"]),
    ]
    cw, top = 390, 84
    fb = font(12)
    for i, (t, name, sub, vig, bullets) in enumerate(cols):
        c = TIER_COL[t]
        x = 24 + i * (cw + 20)
        d.rounded_rectangle([x, top, x + cw, top + 500], 12, fill=(24, 18, 38), outline=c, width=3)
        d.text((x + 18, top + 14), name, font=font(20, True), fill=c)
        d.text((x + 18, top + 44), sub, font=font(14), fill=(220, 220, 235))
        v = upscale(vig, s)
        vd = ImageDraw.Draw(v)
        if t == 0:
            bubble(vd, 10, 70, "Fresh bread!", fb)
            bubble(vd, 236, 118, "Repent!", fb)
        elif t == 1:
            bubble(vd, 112, 118, "They say the prophet slips out the east gate at midnight...", fb, 150)
            bubble(vd, 210, 140, "believe? repeat?", fb, 120)
        else:
            bubble(vd, 120, 136, "I must play my part, even if the script is written in blood.", fb, 220)
        im.paste(v, (x + 15, top + 74))
        d.rectangle([x + 14, top + 73, x + 15 + v.width, top + 74 + v.height], outline=shade(c, 0.8), width=2)
        for j, b in enumerate(bullets):
            d.text((x + 18, top + 304 + j * 28), "- " + b, font=font(15), fill=(235, 235, 245))
        if t == 2:
            d.text((x + 18, top + 420), "speech bubble: real qwen3:8b output", font=font(12), fill=(150, 140, 180))
            d.text((x + 18, top + 436), "from the Phase 1 smoke test", font=font(12), fill=(150, 140, 180))
    # promotion arrows between cards
    for ax in (24 + cw + 1, 24 + 2 * cw + 21):
        d.polygon([(ax, 230), (ax + 8, 230), (ax + 8, 220), (ax + 18, 240), (ax + 8, 260), (ax + 8, 250), (ax, 250)],
                  fill=(255, 220, 140))
    # what drives each column
    by = top + 530
    d.rounded_rectangle([24, by, 24 + cw, by + 70], 10, fill=(24, 18, 38), outline=(110, 110, 120), width=2)
    d.text((40, by + 12), "RUST SIM", font=font(15, True), fill=(255, 150, 90))
    d.text((40, by + 38), "every tick, deterministic, no GPU", font=font(13), fill=(220, 220, 235))
    bx0, bx1 = 24 + cw + 20, 24 + 3 * cw + 40
    for cx in (bx0 + cw // 2, bx0 + cw + 20 + cw // 2):
        d.line([(cx, top + 500), (cx, by)], fill=(200, 160, 255), width=3)
    d.rounded_rectangle([bx0, by, bx1, by + 70], 10, fill=(30, 20, 48), outline=(200, 160, 255), width=2)
    d.text((bx0 + 16, by + 12), "ONE SHARED BRAIN   qwen3:8b via Ollama", font=font(15, True), fill=(200, 160, 255))
    d.text((bx0 + 16, by + 38), "~4.7 s per decision  ·  ~750 decisions / hour  ·  VRAM", font=font(13), fill=(220, 220, 235))
    vx = bx0 + 16 + int(d.textlength("~4.7 s per decision  ·  ~750 decisions / hour  ·  VRAM ", font=font(13)))
    d.rectangle([vx, by + 40, vx + 160, by + 52], outline=(200, 160, 255))
    d.rectangle([vx + 2, by + 42, vx + 2 + int(156 * 6.4 / 8), by + 50], fill=(170, 120, 230))
    d.text((vx + 170, by + 38), "6.4 / 8 GB", font=font(13), fill=(220, 220, 235))
    d.text((24, H_ - 30), "Promotion / demotion: an agent at the centre of a rumor chain moves up a tier, then drops back when it quiets down.",
           font=font(14), fill=(255, 220, 140))
    save(im, "agent-tiers.png")


def box(d, xy, title, lines, col):
    d.rounded_rectangle(xy, 12, fill=(24, 18, 38), outline=col, width=3)
    d.text((xy[0] + 16, xy[1] + 12), title, font=font(18, True), fill=col)
    for i, l in enumerate(lines):
        d.text((xy[0] + 16, xy[1] + 44 + i * 22), l, font=font(14), fill=(230, 230, 240))


def arrow(d, a, b, label, col=(200, 190, 230)):
    d.line([a, b], fill=col, width=3)
    ang = math.atan2(b[1] - a[1], b[0] - a[0])
    for s in (a, b):
        pass
    hx, hy = b
    d.polygon([(hx, hy), (hx - 14 * math.cos(ang - 0.4), hy - 14 * math.sin(ang - 0.4)),
               (hx - 14 * math.cos(ang + 0.4), hy - 14 * math.sin(ang + 0.4))], fill=col)
    mx, my = (a[0] + b[0]) // 2, (a[1] + b[1]) // 2
    tw = d.textlength(label, font=font(13))
    d.rectangle([mx - tw / 2 - 6, my - 12, mx + tw / 2 + 6, my + 10], fill=(14, 10, 22))
    d.text((mx - tw / 2, my - 9), label, font=font(13), fill=col)


def make_architecture():
    im = Image.new("RGB", (1240, 520), (14, 10, 22))
    d = ImageDraw.Draw(im)
    d.text((24, 16), "ONE SIM, THREE LANGUAGES", font=font(22, True), fill=(230, 210, 255))
    box(d, (24, 90, 334, 330), "TypeScript  client/", ["Three.js pixel renderer", "ortho camera, nearest sampling", "fog / ghost / light shaders", "role input layers:", "  observer  god  cult  investigator"], (90, 170, 255))
    box(d, (464, 90, 774, 330), "Rust  sim/", ["world, tick loop, seeded RNG", "Tier 0 utility-AI", "rumor propagation + ancestry", "doomsday clock, hidden truth", "tier scheduler + promotion"], (255, 150, 90))
    box(d, (894, 90, 1216, 330), "Python  brain/", ["prompts + structured output", "embedding memory retrieval", "eval harness + notebooks", "rumor-spread analysis", "(later) LoRA fine-tune"], (130, 220, 130))
    box(d, (894, 400, 1216, 490), "Ollama", ["one qwen3:8b for T1 + T2, RTX 3060 Ti"], (200, 160, 255))
    arrow(d, (334, 190), (464, 190), "websocket")
    arrow(d, (464, 250), (334, 250), "snapshots")
    arrow(d, (774, 190), (894, 190), "decision req")
    arrow(d, (894, 250), (774, 250), "belief + words")
    arrow(d, (1055, 330), (1055, 400), "HTTP")
    d.text((24, 360), "Rust never calls a model directly - all prompting lives in Python.", font=font(15), fill=(255, 220, 140))
    d.text((24, 388), "Human actions (god whispers, cult dialogue, investigator questions) are just events into the sim.", font=font(15), fill=(255, 220, 140))
    save(im, "architecture.png")


def crop_scene(base, box_, s=3):
    return upscale(base.crop(box_), s)


def make_roles():
    base, pos = scene(seed=7, mode="calm", doom=0.35, tint=(200, 200, 230), fog=0.22, ghosts=3, windows_on=0.85, with_tiers=True)
    cb = (130, 40, 330, 160)  # 200 x 120 crop
    s = 3
    tiles = []
    f = font(14)
    # observer
    im = crop_scene(base, cb, s)
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, im.width, 34], fill=(10, 8, 16))
    d.text((12, 8), "OBSERVER  -  watch, pause, scrub the rumor trace", font=font(15, True), fill=(255, 220, 140))
    d.line([(60, 330), (200, 250), (330, 270)], fill=(255, 230, 150), width=4)
    bubble(d, 350, 200, "I heard it from Odo.", f, 200)
    tiles.append(im)
    # god
    im = crop_scene(base, cb, s)
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, im.width, 34], fill=(10, 8, 16))
    d.text((12, 8), "INVISIBLE GOD  -  whisper a claim into a mind", font=font(15, True), fill=(255, 220, 140))
    cx, cy = 360, 270
    for r in (18, 34, 52):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=(200, 170, 255), width=2)
    d.rounded_rectangle([20, 360 - 120, 400, 340], 8, fill=(18, 12, 30), outline=(150, 110, 200), width=2)
    d.text((34, 252), "WHISPER to Brother Aldric:", font=f, fill=(180, 160, 220))
    d.text((34, 278), "\"The well water tastes of ash.\"", font=font(16, True), fill=(255, 255, 255))
    d.text((34, 310), "[ send ]   spreads in ~4 min", font=f, fill=(150, 220, 150))
    tiles.append(im)
    # cult member
    im = crop_scene(base, cb, s)
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, im.width, 34], fill=(10, 8, 16))
    d.text((12, 8), "CULT MEMBER  -  walk, talk, be believed or doubted", font=font(15, True), fill=(255, 220, 140))
    px, py = 280, 250
    d.polygon([(px - 10, py - 80), (px + 10, py - 80), (px, py - 66)], fill=(120, 255, 160))
    d.rounded_rectangle([20, 262, 400, 346], 8, fill=(18, 12, 30), outline=(150, 110, 200), width=2)
    d.text((34, 270), "Odo: \"Do you trust the prophet?\"", font=font(15, True), fill=(255, 255, 255))
    for i, c in enumerate(["1. \"He has never lied to me.\"", "2. \"I saw him by the east gate.\"", "3. say nothing"]):
        d.text((34, 296 + i * 16), c, font=font(13), fill=(210, 200, 240))
    tiles.append(im)
    # investigator
    im = crop_scene(base, cb, s)
    d = ImageDraw.Draw(im)
    d.rectangle([0, 0, im.width, 34], fill=(10, 8, 16))
    d.text((12, 8), "INVESTIGATOR  -  piece truth from unreliable testimony", font=font(15, True), fill=(255, 220, 140))
    d.rounded_rectangle([14, 48, 300, 346], 8, fill=(54, 38, 24), outline=(150, 110, 70), width=3)
    notes = [(24, 62, "Marta: east gate", 0.78), (160, 70, "Odo: a stranger", 0.55), (24, 150, "Aldric: betrayal!", 0.30),
             (160, 170, "GHOST: blue lamp", 0.85), (24, 250, "Elder: a hoax", 0.40)]
    for x, y, t, c in notes:
        d.rectangle([x, y, x + 128, y + 60], fill=(240, 224, 150))
        d.text((x + 6, y + 6), t, font=font(12), fill=(40, 30, 20))
        d.rectangle([x + 6, y + 46, x + 6 + int(110 * c), y + 52], fill=(180, 60, 50))
    d.line([(90, 92), (222, 100)], fill=(190, 40, 40), width=2)
    d.line([(222, 100), (222, 200)], fill=(190, 40, 40), width=2)
    d.rounded_rectangle([312, 48, 586, 120], 8, fill=(18, 12, 30), outline=(150, 110, 200), width=2)
    d.text((324, 56), "ACCUSE:  the prophet", font=font(14, True), fill=(255, 255, 255))
    d.text((324, 82), "confidence 61%", font=font(13), fill=(255, 200, 120))
    tiles.append(im)
    tw, th = tiles[0].size
    out = Image.new("RGB", (tw * 2 + 12, th * 2 + 12), (10, 8, 16))
    for i, t in enumerate(tiles):
        out.paste(t, ((i % 2) * (tw + 12), (i // 2) * (th + 12)))
    save(out, "roles.png")


def make_sprite_sheet():
    """A close look at the sprite language: people, tiers, and the haunted layer."""
    im = Image.new("RGB", (1240, 560), (24, 40, 26))
    d = ImageDraw.Draw(im)
    d.text((24, 14), "SPRITE LANGUAGE (placeholder - replaced by CC0 packs)", font=font(20, True), fill=(230, 240, 220))
    labels = ["villager", "cultist", "prophet", "T1 ring", "T2 ring", "kneeling", "ghost"]
    drawers = [
        lambda dd: agent(dd, 7, 15, (80, 110, 150)),
        lambda dd: agent(dd, 7, 15, (96, 40, 110), hood=True),
        lambda dd: agent(dd, 7, 15, (150, 24, 40), hood=True),
        lambda dd: agent(dd, 7, 15, (120, 140, 80), tier=1),
        lambda dd: agent(dd, 7, 15, (160, 130, 70), tier=2),
        lambda dd: agent(dd, 7, 15, (96, 40, 110), hood=True, kneel=True),
    ]
    for i, lab in enumerate(labels):
        x = 24 + i * 150
        panel_bg = Image.new("RGB", (14, 18), (84, 128, 66))
        if i < 6:
            sp = Image.new("RGBA", (14, 18), (84, 128, 66, 255))
            drawers[i](ImageDraw.Draw(sp))
        else:
            sp = Image.new("RGBA", (14, 18), (84, 128, 66, 255))
            lay = Image.new("RGBA", (14, 18), (0, 0, 0, 0))
            ghost(lay, 7, 17 if i == 7 else 15, None, 150, big=(i == 7))
            sp = Image.alpha_composite(sp, lay)
        sp = sp.resize((14 * 8, 18 * 8), Image.NEAREST)
        im.paste(sp, (x, 70))
        d.text((x, 222), lab, font=font(15), fill=(240, 240, 235))
    # buildings strip
    strip, sd = ground(random.Random(3))
    sd.rectangle([0, 0, PW, PH], fill=GRASS)
    house(sd, 8, 70, 62, 50, (172, 140, 100), (120, 56, 48))
    church(sd, 96, 50)
    tavern(sd, 210, 70)
    shrine(sd, 340, 104, 0.5)
    tombstone(sd, 400, 110)
    tree(sd, 28, 40, random.Random(1))
    well(sd, 190, 40)
    strip = strip.crop((0, 0, 416, 130)).resize((416 * 2, 130 * 2), Image.NEAREST)
    im.paste(strip.convert("RGB"), (24, 270))
    save(im, "sprites.png")


if __name__ == "__main__":
    make_hero()
    make_doom_triptych()
    make_rumor()
    make_tiers()
    make_architecture()
    make_roles()
    make_sprite_sheet()


