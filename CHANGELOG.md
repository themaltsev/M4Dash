# Changelog

## 0.1.48 — synchronize the native title bar theme

- Apply the selected M4Dash light/dark colors to the Windows title bar and caption text.

## 0.1.47 — improve settings text contrast

- Use high-contrast Windows-style text colors in both light and dark themes.
- Keep the settings UI on the Windows system font, separate from taskbar typography.

## 0.1.46 — Windows-style light/dark colors and system accent

- Match the settings palette to Windows light and dark appearance.
- Read and refresh the selected Windows accent color dynamically, with a blue fallback.

## 0.1.45 — respect tray launch mode

- Keep the GUI hidden when launching a second instance while tray mode is selected.
- Preserve the selected visibility when Windows elevation restarts the app.

## 0.1.44 — safe in-place font preservation

- Skip copying custom fonts when Setup is upgrading in the same install directory.

## 0.1.43 — show RAM usage percentage

- Append the used RAM percentage to the taskbar item and sensor settings preview.

## 0.1.42 — optional desktop shortcut

- Add a localized, initially checked Setup checkbox to create the desktop shortcut.
- Remove an existing M4Dash desktop shortcut when the checkbox is unchecked.

## 0.1.41 — brand Setup and uninstaller icons

- Use the M4Dash monitor logo as the Windows Setup executable, installer-window, and uninstaller icon.

## 0.1.40 — rename executable and install folder

- Build and install the app as `M4Dash.exe` under `%LOCALAPPDATA%\Programs\M4Dash`.
- Migrate settings, custom fonts, and autostart target from the previous M4 PC Monitor install.
- Register the branded ICO as the installed-app icon in Windows Settings and use it on M4Dash shortcuts.
- Publish only M4Dash-named Setup and Portable assets; remove legacy-name Setup aliases.

## 0.1.38 — M4Dash release branding and documentation

- Rename Windows Setup and Portable release assets to M4Dash and use M4Dash as the GitHub release title.
- Keep a legacy Setup filename alongside the renamed installer so v0.1.37 and earlier updaters can still update.
- Rewrite the repository README around installation, monitoring, taskbar layout, tray mode, and updates.

## 0.1.37 — rename displayed app to M4Dash

- Use M4Dash in the window, app header, tray, update dialogs, installer, and shortcuts.
- Migrate old Start Menu/Desktop shortcut labels during Setup while keeping the existing installation directory for in-place updates.

## 0.1.36 — repository rename

- Point the automatic GitHub update checker and release documentation to the renamed M4Dash repository.

## 0.1.35 — separate app and taskbar typography

- Use a larger Windows-style Segoe UI font in the settings interface, independent of taskbar typography controls.
- Keep the selected font and text size exclusively for taskbar metrics, and label the controls accordingly.

## 0.1.34 — update controls and refreshed app icon

- Add a localized manual GitHub update check/status block to General, alongside the automatic startup check.
- Replace the app icon with the supplied monitor/health waveform artwork across Windows, tray, Portable, Linux, and macOS icon assets.
- Use the official portable NSIS compiler archive in Windows CI instead of Chocolatey.

## 0.1.33 — manual update controls

- Add a localized Updates block to General with a manual GitHub check button and current check/update status.
- Keep automatic startup checks and confirmation-gated installation.

## 0.1.32 — system theme and Windows setup/update

- Replace the light/dark checkbox with System, Light, and Dark theme modes; follow Windows preference by default and migrate saved legacy choices.
- Check GitHub Releases on startup and offer updates; download and validate the Windows setup only after confirmation, then update in place while preserving settings.
- Provide an NSIS Windows installer and a reproducible setup packaging script.

## 0.1.31 — keep taskbar metrics live in tray mode

- Refresh the native taskbar overlay from a background worker so telemetry continues when the main window is hidden and egui no longer emits frames.


## 0.1.30 — start hidden in tray mode

- Create the main window hidden when the saved startup mode is the system tray, so it no longer flashes open before being hidden.


## 0.1.29 — default to English

- Set English as the default interface language, including when older settings files do not yet contain a language choice.


## 0.1.28 — add multilingual interface

- Add persistent language selection in General: Russian, English, Brazilian Portuguese, Persian, Spanish, French, Italian, and Chinese.
- Localize the main tabs and sensor, layout, LM Studio, and General settings UI.


## 0.1.27 — make theme selection work

- Apply the selected light/dark egui visuals and make light theme the default for fresh settings.
- Move all appearance controls into the General tab and remove the separate Appearance tab.


## 0.1.26 — consolidate LM Studio settings

- Move LM Studio block name, API address, and `lms.exe` path controls into the Sensors tab and remove the separate LM Studio tab.


## 0.1.25 — remove duplicate autosave hint

- Remove the persistent autosave helper line; the footer now displays only the single status message after a settings change.


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
