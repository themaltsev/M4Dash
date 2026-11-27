# M4 PC Monitor Portable — Windows x64

1. Распакуйте все файлы архива в доступную для записи папку (EXE, ICO, `hardware-sensors.ps1`, `LibreHardwareMonitorLib.dll` и лицензию).
2. Запустите `M4PCMonitor.exe`.
3. Первый запуск создаёт `settings.json` рядом с EXE. При обновлении заменяйте EXE и не удаляйте `settings.json`.
4. Ярлык рабочего стола можно создать вручную или держать рядом со своей portable-папкой.

Настройки автозапуска выключены по умолчанию. Данные CPU и материнской платы берутся из LibreHardwareMonitor; прочие устройства и локальный LM Studio опрашиваются приложением. Выбранные показатели отображаются текстовым двухстрочным overlay в области панели задач. Настройте блоки и их порядок во вкладке «Расположение». Отмеченные сетевые интерфейсы суммируются в одном блоке «Сеть»; сами интерфейсы отдельными колонками не выводятся.

LibreHardwareMonitor is distributed under the Mozilla Public License 2.0; see `LICENSE-LibreHardwareMonitor.txt`.
