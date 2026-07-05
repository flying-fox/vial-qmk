#!/usr/bin/env python3
"""Primer29 firmware consistency checker.

Validates the three-way 29-key agreement between keyboard.json, vial.json
and the keymap.c files, plus matrix-coordinate sanity. Run from anywhere;
the keyboard directory is located relative to this script.

Exit code 0 = all checks passed, 1 = failure (with a message).
"""

import json
import re
import sys
from pathlib import Path

# script lives at <repo>/.claude/skills/primer29-firmware/scripts/validate.py
KB_DIR = Path(__file__).resolve().parents[4] / "keyboards" / "yushakobo" / "primer29"
EXPECTED_KEYS = 29
MATRIX_ROWS = 10
MATRIX_COLS = 4


def fail(msg):
    print(f"FAIL: {msg}")
    sys.exit(1)


def layer_arg_counts(src):
    """Count top-level arguments of each LAYOUT(...) block (paren-aware)."""
    counts = []
    for m in re.finditer(r"LAYOUT\(", src):
        i, depth, args = m.end(), 1, 1
        while depth > 0:
            ch = src[i]
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
            elif ch == "," and depth == 1:
                args += 1
            i += 1
        counts.append(args)
    return counts


def main():
    if not KB_DIR.is_dir():
        fail(f"keyboard directory not found: {KB_DIR}")

    # keyboard.json
    kb = json.loads((KB_DIR / "keyboard.json").read_text(encoding="utf-8"))
    layout = kb["layouts"]["LAYOUT"]["layout"]
    coords = [tuple(k["matrix"]) for k in layout]
    if len(coords) != EXPECTED_KEYS:
        fail(f"keyboard.json LAYOUT has {len(coords)} keys, expected {EXPECTED_KEYS}")
    if len(set(coords)) != EXPECTED_KEYS:
        fail("keyboard.json LAYOUT has duplicate matrix coordinates")
    bad = [c for c in coords if not (0 <= c[0] < MATRIX_ROWS and 0 <= c[1] < MATRIX_COLS)]
    if bad:
        fail(f"matrix coordinates out of {MATRIX_ROWS}x{MATRIX_COLS} range: {bad}")
    ms = kb.get("matrix_size")
    if ms != {"rows": MATRIX_ROWS, "cols": MATRIX_COLS}:
        fail(f"keyboard.json matrix_size is {ms}, expected rows={MATRIX_ROWS} cols={MATRIX_COLS}")
    if "matrix_pins" in kb:
        fail("keyboard.json must NOT contain matrix_pins (duplex matrix; pins live in config.h)")
    print(f"keyboard.json: OK ({EXPECTED_KEYS} keys, unique, in range, matrix_size correct)")

    # vial.json
    vial = json.loads((KB_DIR / "keymaps" / "vial" / "vial.json").read_text(encoding="utf-8"))
    if vial["matrix"] != {"rows": MATRIX_ROWS, "cols": MATRIX_COLS}:
        fail(f"vial.json matrix is {vial['matrix']}")
    labels = []
    for row in vial["layouts"]["keymap"]:
        for item in row:
            if isinstance(item, str):
                r, c = item.split(",")
                labels.append((int(r), int(c)))
    if len(labels) != EXPECTED_KEYS:
        fail(f"vial.json KLE has {len(labels)} keys, expected {EXPECTED_KEYS}")
    if set(labels) != set(coords):
        missing = set(coords) - set(labels)
        extra = set(labels) - set(coords)
        fail(f"vial.json labels mismatch keyboard.json (missing={missing}, extra={extra})")
    print(f"vial.json: OK ({EXPECTED_KEYS} KLE keys, labels match keyboard.json)")

    # keymap.c files
    for km in ["default", "vial"]:
        src = (KB_DIR / "keymaps" / km / "keymap.c").read_text(encoding="utf-8")
        counts = layer_arg_counts(src)
        if not counts:
            fail(f"keymaps/{km}/keymap.c: no LAYOUT() blocks found")
        wrong = [(i, c) for i, c in enumerate(counts) if c != EXPECTED_KEYS]
        if wrong:
            fail(f"keymaps/{km}/keymap.c: layers with wrong key count (layer, count): {wrong}")
        print(f"keymaps/{km}/keymap.c: OK ({len(counts)} layers x {EXPECTED_KEYS} keys)")

    print("ALL CHECKS PASSED")


if __name__ == "__main__":
    main()
