use crate::{
    config::Settings,
    hardware::{ExtraSnapshot, Snapshot},
    llm::{LlmPhase, LlmSnapshot},
};
use eframe::egui;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

const BLOCKS: &[(&str, &str)] = &[
    ("cpu", "CPU"),
    ("gpu", "GPU"),
    ("ram", "RAM"),
    ("network", "Сеть"),
    ("motherboard", "Материнская плата"),
    ("llm", "LM Studio"),
];

pub struct MonitorApp {
    settings: Settings,
    hardware: Arc<RwLock<Snapshot>>,
    extra: Arc<RwLock<ExtraSnapshot>>,
    llm: Arc<RwLock<LlmSnapshot>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    tab: usize,
    status: String,
    launch_at_login: bool,
}

impl MonitorApp {
    pub fn new(
        settings: Settings,
        hardware: Arc<RwLock<Snapshot>>,
        extra: Arc<RwLock<ExtraSnapshot>>,
        llm: Arc<RwLock<LlmSnapshot>>,
        shutdown: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            settings,
            hardware,
            extra,
            llm,
            shutdown,
            tab: 0,
            status: String::new(),
            launch_at_login: crate::startup::is_enabled(),
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
            Err(e) => self.status = format!("Не удалось изменить автозапуск: {e}"),
        }
    }
    fn name(&self, key: &str, fallback: &str) -> String {
        self.settings
            .custom_names
            .get(key)
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| fallback.into())
    }
    fn edit_name(&mut self, ui: &mut egui::Ui, key: &str, fallback: &str) {
        let value = self.settings.custom_names.entry(key.into()).or_default();
        ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(fallback)
                .desired_width(150.0),
        );
    }
}

impl Drop for MonitorApp {
    fn drop(&mut self) {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Release);
    }
}

pub fn icon_data() -> egui::IconData {
    const N: usize = 32;
    let mut rgba = vec![0; N * N * 4];
    for p in rgba.chunks_exact_mut(4) {
        p.copy_from_slice(&[24, 29, 40, 255]);
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
    let o = (y * 32 + x) * 4;
    rgba[o..o + 4].copy_from_slice(&color);
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(self.settings.refresh_ms.max(200)));
        let s = self.hardware.read().map(|v| v.clone()).unwrap_or_default();
        let x = self.extra.read().map(|v| v.clone()).unwrap_or_default();
        let l = self.llm.read().map(|v| v.clone()).unwrap_or_default();
        let items = taskbar_items(&self.settings, &s, &x, &l);
        let order = ordered_keys(&mut self.settings.taskbar_order, &items);
        let visible: Vec<String> = order
            .iter()
            .filter_map(|k| {
                items
                    .iter()
                    .find(|(id, _, _)| id == k)
                    .map(|(_, a, b)| format!("{a}\n{b}"))
            })
            .collect();
        if self.settings.show_taskbar {
            crate::taskbar::show_text_overlay(
                ctx,
                visible,
                self.settings.font_size,
                self.settings.text_color,
            );
        }
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("M4 PC Monitor");
                ui.label("Portable · Windows 11 x64");
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for (i, title) in [
                    "Датчики",
                    "Расположение",
                    "Оформление",
                    "Единицы",
                    "LM Studio",
                    "Общие",
                ]
                .iter()
                .enumerate()
                {
                    if ui.selectable_label(self.tab == i, *title).clicked() {
                        self.tab = i;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.tab {
                    0 => self.sensor_page(ui, &s, &x, &l),
                    1 => self.layout_page(ui, &s),
                    2 => {
                        ui.heading("Оформление");
                        ui.add(
                            egui::Slider::new(&mut self.settings.font_size, 9.0..=24.0)
                                .text("Размер текста"),
                        );
                        ui.color_edit_button_srgb(&mut self.settings.text_color);
                        ui.checkbox(&mut self.settings.dark_theme, "Тёмная тема");
                    }
                    3 => {
                        ui.heading("Единицы измерения");
                        ui.label("Скорости: MiB/s; память: GiB; частота: MHz; температура: °C.");
                    }
                    4 => self.llm_page(ui),
                    _ => self.general_page(ui),
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Сохранить настройки").clicked() {
                        self.save();
                    }
                    ui.label(&self.status);
                });
            });
        });
    }
}

