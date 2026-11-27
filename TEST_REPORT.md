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
