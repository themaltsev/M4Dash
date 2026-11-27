# Test report

## 2026-11-27 — Portable v0.1.14 taskbar behavior / v0.1.15 font import

- `cargo fmt --all`, `cargo check --target x86_64-pc-windows-gnu`: passed for v0.1.15 source after adding custom-font picker/registration. Release and HOME-PC deployment of v0.1.15 remain pending.
- v0.1.14 release build passed and was deployed to `%LOCALAPPDATA%\M4 PC Monitor Portable`; `settings.json` remained 1089 bytes with original timestamp, desktop `.lnk` target/icon resolve to the persistent EXE, HKCU Run value absent.
- Interactive HOME-PC desktop capture showed changing nonzero metrics (CPU ~29%, GPU ~1%/53°C, aggregate network ~8.6/8.9 Mb/s, disk counters and CPU frequency), aligned immediately left of the tray notification arrow. Data/style hash changes now invalidate and synchronously repaint the native GDI child.
- User font addition in v0.1.15 opens the Windows TTF/OTF picker, parses the family name, copies the font under Portable `fonts/`, records only the relative filename in `settings.json`, loads it into egui and registers a process-private GDI font. It does not install fonts system-wide.
- CPU temperature is still N/A in taskbar output; despite an earlier direct provider probe, propagation into the displayed snapshot remains unresolved. LM Studio had no model loaded in the latest screenshot.
- NSIS and TrafficMonitor are untouched; autostart remains disabled.

## 2026-11-27 — первичный preflight

- GitHub repository reachability: успешна, репозиторий пустой.
- HOME-PC SSH: успешен как `home-pc\\user`.
- HOME-PC Rust toolchain: не найден через `cmd /c rustc --version` и `cargo --version`.
- HOME-PC Inno Setup: `iscc.exe` не найден в PATH.
- HOME-PC LM Studio CLI: найден `C:\Users\user\.lmstudio\bin\lms.exe`.
- Ubuntu Rust/Cargo: 1.87.0; MinGW-w64 найден.
- Исходный TrafficMonitor parser tests: прочитаны, но ещё не запущены (PowerShell/runtime harness не подтверждён на этой машине).

## 2026-11-27 — cross compile

- Ubuntu Rust/Cargo: 1.100.0; target `x86_64-pc-windows-gnu` установлен.
- `cargo check --target x86_64-pc-windows-gnu`: успешно; были предупреждения о пока неиспользуемых platform/dependency imports.
- `cargo build --release --target x86_64-pc-windows-gnu`: успешно; локальный EXE находится в `target/x86_64-pc-windows-gnu/release/M4PCMonitor.exe`.
- HOME-PC SSH повторно проверен, но недоступен: `No route to host`.
- Это подтверждает cross-compilation, но не Windows запуск или работу аппаратных датчиков.

## 2026-11-27 — HOME-PC runtime probe

- SSH к HOME-PC восстановлен; временный EXE запускался из Task Scheduler с интерактивным токеном вошедшего пользователя (session 1). После проверки временная задача, EXE в `%TEMP%` и дочерние процессы probe удалены.
- Процесс M4PCMonitor оставался запущен и создал дочерний `lms.exe log stream --source runtime`.
- В исходном варианте CLI запускался без `CREATE_NO_WINDOW`, что создавало дочерний `conhost.exe`; поэтому добавлен Windows creation flag `CREATE_NO_WINDOW`.
- После исправления Win32 enumeration не обнаружил видимых окон у процесса приложения и его дочерних процессов; скрытый `conhost.exe` может оставаться системной частью консольного процесса. Отдельное видимое окно терминала не открывается в этой проверке.
- В профиле HOME-PC нет M4 PC Monitor в HKCU Run, Startup-папке или Scheduled Tasks. NSIS installer содержит необязательную отключённую по умолчанию компоненту создания Startup shortcut.
- Ограничение: удалённая проверка подтвердила жизнь процесса в интерактивном сеансе, но `MainWindowHandle` был нулевым; видимое окно настроек и аппаратные датчики требуют проверки непосредственно на рабочем столе.

## 2026-11-27 — Portable и Windows 11 runtime