impl MonitorApp {
    fn sensor_page(&mut self, ui: &mut egui::Ui, s: &Snapshot, x: &ExtraSnapshot, l: &LlmSnapshot) {
        ui.heading("Датчики и обнаруженное оборудование");
        ui.label("Оборудование определяется автоматически; выберите блоки и устройства для панели задач.");
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_cpu, "");
            self.edit_name(ui, "cpu", "CPU");
        });
        ui.label(format!(
            "{} · {} потоков · {:.0}% · {} MHz · температура {}",
            s.cpu_name,
            s.cpu_threads,
            s.cpu_usage,
            s.cpu_frequency_mhz,
            x.cpu_temperature_c
                .map(|v| format!("{v:.0}°C"))
                .unwrap_or_else(|| "N/A".into())
        ));
        if let Some(error) = &x.provider_error {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Провайдер датчиков: {error}"),
            );
        } else {
            ui.weak(format!(
                "Источник температуры: {}",
                x.cpu_temperature_sensor
                    .as_deref()
                    .unwrap_or("ожидание данных")
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_gpu, "");
            self.edit_name(ui, "gpu", "NVIDIA GPU");
        });
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
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_ram, "");
            self.edit_name(ui, "ram", "RAM");
        });
        ui.label(format!(
            "{:.1}/{:.1} GiB",
            s.ram_used_bytes as f64 / 1073741824.0,
            s.ram_total_bytes as f64 / 1073741824.0
        ));
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_motherboard, "");
            self.edit_name(ui, "motherboard", "Материнская плата");
        });
        ui.label(
            x.motherboard_name
                .as_deref()
                .unwrap_or("Материнская плата не обнаружена"),
        );
        for sensor in &x.motherboard_sensors {
            ui.label(format!(
                "{} {}: {:.0}{}",
                sensor.kind,
                sensor.name,
                sensor.value,
                if sensor.kind == "Fan" { " RPM" } else { "°C" }
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_disks, "");
            ui.label("Диски");
        });
        for d in &s.disks {
            let id = format!("disk:{}", d.id);
            let mut visible = self
                .settings
                .disk_visibility
                .get(&d.id)
                .copied()
                .unwrap_or(true);
            ui.horizontal(|ui| {
                ui.checkbox(&mut visible, "");
                self.edit_name(ui, &id, &d.name);
            });
            self.settings.disk_visibility.insert(d.id.clone(), visible);
            ui.label(format!(
                "{} · {:.1} GiB · чтение {:.2} MiB/s · запись {:.2} MiB/s",
                d.kind,
                d.total_bytes as f64 / 1073741824.0,
                d.read_bytes_per_sec as f64 / 1048576.0,
                d.written_bytes_per_sec as f64 / 1048576.0
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_network, "");
            self.edit_name(ui, "network", "Сеть (суммарно)");
        });
        for n in &s.networks {
            let id = format!("network:{}", n.name);
            let mut visible = self
                .settings
                .network_visibility
                .get(&n.name)
                .copied()
                .unwrap_or(true);
            ui.horizontal(|ui| {
                ui.checkbox(&mut visible, "");
                self.edit_name(ui, &id, &n.name);
            });
            self.settings
                .network_visibility
                .insert(n.name.clone(), visible);
            ui.label(format!(
                "↓ {:.2} MiB/s · ↑ {:.2} MiB/s",
                n.received_bytes_per_sec as f64 / 1048576.0,
                n.sent_bytes_per_sec as f64 / 1048576.0
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_llm, "");
            ui.label(self.name("llm", "LM Studio"));
        });
        ui.label(format!(
            "{} · {} · {}",
            l.model.as_deref().unwrap_or("Модель не обнаружена"),
            phase_name(l.phase),
            l.speed_tps
                .map(|v| format!("{v:.2} токен/с"))
                .unwrap_or_default()
        ));
    }
    fn layout_page(&mut self, ui: &mut egui::Ui, s: &Snapshot) {
        ui.heading("Порядок показателей в панели задач");
        ui.label(
            "Перемещайте элементы кнопками ↑ и ↓. Порядок сохраняется в settings.json рядом с EXE.",
        );
        let mut items: Vec<(String, String, String)> = BLOCKS
            .iter()
            .map(|(id, n)| ((*id).into(), (*n).into(), String::new()))
            .collect();
        items.extend(s.disks.iter().map(|d| {
            (
                format!("disk:{}", d.id),
                self.name(&format!("disk:{}", d.id), &d.name),
                String::new(),
            )
        }));
        let keys = ordered_keys(&mut self.settings.taskbar_order, &items);
        for i in 0..keys.len() {
            let key = &keys[i];
            let label = items
                .iter()
                .find(|x| &x.0 == key)
                .map(|x| x.1.as_str())
                .unwrap_or(key);
            ui.horizontal(|ui| {
                ui.label(label);
                if ui.small_button("↑").clicked() && i > 0 {
                    self.settings.taskbar_order.swap(i, i - 1);
                }
                if ui.small_button("↓").clicked() && i + 1 < keys.len() {
                    self.settings.taskbar_order.swap(i, i + 1);
                }
            });
        }
    }
    fn llm_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Настройки LM Studio");
        ui.label("Название блока");
        self.edit_name(ui, "llm", "LM Studio");
        ui.label("Адрес API");
        ui.text_edit_singleline(&mut self.settings.llm_api);
        ui.label("Путь к lms.exe (пусто — автообнаружение)");
        ui.text_edit_singleline(&mut self.settings.lms_path);
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
        ui.checkbox(
            &mut self.settings.show_taskbar,
            "Показывать текстовый мониторинг в панели задач",
        );
        let mut a = self.launch_at_login;
        if ui.checkbox(&mut a, "Запускать вместе с Windows").changed() {
            self.update_autostart(a);
        }
        ui.label("Настройки сохраняются рядом с EXE и переживают обновление Portable.");
    }
}

