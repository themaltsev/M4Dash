# Changelog

## 0.1.1 — Portable runtime hotfix

- Kill the LM Studio runtime CLI when M4 PC Monitor exits, preventing orphaned collector processes.

## 0.1.0 — разработка

- Создан отдельный Rust/egui каркас.
- Добавлены базовые hardware snapshots (sysinfo) и прямой NVIDIA NVML provider.
- Добавлен постоянный runtime-stream reader LM Studio и начальный parser событий PP/TG/lifecycle.
- Добавлены базовые вкладки настроек и атомарная запись конфигурации.
