# M4 PC Monitor — Portable progress

Обновлено: 2026-11-27

## Текущий приоритет

Только Portable-версия Windows 11 x64. NSIS отложен. HOME-PC получает каждую успешную сборку в постоянный каталог `%LOCALAPPDATA%\M4 PC Monitor Portable`; обновление копирует только EXE, не трогая `settings.json` и иконку. Ярлык находится в фактической Desktop-папке текущего пользователя.

## Реализовано в текущей сборке

- Настройки рядом с EXE (`settings.json`); файл создаётся при первом старте, запись через временный файл и атомарную замену.
- Рабочая галочка автозапуска в HKCU Run; default выключен.
- Нативный Windows Taskbar progress indicator, процент отражает текущую загрузку CPU.
- Реальные показатели CPU utilization/frequency, RAM, volumes, network, NVIDIA GPU через NVML.
- Дисковые и сетевые скорости вычисляются из дельт счётчиков.
- Runtime state LM Studio из CLI/API; `lms.exe` запускается с `CREATE_NO_WINDOW`.
- Дочерний `lms.exe` завершается при закрытии приложения, не оставляя runtime-stream orphan.
- Portable app icon; отдельная иконка у desktop shortcut.
- На HOME-PC проверен запуск через фактический `.lnk` в интерактивном пользовательском сеансе; приложение оставлено запущенным.

## Фактические ограничения

- CPU Tctl/Tdie provider ещё не подключён; на HOME-PC CPU temp отображается N/A. Не подменяем его ACPI/материнским показателем.
- Taskbar integration пока показывает штатную полосу загрузки CPU, не желаемый двухстрочный текстовый монитор. Скриншот подтвердил видимую полосу под app icon.
- На снимке HOME-PC LM Studio сообщила отсутствие загруженной модели; LLM token rate отсутствовал.
- Tray и редактор порядка/ширин колонок ещё впереди.
- NSIS installer не изменяется в этой фазе.

## HOME-PC

- Постоянная папка: `%LOCALAPPDATA%\M4 PC Monitor Portable`.
- Desktop shortcut: `M4 PC Monitor.lnk`, путь Desktop получен из Windows Shell Known Folder API.
- Автозапуск HKCU Run отсутствует.
- Текущий экземпляр: M4PCMonitor.exe, запущен через ярлык; не трогать TrafficMonitor.
