# M4 PC Monitor icons

`M4PCMonitor.svg` is the canonical supplied vector. Generated deliverables:

- `windows/M4PCMonitor.ico`: multi-resolution 16, 24, 32, 48, 64, 128, and 256 px Windows icon.
- `macos/M4PCMonitor.icns`: PNG-backed ICNS with 16–1024 px icon representations.
- `linux/hicolor/`: standard application PNG sizes and `linux/scalable/apps/m4-pc-monitor.svg`.
- `app-64.rgba`: embedded 64×64 RGBA pixels used by the Rust window and tray icon.

Regenerate all raster and container formats from the SVG with `python3 scripts/generate-icons.py` (requires ImageMagick `convert` and Pillow). The existing portable EXE and shortcut use the same Windows ICO.
