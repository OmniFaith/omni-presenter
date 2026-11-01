#!/usr/bin/env python3
"""Inventory native UI literals and check the shared translation catalogs.

This development command only reads project files. Run with --area to review
one workspace at a time, or --catalogs to validate all translation resources.
"""
import argparse
import collections
import difflib
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
I18N = ROOT / "src/core/i18n"
AREAS = {
    "workspace": ["src/ui/components", "src/ui/containers", "src/ui/presentation.rs", "src/ui/modals/editor.rs", "src/ui/modals/editor_versions.rs", "src/ui/modals/text_editor.rs", "src/ui/modals/verse_saver.rs", "src/ui/modals/folder_creator.rs", "src/ui/modals/web_lyrics.rs", "src/ui/pages/bible"],
    "design": ["src/ui/pages/design", "src/core/design.rs", "src/core/design_style.rs"],
    "settings": ["src/ui/modals/settings", "src/ui/modals/theme_creator", "src/ui/modals/presentation_settings.rs", "src/ui/modals/preview.rs"],
    "services": ["src/app", "src/services", "src/core/preferences.rs", "src/core/models", "src/core/state", "src/core/layout", "src/core/bible.rs", "src/core/slides.rs"],
    "chat": ["src/ui/pages/chat"],
}
TOKEN = re.compile(r'//[^\n]*|/\*.*?\*/|[bc]?r(?P<rawhash>\#*)".*?"(?P=rawhash)|"(?:\\.|[^"\\])*"|\b[A-Za-z_][A-Za-z_0-9]*\b|::|[^\s]', re.S)
DISPLAY = {"label", "heading", "strong", "small", "button", "checkbox", "radio_value", "selectable_label", "selectable_value", "menu_button", "on_hover_text", "on_disabled_hover_text", "hint_text", "selected_text", "colored_label", "prefix", "suffix", "set_title", "add_filter", "set_file_name", "caption", "note", "section", "section_title", "section_header", "card_header", "setting_toggle", "styled_button", "context_menu_item", "context_menu_danger_item", "context_menu_disabled_item", "context_menu_submenu", "property_slider", "property_slider_pct", "tooltip"}
CONSTRUCTORS = {"RichText::new", "Label::new", "Button::new", "Window::new", "CollapsingHeader::new"}
IDS = {"from_id_salt", "id_salt", "make_persistent_id", "with", "include_str!", "include_bytes!", "new_id", "get_temp", "insert_temp"}
NEUTRAL = {"", " ", "X", "Y", "W", "H", "R", "G", "B", "A", "H:", "W:", "X:", "Y:", "X: ", "Y: ", "B", "I", "U", "S", "Aa", "RGB", "RGBA", "HEX", "HTTP", "HTTPS", "URL", "ID", "JSON", "PNG", "OBS", "N/A", "px", " px", "ms", " ms", "s", " s", "min", " min", "dB", "Proportional", "Monospace", "Serif", "Montserrat", "Inter", "Roboto", "Arial", "Theme", "English", "Español", "Português (Brasil)", "Ctrl", "Alt", "Shift", "Cmd", "Enter", "Escape", "Space", "Backspace", "Delete", "Tab", "F11", "F12"}


def catalogs():
    locales_dir = ROOT / "locales"
    codes = ["en", "pt-BR", "es"]
    merged = [{}, {}, {}]
    for index, code in enumerate(codes):
        folder = locales_dir / code
        if not folder.is_dir():
            continue
        for ftl_file in sorted(folder.glob("*.ftl")):
            text = ftl_file.read_text()
            for match in re.finditer(r'^([a-zA-Z0-9_-]+)\s*=\s*(.+?)(?=\n[a-zA-Z0-9_-]+\s*=|\Z)', text, re.M | re.S):
                msg_id = match.group(1).strip()
                val = match.group(2).strip()
                merged[index][msg_id] = val
    return merged



def source_files(area):
    found = set()
    for relative in AREAS[area]:
        path = ROOT / relative
        found.update(path.rglob("*.rs") if path.is_dir() else [path])
    return sorted(path for path in found if "tests" not in path.stem)


LOCALIZED_HELPERS = {"caption", "note", "section", "section_title", "section_header", "card_header", "setting_toggle", "styled_button", "context_menu_item", "context_menu_danger_item", "context_menu_disabled_item", "context_menu_submenu", "property_slider", "property_slider_pct"}


