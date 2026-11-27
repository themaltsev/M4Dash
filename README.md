# M4 PC Monitor — Windows x64

Hardware and local LM Studio monitor for Windows 11 x64. Current installers and Portable archives are published on [GitHub Releases](https://github.com/themaltsev/m4-pc-monitor/releases/latest).

## Install

- **Setup:** run `M4-PC-Monitor-Setup-<version>-x64.exe`. It installs per-user under `%LOCALAPPDATA%\Programs\M4 PC Monitor`, creates Start Menu and desktop shortcuts, and preserves settings when updating in place.
- **Portable:** extract the full `M4PCMonitor-Portable-<version>-x64.zip` into a writable folder and run `M4PCMonitor.exe`. Keep `settings.json` beside the executable when replacing files.

The application checks GitHub Releases at startup and has a manual **Check for updates** button in General. If a newer version has a Windows Setup asset, it asks before downloading; after confirmation it validates the executable (and the SHA-256 sidecar when present), exits, installs into the current application folder, and restarts. Settings are not replaced by Setup updates. Windows may request elevation when launching because LibreHardwareMonitor needs it for some sensors.

## Settings

- **Sensors:** detected hardware and LM Studio API/CLI configuration.
- **Layout:** select sensor rows, groups, order, and column spacing for the taskbar overlay.
- **General:** startup mode (`In a window` or `In tray`), Windows autostart, refresh interval, language, appearance, and update controls. English is the default; Russian, Brazilian Portuguese, Persian, Spanish, French, Italian, and Chinese are available. Theme defaults to **System** and can be switched to Light or Dark. The app uses a larger Windows-style Segoe UI font; font, text size, and color controls apply only to the taskbar overlay.

The app icon is the monitor-and-waveform artwork shared by the window, taskbar/tray, and release assets.

In tray startup, the main window stays hidden while taskbar metrics continue refreshing. CPU and motherboard sensors come from LibreHardwareMonitor; network, disks, GPU, and local LM Studio are polled by the app. RAM bandwidth is not shown unless a trustworthy live provider becomes available.

## Build

The release workflow builds the Windows MSVC binary, Portable ZIP, NSIS Setup, and SHA-256 sidecar on Windows. To build the app locally with the GNU target:

```powershell
cargo build --release --target x86_64-pc-windows-gnu
./scripts/package-portable.ps1 -TargetTriple x86_64-pc-windows-gnu
./scripts/build-windows-setup.ps1 -TargetTriple x86_64-pc-windows-gnu
```

The setup script requires NSIS 3. macOS and Linux currently have no native app build; the icon artwork in the repository is not a functional cross-platform port.
