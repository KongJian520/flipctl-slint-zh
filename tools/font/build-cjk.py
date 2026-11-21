#!/usr/bin/env python3
"""Build the three FlipCTL CJK faces: each original face's Latin glyphs verbatim,
plus Fusion Pixel's full CJK repertoire placed on the 16px grid.

Why a generated font rather than a system one: the panel's text is 16px, the pixel
fonts are designed on a 16-unit grid, and a CJK face designed at 12px would be scaled
1.33x by the rasteriser and antialias. Placing each source pixel on `upem/16` units
makes 16px land exactly one device pixel per source pixel.

Why per-role faces rather than one: a Slint `Text` has one family, and the three roles
differ in upem (1024 / 16 / 1024) and in ascent (9 / 12 / 13), so one face cannot fit
all three without clipping a glyph row.

Output is a derivative of Fusion Pixel Font (OFL-1.1), so the family names are new and
the reserved names are not used; see REUSE.toml.
"""
import argparse, os, sys
from fontTools.ttLib import TTFont
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from pathops import Path as SkPath

ROLES = [
    # (output file, source TTF suffix, new family name, which BDF it takes its CJK from)
    # The title band is 11px tall, so its face carries the 10px design's 9x9 ink; the
    # two row faces have room for the 12px design's 11x11.
    ("FlipCTL-Title.ttf", "HaxrCorp4090-FlipCTL/HaxrCorp4090-FlipCTL.ttf", "FlipCTL Title", "title"),
    ("FlipCTL-Row.ttf", "Busy9px-FlipCTL/Busy9px-FlipCTL.ttf", "FlipCTL Row", "body"),
    ("FlipCTL-RowActive.ttf", "Born2bSportyV2-FlipCTL/Born2bSportyV2-FlipCTL.ttf", "FlipCTL RowActive", "body"),
]


def _in_gb2312(cp):
    try:
        chr(cp).encode("gb2312")
        return True
    except UnicodeEncodeError:
        return False


def parse_bdf(path):
    g, cur, bm, inbm = {}, None, [], False
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.rstrip("\n")
        if line.startswith("STARTCHAR"):
            cur, bm, inbm = {}, [], False
        elif line.startswith("ENCODING"):
            cur["enc"] = int(line.split()[1])
        elif line.startswith("BBX"):
            cur["bbx"] = tuple(int(x) for x in line.split()[1:5])
        elif line.startswith("DWIDTH"):
            cur["dw"] = int(line.split()[1])
        elif line.startswith("BITMAP"):
            inbm = True
        elif line.startswith("ENDCHAR"):
            cur["bm"] = bm
            g[cur["enc"]] = cur
            inbm = False
        elif inbm:
            bm.append(line)
    return g


def cjk_glyph(gl, unit):
    """A CJK bitmap as one unioned outline, on the 16px grid, baseline at y=0."""
    w, h, xo, yo = gl["bbx"]
    rects = []
    for r in range(h):
        hexs = gl["bm"][r] if r < len(gl["bm"]) else "0"
        v, bits = int(hexs, 16), 4 * len(hexs)
        top = (yo + h - r) * unit
        bot = (yo + h - r - 1) * unit
        for c in range(w):
            if (v >> (bits - 1 - c)) & 1:
                rects.append(((xo + c) * unit, bot, (xo + c + 1) * unit, top))
    pen = TTGlyphPen(None)
    if rects:
        sk = SkPath()
        for (x0, y0, x1, y1) in rects:
            sk.moveTo(x0, y0); sk.lineTo(x1, y0); sk.lineTo(x1, y1); sk.lineTo(x0, y1); sk.close()
        sk.simplify()          # adjacent pixel squares share edges; union them
        sk.draw(pen)
    return pen.glyph()