def literals(path, all_human=False, details=False):
    source = path.read_text()
    test_modules = re.compile(r'#\[cfg\(test\)\]\s*(?:#\[path\s*=\s*"[^"]+"\]\s*)?mod\s+\w+\s*([;{])')
    for match in reversed(list(test_modules.finditer(source))):
        finish = match.end()
        if match.group(1) == "{":
            depth = 1
            for token in TOKEN.finditer(source, finish):
                if token.group() == "{": depth += 1
                elif token.group() == "}": depth -= 1
                if depth == 0:
                    finish = token.end()
                    break
        source = source[:match.start()] + re.sub(r'[^\n]', ' ', source[match.start():finish]) + source[finish:]
    tokens = list(TOKEN.finditer(source))
    stack = []
    for index, token in enumerate(tokens):
        value = token.group()
        if value.startswith(("//", "/*")):
            continue
        if value in {"(", "[", "{"}:
            callee = ""
            if value == "(":
                cursor = index - 1
                parts = []
                while cursor >= 0 and (re.fullmatch(r"[\w!]+", tokens[cursor].group()) or tokens[cursor].group() == "::"):
                    parts.append(tokens[cursor].group())
                    cursor -= 1
                callee = "".join(reversed(parts))
            stack.append([value, callee, 0, token.start()])
            continue
        if value in {")", "]", "}"}:
            if stack:
                stack.pop()
            continue
        if value == "," and stack:
            stack[-1][2] += 1
            continue
        if not value.startswith('"'):
            continue
        try:
            text = json.loads(value)
        except ValueError:
            continue
        if text in NEUTRAL or not any(c.isalpha() for c in text):
            continue
        calls = [(name, argument) for _, name, argument, _ in stack if name]
        if not calls and not all_human:
            continue
        nearest = calls[-1][0].split("::")[-1] if calls else ""
        if nearest in IDS or nearest in {"text", "format", "localize", "tr", "message"}:
            continue
        if any(name.split("::")[-1] in {"message", "tr", "localize"} for name, _ in calls):
            continue
        visible = any(name.split("::")[-1] in DISPLAY or name.endswith(tuple(CONSTRUCTORS)) for name, _ in calls)
        formatted = any(name == "format!" for name, _ in calls)
        # Machine values in selectable_value and format strings remain untouched.
        if re.fullmatch(r"[a-z0-9_./:#?%+-]+", text):
            continue
        if not (visible or formatted or all_human):
            continue
        if any(name.startswith(("assert", "debug_assert", "panic", "eprintln", "println")) for name, _ in calls):
            continue
        if details:
            yield token.start(), token.end(), text, calls, [frame.copy() for frame in stack], source
        else:
            yield source.count("\n", 0, token.start()) + 1, text


def migration_patch(area, texts):
    print("*** Begin Patch")
    for path in source_files(area):
        if "/ui/" not in str(path):
            continue
        source = path.read_text()
        replacements = {}
        for start, end, text, calls, stack, _ in literals(path, details=True):
            if text not in texts[1]:
                continue
            if any(name.split("::")[-1] in LOCALIZED_HELPERS for name, _ in calls):
                continue
            macro = next((frame for frame in reversed(stack) if frame[1] == "format!"), None)
            if macro:
                if not any(name.split("::")[-1] in DISPLAY or name.endswith(tuple(CONSTRUCTORS)) for name, _ in calls[:-1]):
                    continue
                opening = macro[3]
                depth = 1
                finish = None
                for match in TOKEN.finditer(source, opening + 1):
                    if match.group() == "(": depth += 1
                    elif match.group() == ")": depth -= 1
                    if depth == 0:
                        finish = match.end()
                        break
                if finish:
                    begin = source.rfind("format!", 0, opening)
                    original = source[begin:finish]
                    replacements[(begin, finish)] = f"crate::ui::i18n::message(ui, &{original})"
            elif any(name.split("::")[-1] in DISPLAY or name.endswith(tuple(CONSTRUCTORS)) for name, _ in calls):
                replacements[(start, end)] = f"crate::ui::i18n::text(ui, {source[start:end]})"
        updated = source
        for (start, end), replacement in sorted(replacements.items(), reverse=True):
            updated = updated[:start] + replacement + updated[end:]
        if updated != source:
            print(f"*** Update File: {path.relative_to(ROOT)}")
            for line in list(difflib.unified_diff(source.splitlines(), updated.splitlines(), n=3, lineterm=""))[2:]:
                print("@@" if line.startswith("@@") else line)
    print("*** End Patch")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--area", choices=AREAS)
    parser.add_argument("--catalogs", action="store_true")
    parser.add_argument("--missing", action="store_true")
    parser.add_argument("--all-human", action="store_true")
    parser.add_argument("--patch", action="store_true")
    args = parser.parse_args()
    texts = catalogs()
    if args.patch:
        if not args.area:
            parser.error("--patch requires --area")
        migration_patch(args.area, texts)
        return 0
    if args.catalogs:
        errors = []
        placeholders = lambda value: set(re.findall(r"\{([^{}]+)\}", value))
        for index, catalog in enumerate(texts):
            if catalog.keys() != texts[0].keys():
                errors.append(f"catalog {index}: different keys")
            for key, value in catalog.items():
                if not isinstance(value, str) or not value.strip():
                    errors.append(f"{key}: empty translation")
                elif placeholders(value) != placeholders(texts[0][key]):
                    errors.append(f"{key}: different placeholders")
        if errors:
            print("\n".join(errors))
            return 1
        print(f"Catalogs: {len(texts[0])} entries, complete in en / pt-BR / es.")
    if args.area:
        inventory = collections.defaultdict(list)
        for path in source_files(args.area):
            for line, text in literals(path, all_human=args.all_human):
                if not args.missing or text not in texts[1]:
                    inventory[text].append(f"{path.relative_to(ROOT)}:{line}")
        print(json.dumps(inventory, ensure_ascii=False, indent=2))
        print(f"{args.area}: {len(inventory)} unique candidate literals", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
