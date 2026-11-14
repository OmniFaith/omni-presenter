#!/usr/bin/env python3
"""Build and embed custom SVG icons from assets/icons/ into assets/fonts/custom-icons.ttf.

Every icon used in the Omni Presenter application is mapped from its icon name
and Unicode codepoint to its corresponding custom SVG icon from the assets/icons
folder. These vector glyphs are compiled directly into custom-icons.ttf so that
both desktop native egui and web interfaces render the custom icons seamlessly.
"""

import math
import os
import pathlib
import re
import shutil
import subprocess
import tempfile
import time
import xml.etree.ElementTree as ET
from concurrent.futures import ThreadPoolExecutor

from fontTools.ttLib import TTFont
from fontTools.svgLib.path import SVGPath, shapes
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.transformPen import TransformPen

ROOT = pathlib.Path(__file__).resolve().parent.parent
FONT_PATH = ROOT / "assets/fonts/custom-icons.ttf"
ICONS_DIR = ROOT / "assets/icons"

# Monkey-patch fontTools SVG transform to robustly handle translate/scale/matrix/rotate
def robust_transform(raw_value: str):
    m = re.match(r"matrix\((.*?)\)", raw_value)
    if m:
        vals = [float(p) for p in re.split(r"[\s,]+", m.group(1).strip()) if p]
        return tuple(vals)
    m = re.match(r"translate\((.*?)\)", raw_value)
    if m:
        vals = [float(p) for p in re.split(r"[\s,]+", m.group(1).strip()) if p]
        tx = vals[0]
        ty = vals[1] if len(vals) > 1 else 0.0
        return (1.0, 0.0, 0.0, 1.0, tx, ty)
    m = re.match(r"scale\((.*?)\)", raw_value)
    if m:
        vals = [float(p) for p in re.split(r"[\s,]+", m.group(1).strip()) if p]
        sx = vals[0]
        sy = vals[1] if len(vals) > 1 else sx
        return (sx, 0.0, 0.0, sy, 0.0, 0.0)
    m = re.match(r"rotate\((.*?)\)", raw_value)
    if m:
        vals = [float(p) for p in re.split(r"[\s,]+", m.group(1).strip()) if p]
        a = math.radians(vals[0])
        ca = math.cos(a)
        sa = math.sin(a)
        if len(vals) == 3:
            cx, cy = vals[1], vals[2]
            return (ca, sa, -sa, ca, cx - cx * ca + cy * sa, cy - cx * sa - cy * ca)
        return (ca, sa, -sa, ca, 0.0, 0.0)
    return (1.0, 0.0, 0.0, 1.0, 0.0, 0.0)

shapes._transform = robust_transform