- `cargo check --target x86_64-pc-windows-gnu`: успешно.
- `cargo build --release --target x86_64-pc-windows-gnu`: успешно; Windows GUI EXE 6,277,120 bytes.
- Новая сборка синхронизирована в `%LOCALAPPDATA%\M4 PC Monitor Portable\M4PCMonitor.exe`; рядом `M4PCMonitor.ico`.
- Создан ярлык `C:\Users\user\Desktop\M4 PC Monitor.lnk` через shell-resolved Desktop folder. Проверены TargetPath, WorkingDirectory и IconLocation; все пути существуют.
- Приложение запущено именно через `.lnk` в интерактивной сессии `HOME-PC\user`; получены непустые MainWindowHandle и заголовок `M4 PC Monitor`. Окно захвачено непосредственно в desktop-сеансе.
- Реальные данные окна: AMD Ryzen 5 5600X, 12 threads, частота около 3.7 GHz; NVIDIA GeForce RTX 3070 с загрузкой, VRAM и температурой (около 59–61°C); RAM 9.5/31.9 GiB; дисковые тома и два сетевых интерфейса.
- Скорости диска/сети выводятся как MiB/s; при простое могут округляться к 0.00.
- CPU temperature показывает N/A — portable CPU Tctl/Tdie provider пока отсутствует.
- LM Studio UI показывает отсутствие загруженной модели; состояние/скорость токенов отсутствуют, не фальсифицируются.
- Windows Taskbar на кнопке M4 PC Monitor показал прогресс-полоску, соответствующую CPU (на снимке загрузка была низкой). Двухстрочное текстовое отображение в панели задач пока не реализовано.
- В дереве процесса M4PCMonitor → lms.exe присутствует console host процесс, но Win32 enumeration видимых окон процесса нашёл только окна M4 PC Monitor, без видимого окна терминала.
- `settings.json` создан автоматически рядом с EXE; автозапуск в HKCU Run отсутствует.
- TrafficMonitor не изменялся.

## 2026-11-27 — Portable v0.1.1 lifecycle hotfix

- Добавлено `kill_on_drop(true)` для управляемого `lms.exe`; сборка `cargo check` и release cross-build успешны.
- EXE обновлён в той же Portable-папке, существующий `settings.json` не заменялся.
- Ярлык `.lnk` снова запустил окно в интерактивной сессии HOME-PC; instance оставлен запущенным.

## 2026-11-27 — Portable v0.1.2 UI, taskbar, and sensors

- `cargo fmt` and `cargo check --target x86_64-pc-windows-gnu`: passed; release Windows GUI build v0.1.2 passed (PE32+ x86-64).
- Actual interactive HOME-PC desktop capture showed the M4 PC Monitor window and two-line text metrics in the taskbar region (CPU/GPU/RAM/LLM/storage/network); the previous CPU progress bar is no longer the sole integration.
- Layout tab now exposes saved up/down ordering for blocks and dynamic devices; disk/network devices each have visibility checkboxes and editable labels. LM Studio fields moved to their own tab.
- Shortcut resolved to `C:\Users\user\Desktop\M4 PC Monitor.lnk`, target `%LOCALAPPDATA%\M4 PC Monitor Portable\M4PCMonitor.exe`, icon `M4PCMonitor.ico,0`; target and icon exist.
- Deployment preserved existing `settings.json` (344 bytes, unchanged by the deployment script); launch-at-login Run value was removed, so autostart remains disabled.
- Direct live LHM provider probe returned Ryzen 5 5600X `Core (Tctl/Tdie)` 39.5°C and Gigabyte B550 GAMING X V2 board sensors (VRM MOS, chipset, fans, etc.). Provider process is running as a hidden child of M4PCMonitor with `CREATE_NO_WINDOW`.
- In the captured taskbar frame CPU still displayed N/A, despite the independent provider probe returning temperature; app-side sample propagation remains to be resolved. Do not treat CPU temperature display as verified yet.
- Taskbar ordering controls are implemented and compiled; actual user reorder interaction was not independently exercised in this remote session.
- No NSIS changes; TrafficMonitor files/settings/tasks were not touched.

## 2026-11-27 — Portable v0.1.4 network aggregation