def build(out_dir, bdf_path, fonts_dir, charset="full", bdf_title_path=None):
    bdfs = {"body": parse_bdf(bdf_path), "title": parse_bdf(bdf_title_path or bdf_path)}
    def cjk_of(which):
        return {cp: gl for cp, gl in bdfs[which].items() if 0x4E00 <= cp <= 0x9FFF}
    cjk_body = cjk_of("body")
    if charset == "gb2312":
        # 全量 19546 字 → 3.7 MB/套，三套 11 MB 会把 debug 链接打爆（ld 被 OOM 杀）。
        # GB2312 覆盖 6763 个常用字，是"能用"和"能构建"之间的取舍点。
        cjk_body = {cp: gl for cp, gl in cjk_body.items() if _in_gb2312(cp)}
    print(f"BDF body: {len(cjk_of('body'))} CJK glyphs, title: {len(cjk_of('title'))}", flush=True)

    for out_name, src_rel, family, which in ROLES:
        src = TTFont(os.path.join(fonts_dir, src_rel))
        src_upem = src["head"].unitsPerEm
        # One grid for all three faces. Busy9px is drawn on a 16-unit em, and
        # FreeType hints such a tiny coordinate space badly enough to antialias a
        # 1px stem — the panel's own rasteriser does not hint and does not care, but
        # every other tool that opens the file does. Scaling its Latin outlines by
        # 64 keeps the geometry identical (16px still lands one pixel per design
        # pixel) and makes the file look the same everywhere.
        upem = 1024
        unit = upem // 16
        scale = upem // src_upem
        hhea, os2, head = src["hhea"], src["OS/2"], src["head"]
        ascent_px = hhea.ascent * scale // unit

        src_order = src.getGlyphOrder()
        src_cmap = src.getBestCmap()
        src_glyphset = src.getGlyphSet()
        src_hmtx = src["hmtx"]

        glyphs, metrics, cmap = {}, {}, {}
        for cp, gname in sorted(src_cmap.items()):
            pen = TTGlyphPen(None, handleOverflowingTransforms=True)
            rec = DecomposingRecordingPen(src_glyphset)
            src_glyphset[gname].draw(rec)
            for op, args in rec.value:
                if scale != 1 and op in ("moveTo", "lineTo", "curveTo"):
                    args = [(x * scale, y * scale) for (x, y) in args]
                elif scale != 1 and op == "qCurveTo":
                    args = [None if p is None else (p[0] * scale, p[1] * scale) for p in args]
                pen.__getattribute__(op)(*args)
            glyphs[gname] = pen.glyph()
            adv, lsb = src_hmtx[gname]
            metrics[gname] = (adv * scale, lsb * scale)
            cmap[cp] = gname
        latin_count = len(cmap)

        cjk = cjk_of(which)
        if charset == "gb2312":
            cjk = {cp: gl for cp, gl in cjk.items() if _in_gb2312(cp)}
        for cp, gl in sorted(cjk.items()):
            name = f"uni{cp:04X}"
            if name in glyphs:
                continue
            glyphs[name] = cjk_glyph(gl, unit)
            metrics[name] = (gl["dw"] * unit, 0)
            cmap[cp] = name

        if ".notdef" not in glyphs:
            glyphs[".notdef"] = TTGlyphPen(None).glyph()
            metrics[".notdef"] = (unit * 6, 0)
        names = [".notdef"] + [n for n in glyphs if n != ".notdef"]
        fb = FontBuilder(upem, isTTF=True)
        fb.setupGlyphOrder(names)
        fb.setupCharacterMap(cmap)
        fb.setupGlyf(glyphs)
        fb.setupHorizontalMetrics({n: metrics.get(n, (unit * 6, 0)) for n in names})
        fb.setupHorizontalHeader(ascent=hhea.ascent * scale, descent=hhea.descent * scale)
        fb.setupNameTable({"familyName": family, "styleName": "Regular",
                           "uniqueFontIdentifier": family, "fullName": family,
                           "psName": family.replace(" ", ""), "version": "Version 1.0"})
        fb.setupOS2(sTypoAscender=os2.sTypoAscender * scale, sTypoDescender=os2.sTypoDescender * scale,
                    usWinAscent=os2.usWinAscent * scale, usWinDescent=os2.usWinDescent * scale,
                    sxHeight=os2.sxHeight * scale, sCapHeight=os2.sCapHeight * scale)
        fb.setupPost()
        fb.font["OS/2"].recalcUnicodeRanges(fb.font)
        fb.font["OS/2"].ulCodePageRange1 |= 1 << 18
        fb.font["head"].flags = head.flags

        out = os.path.join(out_dir, out_name)
        fb.save(out)
        size = os.path.getsize(out)
        print(f"  {out_name:<22} upem {upem:>4} unit {unit:>2} ascent {ascent_px:>2}px "
              f"latin {latin_count} cjk {len(cjk)}  {size/1048576:.2f} MB", flush=True)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--bdf", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--fonts", required=True, help="third_party/flipctl-fonts")
    ap.add_argument("--charset", default="full", choices=["full", "gb2312"])
    ap.add_argument("--bdf-title", default=None, help="10px BDF for the title face")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    build(a.out, a.bdf, a.fonts, a.charset, a.bdf_title)
