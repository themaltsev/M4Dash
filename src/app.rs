use crate::{
    config::{Settings, SpeedUnit, ThemeMode},
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

static TASKBAR_ACTION: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub(crate) fn taskbar_menu_action(action: u8) {
    TASKBAR_ACTION.fetch_or(action, std::sync::atomic::Ordering::Release);
    wake_native_window();
}

pub struct MonitorApp {
    settings: Settings,
    hardware: Arc<RwLock<Snapshot>>,
    extra: Arc<RwLock<ExtraSnapshot>>,
    llm: Arc<RwLock<LlmSnapshot>>,
    overlay_settings: Arc<RwLock<Settings>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    tab: usize,
    status: String,
    launch_at_login: bool,
    active_language: Option<crate::config::Language>,
    tray: Option<tray_icon::TrayIcon>,
    tray_open: tray_icon::menu::MenuId,
    tray_exit: tray_icon::menu::MenuId,
    exiting: bool,
    initial_visibility_pending: bool,
    tray_actions: Arc<std::sync::atomic::AtomicU8>,
    tray_handlers_registered: bool,
    settings_save_deadline: Option<std::time::Instant>,
    system_theme_dark: bool,
    system_theme_checked_at: std::time::Instant,
    system_accent_color: [u8; 3],
    applied_titlebar_dark: Option<bool>,
}

impl MonitorApp {
    pub fn new(
        settings: Settings,
        hardware: Arc<RwLock<Snapshot>>,
        extra: Arc<RwLock<ExtraSnapshot>>,
        llm: Arc<RwLock<LlmSnapshot>>,
        shutdown: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        crate::updater::start_check();
        let system_theme_dark = crate::config::system_prefers_dark_theme();
        let system_accent_color = crate::config::system_accent_color().unwrap_or([0, 120, 215]);
        let overlay_settings = Arc::new(RwLock::new(settings.clone()));
        start_taskbar_refresh(
            overlay_settings.clone(),
            hardware.clone(),
            extra.clone(),
            llm.clone(),
            shutdown.clone(),
        );
        let menu = tray_icon::menu::Menu::new();
        let open = tray_icon::menu::MenuItem::new(
            crate::i18n::tr(settings.language, "Открыть"),
            true,
            None,
        );
        let exit =
            tray_icon::menu::MenuItem::new(crate::i18n::tr(settings.language, "Выход"), true, None);
        let tray_open = open.id().clone();
        let tray_exit = exit.id().clone();
        let _ = menu.append_items(&[&open, &exit]);
        let icon_data = icon_data();
        let tray = tray_icon::Icon::from_rgba(icon_data.rgba, icon_data.width, icon_data.height)
            .ok()
            .and_then(|icon| {
                tray_icon::TrayIconBuilder::new()
                    .with_tooltip("M4Dash")
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
            overlay_settings,
            shutdown,
            tab: 0,
            status: String::new(),
            launch_at_login: crate::startup::is_enabled(),
            active_language: None,
            tray,
            tray_open,
            tray_exit,
            exiting: false,
            initial_visibility_pending: true,
            tray_actions: Arc::new(std::sync::atomic::AtomicU8::new(0)),
            tray_handlers_registered: false,
            settings_save_deadline: None,
            system_theme_dark,
            system_theme_checked_at: std::time::Instant::now(),
            system_accent_color,
            applied_titlebar_dark: None,
        }
    }
    fn save(&mut self) {
        self.status = match self.settings.save() {
            Ok(()) => crate::i18n::tr(
                self.settings.language,
                "Изменения автоматически сохранены рядом с приложением",
            )
            .into(),
            Err(e) => format!(
                "{}: {e}",
                crate::i18n::tr(self.settings.language, "Ошибка сохранения")
            ),
        };
    }
    fn update_autostart(&mut self, enabled: bool) {
        match crate::startup::set_enabled(enabled) {
            Ok(()) => {
                self.launch_at_login = enabled;
                self.status = if enabled {
                    crate::i18n::tr(self.settings.language, "Автозапуск включён")
                } else {
                    crate::i18n::tr(self.settings.language, "Автозапуск выключен")
                }
                .into();
            }
            Err(e) => {
                self.status = format!(
                    "{}: {e}",
                    crate::i18n::tr(self.settings.language, "Не удалось изменить автозапуск")
                )
            }
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
        if let Err(error) = self.settings.save() {
            tracing::warn!("Could not save portable settings on exit: {error}");
        }
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Release);
    }
}

pub fn icon_data() -> egui::IconData {
    egui::IconData {
        rgba: include_bytes!("../assets/icons/app-64.rgba").to_vec(),
        width: 64,
        height: 64,
    }
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use std::sync::atomic::Ordering;
        let language = self.settings.language;
        let previous_settings = self.settings.clone();
        if let Ok(mut overlay_settings) = self.overlay_settings.write() {
            *overlay_settings = self.settings.clone();
        }
        if !self.tray_handlers_registered {
            self.tray_handlers_registered = true;
            let actions = self.tray_actions.clone();
            let open_id = self.tray_open.clone();
            let exit_id = self.tray_exit.clone();
            let wake = ctx.clone();
            tray_icon::menu::MenuEvent::set_event_handler(Some(
                move |event: tray_icon::menu::MenuEvent| {
                    let flag = if event.id == open_id {
                        1
                    } else if event.id == exit_id {
                        2
                    } else {
                        0
                    };
                    if flag != 0 {
                        actions.fetch_or(flag, Ordering::Release);
                        // A hidden winit window may not receive redraws. Make it
                        // visible through Win32 first so egui can process the action.
                        wake_native_window();
                        wake.request_repaint();
                    }
                },
            ));
            let actions = self.tray_actions.clone();
            let wake = ctx.clone();
            tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
                if matches!(
                    event,
                    tray_icon::TrayIconEvent::DoubleClick {
                        button: tray_icon::MouseButton::Left,
                        ..
                    }
                ) {
                    actions.fetch_or(1, Ordering::Release);
                    wake_native_window();
                    wake.request_repaint();
                }
            }));
        }
        if self.initial_visibility_pending {
            self.initial_visibility_pending = false;
            if self.settings.start_in_tray && self.tray.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }
        let actions =
            self.tray_actions.swap(0, Ordering::AcqRel) | TASKBAR_ACTION.swap(0, Ordering::AcqRel);
        if crate::updater::update_ready() {
            self.exiting = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if actions & 2 != 0 {
            self.exiting = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if actions & 1 != 0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if ctx.input(|input| input.viewport().close_requested())
            && !self.exiting
            && self.tray.is_some()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        ctx.request_repaint_after(Duration::from_millis(self.settings.refresh_ms.max(200)));
        if self.system_theme_checked_at.elapsed() >= Duration::from_secs(1) {
            if self.settings.theme_mode == ThemeMode::System {
                self.system_theme_dark = crate::config::system_prefers_dark_theme();
            }
            self.system_accent_color =
                crate::config::system_accent_color().unwrap_or([0, 120, 215]);
            self.system_theme_checked_at = std::time::Instant::now();
        }
        if self.active_language != Some(language) {
            apply_windows_ui_style(ctx, language);
            self.active_language = Some(language);
        }
        let s = self.hardware.read().map(|v| v.clone()).unwrap_or_default();
        let x = self.extra.read().map(|v| v.clone()).unwrap_or_default();
        let l = self.llm.read().map(|v| v.clone()).unwrap_or_default();
        let items = grouped_items(&self.settings, taskbar_items(&self.settings, &s, &x, &l));
        let order = ordered_keys(&mut self.settings.taskbar_order, &items);
        let visible: Vec<(String, u16)> = order
            .iter()
            .filter_map(|k| {
                items
                    .iter()
                    .find(|(id, _, _)| id == k)
                    .filter(|(_, a, b)| !a.is_empty() || !b.is_empty())
                    .map(|(id, a, b)| {
                        let width = id
                            .strip_prefix("group:")
                            .and_then(|x| x.parse::<usize>().ok())
                            .and_then(|index| self.settings.layout_groups.get(index))
                            .map(|group| group.width_chars)
                            .unwrap_or(28);
                        (format!("{a}\n{b}"), width)
                    })
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
            crate::taskbar::hide_text_overlay();
        }
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("M4Dash");
                ui.label("Windows 11 x64");
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for (i, title) in ["Датчики", "Расположение", "Общие"].iter().enumerate()
                {
                    if ui
                        .selectable_label(self.tab == i, crate::i18n::tr(language, title))
                        .clicked()
                    {
                        self.tab = i;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.tab {
                    0 => self.sensor_page(ui, &s, &x, &l),
                    1 => self.layout_page(ui, &s, &x),
                    _ => self.general_page(ui),
                }
                ui.separator();
                if !self.status.is_empty() {
                    ui.label(&self.status);
                }
            });
        });
        if self.settings != previous_settings {
            self.settings_save_deadline =
                Some(std::time::Instant::now() + Duration::from_millis(350));
        }
        if self
            .settings_save_deadline
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
        {
            self.settings_save_deadline = None;
            self.save();
        }
        let dark_theme = self.settings.theme_mode.is_dark(self.system_theme_dark);
        if self.applied_titlebar_dark != Some(dark_theme) && set_windows_titlebar_theme(dark_theme)
        {
            self.applied_titlebar_dark = Some(dark_theme);
        }
        let visuals = windows_visuals(dark_theme, self.system_accent_color);
        if ctx.style().visuals != visuals {
            ctx.set_visuals(visuals);
        }
    }
}