# Phosphor icon name -> SVG file mapping
ICON_TO_SVG = {
    # Playback & Media Controls
    "PLAY": "player-play.svg",
    "PAUSE": "player-pause.svg",
    "STOP": "player-stop.svg",
    "SKIP_FORWARD": "player-skip-next.svg",
    "SKIP_BACK": "player-skip-previous.svg",
    "REPEAT": "refresh.svg",
    "ARROWS_CLOCKWISE": "refresh.svg",
    "ARROW_COUNTER_CLOCKWISE": "restore.svg",

    # Carets & Navigation
    "CARET_DOWN": "chevron-down.svg",
    "CARET_UP": "chevron-up.svg",
    "CARET_LEFT": "chevron-left.svg",
    "CARET_RIGHT": "chevron-right.svg",

    # Directional Arrows
    "ARROW_DOWN": "arrow-down.svg",
    "ARROW_UP": "arrow-up.svg",
    "ARROW_LEFT": "arrow-left.svg",
    "ARROW_RIGHT": "arrow-right.svg",
    "ARROW_LINE_UP": "arrow-alt-up.svg",
    "ARROWS_DOWN_UP": "arrow-up-down.svg",
    "ARROWS_LEFT_RIGHT": "arrow-right-left.svg",
    "ARROWS_IN": "arrow-shrink-horizontal.svg",
    "ARROWS_OUT": "arrow-expand-horizontal.svg",
    "ARROW_SQUARE_OUT": "open-new.svg",
    "ARROW_U_UP_LEFT": "arrow-thick-back.svg",
    "ARROW_U_UP_RIGHT": "arrow-thick-forward.svg",

    # Common Actions & CRUD
    "PLUS": "plus.svg",
    "PLUS_CIRCLE": "add-round.svg",
    "MINUS": "remove.svg",
    "TRASH": "trash.svg",
    "X": "cross.svg",
    "CHECK": "check.svg",
    "CHECKS": "square-ok.svg",
    "CHECK_CIRCLE": "check.svg",
    "CHECK_SQUARE": "square-ok.svg",
    "PROHIBIT": "cross.svg",

    # Search & Inspection
    "MAGNIFYING_GLASS": "search.svg",
    "MAGNIFYING_GLASS_PLUS": "zoom-in.svg",
    "MAGNIFYING_GLASS_MINUS": "zoom-out.svg",

    # Settings & Configuration
    "GEAR": "settings.svg",
    "GEAR_SIX": "settings-2.svg",
    "SLIDERS": "settings-3.svg",
    "SLIDERS_HORIZONTAL": "settings-4.svg",

    # Editing & Tools
    "PENCIL": "edit-line.svg",
    "PENCIL_SIMPLE": "edit-square.svg",
    "COPY": "content-copy.svg",
    "COPY_SIMPLE": "content-copy-2.svg",
    "CLIPBOARD": "clipboard-ok.svg",
    "SCISSORS": "edit-line.svg",
    "MAGIC_WAND": "asterisk.svg",
    "PAINT_BRUSH": "drawing-compass.svg",
    "PALETTE": "contrast.svg",
    "BEZIER_CURVE": "drawing-compass.svg",

    # Files & Storage
    "FLOPPY_DISK": "disk-external.svg",
    "DOWNLOAD_SIMPLE": "arrow-alt-down.svg",
    "UPLOAD_SIMPLE": "arrow-alt-up.svg",
    "FILE": "page.svg",
    "FILE_TEXT": "select-text.svg",
    "FILE_PLUS": "add-square.svg",
    "FOLDER": "folder.svg",
    "FOLDER_OPEN": "folder.svg",
    "FOLDER_PLUS": "folder-add.svg",
    "FOLDER_MINUS": "remove-square.svg",

    # Visibility
    "EYE": "eye-open.svg",
    "EYE_SLASH": "eye-closed.svg",

    # Media & Content
    "MUSIC_NOTE": "music-note-2.svg",
    "MUSIC_NOTES": "music-note.svg",
    "BOOK_OPEN": "book-open.svg",
    "BOOKS": "books.svg",
    "CHURCH": "christian-cross.svg",
    "CHAT_CIRCLE_TEXT": "chat-circle-2.svg",
    "CHAT_TEARDROP_TEXT": "chat-circle.svg",
    "STACK": "Layers.svg",
    "IMAGE": "picture.svg",
    "CAMERA": "photo-camera.svg",
    "VIDEO": "video.svg",
    "FILM_STRIP": "video-clip.svg",
    "MICROPHONE": "microphone.svg",
    "SPEAKER_HIGH": "volume-up.svg",
    "SPEAKER_LOW": "volume-down.svg",
    "SPEAKER_SLASH": "volume-mute.svg",
    "BELL": "bell.svg",
    "MEGAPHONE": "microphone.svg",

    # Typography
    "TEXT_T": "text.svg",
    "TEXT_B": "text-format-bold.svg",
    "TEXT_ITALIC": "text-format-italic.svg",
    "TEXT_UNDERLINE": "text-format-underline.svg",
    "TEXT_STRIKETHROUGH": "text-format-strikethrough.svg",
    "TEXT_ALIGN_LEFT": "text-align-left.svg",
    "TEXT_ALIGN_CENTER": "text-align-center.svg",
    "TEXT_ALIGN_RIGHT": "text-align-right.svg",
    "TEXT_ALIGN_JUSTIFY": "text-align-justfiy.svg",
    "TEXT_AA": "text-case-upper.svg",
    "TEXT_INDENT": "chevron-right.svg",
    "TEXT_OUTDENT": "chevron-left.svg",

    # Hardware & System
    "MONITOR": "monitor.svg",
    "TELEVISION": "monitor.svg",
    "TELEVISION_SIMPLE": "monitor.svg",
    "DESKTOP": "monitor-2.svg",
    "PROJECTOR_SCREEN": "monitor-2.svg",
    "KEYBOARD": "keyboard.svg",
    "DEVICE_MOBILE": "phone-ringer.svg",
    "WIFI_HIGH": "wifi.svg",
    "POWER": "power.svg",
    "GLOBE": "globe.svg",
    "CLOCK": "watch-1.svg",
    "TIMER": "smart-watch.svg",
    "CALENDAR_BLANK": "calendar.svg",
    "PUZZLE_PIECE": "puzzle.svg",
    "QR_CODE": "grid-view.svg",
    "USER": "person.svg",
    "USERS_THREE": "person-group.svg",
    "HEART": "heart.svg",
    "STAR": "star.svg",
    "LINK": "link.svg",
    "LINK_SIMPLE": "link.svg",
    "TAG": "tag-01.svg",
    "PUSH_PIN": "bookmark.svg",
    "INFO": "info.svg",
    "WARNING": "warning.svg",
    "WARNING_CIRCLE": "warning.svg",
    "QUESTION": "help.svg",
    "LIGHTNING": "light.svg",
    "SUN": "Sun.svg",
    "MOON": "night.svg",
    "DROP": "drop.svg",
    "CIRCLE_HALF": "contrast.svg",
    "MAGNET": "attach-file.svg",
    "TARGET": "camera-focus.svg",
    "TROPHY": "star.svg",
    "INFINITY": "refresh.svg",
    "PULSE": "heart.svg",

    # UI Layout & Indicators
    "FUNNEL": "funnel.svg",
    "LAYOUT": "tile.svg",
    "ROWS": "tile-2.svg",
    "COLUMNS": "tile.svg",
    "SQUARE": "square-more.svg",
    "RECTANGLE": "card.svg",
    "SQUARES_FOUR": "grid-view.svg",
    "CIRCLE": "bitcoin-circle.svg",
    "CUBE": "tile-2.svg",
    "CARDS": "grid-view.svg",
    "CHECKERBOARD": "grid-view.svg",
    "DOTS_THREE": "circle-more.svg",
    "DOTS_THREE_CIRCLE": "circle-more.svg",
    "DOTS_THREE_VERTICAL": "more-vert.svg",
    "DOTS_SIX_VERTICAL": "drag-indicator.svg",
    "CURSOR": "cursor.svg",
    "LIST": "list.svg",
    "LIST_BULLETS": "list.svg",
    "LIST_DASHES": "list.svg",
    "LIST_NUMBERS": "order.svg",
    "LIST_PLUS": "addlist.svg",
    "LOCK_SIMPLE": "lock.svg",
    "LOCK_SIMPLE_OPEN": "lock-open.svg",
    "PAPER_PLANE_RIGHT": "arrow-alt-right.svg",
    "HAND_WAVING": "hand-open.svg",
    "SMILEY": "face-happy.svg",
}

