# Changelog

## 0.1.6 — taskbar child hosting, font selector, CPU fallback

- Host the text renderer as a child window of `Shell_TrayWnd`, not a topmost window floating over the desktop.
- Add system-font selection (Segoe UI, Arial, Tahoma, Consolas, Cascadia Mono) in Appearance.
- Prevent wrapping within metric lines; preserve exactly two lines per taskbar item.
- Fall back to the motherboard's CPU temperature sensor when the CPU Tctl/Tdie sensor is unavailable.

## 0.1.5 — explicit visible app windows

- Explicitly request visible/active state for the settings window and visible state for the taskbar overlay on first creation.

## 0.1.4 — aggregate network block

- Network interface checkboxes now only select counters included in aggregate rates.
- Taskbar shows one reorderable/custom-name Network block with summed upload and download speeds.
- Network interfaces no longer become individual taskbar columns.

## 0.1.3 — sensor propagation diagnostics

- Expose the detected CPU temperature source and provider/JSON-stream errors in the Sensors tab.
- Surface JSON parse and provider stream errors in live sensor state.

## 0.1.2 — Portable UI, sensors, and taskbar text

- Replaced the CPU progress indicator with a transparent two-line taskbar text overlay.
- Added editable per-block/per-device labels and per-disk/per-network-interface visibility.
- Added saved taskbar ordering controls (up/down) for standard blocks, each disk, and each interface.
- Moved LM Studio controls to a dedicated tab.
- Bundled LibreHardwareMonitor CPU/motherboard sensor bridge and MPL-2.0 notice.
- Added a repeatable HOME-PC portable deployment script that preserves `settings.json`, recreates the desktop shortcut, and leaves autostart disabled.

## 0.1.1 — Portable runtime hotfix

- Kill the LM Studio runtime CLI when M4 PC Monitor exits, preventing orphaned collector processes.

## 0.1.0 — разработка

- Создан отдельный Rust/egui каркас.
- Добавлены базовые hardware snapshots (sysinfo) и прямой NVIDIA NVML provider.
- Добавлен постоянный runtime-stream reader LM Studio и начальный parser событий PP/TG/lifecycle.
- Добавлены базовые вкладки настроек и атомарная запись конфигурации.