fn set_windows_titlebar_theme(dark: bool) -> bool {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::COLORREF,
            Graphics::Dwm::{
                DwmSetWindowAttribute, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
            },
            UI::WindowsAndMessaging::FindWindowW,
        },
    };

    let title: Vec<u16> = "M4Dash\0".encode_utf16().collect();
    let Ok(hwnd) = (unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) }) else {
        return false;
    };
    let (caption_rgb, text_rgb) = if dark {
        ([31, 32, 35], [245, 245, 245])
    } else {
        ([240, 240, 240], [20, 20, 20])
    };
    let to_colorref =
        |rgb: [u8; 3]| COLORREF(rgb[0] as u32 | ((rgb[1] as u32) << 8) | ((rgb[2] as u32) << 16));
    let caption = to_colorref(caption_rgb);
    let text = to_colorref(text_rgb);
    let dark_mode = i32::from(dark);
    unsafe {
        // Keep the native title bar aligned with the app's explicit theme,
        // even when the Windows-wide appearance is set to the opposite mode.
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark_mode as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            (&caption as *const COLORREF).cast(),
            std::mem::size_of::<COLORREF>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TEXT_COLOR,
            (&text as *const COLORREF).cast(),
            std::mem::size_of::<COLORREF>() as u32,
        );
    }
    true
}