# Explicit sidebar and custom codepoints
CUSTOM_CODEPOINTS = {
    0xE001: "music-note-2.svg",
    0xE002: "christian-bible.svg",
    0xE003: "chat-circle-2.svg",
    0xE004: "Layers.svg",
}

def load_phosphor_codepoints() -> dict[str, int]:
    """Find codepoints.rs in cargo registry cache or fallback dictionary."""
    import glob
    candidates = glob.glob(
        os.path.expanduser("~/.cargo/registry/src/*/egui-phosphor-*/src/variants/codepoints.rs")
    )
    if not candidates:
        raise FileNotFoundError("Could not find egui-phosphor codepoints.rs")
    with open(candidates[0], "r", encoding="utf-8") as f:
        content = f.read()
    pattern = re.compile(r"pub const ([A-Z0-9_]+): &str = \"\\u\{([0-9A-Fa-f]+)\}\";")
    return {name: int(hex_str, 16) for name, hex_str in pattern.findall(content)}

def clean_svg_file(svg_path: str, out_path: str) -> None:
    """Filter out invisible elements (e.g. stroke-to-path leftover guides with opacity 0)."""
    tree = ET.parse(svg_path)
    root = tree.getroot()
    for parent in list(root.iter()):
        for child in list(parent):
            style_str = child.attrib.get("style", "")
            style = dict(part.split(":", 1) for part in style_str.split(";") if ":" in part)

            # Check display / opacity
            if child.attrib.get("display") == "none" or style.get("display") == "none":
                parent.remove(child)
                continue
            if child.attrib.get("opacity") == "0" or style.get("opacity") == "0":
                parent.remove(child)
                continue

            fill = child.attrib.get("fill") or style.get("fill")
            fill_op = child.attrib.get("fill-opacity") or style.get("fill-opacity", "1")
            has_no_fill = (fill == "none" or fill_op == "0")

            stroke = child.attrib.get("stroke") or style.get("stroke")
            stroke_op = child.attrib.get("stroke-opacity") or style.get("stroke-opacity", "1")
            stroke_w = child.attrib.get("stroke-width") or style.get("stroke-width", "1")
            has_no_stroke = (stroke in (None, "none") or stroke_op == "0" or stroke_w == "0")

            if has_no_fill and has_no_stroke and child.tag.split("}")[-1] in (
                "path", "rect", "circle", "ellipse", "polygon", "polyline", "line"
            ):
                parent.remove(child)
    tree.write(out_path)

