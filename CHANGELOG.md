# Changelog

## 0.1.24 — remove clipped unavailable RAM bandwidth marker

- Keep the taskbar RAM row to type, frequency, and used/total capacity when no live DRAM bandwidth provider is available; this fits the existing fixed-width column instead of showing a clipped `RAM BW N/A` fragment.


## 0.1.23 — group sensor selection and compact CPU/RAM rows

- Replace the two group row dropdowns with checkbox selection and expose individual motherboard sensors as selectable group items.
- Render CPU as `CPU 3.7 GHz: 10% 35°C` and RAM as `DDR4 3.6 GHz: 14/32 GB · RAM BW N/A GB/s`; RAM bandwidth remains unavailable until a trustworthy provider exists.
- Save settings automatically after edits (debounced, with a final save on exit); remove the manual save button and Network rename input.
- Use `MBR` as the motherboard label by default and allow renaming individual motherboard sensors.


## 0.1.22 — cross-platform icon pack

- Add the supplied SVG as the canonical icon source, with multi-resolution Windows ICO, macOS ICNS, Linux hicolor PNG/SVG assets, and an embedded app/tray icon.
- Use the branded icon in the application window and Portable desktop shortcut.


## 0.1.21 — fixed group widths

- Add an adjustable fixed character-based width for each group; pixel widths scale with the selected font and size rather than live metrics.
- Add a right-click menu on taskbar metrics with Open and Exit commands.

## 0.1.20 — groups-only taskbar layout

- Display only user-configured groups, with network RX/TX, CPU/RAM and GPU/LLM as defaults for settings without group configuration.
- Keep existing groups and remove ungrouped entries from layout ordering.

## 0.1.19 — sensor permissions and tray wake

- Request Windows elevation on normal launch so LibreHardwareMonitor can access supported CPU temperature sensors.
- Restore the native window when a tray action arrives, allowing the event loop to process Open/Exit while the window is hidden.

## 0.1.18 — user-defined taskbar groups

- Add groups with independent top and bottom row selectors based on enabled sensors; selecting a row moves it from its default column into the group.
- Save groups in portable settings, retaining earlier configurations when no groups are defined.
- Wake the UI event loop when a tray menu action arrives, including while the window is hidden.

## 0.1.17 — tray and empty-row crash fix

- Add startup mode setting and tray menu with Open and Exit; closing the window hides it when the tray is available.
- Avoid passing a dangling empty-string buffer to DrawTextW when one row in a taskbar column is disabled.
- Guard against zero-width taskbar geometry during Explorer transitions.

## 0.1.16 — grouped taskbar layout and single instance

- Group output into three two-line columns (network receive/send, CPU/RAM, GPU/LLM) and keep CPU metrics on one unwrapped line.
- Include RAM type and configured speed when SMBIOS provides it; log LibreHardwareMonitor startup/sample status and flush its JSON stream for temperature diagnosis.
- Hide disabled blocks/devices from the taskbar-order editor, remove the now-redundant Units tab, and prevent duplicate app/taskbar instances.

## 0.1.15 — Portable custom fonts

- Add a TTF/OTF font picker, copy the selected font to the Portable `fonts` directory, and save its family/file mapping beside the EXE.
- Load custom fonts in the settings UI and register them as process-private GDI fonts for taskbar output; no system-wide install is performed.

## 0.1.14 — live taskbar updates and speed units

- Redraw taskbar output when metrics or visual style change and align text to the notification area/tray arrow.
- Add separate MiB/s or Mb/s selectors for aggregate network and each disk; remove ineffective interface rename fields.

## 0.1.13 — keep metrics clear of pinned taskbar icons

- Add a Win11 composition-host gutter after the legacy task-list rectangle to avoid painting over search and pinned application buttons.

## 0.1.12 — taskbar control-area fallback

- Use the Windows 11 `ReBarWindow32` bounds when the nested task-list HWND is not directly discoverable, so metrics start after pinned applications.

## 0.1.11 — reserve Windows 11 taskbar controls

- Position the metric area after the actual task-list/search control and before the notification area, preventing overlap with Start, search, and pinned app icons.

## 0.1.10 — TrafficMonitor-style taskbar hosting

- Create the layered taskbar renderer as a top-level native window, then parent it to `Shell_TrayWnd` and explicitly switch its style to `WS_CHILD`, matching TrafficMonitor's working HWND lifecycle.
- Treat the null previous-parent return from `SetParent` as success when `GetParent` confirms the new parent.
- Limit startup diagnostic messages to one entry per failure class.

## 0.1.9 — taskbar startup diagnostics

- Record one-time taskbar renderer/window creation diagnostics next to the portable executable to make HWND failures actionable.

## 0.1.8 — Windows 11 composition visibility fix

- Use the layered color-key GDI rendering path used by TrafficMonitor so the taskbar composition bridge does not cover metric text.

## 0.1.7 — native Windows 11 taskbar child

- Replace the secondary egui viewport and after-the-fact HWND lookup with a native Win32 child window created directly under `Shell_TrayWnd`.
- Render the configured font, color, and two taskbar rows in the child window; position it between the Start and notification-area windows when Explorer exposes them.

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
## 0.1.14 — live appearance and speed units

- Force an immediate native taskbar repaint when font family, size, or text color changes, even when metric text is unchanged.
- Add selectable MiB/s or Mb/s units for aggregate network rates and each disk.
- Remove ineffective per-interface rename fields; interface checkboxes only control which counters contribute to the aggregate.
