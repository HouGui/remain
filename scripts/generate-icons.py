#!/usr/bin/env python3
"""Regenerate platform icons from packaging/remain.svg using Inkscape and Pillow."""

from pathlib import Path
import subprocess
import tempfile

from PIL import Image

packaging = Path(__file__).resolve().parent.parent / "packaging"
with tempfile.TemporaryDirectory() as directory:
    raster = Path(directory) / "remain.png"
    subprocess.run(
        ["inkscape", str(packaging / "remain.svg"), "--export-width=1024",
         f"--export-filename={raster}"],
        check=True,
    )
    with Image.open(raster) as image:
        image = image.convert("RGBA")
        image.resize((256, 256), Image.Resampling.LANCZOS).save(packaging / "remain.png")
        image.save(packaging / "remain.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
        image.save(packaging / "remain.icns")