- Windows x64 `cargo check` and release build passed; v0.1.4 was copied to the permanent Portable directory on HOME-PC.
- Network now contributes a single ordered taskbar block: upload/download rates sum only interfaces whose individual checkboxes are enabled; interfaces are not separate taskbar columns.
- Shortcut still targets the installed EXE and icon; autostart registry value is absent. The currently running app is PID 20836 in session 1.
- Settings were not replaced during deployment. The current settings file is 1171 bytes (the application has since persisted its expanded schema).
- The process is running, but `MainWindowHandle` remains zero when launched via the remote scheduled-task route. The taskbar appearance cannot be confidently separated from existing TrafficMonitor from the captured desktop; owner should verify the window and aggregate output by opening the desktop shortcut interactively.

## 2026-11-27 — Portable v0.1.5 visibility attempt

- `cargo fmt --check`, Windows x64 `cargo check`, and release build passed. v0.1.5 was deployed to HOME-PC Portable; the existing settings file was not replaced and the Desktop link/icon remain valid.
- Root and taskbar viewports now explicitly request visible state; root also requests active state. Remote launch still reports `MainWindowHandle=0`, so a visible settings window is not verified via SSH/Task Scheduler.
- App process is running in session 1; HKCU Run autostart is absent. Aggregate network block is implemented but must be confirmed interactively alongside taskbar ordering.

## 2026-11-27 — Portable v0.1.6 taskbar/font/CPU update

- Windows x64 `cargo check` and release build passed; v0.1.6 is deployed to the persistent Portable directory. Settings and desktop shortcut are preserved; autostart remains off.
- Taskbar view now reparents the text window to `Shell_TrayWnd`, sets `WS_CHILD`, and positions it in the taskbar client area. Text labels use no-wrap mode, two lines per ordered column.
- Appearance now offers system font choices; files for Segoe UI, Arial, and Consolas exist on HOME-PC.
- Live provider probe on HOME-PC returns `Core (Tctl/Tdie)` about 43.5°C and the Gigabyte B550 board CPU sensor about 43°C. The provider script has a motherboard CPU-sensor fallback.
- Important: the active HOME-PC GUI could not be inspected from SSH. A Task Scheduler screenshot/probe saw `Shell_TrayWnd` but did not find the M4 viewport; therefore actual child-parent attachment and visible CPU display still require owner-side confirmation from the desktop. The source does not claim these runtime checks passed.

## 2026-11-27 — Portable v0.1.3 sensor diagnostics

- Release build v0.1.3 passed `cargo check` and `cargo build --release --target x86_64-pc-windows-gnu`; updated binary was copied to the persistent HOME-PC Portable directory without changing the existing 344-byte `settings.json`.
- Direct provider probe on HOME-PC returns CPU `Core (Tctl/Tdie)` at ~39.5°C and Gigabyte B550 board sensors; provider runs as a child of the M4 process with no window.
- The deployed app process is active in session 1 and the desktop shortcut resolves to the correct executable/icon, but its `MainWindowHandle` remains 0 when launched by remote Task Scheduler. Therefore the Sensors-tab live value and the new ordering controls could not be directly clicked/confirmed in this session.
- Desktop capture contains existing TrafficMonitor taskbar metrics, so it is not sufficient to distinguish/confirm M4's overlay. Do not claim M4 taskbar rendering is proven by that capture.
- Autostart remains disabled. NSIS and TrafficMonitor remain untouched. Portable v0.1.3 is installed and its process left running for owner testing.

## 2026-11-27 — Portable v0.1.22 brand icon pack

- Generated a Windows multi-size ICO, PNG-backed macOS ICNS, Linux hicolor PNG sizes plus scalable SVG, and an embedded 64×64 RGBA variant from the user-supplied SVG. The Rust app window/tray and Portable shortcut now share the branded artwork.
- `cargo build --release --target x86_64-pc-windows-gnu` passed. EXE SHA-256: `5dd45c46ebc485880d120461e837815ce7aa0153ca13f2e29a42204c692dadb4`.
- Deployed to `C:\Users\user\AppData\Local\M4 PC Monitor Portable`; verified EXE and ICO exist, `settings.json` is preserved, and process PID 29800 is running.
- Verified shortcut at `C:\Users\user\Desktop\M4 PC Monitor.lnk`, target `...\M4PCMonitor.exe`, icon `...\M4PCMonitor.ico,0`; HKCU Run autostart remains disabled.
- This deployment check confirms packaging/paths/process only; no new sensor or taskbar-rendering claim is made here. TrafficMonitor was not modified.