fn windows_visuals(dark: bool, accent_rgb: [u8; 3]) -> egui::Visuals {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let accent = egui::Color32::from_rgb(accent_rgb[0], accent_rgb[1], accent_rgb[2]);
    let accent_soft = egui::Color32::from_rgba_unmultiplied(
        accent_rgb[0],
        accent_rgb[1],
        accent_rgb[2],
        if dark { 52 } else { 34 },
    );
    let (background, control, border) = if dark {
        (
            egui::Color32::from_rgb(31, 32, 35),
            egui::Color32::from_rgb(41, 42, 46),
            egui::Color32::from_rgb(86, 88, 94),
        )
    } else {
        (
            egui::Color32::from_rgb(240, 240, 240),
            egui::Color32::WHITE,
            egui::Color32::from_rgb(130, 132, 136),
        )
    };
    let foreground = if dark {
        egui::Color32::from_rgb(245, 245, 245)
    } else {
        egui::Color32::from_rgb(20, 20, 20)
    };

    visuals.window_fill = background;
    visuals.panel_fill = background;
    visuals.window_stroke = egui::Stroke::new(1.0_f32, border);
    visuals.override_text_color = Some(foreground);
    visuals.extreme_bg_color = control;
    visuals.code_bg_color = control;
    visuals.faint_bg_color = if dark {
        egui::Color32::from_rgb(38, 39, 43)
    } else {
        egui::Color32::from_rgb(247, 247, 247)
    };
    visuals.hyperlink_color = accent;
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = egui::Stroke::new(
        1.0_f32,
        if dark {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(20, 20, 20)
        },
    );
    visuals.widgets.noninteractive.bg_fill = background;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, border);
    visuals.widgets.noninteractive.fg_stroke.color = foreground;
    visuals.widgets.inactive.bg_fill = control;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, border);
    visuals.widgets.inactive.fg_stroke.color = foreground;
    visuals.widgets.hovered.bg_fill = accent_soft;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, accent);
    visuals.widgets.hovered.fg_stroke.color = foreground;
    visuals.widgets.active.bg_fill = accent_soft;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, accent);
    visuals.widgets.active.fg_stroke.color = foreground;
    visuals.widgets.open.bg_fill = control;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, accent);
    visuals.widgets.open.fg_stroke.color = foreground;
    visuals
}

