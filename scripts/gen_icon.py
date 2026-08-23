#!/usr/bin/env python3
"""Generate Asol Naki? app icons (all sizes) from a vector-drawn master via Pillow.

Design: rounded-square deep teal->navy gradient, white laptop outline, green check +
amber warning glyph on the screen. Trust-tool aesthetic: clean, clinical, no text.
"""
from PIL import Image, ImageDraw

SS = 4  # supersample factor for smooth edges
S = 1024 * SS
NAVY = (13, 27, 62)
TEAL = (19, 78, 94)
GREEN = (46, 204, 113)
AMBER = (241, 196, 15)
WHITE = (245, 250, 252)

img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(img)

# Rounded-square gradient background
grad = Image.new("RGBA", (S, S))
gd = ImageDraw.Draw(grad)
for y in range(S):
    t = y / S
    r = int(TEAL[0] + (NAVY[0] - TEAL[0]) * t)
    g = int(TEAL[1] + (NAVY[1] - TEAL[1]) * t)
    b = int(TEAL[2] + (NAVY[2] - TEAL[1023 if False else 3 if False else 2]) * t)
    gd.line([(0, y), (S, y)], fill=(r, g, b, 255))

mask = Image.new("L", (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle([0, 0, S - 1, S - 1], radius=180 * SS, fill=255)
img.paste(grad, (0, 0), mask)

def sc(v):  # scale helper
    return int(v * SS)

# Laptop: screen (rounded rect) + base bar
sx0, sy0, sx1, sy1 = sc(220), sc(260), sc(804), sc(640)
d.rounded_rectangle([sx0, sy0, sx1, sy1], radius=sc(28),
                    outline=WHITE, width=sc(22))
d.rounded_rectangle([sc(150), sc(700), sc(874), sc(790)], radius=sc(40),
                    fill=WHITE)
# hinge notch
d.rounded_rectangle([sc(430), sc(668), sc(594), sc(706)], radius=sc(18), fill=WHITE)

# Green check on screen (thick polyline)
w = sc(56)
pts = [(360, 470), (455, 560), (660, 330)]
d.line(pts, fill=GREEN, width=w, joint="curve")
for p in (pts[0], pts[2]):
    d.ellipse([p[0]-w//2, p[1]-w//2, p[0]+w//2, p[1]+w//2], fill=GREEN)
# round the elbow too
mid = pts[1]
d.ellipse([mid[0]-w//2, mid[1]-w//2, mid[0]+w//2, mid[1]+w//2], fill=GREEN)

# Amber warning triangle, bottom-right of screen
tx, ty, ts = 690, 470, 150
tri = [(tx, ty + ts), (tx + ts // 2, ty), (tx + ts, ty + ts)]
d.polygon(tri, fill=AMBER)
cx = tx + ts // 2
d.rounded_rectangle([cx - 12, ty + 52, cx + 12, ty + 96], radius=10, fill=(30, 41, 59))
d.ellipse([cx - 13, ty + 108, cx + 13, ty + 134], fill=(30, 41, 59))

out = img.resize((1024, 1024), Image.LANCZOS)
out.save("/tmp/an-scaffold/icon-master.png")
print("saved /tmp/an-scaffold/icon-master.png")