fn taskbar_items(
    settings: &Settings,
    s: &Snapshot,
    x: &ExtraSnapshot,
    l: &LlmSnapshot,
) -> Vec<(String, String, String)> {
    let mut v = Vec::new();
    let name = |k: &str, d: &str| {
        settings
            .custom_names
            .get(k)
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| d.into())
    };
    if settings.show_cpu {
        v.push((
            "cpu".into(),
            format!(
                "{}: {:.0}% {}",
                name("cpu", "CPU"),
                s.cpu_usage,
                x.cpu_temperature_c
                    .map(|n| format!("{n:.0}°C"))
                    .unwrap_or_else(|| "N/A".into())
            ),
            format!("{} MHz", s.cpu_frequency_mhz),
        ));
    }
    if settings.show_gpu {
        v.push((
            "gpu".into(),
            format!(
                "{}: {}% {}",
                name("gpu", "GPU"),
                opt(s.gpu_utilization_pct),
                s.gpu_temp_c
                    .map(|n| format!("{n}°C"))
                    .unwrap_or_else(|| "N/A".into())
            ),
            format!(
                "VRAM {}/{} MiB",
                opt(s.vram_used_mib),
                opt(s.vram_total_mib)
            ),
        ));
    }
    if settings.show_ram {
        v.push((
            "ram".into(),
            name("ram", "RAM"),
            format!(
                "{:.1}/{:.1} GiB",
                s.ram_used_bytes as f64 / 1073741824.0,
                s.ram_total_bytes as f64 / 1073741824.0
            ),
        ));
    }
    if settings.show_motherboard {
        if let Some(board) = &x.motherboard_name {
            let vals = x
                .motherboard_sensors
                .iter()
                .filter(|q| q.kind == "Temperature")
                .take(2)
                .map(|q| format!("{} {:.0}°", q.name, q.value))
                .collect::<Vec<_>>()
                .join(" · ");
            v.push(("motherboard".into(), name("motherboard", board), vals));
        }
    }
    if settings.show_llm {
        v.push((
            "llm".into(),
            name("llm", "LLM"),
            format!(
                "{} · {:.1} t/s",
                l.model.as_deref().unwrap_or("NO MODEL"),
                l.speed_tps.unwrap_or(0.0)
            ),
        ));
    }
    if settings.show_disks {
        for d in &s.disks {
            if settings.disk_visibility.get(&d.id).copied().unwrap_or(true) {
                let id = format!("disk:{}", d.id);
                v.push((
                    id.clone(),
                    name(&id, &d.name),
                    format!(
                        "R/W {:.1}/{:.1} MiB/s",
                        d.read_bytes_per_sec as f64 / 1048576.0,
                        d.written_bytes_per_sec as f64 / 1048576.0
                    ),
                ));
            }
        }
    }
    if settings.show_network {
        let (sent, received) = s
            .networks
            .iter()
            .filter(|n| {
                settings
                    .network_visibility
                    .get(&n.name)
                    .copied()
                    .unwrap_or(true)
            })
            .fold((0_u64, 0_u64), |(tx, rx), n| {
                (
                    tx.saturating_add(n.sent_bytes_per_sec),
                    rx.saturating_add(n.received_bytes_per_sec),
                )
            });
        v.push((
            "network".into(),
            format!(
                "{} ↑ {:.2} MiB/s",
                name("network", "Сеть"),
                sent as f64 / 1048576.0
            ),
            format!("↓ {:.2} MiB/s", received as f64 / 1048576.0),
        ));
    }
    v
}
fn ordered_keys(order: &mut Vec<String>, items: &[(String, String, String)]) -> Vec<String> {
    let valid: Vec<String> = items.iter().map(|i| i.0.clone()).collect();
    order.retain(|k| valid.contains(k));
    for k in &valid {
        if !order.contains(k) {
            order.push(k.clone());
        }
    }
    order.clone()
}
fn opt<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map(|x| x.to_string()).unwrap_or_else(|| "N/A".into())
}
fn phase_name(p: LlmPhase) -> &'static str {
    match p {
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
