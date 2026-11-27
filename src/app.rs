use crate::{
    config::{Settings, SpeedUnit},
    hardware::{ExtraSnapshot, Snapshot},
    llm::{LlmPhase, LlmSnapshot},
};
use eframe::egui;
use std::os::windows::ffi::OsStringExt;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct MonitorApp {
    settings: Settings,
    hardware: Arc<RwLock<Snapshot>>,
    extra: Arc<RwLock<ExtraSnapshot>>,
    llm: Arc<RwLock<LlmSnapshot>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    tab: usize,
    status: String,
    launch_at_login: bool,
    active_font_family: String,
    tray: Option<tray_icon::TrayIcon>,
    tray_open: tray_icon::menu::MenuId,
    tray_exit: tray_icon::menu::MenuId,
    exiting: bool,
    initial_visibility_pending: bool,
}

impl MonitorApp {
    pub fn new(
        settings: Settings,
        hardware: Arc<RwLock<Snapshot>>,
        extra: Arc<RwLock<ExtraSnapshot>>,
        llm: Arc<RwLock<LlmSnapshot>>,
        shutdown: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        let menu = tray_icon::menu::Menu::new();
        let open = tray_icon::menu::MenuItem::new("Открыть", true, None);
        let exit = tray_icon::menu::MenuItem::new("Выход", true, None);
        let tray_open = open.id().clone();
        let tray_exit = exit.id().clone();
        let _ = menu.append_items(&[&open, &exit]);
        let icon_data = icon_data();
        let tray = tray_icon::Icon::from_rgba(icon_data.rgba, icon_data.width, icon_data.height)
            .ok()
            .and_then(|icon| {
                tray_icon::TrayIconBuilder::new()
                    .with_tooltip("M4 PC Monitor")
                    .with_icon(icon)
                    .with_menu(Box::new(menu))
                    .build()
                    .ok()
            });
        Self {
            settings,
            hardware,
            extra,
            llm,
            shutdown,
            tab: 0,
            status: String::new(),
            launch_at_login: crate::startup::is_enabled(),
            active_font_family: String::new(),
            tray,
            tray_open,
            tray_exit,
            exiting: false,
            initial_visibility_pending: true,
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
        if self.initial_visibility_pending {
            self.initial_visibility_pending = false;
            if self.settings.start_in_tray && self.tray.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }
        while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == self.tray_open {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else if event.id == self.tray_exit {
                self.exiting = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            if matches!(
                event,
                tray_icon::TrayIconEvent::DoubleClick {
                    button: tray_icon::MouseButton::Left,
                    ..
                }
            ) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        if ctx.input(|input| input.viewport().close_requested())
            && !self.exiting
            && self.tray.is_some()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        ctx.request_repaint_after(Duration::from_millis(self.settings.refresh_ms.max(200)));
        if self.active_font_family != self.settings.font_family {
            let custom_path = self
                .settings
                .custom_fonts
                .get(&self.settings.font_family)
                .map(|file| portable_font_dir().join(file));
            apply_selected_font(ctx, &self.settings.font_family, custom_path.as_deref());
            self.active_font_family
                .clone_from(&self.settings.font_family);
        }
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
                &self.settings.font_family,
                self.settings.column_spacing_px,
                self.settings
                    .custom_fonts
                    .get(&self.settings.font_family)
                    .map(|file| portable_font_dir().join(file))
                    .as_deref(),
            );
        } else {
            crate::taskbar::hide_text_overlay(ctx);
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
                        egui::ComboBox::from_label("Шрифт")
                            .selected_text(&self.settings.font_family)
                            .show_ui(ui, |ui| {
                                for family in
                                    ["Segoe UI", "Arial", "Tahoma", "Consolas", "Cascadia Mono"]
                                {
                                    ui.selectable_value(
                                        &mut self.settings.font_family,
                                        family.to_owned(),
                                        family,
                                    );
                                }
                                for family in self.settings.custom_fonts.keys() {
                                    ui.selectable_value(
                                        &mut self.settings.font_family,
                                        family.clone(),
                                        family,
                                    );
                                }
                            });
                        if ui.button("Добавить шрифт…").clicked() {
                            match choose_font_file() {
                                Ok(Some(path)) => match install_custom_font(&path) {
                                    Ok((family, filename)) => {
                                        self.settings.custom_fonts.insert(family.clone(), filename);
                                        self.settings.font_family = family.clone();
                                        self.active_font_family.clear();
                                        self.save();
                                        self.status =
                                            format!("Шрифт {family} добавлен в Portable и выбран");
                                    }
                                    Err(error) => {
                                        self.status = format!("Не удалось добавить шрифт: {error}")
                                    }
                                },
                                Ok(None) => {}
                                Err(error) => {
                                    self.status =
                                        format!("Не удалось открыть выбор шрифта: {error}")
                                }
                            }
                        }
                        ui.add(
                            egui::Slider::new(&mut self.settings.font_size, 9.0..=24.0)
                                .text("Размер текста"),
                        );
                        ui.color_edit_button_srgb(&mut self.settings.text_color);
                        ui.checkbox(&mut self.settings.dark_theme, "Тёмная тема");
                    }
                    3 => self.llm_page(ui),
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

fn apply_selected_font(ctx: &egui::Context, family: &str, custom_path: Option<&Path>) {
    if let Some(path) = custom_path {
        match std::fs::read(path) {
            Ok(bytes) => {
                let mut fonts = egui::FontDefinitions::default();
                let key = format!("m4-portable-font-{family}");
                fonts.font_data.insert(
                    key.clone(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, key);
                ctx.set_fonts(fonts);
                return;
            }
            Err(error) => tracing::warn!("Could not load custom font {}: {error}", path.display()),
        }
    }
    let file = match family {
        "Arial" => "arial.ttf",
        "Tahoma" => "tahoma.ttf",
        "Consolas" => "consola.ttf",
        "Cascadia Mono" => "CascadiaMono.ttf",
        _ => "segoeui.ttf",
    };
    let path = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts")
        .join(file);
    let mut fonts = egui::FontDefinitions::default();
    match std::fs::read(&path) {
        Ok(bytes) => {
            let key = format!("m4-system-font-{family}");
            fonts.font_data.insert(
                key.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, key);
        }
        Err(error) => tracing::warn!("Could not load font {}: {error}", path.display()),
    }
    ctx.set_fonts(fonts);
}

fn portable_font_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fonts")
}

fn install_custom_font(source: &Path) -> anyhow::Result<(String, String)> {
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    anyhow::ensure!(
        extension == "ttf" || extension == "otf",
        "поддерживаются TTF и OTF"
    );
    let bytes = std::fs::read(source)?;
    let face = ttf_parser::Face::parse(&bytes, 0).map_err(|error| {
        anyhow::anyhow!("файл шрифта повреждён или не поддерживается: {error:?}")
    })?;
    let family = face
        .names()
        .into_iter()
        .filter(|name| {
            name.name_id == ttf_parser::name_id::TYPOGRAPHIC_FAMILY
                || name.name_id == ttf_parser::name_id::FAMILY
        })
        .find_map(|name| name.to_string())
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| anyhow::anyhow!("в файле не найдено имя семейства шрифта"))?;

    let directory = portable_font_dir();
    std::fs::create_dir_all(&directory)?;
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("custom-font");
    let mut filename = format!("{stem}.{extension}");
    let mut suffix = 2;
    while directory.join(&filename).exists() {
        filename = format!("{stem}-{suffix}.{extension}");
        suffix += 1;
    }
    std::fs::copy(source, directory.join(&filename))?;
    Ok((family, filename))
}

fn choose_font_file() -> anyhow::Result<Option<PathBuf>> {
    use windows::{
        core::{PCWSTR, PWSTR},
        Win32::{
            Foundation::HWND,
            UI::Controls::Dialogs::{
                GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR,
                OFN_PATHMUSTEXIST, OPENFILENAMEW,
            },
        },
    };

    let filter: Vec<u16> = "Font files (*.ttf;*.otf)\0*.ttf;*.otf\0All files (*.*)\0*.*\0\0"
        .encode_utf16()
        .collect();
    let title: Vec<u16> = "Добавить шрифт в M4 PC Monitor\0".encode_utf16().collect();
    let mut buffer = vec![0u16; 32_768];
    let mut dialog = OPENFILENAMEW::default();
    dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = HWND::default();
    dialog.lpstrFilter = PCWSTR(filter.as_ptr());
    dialog.lpstrFile = PWSTR(buffer.as_mut_ptr());
    dialog.nMaxFile = buffer.len() as u32;
    dialog.lpstrTitle = PCWSTR(title.as_ptr());
    dialog.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    if !unsafe { GetOpenFileNameW(&mut dialog) }.as_bool() {
        return Ok(None);
    }
    let end = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..end],
    ))))
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
            "{} {}{:.1}/{:.1} GiB",
            x.memory_type.as_deref().unwrap_or("RAM"),
            x.memory_speed_mhz
                .map(|mhz| format!("{:.1}GHz ", mhz as f64 / 1000.0))
                .unwrap_or_default(),
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
            let mut selected_unit = self
                .settings
                .disk_speed_units
                .get(&d.id)
                .copied()
                .unwrap_or_default();
            let mut visible = self
                .settings
                .disk_visibility
                .get(&d.id)
                .copied()
                .unwrap_or(true);
            ui.horizontal(|ui| {
                ui.checkbox(&mut visible, "");
                self.edit_name(ui, &id, &d.name);
                speed_unit_selector(ui, &format!("disk-speed-unit-{}", d.id), &mut selected_unit);
            });
            self.settings.disk_visibility.insert(d.id.clone(), visible);
            self.settings
                .disk_speed_units
                .insert(d.id.clone(), selected_unit);
            ui.label(format!(
                "{} · {:.1} GiB · чтение {:.2} {} · запись {:.2} {}",
                d.kind,
                d.total_bytes as f64 / 1073741824.0,
                selected_unit.value(d.read_bytes_per_sec),
                selected_unit.label(),
                selected_unit.value(d.written_bytes_per_sec),
                selected_unit.label()
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_network, "");
            self.edit_name(ui, "network", "Сеть (суммарно)");
        });
        ui.horizontal(|ui| {
            ui.label("Скорость суммы:");
            speed_unit_selector(
                ui,
                "aggregate-network-speed-unit",
                &mut self.settings.network_speed_unit,
            );
        });
        for n in &s.networks {
            let mut visible = self
                .settings
                .network_visibility
                .get(&n.name)
                .copied()
                .unwrap_or(true);
            ui.horizontal(|ui| {
                ui.checkbox(&mut visible, &n.name);
            });
            self.settings
                .network_visibility
                .insert(n.name.clone(), visible);
            ui.label(format!(
                "↓ {:.2} {} · ↑ {:.2} {}",
                self.settings
                    .network_speed_unit
                    .value(n.received_bytes_per_sec),
                self.settings.network_speed_unit.label(),
                self.settings.network_speed_unit.value(n.sent_bytes_per_sec),
                self.settings.network_speed_unit.label()
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
            "Здесь показаны только включённые блоки и устройства. Порядок сохраняется рядом с EXE.",
        );
        ui.add(
            egui::Slider::new(&mut self.settings.column_spacing_px, 0..=100)
                .text("Отступ между колонками")
                .suffix(" px"),
        );
        let mut items: Vec<(String, String, String)> = Vec::new();
        if self.settings.show_network {
            items.push((
                "network".into(),
                self.name("network", "Сеть"),
                String::new(),
            ));
        }
        if self.settings.show_cpu || self.settings.show_ram {
            items.push(("cpu_ram".into(), "CPU + RAM".into(), String::new()));
        }
        if self.settings.show_gpu || self.settings.show_llm {
            items.push(("gpu_llm".into(), "GPU + LLM".into(), String::new()));
        }
        if self.settings.show_motherboard {
            items.push((
                "motherboard".into(),
                self.name("motherboard", "Материнская плата"),
                String::new(),
            ));
        }
        if self.settings.show_disks {
            items.extend(s.disks.iter().filter_map(|disk| {
                self.settings
                    .disk_visibility
                    .get(&disk.id)
                    .copied()
                    .unwrap_or(true)
                    .then(|| {
                        let id = format!("disk:{}", disk.id);
                        (id.clone(), self.name(&id, &disk.name), String::new())
                    })
            }));
        }
        if items.is_empty() {
            ui.weak("Нет включённых блоков для панели задач.");
            return;
        }
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
        egui::ComboBox::from_label("Открывать программу")
            .selected_text(if self.settings.start_in_tray {
                "В трее"
            } else {
                "В окне"
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.settings.start_in_tray, true, "В трее");
                ui.selectable_value(&mut self.settings.start_in_tray, false, "В окне");
            });
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
    if settings.show_cpu || settings.show_ram {
        let cpu_line = if settings.show_cpu {
            format!(
                "{}: {:.0}% {} {:.1} GHz",
                name("cpu", "CPU"),
                s.cpu_usage,
                x.cpu_temperature_c
                    .map(|temperature| format!("{temperature:.0}°C"))
                    .unwrap_or_else(|| "N/A".into()),
                s.cpu_frequency_mhz as f64 / 1000.0
            )
        } else {
            String::new()
        };
        let ram_line = if settings.show_ram {
            let ram_type = x.memory_type.as_deref().unwrap_or("RAM");
            let speed = x
                .memory_speed_mhz
                .map(|mhz| format!("{:.1}GHz ", mhz as f64 / 1000.0))
                .unwrap_or_default();
            format!(
                "{} {}{:.1}/{:.1} GiB",
                name("ram", ram_type),
                speed,
                s.ram_used_bytes as f64 / 1073741824.0,
                s.ram_total_bytes as f64 / 1073741824.0
            )
        } else {
            String::new()
        };
        v.push(("cpu_ram".into(), cpu_line, ram_line));
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
    let gpu_line = if settings.show_gpu {
        format!(
            "{}: {}% {} {}/{} GiB",
            name("gpu", "GPU"),
            opt(s.gpu_utilization_pct),
            s.gpu_temp_c
                .map(|temperature| format!("{temperature}°C"))
                .unwrap_or_else(|| "N/A".into()),
            s.vram_used_mib
                .map(|value| format!("{:.1}", value as f64 / 1024.0))
                .unwrap_or_else(|| "N/A".into()),
            s.vram_total_mib
                .map(|value| format!("{:.0}", value as f64 / 1024.0))
                .unwrap_or_else(|| "N/A".into())
        )
    } else {
        String::new()
    };
    let llm_line = if settings.show_llm {
        format!(
            "{}: {} · {:.1} t/s",
            name("llm", "LLM"),
            l.model.as_deref().unwrap_or("NO MODEL"),
            l.speed_tps.unwrap_or(0.0)
        )
    } else {
        String::new()
    };
    if settings.show_gpu || settings.show_llm {
        v.push(("gpu_llm".into(), gpu_line, llm_line));
    }
    if settings.show_disks {
        for d in &s.disks {
            if settings.disk_visibility.get(&d.id).copied().unwrap_or(true) {
                let id = format!("disk:{}", d.id);
                let unit = settings
                    .disk_speed_units
                    .get(&d.id)
                    .copied()
                    .unwrap_or_default();
                v.push((
                    id.clone(),
                    name(&id, &d.name),
                    format!(
                        "R/W {:.1}/{:.1} {}",
                        unit.value(d.read_bytes_per_sec),
                        unit.value(d.written_bytes_per_sec),
                        unit.label()
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
                "↓ {:.2} {}",
                settings.network_speed_unit.value(received),
                settings.network_speed_unit.label()
            ),
            format!(
                "↑ {:.2} {}",
                settings.network_speed_unit.value(sent),
                settings.network_speed_unit.label()
            ),
        ));
    }
    v
}

fn speed_unit_selector(ui: &mut egui::Ui, id: &str, unit: &mut SpeedUnit) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(unit.label())
        .width(78.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(unit, SpeedUnit::MebiBytesPerSecond, "MiB/s");
            ui.selectable_value(unit, SpeedUnit::MegaBitsPerSecond, "Mb/s");
        });
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
