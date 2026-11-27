//! Text metrics hosted as a child window of the Windows taskbar.
use eframe::egui;
use std::sync::atomic::{AtomicU64, Ordering};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::RECT,
        UI::WindowsAndMessaging::{
            FindWindowW, GetClientRect, GetParent, GetWindowLongPtrW, GetWindowRect, SetParent,
            SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_TOP,
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOWNA, WS_CHILD,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
        },
    },
};

const VIEWPORT_TITLE: &str = "M4 PC Monitor Taskbar";
const DOCK_INTERVAL_MS: u64 = 750;
static LAST_DOCK_MS: AtomicU64 = AtomicU64::new(0);

pub fn show_text_overlay(
    ctx: &egui::Context,
    columns: Vec<String>,
    font_size: f32,
    color: [u8; 3],
) {
    let viewport_id = egui::ViewportId::from_hash_of("m4-pc-monitor-taskbar-text");
    if columns.is_empty() {
        hide_text_overlay(ctx);
        return;
    }

    let ppp = ctx.pixels_per_point().max(1.0);
    let (x, y, width, height) = taskbar_screen_rect(ppp).unwrap_or((0.0, 0.0, 800.0, 48.0));
    let left_reserve = 190.0_f32.min(width * 0.18);
    let right_reserve = 250.0_f32.min(width * 0.22);
    let builder = egui::ViewportBuilder::default()
        .with_title(VIEWPORT_TITLE)
        .with_visible(true)
        .with_position([x + left_reserve, y])
        .with_inner_size([(width - left_reserve - right_reserve).max(240.0), height])
        .with_decorations(false)
        .with_resizable(false)
        .with_taskbar(false)
        .with_transparent(true)
        .with_mouse_passthrough(true);

    ctx.show_viewport_deferred(viewport_id, builder, move |overlay, _| {
        if let Err(error) = dock_window_inside_taskbar(overlay.pixels_per_point().max(1.0)) {
            tracing::debug!("Taskbar child-window docking: {error}");
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(overlay, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
                    ui.add_space(6.0);
                    for text in &columns {
                        ui.vertical(|ui| {
                            for line in text.lines().take(2) {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(line)
                                            .size(font_size)
                                            .color(egui::Color32::from_rgb(
                                                color[0], color[1], color[2],
                                            ))
                                            .strong(),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
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

pub fn hide_text_overlay(ctx: &egui::Context) {
    let id = egui::ViewportId::from_hash_of("m4-pc-monitor-taskbar-text");
    ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Visible(false));
}

fn taskbar_screen_rect(ppp: f32) -> Option<(f32, f32, f32, f32)> {
    let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
    let rect = unsafe {
        let hwnd = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()).ok()?;
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        rect
    };
    Some((
        rect.left as f32 / ppp,
        rect.top as f32 / ppp,
        (rect.right - rect.left) as f32 / ppp,
        (rect.bottom - rect.top) as f32 / ppp,
    ))
}

fn dock_window_inside_taskbar(ppp: f32) -> windows::core::Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let last = LAST_DOCK_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < DOCK_INTERVAL_MS {
        return Ok(());
    }

    let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
    let title: Vec<u16> = format!("{VIEWPORT_TITLE}\0").encode_utf16().collect();
    unsafe {
        let taskbar = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null())?;
        let child = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr()))?;
        if GetParent(child).unwrap_or_default() != taskbar {
            SetParent(child, Some(taskbar))?;
            let style = GetWindowLongPtrW(child, GWL_STYLE);
            let style = (style & !(WS_POPUP.0 as isize)) | WS_CHILD.0 as isize;
            SetWindowLongPtrW(child, GWL_STYLE, style);
            let ex_style = GetWindowLongPtrW(child, GWL_EXSTYLE)
                | WS_EX_NOACTIVATE.0 as isize
                | WS_EX_TOOLWINDOW.0 as isize
                | WS_EX_TRANSPARENT.0 as isize;
            SetWindowLongPtrW(child, GWL_EXSTYLE, ex_style);
        }

        let mut client = RECT::default();
        GetClientRect(taskbar, &mut client)?;
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        let left = (190.0 * ppp).min(width as f32 * 0.18) as i32;
        let right = (250.0 * ppp).min(width as f32 * 0.22) as i32;
        let content_width = (width - left - right).max(240);
        SetWindowPos(
            child,
            Some(HWND_TOP),
            left,
            0,
            content_width,
            height.max(40),
            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        )?;
        let _ = ShowWindow(child, SW_SHOWNA);
    }
    LAST_DOCK_MS.store(now, Ordering::Relaxed);
    Ok(())
}
