use crate::{
    config::Settings,
    hardware::Snapshot,
    llm::{LlmPhase, LlmSnapshot},
};
use eframe::egui;
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

pub struct MonitorApp {
    settings: Settings,
    hardware: Arc<RwLock<Snapshot>>,
    llm: Arc<RwLock<LlmSnapshot>>,
    tab: usize,
    status: String,
    launch_at_login: bool,
    last_taskbar_update: Instant,
}

impl MonitorApp {
    pub fn new(
        settings: Settings,
        hardware: Arc<RwLock<Snapshot>>,
        llm: Arc<RwLock<LlmSnapshot>>,
    ) -> Self {
        Self {
            settings,
            hardware,
            llm,
            tab: 0,
            status: String::new(),
            launch_at_login: crate::startup::is_enabled(),
            last_taskbar_update: Instant::now() - Duration::from_secs(2),
        }
    }

    fn save(&mut self) {
        self.status = match self.settings.save() {
            Ok(()) => "Настройки сохранены рядом с приложением".into(),
            Err(e) => format!("Ошибка сохранения: {e}"),
        };
    }

    fn update_autostart(&mut self, enabled: bool) {
        match crate::startup::set_enabled(enabled) {
            Ok(()) => {
                self.launch_at_login = enabled;
                self.status = if enabled {
                    "Автозапуск включён"
                } else {
                    "Автозапуск выключен"
                }
                .into();
            }
            Err(error) => self.status = format!("Не удалось изменить автозапуск: {error}"),
        }
    }
}

pub fn icon_data() -> egui::IconData {
    const N: usize = 32;
    let mut rgba = vec![0; N * N * 4];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[24, 29, 40, 255]);
    }
    for y in 5..24 {
        for x in 4..28 {
            if x == 4 || x == 27 || y == 5 || y == 23 {
                pixel(&mut rgba, x, y, [190, 199, 215, 255]);
            }
        }
    }
    for (x, y) in [(8, 17), (11, 14), (14, 16), (17, 11), (20, 13), (23, 9)] {
        pixel(&mut rgba, x, y, [239, 68, 68, 255]);
        if y + 1 < 23 {
            pixel(&mut rgba, x, y + 1, [239, 68, 68, 255]);
        }
    }
    for x in 14..19 {
        pixel(&mut rgba, x, 25, [190, 199, 215, 255]);
    }
    for x in 11..22 {
        pixel(&mut rgba, x, 27, [190, 199, 215, 255]);
    }
    egui::IconData {
        rgba,
        width: N as u32,
        height: N as u32,
    }
}

fn pixel(rgba: &mut [u8], x: usize, y: usize, color: [u8; 4]) {
    let offset = (y * 32 + x) * 4;
    rgba[offset..offset + 4].copy_from_slice(&color);
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(self.settings.refresh_ms.max(200)));
        let snapshot = self.hardware.read().map(|s| s.clone()).unwrap_or_default();
        let llm = self.llm.read().map(|s| s.clone()).unwrap_or_default();

        if self.last_taskbar_update.elapsed() >= Duration::from_secs(1) {
            let cpu = self
                .settings
                .show_taskbar
                .then_some(snapshot.cpu_usage.round().clamp(0.0, 100.0) as u32);
            if let Err(error) = crate::taskbar::update(cpu) {
                tracing::debug!("Unable to update taskbar monitor: {error}");
            }
            self.last_taskbar_update = Instant::now();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("M4 PC Monitor");
                ui.label("Portable · Windows 11 x64");
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for (index, title) in ["Датчики", "Расположение", "Оформление", "Единицы", "Общие"]
                    .iter()
                    .enumerate()
                {
                    if ui.selectable_label(self.tab == index, *title).clicked() {
                        self.tab = index;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.tab {
                    0 => self.sensor_page(ui, &snapshot, &llm),
                    1 => {
                        ui.heading("Расположение показателей");
                        ui.label("Taskbar сейчас показывает загрузку CPU стандартным индикатором прогресса Windows.");
                        ui.label("Редактор компактных строк, колонок и порядка датчиков — в разработке.");
                    }
                    2 => {
                        ui.heading("Оформление");
                        ui.add(egui::Slider::new(&mut self.settings.font_size, 9.0..=24.0).text("Размер текста"));
                        ui.color_edit_button_srgb(&mut self.settings.text_color);
                        ui.checkbox(&mut self.settings.dark_theme, "Тёмная тема");
                    }
                    3 => {
                        ui.heading("Единицы измерения");
                        ui.label("Скорости сети: MiB/s; память: GiB; частота: MHz; температура: °C.");
                    }
                    _ => self.general_page(ui),
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Сохранить настройки").clicked() { self.save(); }
                    ui.label(&self.status);
                });
            });
        });
    }
}

