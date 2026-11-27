# Test report

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

## 2026-11-27 — Portable v0.1.3 sensor diagnostics

- Release build v0.1.3 passed `cargo check` and `cargo build --release --target x86_64-pc-windows-gnu`; updated binary was copied to the persistent HOME-PC Portable directory without changing the existing 344-byte `settings.json`.
- Direct provider probe on HOME-PC returns CPU `Core (Tctl/Tdie)` at ~39.5°C and Gigabyte B550 board sensors; provider runs as a child of the M4 process with no window.
- The deployed app process is active in session 1 and the desktop shortcut resolves to the correct executable/icon, but its `MainWindowHandle` remains 0 when launched by remote Task Scheduler. Therefore the Sensors-tab live value and the new ordering controls could not be directly clicked/confirmed in this session.
- Desktop capture contains existing TrafficMonitor taskbar metrics, so it is not sufficient to distinguish/confirm M4's overlay. Do not claim M4 taskbar rendering is proven by that capture.
- Autostart remains disabled. NSIS and TrafficMonitor remain untouched. Portable v0.1.3 is installed and its process left running for owner testing.
