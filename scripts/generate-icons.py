#!/usr/bin/env python3
"""Generate the M4 PC Monitor icon pack from assets/icons/M4PCMonitor.svg.
Requires ImageMagick (`convert`) and Pillow (`python -m pip install Pillow`).
"""
from pathlib import Path
import io
import struct
import subprocess
from PIL import Image

ROOT = Path(__file__).resolve().parents[1] / "assets" / "icons"
SOURCE = ROOT / "M4PCMonitor.svg"
SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
images = {}
for size in SIZES:
    output = ROOT / "linux" / "hicolor" / f"{size}x{size}" / "apps" / "m4-pc-monitor.png"
    output.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["convert", "-background", "none", str(SOURCE), "-resize", f"{size}x{size}", str(output)], check=True)
    images[size] = Image.open(output).convert("RGBA")

(ROOT / "linux" / "scalable" / "apps" / "m4-pc-monitor.svg").write_bytes(SOURCE.read_bytes())
images[256].save(ROOT / "windows" / "M4PCMonitor.ico", format="ICO", sizes=[(n, n) for n in [16, 24, 32, 48, 64, 128, 256]])
(ROOT.parent / "M4PCMonitor.ico").write_bytes((ROOT / "windows" / "M4PCMonitor.ico").read_bytes())
(ROOT / "app-64.rgba").write_bytes(images[64].tobytes())

# macOS ICNS using PNG-backed icon family chunks.
entries = [(16, b"icp4"), (32, b"icp5"), (64, b"icp6"), (128, b"ic07"), (256, b"ic08"), (512, b"ic09"), (1024, b"ic10")]
chunks = []
for size, kind in entries:
    encoded = io.BytesIO()
    images[size].save(encoded, format="PNG")
    payload = encoded.getvalue()
    chunks.append(kind + struct.pack(">I", len(payload) + 8) + payload)
contents = b"icns" + struct.pack(">I", 8 + sum(map(len, chunks))) + b"".join(chunks)
(ROOT / "macos" / "M4PCMonitor.icns").write_bytes(contents)
print("Generated Windows ICO, macOS ICNS, Linux hicolor icons, and embedded 64x64 RGBA.")