impl MonitorApp {
    fn sensor_page(&mut self, ui: &mut egui::Ui, s: &Snapshot, llm: &LlmSnapshot) {
        ui.heading("Датчики и обнаруженное оборудование");
        ui.label("Показания обновляются автоматически. Недоступные датчики отображаются как N/A.");
        ui.separator();
        ui.checkbox(&mut self.settings.show_cpu, "CPU");
        ui.label(format!(
            "{} · {} потоков · {:.0}% · {} MHz · температура {}",
            s.cpu_name,
            s.cpu_threads,
            s.cpu_usage,
            s.cpu_frequency_mhz,
            s.cpu_temperature_c
                .map(|v| format!("{v:.0}°C"))
                .unwrap_or_else(|| "N/A".into())
        ));
        ui.checkbox(&mut self.settings.show_gpu, "NVIDIA GPU");
        ui.label(format!(
            "{} · загрузка {} · VRAM {}/{} MiB · температура {}",
            s.gpu_name.as_deref().unwrap_or("N/A"),
            opt(s.gpu_utilization_pct),
            opt(s.vram_used_mib),
            opt(s.vram_total_mib),
            s.gpu_temp_c
                .map(|v| format!("{v}°C"))
                .unwrap_or_else(|| "N/A".into())
        ));
        ui.checkbox(&mut self.settings.show_ram, "RAM");
        ui.label(format!(
            "{:.1}/{:.1} GiB",
            s.ram_used_bytes as f64 / 1_073_741_824.0,
            s.ram_total_bytes as f64 / 1_073_741_824.0
        ));
        ui.checkbox(&mut self.settings.show_disks, "Диски");
        for disk in &s.disks {
            ui.label(format!(
                "{} · {} · {:.1} GiB · чтение {:.2} MiB/s · запись {:.2} MiB/s",
                disk.name,
                disk.kind,
                disk.total_bytes as f64 / 1_073_741_824.0,
                disk.read_bytes_per_sec as f64 / 1_048_576.0,
                disk.written_bytes_per_sec as f64 / 1_048_576.0
            ));
        }
        ui.checkbox(&mut self.settings.show_network, "Сеть");
        for net in &s.networks {
            ui.label(format!(
                "{} · ↓ {:.2} MiB/s · ↑ {:.2} MiB/s",
                net.name,
                net.received_bytes_per_sec as f64 / 1_048_576.0,
                net.sent_bytes_per_sec as f64 / 1_048_576.0
            ));
        }
        ui.checkbox(&mut self.settings.show_llm, "LM Studio");
        ui.label(format!(
            "{} · {} · {}",
            llm.model.as_deref().unwrap_or("Модель не обнаружена"),
            phase_name(llm.phase),
            llm.speed_tps
                .map(|v| format!("{v:.2} токен/с"))
                .unwrap_or_default()
        ));
    }

    fn general_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Общие настройки · Portable");
        egui::ComboBox::from_label("Интервал обновления")
            .selected_text(format!("{} мс", self.settings.refresh_ms))
            .show_ui(ui, |ui| {
                for ms in [500, 1000, 2000] {
                    ui.selectable_value(&mut self.settings.refresh_ms, ms, format!("{ms} мс"));
                }
            });
        ui.label("Адрес API LM Studio");
        ui.text_edit_singleline(&mut self.settings.llm_api);
        ui.label("Путь к lms.exe (пусто — автообнаружение)");
        ui.text_edit_singleline(&mut self.settings.lms_path);
        ui.checkbox(
            &mut self.settings.show_taskbar,
            "Мониторинг в панели задач (индикатор загрузки CPU)",
        );
        let mut autostart = self.launch_at_login;
        if ui
            .checkbox(&mut autostart, "Запускать вместе с Windows")
            .changed()
        {
            self.update_autostart(autostart);
        }
        ui.label(
            "Настройки сохраняются в settings.json рядом с EXE и переживают обновление приложения.",
        );
    }
}

fn opt<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "N/A".into())
}

fn phase_name(phase: LlmPhase) -> &'static str {
    match phase {
        LlmPhase::Loading => "Загрузка",
        LlmPhase::Preparing => "Подготовка",
        LlmPhase::Pp => "Prompt processing",
        LlmPhase::Tg => "Генерация",
        LlmPhase::Unloading => "Выгрузка",
        LlmPhase::Idle => "Ожидание",
        LlmPhase::Offline => "LM Studio offline",
        LlmPhase::Unknown => "Статус неизвестен",
        LlmPhase::NoModel => "Нет загруженной модели",
    }
}
