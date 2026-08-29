#!/usr/bin/env python3
"""Split the shipped stylesheet into a light and a dark variant.

Storybook has to switch schemes at runtime, and a `light-dark()` inside a
custom property is not re-resolved when `color-scheme` changes, so a toggle
cannot drive it. Both variants are derived from the stylesheet the build
actually emits, so only the colour values differ from what ships.
"""
import pathlib
import re
import sys


def pick(css: str, index: int) -> str:
    out, i = [], 0
    while True:
        start = css.find("light-dark(", i)
        if start == -1:
            out.append(css[i:])
            return "".join(out)
        out.append(css[i:start])
        depth, j = 0, start + len("light-dark(") - 1
        while j < len(css):
            if css[j] == "(":
                depth += 1
            elif css[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        args = [a.strip() for a in css[start + len("light-dark(") : j].split(",")]
        out.append(args[index])
        i = j + 1


def main() -> int:
    source, out_dir = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    css = source.read_text()
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, index in (("light", 0), ("dark", 1)):
        flat = pick(css, index)
        flat = re.sub(r"color-scheme:[^;]+;", f"color-scheme: only {name};", flat, count=1)
        (out_dir / f"main.{name}.css").write_text(flat)
        print(f"  main.{name}.css")
    return 0


if __name__ == "__main__":
    sys.exit(main())