## 2026-11-27 — Portable v0.1.23 groups and sensor rows

- Windows x64 release build passed after `cargo fmt`; Portable deployed through `scripts/deploy-portable.ps1` to the persistent HOME-PC directory.
- Updated layout replaces row dropdowns with checkbox selection; individual motherboard sensors can be enabled for groups and renamed. Network rename input removed; motherboard fallback label is MBR.
- Settings now auto-save after a 350 ms quiet period and again on exit; manual Save button removed. Existing `settings.json` remained present and was migrated with new defaults (2914 bytes after launch); autostart remains disabled.
- The process is running in HOME-PC Portable (PID 39496), desktop shortcut targets the permanent EXE and ICO; deployed EXE SHA-256 is `6bc8fd5906553ab0ac281a10b8a7694ba077ba3601978a35685da16abeeb69d0`. SSH reports `MainWindowHandle=0` even though `start_in_tray=false`, so this remote deployment check does not confirm the visible settings window or taskbar rendering.
- Investigated RAM bandwidth feasibility. Current LibreHardwareMonitor integration only reads the CPU/motherboard tree plus SMBIOS memory type/speed; its Memory hardware module exposes usage/load, while DIMM telemetry provides SPD timing/capacity/thermal fields rather than live DRAM throughput. RAM bandwidth is therefore rendered as N/A, not misrepresented by a disk or page-write counter.
- TrafficMonitor was not changed.

## 2026-11-27 — Portable v0.1.24 clipped RAM row cleanup

- Windows x64 release build passed; the RAM taskbar line now contains only type, configured frequency, and used/total capacity, so it fits the existing 25-character group width without the clipped `RAM B…` placeholder.
- Live RAM bandwidth was not found in the current data providers. AMD uProf's system-analysis support matrix lists Ryzen 5000 as unsupported for uProfPcm; the existing LibreHardwareMonitor memory integration reports capacity/usage and DIMM SPD data, not live DRAM throughput.
- Deployed to HOME-PC Portable; `settings.json` remains 2914 bytes, shortcut target/icon remain valid, autostart remains disabled, and process PID 24456 is running. Remote SSH still reports `MainWindowHandle=0`, so this verifies process/deployment but not the visible taskbar pixels.
- TrafficMonitor was not changed.

## 2026-11-27 — Portable v0.1.25 duplicate autosave status

- Windows x64 release build passed; removed the always-visible autosave hint so the settings footer no longer repeats the autosave status.
- Deployed to HOME-PC Portable with settings preserved (2914 bytes), shortcut target/icon intact, and autostart disabled. Process PID 38452 is running; remote `MainWindowHandle=0` means the duplicate-free visual footer itself was not inspected on desktop.
- TrafficMonitor was not changed.

## 2026-11-27 — Portable v0.1.26 LM Studio settings location

- Windows x64 release build passed. LM Studio block name, API address, and `lms.exe` path controls moved below hardware sensors; the separate tab was removed and remaining tab indices adjusted.
- Deployed to HOME-PC Portable; existing settings file remained present (2914 bytes), shortcut target/icon remain valid, autostart is disabled, and process PID 11688 is running.
- Remote process check reports `MainWindowHandle=0`; the new tab placement was not visually verified through SSH. TrafficMonitor was not changed.

## 2026-11-27 — Portable v0.1.31 tray-mode overlay refresh

- Windows x64 release build passed. Moved taskbar metrics refresh to an independent worker so the hidden egui viewport no longer freezes the first (empty) telemetry frame.
- On HOME-PC, verified `start_in_tray=True`; LibreHardwareMonitor reported Ryzen 5 5600X, CPU temperature ~53°C, DDR4 at 3600 MHz, and 11 board sensors.
- Captured the desktop about 3 seconds after startup: no M4 PC Monitor interface was visible, while the taskbar showed current CPU/GPU/RAM values (CPU 3.7 GHz, 19%, 54°C; DDR4 3.6 GHz, 17/32 GB; RTX 3070 9%, 63°C). After the window settled, the process had no main settings window (`MainWindowHandle=0`).
- The interactive diagnostic capture temporarily opened its own PowerShell window; the capture task/process and screenshots were removed after verification. The M4 app itself remained in tray mode. Existing settings were preserved.
