#!/usr/bin/env python3
"""Render every Nexus PDF icon from nexus/assets/nexus-pdf.svg (SonnyTech, CC0-1.0).

Needs: pip install resvg-py pillow. The outputs are committed, so builds never need these tools.

    python3 nexus/assets/render-icons.py
"""
import io
import pathlib
import shutil

import resvg_py
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
SVG = HERE / "nexus-pdf.svg"
APP_ID = "vn.sonnytech.nexus.pdf"


def render(svg: str, size: int) -> Image.Image:
    png = bytes(resvg_py.svg_to_bytes(svg_string=svg, width=size, height=size))
    return Image.open(io.BytesIO(png)).convert("RGBA")


def main() -> None:
    svg = SVG.read_text(encoding="utf-8")
    # macOS icons sit on Apple's grid: an 824/1024 body with a transparent margin.
    mac = svg.replace('viewBox="0 0 512 512"', 'viewBox="-62.136 -62.136 636.272 636.272"')
    assert mac != svg, "expected viewBox=\"0 0 512 512\""

    render(svg, 256).save(HERE / "nexus-pdf-256.png")
    render(mac, 512).save(HERE / "nexus-pdf-macos-512.png")

    for s in (16, 24, 32, 48, 64, 128, 256, 512):
        d = HERE / "hicolor" / f"{s}x{s}" / "apps"
        d.mkdir(parents=True, exist_ok=True)
        render(svg, s).save(d / f"{APP_ID}.png")
    d = HERE / "hicolor" / "scalable" / "apps"
    d.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SVG, d / f"{APP_ID}.svg")

    render(svg, 256).save(HERE / "nexus-pdf.ico", sizes=[(s, s) for s in (16, 20, 24, 32, 40, 48, 64, 128, 256)])
    render(mac, 1024).save(HERE / "nexus-pdf.icns")
    print(f"icons written to {HERE}")


if __name__ == "__main__":
    main()