def convert_svg_to_plain(args: tuple[str, str, str]) -> tuple[str, bool, str]:
    in_path, raw_out_path, clean_out_path = args
    cmd = [
        "inkscape",
        in_path,
        "--actions=select-all:all;selection-ungroup;selection-ungroup;selection-ungroup;object-stroke-to-path;export-plain-svg;export-do",
        "-o",
        raw_out_path,
    ]
    env = os.environ.copy()
    env["DBUS_SESSION_BUS_ADDRESS"] = "disabled:"
    res = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if res.returncode != 0:
        # Retry once after brief pause
        time.sleep(0.1)
        res = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if res.returncode != 0:
        return (os.path.basename(in_path), False, res.stderr)
    try:
        clean_svg_file(raw_out_path, clean_out_path)
    except Exception as e:
        return (os.path.basename(in_path), False, str(e))
    return (os.path.basename(in_path), True, clean_out_path)

def build_glyph(svg_path: str, glyf, scale_factor: float = 1.0):
    sx = (1024.0 / 6.35) * scale_factor
    sy = -(1005.0 / 6.35) * scale_factor

    # Pass 1: compute bounds
    svg_path_obj = SVGPath(svg_path)
    tt_pen = TTGlyphPen(glyf)
    cu2qu_pen = Cu2QuPen(tt_pen, max_err=1.0)
    transform_pen = TransformPen(cu2qu_pen, (sx, 0, 0, sy, 0, 0))
    svg_path_obj.draw(transform_pen)
    glyph = tt_pen.glyph()
    glyph.recalcBounds(glyf)

    # Compute centering shift
    cx = (glyph.xMin + glyph.xMax) / 2.0
    cy = (glyph.yMin + glyph.yMax) / 2.0
    dx = 512.0 - cx
    dy = 448.0 - cy

    # Pass 2: draw final centered glyph
    tt_pen = TTGlyphPen(glyf)
    cu2qu_pen = Cu2QuPen(tt_pen, max_err=1.0)
    transform_pen = TransformPen(cu2qu_pen, (sx, 0, 0, sy, dx, dy))
    svg_path_obj.draw(transform_pen)
    glyph = tt_pen.glyph()
    glyph.recalcBounds(glyf)
    return glyph

def main() -> None:
    print(f"Loading font {FONT_PATH}...")
    font = TTFont(str(FONT_PATH))
    glyf = font["glyf"]
    hmtx = font["hmtx"]

    phosphor_codepoints = load_phosphor_codepoints()

    # Collect all (codepoint, svg_filename) mappings
    codepoint_to_svg: dict[int, str] = {}

    # 1. Add Phosphor mappings
    for icon_name, svg_file in ICON_TO_SVG.items():
        if icon_name not in phosphor_codepoints:
            print(f"Warning: Phosphor constant {icon_name} not found in codepoints.rs")
            continue
        code = phosphor_codepoints[icon_name]
        codepoint_to_svg[code] = svg_file

    # 2. Add custom codepoints (takes priority)
    for code, svg_file in CUSTOM_CODEPOINTS.items():
        codepoint_to_svg[code] = svg_file

    unique_svgs = sorted(set(codepoint_to_svg.values()))
    print(f"Mapped {len(codepoint_to_svg)} codepoints to {len(unique_svgs)} unique custom SVGs.")

    tmpdir = tempfile.mkdtemp(prefix="omni_custom_icons_")
    try:
        tasks = []
        for svg_name in unique_svgs:
            in_file = str(ICONS_DIR / svg_name)
            raw_out_file = os.path.join(tmpdir, "raw_" + svg_name)
            clean_out_file = os.path.join(tmpdir, svg_name)
            tasks.append((in_file, raw_out_file, clean_out_file))

        print(f"Converting and cleaning {len(tasks)} SVGs with Inkscape in parallel...")
        t0 = time.time()
        with ThreadPoolExecutor(max_workers=4) as executor:
            results = list(executor.map(convert_svg_to_plain, tasks))
        t1 = time.time()

        failed = [r for r in results if not r[1]]
        if failed:
            for f in failed:
                print(f"Error converting {f[0]}: {f[2]}")
            raise RuntimeError(f"{len(failed)} SVGs failed to convert.")
        print(f"All {len(results)} SVGs converted and cleaned in {t1 - t0:.2f}s.")

        # Cache converted glyphs by SVG filename
        glyph_cache = {}
        for svg_name in unique_svgs:
            clean_p = os.path.join(tmpdir, svg_name)
            glyph = build_glyph(clean_p, glyf, scale_factor=1.0)
            glyph_cache[svg_name] = glyph

        # Populate font glyf, hmtx, and cmap
        applied_count = 0
        for code, svg_name in codepoint_to_svg.items():
            gname = f"uni{code:04X}"
            glyph = glyph_cache[svg_name]
            glyf[gname] = glyph
            hmtx[gname] = (1024, glyph.xMin)

            for table in font["cmap"].tables:
                table.cmap[code] = gname
            applied_count += 1

        font.save(str(FONT_PATH))
        print(f"Successfully updated {applied_count} icon glyphs in {FONT_PATH}!")

    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)

if __name__ == "__main__":
    main()
