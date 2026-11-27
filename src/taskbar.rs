//! Transparent, click-through text overlay positioned over the Windows taskbar.
use eframe::egui;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::RECT,
        UI::WindowsAndMessaging::{FindWindowW, GetWindowRect},
    },
};

pub fn show_text_overlay(
    ctx: &egui::Context,
    columns: Vec<String>,
    font_size: f32,
    color: [u8; 3],
) {
    if columns.is_empty() {
        return;
    }
    let Some((x, y, width, height)) = taskbar_rect(ctx.pixels_per_point()) else {
        return;
    };
    let viewport_id = egui::ViewportId::from_hash_of("m4-pc-monitor-taskbar-text");
    let builder = egui::ViewportBuilder::default()
        .with_title("M4 PC Monitor Taskbar")
        .with_visible(true)
        .with_position([x, y])
        .with_inner_size([width, height])
        .with_min_inner_size([width, height])
        .with_max_inner_size([width, height])
        .with_decorations(false)
        .with_resizable(false)
        .with_taskbar(false)
        .with_always_on_top()
        .with_transparent(true)
        .with_mouse_passthrough(true);
    ctx.show_viewport_deferred(viewport_id, builder, move |overlay, _| {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(overlay, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(6.0);
                    for text in &columns {
                        ui.vertical(|ui| {
                            for line in text.lines() {
                                ui.label(
                                    egui::RichText::new(line)
                                        .size(font_size)
                                        .color(egui::Color32::from_rgb(
                                            color[0], color[1], color[2],
                                        ))
                                        .strong(),
                                );
                            }
                        });
                        ui.add_space(16.0);
                    }
                });
            });
        overlay.request_repaint_after(std::time::Duration::from_millis(500));
    });
}

fn taskbar_rect(ppp: f32) -> Option<(f32, f32, f32, f32)> {
    let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
    let rect = unsafe {
        let hwnd = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()).ok()?;
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        rect
    };
    let ppp = ppp.max(1.0);
    let taskbar_width = (rect.right - rect.left) as f32 / ppp;
    let taskbar_height = (rect.bottom - rect.top) as f32 / ppp;
    // Reserve pinned applications on the left and notification/system tray on the right.
    let left_reserve = 190.0_f32.min(taskbar_width * 0.18);
    let right_reserve = 250.0_f32.min(taskbar_width * 0.22);
    let x = rect.left as f32 / ppp + left_reserve;
    let width = (taskbar_width - left_reserve - right_reserve).max(240.0);
    Some((x, rect.top as f32 / ppp, width, taskbar_height.max(40.0)))
}