fn wake_native_window() {
    use windows::{
        core::PCWSTR,
        Win32::UI::WindowsAndMessaging::{
            FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
        },
    };
    let title: Vec<u16> = "M4Dash\0".encode_utf16().collect();
    if let Ok(hwnd) = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) } {
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

fn apply_windows_ui_style(ctx: &egui::Context, language: crate::config::Language) {
    // The selected font and point size belong to the taskbar overlay. Keep the
    // settings window independent and use the familiar Windows UI typeface.
    let file = "segoeui.ttf";
    let path = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts")
        .join(file);
    let mut fonts = egui::FontDefinitions::default();
    match std::fs::read(&path) {
        Ok(bytes) => {
            let key = "m4-windows-ui-font";
            fonts.font_data.insert(
                key.to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, key.to_owned());
        }
        Err(error) => tracing::warn!("Could not load font {}: {error}", path.display()),
    }
    add_language_font_fallback(&mut fonts, language);
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    for (text_style, size) in [
        (egui::TextStyle::Body, 16.0),
        (egui::TextStyle::Button, 16.0),
        (egui::TextStyle::Small, 13.0),
        (egui::TextStyle::Heading, 21.0),
        (egui::TextStyle::Monospace, 16.0),
        (egui::TextStyle::Name("Heading".into()), 25.0),
    ] {
        style
            .text_styles
            .insert(text_style, egui::FontId::proportional(size));
    }
    ctx.set_style(style);
}

fn add_language_font_fallback(
    fonts: &mut egui::FontDefinitions,
    language: crate::config::Language,
) {
    let (filename, key) = match language {
        crate::config::Language::Chinese => ("msyh.ttc", "m4-language-fallback-chinese"),
        crate::config::Language::Persian => ("arial.ttf", "m4-language-fallback-persian"),
        _ => return,
    };
    let path = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts")
        .join(filename);
    let Ok(bytes) = std::fs::read(&path) else {
        tracing::warn!("Could not load language fallback font {}", path.display());
        return;
    };
    fonts.font_data.insert(
        key.into(),
        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let fonts_for_family = fonts.families.entry(family).or_default();
        if !fonts_for_family.iter().any(|entry| entry == key) {
            fonts_for_family.insert(1.min(fonts_for_family.len()), key.into());
        }
    }
}

fn portable_font_dir() -> PathBuf {
    crate::config::Settings::data_dir()
        .ok()
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

fn choose_font_file(language: crate::config::Language) -> anyhow::Result<Option<PathBuf>> {
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
    let title: Vec<u16> = format!("{}\0", crate::i18n::tr(language, "Добавить шрифт в M4Dash"))
        .encode_utf16()
        .collect();
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
        let language = self.settings.language;
        ui.heading(crate::i18n::tr(
            language,
            "Датчики и обнаруженное оборудование",
        ));
        ui.label(crate::i18n::tr(language, "Оборудование определяется автоматически; выберите блоки и устройства для панели задач."));
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_cpu, "");
            self.edit_name(ui, "cpu", "CPU");
        });
        ui.label(crate::i18n::tr_format(
            language,
            "{} · {} потоков · {:.0}% · {} MHz · температура {}",
            &[
                s.cpu_name.clone(),
                s.cpu_threads.to_string(),
                format!("{:.0}", s.cpu_usage),
                s.cpu_frequency_mhz.to_string(),
                x.cpu_temperature_c
                    .map(|v| format!("{v:.0}°C"))
                    .unwrap_or_else(|| "N/A".into()),
            ],
        ));
        if let Some(error) = &x.provider_error {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!(
                    "{}: {error}",
                    crate::i18n::tr(language, "Провайдер датчиков")
                ),
            );
        } else {
            ui.weak(crate::i18n::tr_format(
                language,
                "Источник температуры: {}",
                &[x.cpu_temperature_sensor
                    .as_deref()
                    .unwrap_or(crate::i18n::tr(language, "ожидание данных"))
                    .to_owned()],
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_gpu, "");
            self.edit_name(ui, "gpu", "NVIDIA GPU");
        });
        ui.label(crate::i18n::tr_format(
            language,
            "{} · загрузка {} · VRAM {}/{} MiB · температура {}",
            &[
                s.gpu_name.as_deref().unwrap_or("N/A").to_owned(),
                opt(s.gpu_utilization_pct),
                opt(s.vram_used_mib),
                opt(s.vram_total_mib),
                s.gpu_temp_c
                    .map(|v| format!("{v}°C"))
                    .unwrap_or_else(|| "N/A".into()),
            ],
        ));
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_ram, "");
            self.edit_name(ui, "ram", "RAM");
        });
        ui.label(format!(
            "{} {}{:.1}/{:.1} GiB ({})",
            x.memory_type.as_deref().unwrap_or("RAM"),
            x.memory_speed_mhz
                .map(|mhz| format!("{:.1}GHz ", mhz as f64 / 1000.0))
                .unwrap_or_default(),
            s.ram_used_bytes as f64 / 1073741824.0,
            s.ram_total_bytes as f64 / 1073741824.0,
            ram_usage_percent(s)
        ));
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_motherboard, "");
            self.edit_name(ui, "motherboard", "MBR");
        });
        ui.label(
            x.motherboard_name
                .as_deref()
                .unwrap_or(crate::i18n::tr(language, "Материнская плата не обнаружена")),
        );
        for sensor in &x.motherboard_sensors {
            let key = board_sensor_key(sensor);
            let mut selected = self
                .settings
                .motherboard_sensor_visibility
                .get(&key)
                .copied()
                .unwrap_or(false);
            let default_name = format!("{} {}", sensor.kind, sensor.name);
            ui.horizontal(|ui| {
                ui.checkbox(&mut selected, "");
                self.edit_name(ui, &key, &default_name);
                ui.label(format!(
                    "{:.0}{}",
                    sensor.value,
                    board_sensor_unit(&sensor.kind)
                ));
            });
            self.settings
                .motherboard_sensor_visibility
                .insert(key, selected);
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_disks, "");
            ui.label(crate::i18n::tr(language, "Диски"));
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
            ui.label(crate::i18n::tr_format(
                language,
                "{} · {:.1} GiB · чтение {:.2} {} · запись {:.2} {}",
                &[
                    d.kind.clone(),
                    format!("{:.1}", d.total_bytes as f64 / 1073741824.0),
                    format!("{:.2}", selected_unit.value(d.read_bytes_per_sec)),
                    selected_unit.label().into(),
                    format!("{:.2}", selected_unit.value(d.written_bytes_per_sec)),
                    selected_unit.label().into(),
                ],
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.show_network, "");
            ui.label(crate::i18n::tr(language, "Сеть (суммарно)"));
        });
        ui.horizontal(|ui| {
            ui.label(crate::i18n::tr(language, "Скорость суммы:"));
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
            l.model
                .as_deref()
                .unwrap_or(crate::i18n::tr(language, "Модель не обнаружена")),
            phase_name(l.phase, language),
            l.speed_tps
                .map(|v| format!("{v:.2} {}", crate::i18n::tr(language, "токен/с")))
                .unwrap_or_default()
        ));
        ui.separator();
        self.llm_page(ui);
    }
    fn layout_page(&mut self, ui: &mut egui::Ui, s: &Snapshot, x: &ExtraSnapshot) {
        let language = self.settings.language;
        ui.heading(crate::i18n::tr(
            language,
            "Порядок показателей в панели задач",
        ));
        ui.label(crate::i18n::tr(
            language,
            "Здесь показаны только включённые блоки и устройства. Порядок сохраняется рядом с EXE.",
        ));
        ui.add(
            egui::Slider::new(&mut self.settings.column_spacing_px, 0..=100)
                .text(crate::i18n::tr(language, "Отступ между колонками"))
                .suffix(" px"),
        );
        ui.separator();
        ui.heading(crate::i18n::tr(language, "Пользовательские группы"));
        ui.label(crate::i18n::tr(
            language,
            "В каждой группе выберите верхнюю и нижнюю строку из включённых датчиков.",
        ));
        let mut choices: Vec<(String, String)> = Vec::new();
        for (enabled, key, label) in [
            (
                self.settings.show_network,
                "network:rx",
                crate::i18n::tr(language, "Сеть ↓"),
            ),
            (
                self.settings.show_network,
                "network:tx",
                crate::i18n::tr(language, "Сеть ↑"),
            ),
            (self.settings.show_cpu, "cpu", "CPU"),
            (self.settings.show_ram, "ram", "RAM"),
            (self.settings.show_gpu, "gpu", "GPU"),
            (self.settings.show_llm, "llm", "LM Studio"),
            (self.settings.show_motherboard, "motherboard", "MBR"),
        ] {
            if enabled {
                choices.push((key.into(), label.into()));
            }
        }
        if self.settings.show_disks {
            for disk in &s.disks {
                if self
                    .settings
                    .disk_visibility
                    .get(&disk.id)
                    .copied()
                    .unwrap_or(true)
                {
                    let id = format!("disk:{}", disk.id);
                    choices.push((id.clone(), self.name(&id, &disk.name)));
                }
            }
        }
        if self.settings.show_motherboard {
            for sensor in &x.motherboard_sensors {
                let key = board_sensor_key(sensor);
                if self
                    .settings
                    .motherboard_sensor_visibility
                    .get(&key)
                    .copied()
                    .unwrap_or(false)
                {
                    let default_name = format!("{} {}", sensor.kind, sensor.name);
                    choices.push((key.clone(), self.name(&key, &default_name)));
                }
            }
        }
        let mut delete = None;
        for (index, group) in self.settings.layout_groups.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(crate::i18n::tr_format(
                    language,
                    "Группа {}",
                    &[format!("{}", index + 1)],
                ));
                if ui.button(crate::i18n::tr(language, "Удалить")).clicked() {
                    delete = Some(index);
                }
            });
            ui.label(crate::i18n::tr(language, "Верхняя строка"));
            group_row_checkboxes(ui, &mut group.first, &choices, language);
            ui.label(crate::i18n::tr(language, "Нижняя строка"));
            group_row_checkboxes(ui, &mut group.second, &choices, language);
            ui.add(
                egui::Slider::new(&mut group.width_chars, 8..=80)
                    .text(crate::i18n::tr(language, "Ширина группы (символов)")),
            );
        }
        if let Some(index) = delete {
            self.settings.layout_groups.remove(index);
            self.settings.taskbar_order.clear();
        }
        if ui
            .button(crate::i18n::tr(language, "Добавить группу"))
            .clicked()
        {
            self.settings
                .layout_groups
                .push(crate::config::LayoutGroup {
                    first: String::new(),
                    second: String::new(),
                    width_chars: 28,
                });
        }
        ui.separator();
        let mut items: Vec<(String, String, String)> = Vec::new();
        for (index, _) in self.settings.layout_groups.iter().enumerate() {
            items.push((
                format!("group:{index}"),
                crate::i18n::tr_format(language, "Группа {}", &[format!("{}", index + 1)]),
                String::new(),
            ));
        }
        if items.is_empty() {
            ui.weak(crate::i18n::tr(
                language,
                "Нет включённых блоков для панели задач.",
            ));
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
        let language = self.settings.language;
        ui.heading(crate::i18n::tr(language, "Настройки LM Studio"));
        ui.label(crate::i18n::tr(language, "Название блока"));
        self.edit_name(ui, "llm", "LM Studio");
        ui.label(crate::i18n::tr(language, "Адрес API"));
        ui.text_edit_singleline(&mut self.settings.llm_api);
        ui.label(crate::i18n::tr(
            language,
            "Путь к lms.exe (пусто — автообнаружение)",
        ));
        ui.text_edit_singleline(&mut self.settings.lms_path);
    }
    fn general_page(&mut self, ui: &mut egui::Ui) {
        let language = self.settings.language;
        ui.heading(crate::i18n::tr(language, "Общие настройки"));
        egui::ComboBox::from_label(crate::i18n::tr(language, "Язык"))
            .selected_text(language.label())
            .show_ui(ui, |ui| {
                for option in crate::config::Language::ALL {
                    ui.selectable_value(&mut self.settings.language, option, option.label());
                }
            });
        egui::ComboBox::from_label(crate::i18n::tr(language, "Тема приложения"))
            .selected_text(crate::i18n::tr(
                language,
                match self.settings.theme_mode {
                    ThemeMode::System => "Как в системе",
                    ThemeMode::Light => "Светлая",
                    ThemeMode::Dark => "Тёмная",
                },
            ))
            .show_ui(ui, |ui| {
                for (mode, label) in [
                    (ThemeMode::System, "Как в системе"),
                    (ThemeMode::Light, "Светлая"),
                    (ThemeMode::Dark, "Тёмная"),
                ] {
                    ui.selectable_value(
                        &mut self.settings.theme_mode,
                        mode,
                        crate::i18n::tr(language, label),
                    );
                }
            });
        egui::ComboBox::from_label(crate::i18n::tr(language, "Открывать программу"))
            .selected_text(if self.settings.start_in_tray {
                crate::i18n::tr(language, "В трее")
            } else {
                crate::i18n::tr(language, "В окне")
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.settings.start_in_tray,
                    true,
                    crate::i18n::tr(language, "В трее"),
                );
                ui.selectable_value(
                    &mut self.settings.start_in_tray,
                    false,
                    crate::i18n::tr(language, "В окне"),
                );
            });
        egui::ComboBox::from_label(crate::i18n::tr(language, "Интервал обновления"))
            .selected_text(format!("{} мс", self.settings.refresh_ms))
            .show_ui(ui, |ui| {
                for ms in [500, 1000, 2000] {
                    ui.selectable_value(&mut self.settings.refresh_ms, ms, format!("{ms} мс"));
                }
            });
        ui.checkbox(
            &mut self.settings.show_taskbar,
            crate::i18n::tr(language, "Показывать текстовый мониторинг в панели задач"),
        );
        let mut a = self.launch_at_login;
        if ui
            .checkbox(
                &mut a,
                crate::i18n::tr(language, "Запускать вместе с Windows"),
            )
            .changed()
        {
            self.update_autostart(a);
        }
        ui.label(crate::i18n::tr(
            language,
            "Настройки сохраняются рядом с EXE и переживают обновление Portable.",
        ));
        ui.separator();
        ui.heading(crate::i18n::tr(language, "Оформление"));
        ui.label(crate::i18n::tr(
            language,
            "Шрифт и размер относятся только к панели задач",
        ));
        egui::ComboBox::from_label(crate::i18n::tr(language, "Шрифт"))
            .selected_text(&self.settings.font_family)
            .show_ui(ui, |ui| {
                for family in ["Segoe UI", "Arial", "Tahoma", "Consolas", "Cascadia Mono"] {
                    ui.selectable_value(&mut self.settings.font_family, family.to_owned(), family);
                }
                for family in self.settings.custom_fonts.keys() {
                    ui.selectable_value(&mut self.settings.font_family, family.clone(), family);
                }
            });
        if ui
            .button(crate::i18n::tr(language, "Добавить шрифт…"))
            .clicked()
        {
            match choose_font_file(language) {
                Ok(Some(path)) => match install_custom_font(&path) {
                    Ok((family, filename)) => {
                        self.settings.custom_fonts.insert(family.clone(), filename);
                        self.settings.font_family = family.clone();
                        self.status = crate::i18n::tr_format(
                            language,
                            "Шрифт {} добавлен в Portable и выбран",
                            &[family],
                        );
                    }
                    Err(error) => {
                        self.status = format!(
                            "{}: {error}",
                            crate::i18n::tr(language, "Не удалось добавить шрифт")
                        )
                    }
                },
                Ok(None) => {}
                Err(error) => {
                    self.status = format!(
                        "{}: {error}",
                        crate::i18n::tr(language, "Не удалось открыть выбор шрифта")
                    )
                }
            }
        }
        ui.add(
            egui::Slider::new(&mut self.settings.font_size, 9.0..=24.0)
                .text(crate::i18n::tr(language, "Размер текста")),
        );
        ui.color_edit_button_srgb(&mut self.settings.text_color);
        ui.separator();
        ui.heading(crate::i18n::tr(language, "Обновления"));
        ui.label(crate::i18n::tr_format(
            language,
            "Текущая версия: {}",
            &[env!("CARGO_PKG_VERSION").to_owned()],
        ));
        let update_status = crate::updater::status();
        let update_is_busy = matches!(
            &update_status,
            crate::updater::UpdateStatus::Checking | crate::updater::UpdateStatus::Downloading(_)
        );
        if ui
            .add_enabled(
                !update_is_busy,
                egui::Button::new(crate::i18n::tr(language, "Проверить обновления")),
            )
            .clicked()
        {
            crate::updater::check_now();
        }
        let status_text = match update_status {
            crate::updater::UpdateStatus::Idle => {
                crate::i18n::tr(language, "Нажмите кнопку для проверки GitHub").to_owned()
            }
            crate::updater::UpdateStatus::Checking => {
                crate::i18n::tr(language, "Проверка обновлений…").to_owned()
            }
            crate::updater::UpdateStatus::Available(version) => {
                return_update_status(language, "Доступна версия {}", &version)
            }
            crate::updater::UpdateStatus::UpToDate => {
                crate::i18n::tr(language, "Установлена последняя версия").to_owned()
            }
            crate::updater::UpdateStatus::Downloading(version) => {
                return_update_status(language, "Скачивается версия {}…", &version)
            }
            crate::updater::UpdateStatus::Error => {
                crate::i18n::tr(language, "Не удалось проверить обновления").to_owned()
            }
        };
        ui.label(status_text);
    }
}

