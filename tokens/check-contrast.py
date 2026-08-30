#!/usr/bin/env python3
"""Gate every semantic colour role against its surface at WCAG AA.

Deliberately dependency-free: this runs in CI without adding Node to the
core loop, unlike the CSS generation step.
"""
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).parent
TEXT_ROLES = ("text", "text-muted", "accent")
# Syntax roles never sit on the page ground — code blocks wear surface-raised —
# so they are gated against that instead.
SYNTAX_ROLES = ("syntax-comment", "syntax-keyword", "syntax-entity", "syntax-string", "syntax-number")
AA_BODY = 4.5


def load():
    prim = json.loads((HERE / "primitive.tokens.json").read_text())
    sem = json.loads((HERE / "semantic.tokens.json").read_text())
    flat = {}

    def walk(node, prefix=""):
        for key, val in node.items():
            if key.startswith("$") or not isinstance(val, dict):
                continue
            if "$value" in val:
                flat[prefix + key] = val["$value"]
            else:
                walk(val, prefix + key + ".")

    walk(prim)
    return flat, sem


def resolve(value, flat):
    ref = re.findall(r"\{([^}]+)\}", value)
    return flat[ref[0]] if ref else value


def luminance(hex_color):
    raw = hex_color.lstrip("#")
    channels = [int(raw[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    channels = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in channels]
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]


def contrast(a, b):
    la, lb = luminance(a), luminance(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def main():
    flat, sem = load()
    roles = sem["color"]
    failures = []
    for mode in ("light", "dark"):
        surface = resolve(roles["surface"][mode]["$value"], flat)
        for name in TEXT_ROLES:
            value = resolve(roles[name][mode]["$value"], flat)
            ratio = contrast(value, surface)
            status = "ok" if ratio >= AA_BODY else "FAIL"
            print(f"{status:4s} {mode:5s} {name:12s} {value} on {surface}  {ratio:5.2f}:1")
            if ratio < AA_BODY:
                failures.append(f"{mode}/{name} {ratio:.2f}:1 < {AA_BODY}")
        # The selection ground carries body text, so that pairing is gated too.
        wash = resolve(roles["accent-wash"][mode]["$value"], flat)
        text_on_wash = resolve(roles["text"][mode]["$value"], flat)
        ratio = contrast(text_on_wash, wash)
        status = "ok" if ratio >= AA_BODY else "FAIL"
        print(f"{status:4s} {mode:5s} {'text/wash':12s} {text_on_wash} on {wash}  {ratio:5.2f}:1")
        if ratio < AA_BODY:
            failures.append(f"{mode}/text-on-wash {ratio:.2f}:1 < {AA_BODY}")
        raised = resolve(roles["surface-raised"][mode]["$value"], flat)
        for name in SYNTAX_ROLES:
            value = resolve(roles[name][mode]["$value"], flat)
            ratio = contrast(value, raised)
            status = "ok" if ratio >= AA_BODY else "FAIL"
            print(f"{status:4s} {mode:5s} {name:12s} {value} on {raised}  {ratio:5.2f}:1")
            if ratio < AA_BODY:
                failures.append(f"{mode}/{name} {ratio:.2f}:1 < {AA_BODY}")
    if failures:
        print("\n" + "\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
