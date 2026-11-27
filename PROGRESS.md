# M4 PC Monitor — Portable progress

Обновлено: 2026-11-27

## Текущий приоритет

Только Portable-версия Windows 11 x64. NSIS отложен. HOME-PC получает каждую успешную сборку в постоянный каталог `%LOCALAPPDATA%\M4 PC Monitor Portable`; обновление сохраняет `settings.json`. Ярлык создаётся в разрешённой Windows Desktop-папке пользователя.

## Реализовано в Portable v0.1.6

- Настройки рядом с EXE (`settings.json`); файл создаётся при первом старте, запись через временный файл и атомарную замену.
- Рабочая галочка автозапуска в HKCU Run; default выключен.
- Двухстрочные колонки текста; viewport переводится в дочерний HWND `Shell_TrayWnd`, строки без автоматического переноса. Проверка фактического parent HWND ожидает интерактивный desktop-сеанс.
- Редактор порядка отображения; отдельные чекбоксы дисков и сетевых интерфейсов, пользовательские подписи блоков и устройств.
- Один суммарный сетевой блок: выбранные интерфейсы агрегируются по upload/download; чекбоксы интерфейсов только управляют включением в сумму и не создают taskbar-колонки.
- Отдельная вкладка настройки LM Studio.
- LibreHardwareMonitor sidecar для CPU Tctl/Tdie и сенсоров материнской платы (MPL-2.0); отдельный live probe на HOME-PC подтвердил реальные показания.
- Taskbar text child hosted by `Shell_TrayWnd` (not a topmost floating overlay); font family is selectable in Appearance; taskbar lines are no-wrap.
- CPU Tctl/Tdie проверен напрямую (~43.5°C); motherboard CPU sensor включён как fallback для CPU readout.
- Реальные показатели CPU utilization/frequency, RAM, volumes, network, NVIDIA GPU через NVML.
- Дисковые и сетевые скорости вычисляются из дельт счётчиков.
- Runtime state LM Studio из CLI/API; `lms.exe` запускается с `CREATE_NO_WINDOW`.
- Дочерний `lms.exe` завершается при закрытии приложения, не оставляя runtime-stream orphan.
- Portable app icon; отдельная иконка у desktop shortcut.
- На HOME-PC проверен запуск через фактический `.lnk` в интерактивном пользовательском сеансе; приложение оставлено запущенным.

## Текущие ограничения/дальнейшая проверка

- Прямой запуск LHM provider подтверждает CPU Tctl/Tdie и плату. Передачу показания в UI нужно проверить: MainWindowHandle у удалённо запущенного приложения равен 0, taskbar кадр трудно отделить от TrafficMonitor.
- Порядок показателей редактируется и сохраняется, но фактическое нажатие ↑/↓ ещё нужно проверить в открытом окне.
- На снимке HOME-PC LM Studio сообщила отсутствие загруженной модели; LLM token rate отсутствовал.
- Системный tray и тонкая настройка ширин колонок — дальнейшая работа.
- NSIS installer не изменяется в этой фазе.

## HOME-PC

- Постоянная папка: `%LOCALAPPDATA%\M4 PC Monitor Portable`.
- Desktop shortcut: `M4 PC Monitor.lnk`, путь Desktop получен из Windows Shell Known Folder API.
- Автозапуск HKCU Run отсутствует.
- Текущий экземпляр: M4PCMonitor.exe, запущен через desktop shortcut; autostart выключен; TrafficMonitor не изменялся.
- MainWindowHandle при удалённом старте через Task Scheduler равен 0; владельцу нужно открыть desktop shortcut интерактивно, чтобы подтвердить окно настроек/порядок/агрегированный overlay.