fn return_update_status(
    language: crate::config::Language,
    source: &'static str,
    version: &str,
) -> String {
    crate::i18n::tr_format(language, source, &[version.to_owned()])
}

fn start_taskbar_refresh(
    settings: Arc<RwLock<Settings>>,
    hardware: Arc<RwLock<Snapshot>>,
    extra: Arc<RwLock<ExtraSnapshot>>,
    llm: Arc<RwLock<LlmSnapshot>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
) {
    std::thread::Builder::new()
        .name("m4-taskbar-metrics".into())
        .spawn(move || {
            use std::sync::atomic::Ordering;
            while !shutdown.load(Ordering::Acquire) {
                let mut current_settings = settings
                    .read()
                    .map(|value| value.clone())
                    .unwrap_or_default();
                if current_settings.show_taskbar {
                    let snapshot = hardware
                        .read()
                        .map(|value| value.clone())
                        .unwrap_or_default();
                    let extra_snapshot =
                        extra.read().map(|value| value.clone()).unwrap_or_default();
                    let llm_snapshot = llm.read().map(|value| value.clone()).unwrap_or_default();
                    let columns = grouped_items(
                        &current_settings,
                        taskbar_items(&current_settings, &snapshot, &extra_snapshot, &llm_snapshot),
                    );
                    let order = ordered_keys(&mut current_settings.taskbar_order, &columns);
                    let visible = order
                        .iter()
                        .filter_map(|key| {
                            columns
                                .iter()
                                .find(|(id, _, _)| id == key)
                                .filter(|(_, first, second)| {
                                    !first.is_empty() || !second.is_empty()
                                })
                                .map(|(id, first, second)| {
                                    let width = id
                                        .strip_prefix("group:")
                                        .and_then(|index| index.parse::<usize>().ok())
                                        .and_then(|index| current_settings.layout_groups.get(index))
                                        .map(|group| group.width_chars)
                                        .unwrap_or(28);
                                    (format!("{first}\n{second}"), width)
                                })
                        })
                        .collect();
                    let font_path = current_settings
                        .custom_fonts
                        .get(&current_settings.font_family)
                        .map(|file| portable_font_dir().join(file));
                    crate::taskbar::refresh_text_overlay(
                        visible,
                        current_settings.font_size,
                        current_settings.text_color,
                        &current_settings.font_family,
                        current_settings.column_spacing_px,
                        font_path.as_deref(),
                    );
                } else {
                    crate::taskbar::hide_text_overlay();
                }
                std::thread::sleep(Duration::from_millis(current_settings.refresh_ms.max(500)));
            }
        })
        .expect("spawn taskbar metrics refresher");
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
                "{} {} GHz: {:.0}% {}",
                name("cpu", "CPU"),
                if s.cpu_frequency_mhz > 0 {
                    format!("{:.1}", s.cpu_frequency_mhz as f64 / 1000.0)
                } else {
                    "N/A".into()
                },
                s.cpu_usage,
                x.cpu_temperature_c
                    .map(|temperature| format!("{temperature:.0}°C"))
                    .unwrap_or_else(|| "N/A".into()),
            )
        } else {
            String::new()
        };
        let ram_line = if settings.show_ram {
            let ram_type = x.memory_type.as_deref().unwrap_or("RAM");
            let frequency = x
                .memory_speed_mhz
                .map(|mhz| format!("{:.1} GHz", mhz as f64 / 1000.0))
                .unwrap_or_else(|| "N/A GHz".into());
            format!(
                "{} {}: {:.0}/{:.0} GB ({})",
                name("ram", ram_type),
                frequency,
                s.ram_used_bytes as f64 / 1073741824.0,
                s.ram_total_bytes as f64 / 1073741824.0,
                ram_usage_percent(s)
            )
        } else {
            String::new()
        };
        v.push(("cpu_ram".into(), cpu_line, ram_line));
    }
    if settings.show_motherboard {
        if x.motherboard_name.is_some() {
            let vals = x
                .motherboard_sensors
                .iter()
                .filter(|q| q.kind == "Temperature")
                .take(2)
                .map(|q| format!("{} {:.0}°", q.name, q.value))
                .collect::<Vec<_>>()
                .join(" · ");
            v.push(("motherboard".into(), name("motherboard", "MBR"), vals));
        }
        for sensor in &x.motherboard_sensors {
            let key = board_sensor_key(sensor);
            if settings
                .motherboard_sensor_visibility
                .get(&key)
                .copied()
                .unwrap_or(false)
            {
                let default_name = format!("{} {}", sensor.kind, sensor.name);
                let unit = board_sensor_unit(&sensor.kind);
                v.push((
                    key.clone(),
                    name(&key, &default_name),
                    format!("{:.0}{unit}", sensor.value),
                ));
            }
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

fn ram_usage_percent(s: &Snapshot) -> String {
    if s.ram_total_bytes == 0 {
        return "N/A".into();
    }
    let percent = (s.ram_used_bytes as f64 / s.ram_total_bytes as f64 * 100.0).clamp(0.0, 100.0);
    format!("{percent:.0}%")
}

fn group_row_checkboxes(
    ui: &mut egui::Ui,
    selected: &mut String,
    choices: &[(String, String)],
    language: crate::config::Language,
) {
    ui.horizontal_wrapped(|ui| {
        let mut none = selected.is_empty();
        if ui
            .checkbox(&mut none, crate::i18n::tr(language, "Не выбрано"))
            .clicked()
        {
            selected.clear();
        }
        for (key, label) in choices {
            let mut checked = *selected == *key;
            if ui.checkbox(&mut checked, label).changed() {
                if checked {
                    selected.clone_from(key);
                } else if *selected == *key {
                    selected.clear();
                }
            }
        }
    });
}

fn board_sensor_key(sensor: &crate::hardware::BoardSensor) -> String {
    format!("board-sensor:{}:{}", sensor.kind, sensor.name)
}

fn board_sensor_unit(kind: &str) -> &'static str {
    match kind {
        "Temperature" => "°C",
        "Fan" => " RPM",
        "Voltage" => " V",
        "Clock" => " MHz",
        "Load" | "Control" | "Level" | "Humidity" => "%",
        "Power" => " W",
        "Flow" => " L/h",
        "Energy" => " mWh",
        "Noise" => " dBA",
        "Conductivity" => " µS/cm",
        "Data" => " B",
        "Throughput" => " B/s",
        "TimeSpan" => " s",
        "Timing" => " ns",
        "Frequency" => " Hz",
        "Factor" => "×",
        _ => "",
    }
}

fn grouped_items(
    settings: &Settings,
    columns: Vec<(String, String, String)>,
) -> Vec<(String, String, String)> {
    let mut available = std::collections::BTreeMap::<String, String>::new();
    for (key, first, second) in &columns {
        match key.as_str() {
            "network" => {
                available.insert("network:rx".into(), first.clone());
                available.insert("network:tx".into(), second.clone());
            }
            "cpu_ram" => {
                available.insert("cpu".into(), first.clone());
                available.insert("ram".into(), second.clone());
            }
            "gpu_llm" => {
                available.insert("gpu".into(), first.clone());
                available.insert("llm".into(), second.clone());
            }
            _ => {
                available.insert(
                    key.clone(),
                    [first.as_str(), second.as_str()]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
    }
    let mut columns = Vec::new();
    for (index, group) in settings.layout_groups.iter().enumerate() {
        let first = available.get(&group.first).cloned().unwrap_or_default();
        let second = available.get(&group.second).cloned().unwrap_or_default();
        columns.push((format!("group:{index}"), first, second));
    }
    columns
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
fn phase_name(p: LlmPhase, language: crate::config::Language) -> &'static str {
    match p {
        LlmPhase::Loading => crate::i18n::tr(language, "Загрузка"),
        LlmPhase::Preparing => crate::i18n::tr(language, "Подготовка"),
        LlmPhase::Pp => crate::i18n::tr(language, "Prompt processing"),
        LlmPhase::Tg => crate::i18n::tr(language, "Генерация"),
        LlmPhase::Unloading => crate::i18n::tr(language, "Выгрузка"),
        LlmPhase::Idle => crate::i18n::tr(language, "Ожидание"),
        LlmPhase::Offline => crate::i18n::tr(language, "LM Studio offline"),
        LlmPhase::Unknown => crate::i18n::tr(language, "Статус неизвестен"),
        LlmPhase::NoModel => crate::i18n::tr(language, "Нет загруженной модели"),
    }
}
