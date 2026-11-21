# The three faces that carry Chinese

Generated, not drawn: `tools/font/build-cjk.py` takes each original face's Latin
glyphs and Fusion Pixel's CJK and puts them on one 16-unit grid. Built for 16px text,
where one design pixel is one device pixel.

| File | Latin from | CJK from | Licence |
|---|---|---|---|
| `FlipCTL-Title.ttf` | HaxrCorp 4090 (FlipCTL adaptation) | Fusion Pixel 10px | `OFL-1.1 AND CC-BY-SA-3.0` |
| `FlipCTL-Row.ttf` | Busy9px | Fusion Pixel 12px | `OFL-1.1 AND MIT` |
| `FlipCTL-RowActive.ttf` | Born2bSportyV2 | Fusion Pixel 12px | `OFL-1.1` |

The title face carries the 10px design because its ink is 9x9 and the title band is
11px tall; the two row faces carry the 12px design, which has the room to be 11x11.

## Fusion Pixel

CJK from [Fusion Pixel Font](https://github.com/TakWolf/fusion-pixel-font) by TakWolf,
under the SIL Open Font License 1.1 — `LICENSES/OFL-1.1.txt` has the text.

The license reserves the names 「缝合像素」 and "Fusion Pixel"; a derivative must not
carry them, so these faces are named `FlipCTL Title`, `FlipCTL Row` and
`FlipCTL RowActive`, and the family names in the `.slint` files are those. They are
not sold, and the license travels with them.

## The Latin half

Copied from the three faces in the directories beside this one, unchanged, so the
English the panel drew before Chinese existed is the English it draws now. Their terms
are theirs and are annotated in `REUSE.toml`: HaxrCorp 4090 is ShareAlike, which is why
the title face is `AND CC-BY-SA-3.0` rather than OFL alone.

## Rebuilding

    docker run --rm -v "$PWD":/src -w /src python:3.12-slim \
      sh -c 'pip install fonttools skia-pathops && python tools/font/build-cjk.py \
      --bdf fusion-pixel-12px-proportional-zh_hans.bdf \
      --bdf-title fusion-pixel-10px-proportional-zh_hans.bdf \
      --out third_party/flipctl-fonts/FlipCTL-CJK --fonts third_party/flipctl-fonts'

The BDFs come from the Fusion Pixel release archive; `--charset gb2312` builds the
smaller subset if the full repertoire is ever too large for a target.
