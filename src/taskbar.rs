//! Native child window hosted directly inside the Windows taskbar.
//!
//! TrafficMonitor uses a separate native window parented to Shell_TrayWnd.  Do
//! the same here instead of asking egui/winit to create a second top-level
//! viewport and trying to locate/reparent that window after the fact.
use std::os::windows::ffi::OsStrExt;
use std::sync::{
    atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering},
    LazyLock, Mutex, Once,
};
use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            AddFontResourceExW, BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW,
            EndPaint, FillRect, InvalidateRect, SelectObject, SetBkMode, SetTextColor,
            UpdateWindow, BACKGROUND_MODE, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY,
            DT_CALCRECT, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FR_PRIVATE, FW_BOLD,
            OUT_DEFAULT_PRECIS, TRANSPARENT, VARIABLE_PITCH,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, FindWindowExW, FindWindowW, GetClientRect, GetParent,
            GetWindowLongPtrW, GetWindowRect, RegisterClassW, SetLayeredWindowAttributes,
            SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_STYLE, LWA_COLORKEY,
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOWNA, WM_ERASEBKGND, WM_PAINT,
            WNDCLASSW, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_LAYERED, WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
};

const CLASS_NAME: &str = "M4PCMonitor.NativeTaskbarMetrics";
static REGISTER_CLASS: Once = Once::new();
static CHILD: AtomicIsize = AtomicIsize::new(0);
static FONT_SIZE_BITS: AtomicU32 = AtomicU32::new(14.0f32.to_bits());
static COLUMN_SPACING_PX: AtomicU32 = AtomicU32::new(16);
static TEXT_RGB: AtomicU32 = AtomicU32::new(0x00_EB_EB_EBu32);
static DRAW_STYLE_KEY: AtomicU64 = AtomicU64::new(0);
static DRAW_DATA_KEY: AtomicU64 = AtomicU64::new(0);
static FONT_NAME: Mutex<String> = Mutex::new(String::new());
static DRAW_COLUMNS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
static PRIVATE_FONTS: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static FIRST_CALL_LOGGED: AtomicBool = AtomicBool::new(false);
static EMPTY_LOGGED: AtomicBool = AtomicBool::new(false);
static WINDOW_LOGGED: AtomicBool = AtomicBool::new(false);
static CREATE_ERROR_LOGGED: AtomicBool = AtomicBool::new(false);
static POSITION_LOGGED: AtomicBool = AtomicBool::new(false);

pub fn show_text_overlay(
    _ctx: &eframe::egui::Context,
    columns: Vec<String>,
    font_size: f32,
    color: [u8; 3],
    font_family: &str,
    column_spacing_px: u16,
    font_path: Option<&Path>,
) {
    if columns.is_empty() {
        if !EMPTY_LOGGED.swap(true, Ordering::Relaxed) {
            diagnostic("taskbar renderer called with zero visible columns");
        }
        hide_text_overlay(_ctx);
        return;
    }
    if !FIRST_CALL_LOGGED.swap(true, Ordering::Relaxed) {
        diagnostic(&format!(
            "taskbar renderer called with {} columns",
            columns.len()
        ));
    }

    FONT_SIZE_BITS.store(font_size.max(6.0).to_bits(), Ordering::Relaxed);
    COLUMN_SPACING_PX.store(u32::from(column_spacing_px), Ordering::Relaxed);
    if let Some(path) = font_path {
        register_private_font(path);
    }
    let rgb = u32::from(color[0]) | (u32::from(color[1]) << 8) | (u32::from(color[2]) << 16);
    TEXT_RGB.store(rgb, Ordering::Relaxed);
    if let Ok(mut font) = FONT_NAME.lock() {
        *font = font_family.to_string();
    }
    let family_hash = font_family
        .bytes()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    let path_hash = font_path
        .map(|path| {
            path.as_os_str()
                .encode_wide()
                .fold(0xcbf29ce484222325u64, |hash, unit| {
                    (hash ^ u64::from(unit)).wrapping_mul(0x100000001b3)
                })
        })
        .unwrap_or_default();
    let style_key = family_hash
        ^ path_hash.rotate_left(29)
        ^ u64::from(font_size.max(6.0).to_bits()).rotate_left(17)
        ^ u64::from(rgb).rotate_left(41);
    let style_changed = DRAW_STYLE_KEY.swap(style_key, Ordering::Relaxed) != style_key;

    let draw_columns: Vec<(String, String)> = columns
        .iter()
        .map(|column| {
            (
                column.lines().next().unwrap_or_default().to_owned(),
                column.lines().nth(1).unwrap_or_default().to_owned(),
            )
        })
        .collect();
    if let Ok(mut current) = DRAW_COLUMNS.lock() {
        *current = draw_columns.clone();
    }
    let data_key = draw_columns
        .iter()
        .flat_map(|(top, bottom)| {
            top.chars()
                .chain(['\n'])
                .chain(bottom.chars())
                .chain(['\0'])
        })
        .fold(
            0xcbf29ce484222325u64 ^ u64::from(column_spacing_px),
            |hash, character| (hash ^ u64::from(character as u32)).wrapping_mul(0x100000001b3),
        );
    let data_changed = DRAW_DATA_KEY.swap(data_key, Ordering::Relaxed) != data_key;

    unsafe {
        let taskbar = match taskbar_hwnd() {
            Ok(hwnd) => hwnd,
            Err(e) => {
                diagnostic(&format!("FindWindow Shell_TrayWnd failed: {e}"));
                return;
            }
        };
        let child = match ensure_child(taskbar) {
            Ok(hwnd) => hwnd,
            Err(e) => {
                if !CREATE_ERROR_LOGGED.swap(true, Ordering::Relaxed) {
                    diagnostic(&format!("Create taskbar child failed: {e}"));
                }
                return;
            }
        };
        position_child(taskbar, child);
        // Metrics, font, and spacing update the stored draw model directly;
        // invalidate the GDI child whenever any of those values change.
        if style_changed || data_changed {
            let _ = InvalidateRect(Some(child), None, true);
            let _ = UpdateWindow(child);
        }
        let _ = ShowWindow(child, SW_SHOWNA);
        if !WINDOW_LOGGED.swap(true, Ordering::Relaxed) {
            diagnostic(&format!(
                "taskbar child visible: hwnd={:?}, parent={:?}",
                child, taskbar
            ));
        }
    }
}

fn register_private_font(path: &Path) {
    let Ok(path) = std::fs::canonicalize(path) else {
        return;
    };
    let Ok(mut loaded) = PRIVATE_FONTS.lock() else {
        return;
    };
    if loaded.contains(&path) {
        return;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let count = unsafe { AddFontResourceExW(PCWSTR(wide.as_ptr()), FR_PRIVATE, None) };
    if count > 0 {
        loaded.insert(path);
    } else {
        diagnostic("AddFontResourceExW failed for selected portable font");
    }
}

pub fn hide_text_overlay(_ctx: &eframe::egui::Context) {
    let raw = CHILD.load(Ordering::Relaxed);
    if raw != 0 {
        unsafe {
            let hwnd = HWND(raw as *mut _);
            let _ = ShowWindow(hwnd, windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
        }
    }
}

unsafe fn taskbar_hwnd() -> windows::core::Result<HWND> {
    let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
    FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null())
}

unsafe fn ensure_child(taskbar: HWND) -> windows::core::Result<HWND> {
    let existing = CHILD.load(Ordering::Relaxed);
    if existing != 0 {
        let hwnd = HWND(existing as *mut _);
        if windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(hwnd)).as_bool() {
            return Ok(hwnd);
        }
        CHILD.store(0, Ordering::Relaxed);
    }

    REGISTER_CLASS.call_once(|| {
        let class_name: Vec<u16> = format!("{CLASS_NAME}\0").encode_utf16().collect();
        let instance = unsafe { GetModuleHandleW(PCWSTR::null()) }
            .map(|m| m.into())
            .unwrap_or_default();
        let class = WNDCLASSW {
            lpfnWndProc: Some(taskbar_wnd_proc),
            hInstance: instance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        let _ = unsafe { RegisterClassW(&class) };
    });

    let class_name: Vec<u16> = format!("{CLASS_NAME}\0").encode_utf16().collect();
    let title: Vec<u16> = "\0".encode_utf16().collect();
    let hwnd = CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
        PCWSTR(class_name.as_ptr()),
        PCWSTR(title.as_ptr()),
        WS_POPUP,
        0,
        0,
        1,
        1,
        None,
        None,
        Some(GetModuleHandleW(PCWSTR::null())?.into()),
        None,
    )?;
    // Windows 11's taskbar composition bridge can cover an ordinary child
    // HWND. TrafficMonitor's GDI renderer uses a layered color-key window;
    // mirror that so Explorer composites the text over its taskbar surface.
    if let Err(e) = SetLayeredWindowAttributes(hwnd, COLORREF(1), 255, LWA_COLORKEY) {
        diagnostic(&format!("SetLayeredWindowAttributes failed: {e}"));
    }
    // TrafficMonitor creates its taskbar dialog first, then reparents it.
    // Creating a WS_CHILD+WS_EX_LAYERED window directly under Explorer can
    // fail on Windows 11, so follow the same top-level-then-parent sequence.
    let parent_result = SetParent(hwnd, Some(taskbar));
    if parent_result.is_err() && GetParent(hwnd).ok() != Some(taskbar) {
        return parent_result.map(|_| hwnd);
    }
    let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    let style = (style & !(WS_POPUP.0 as isize)) | WS_CHILD.0 as isize | WS_CLIPSIBLINGS.0 as isize;
    SetWindowLongPtrW(hwnd, GWL_STYLE, style);
    CHILD.store(hwnd.0 as isize, Ordering::Relaxed);
    Ok(hwnd)
}

fn diagnostic(message: &str) {
    let Some(path) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("taskbar-diagnostic.log")))
    else {
        return;
    };
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "{timestamp} {message}");
    }
}

unsafe fn position_child(taskbar: HWND, child: HWND) {
    let mut client = RECT::default();
    if GetClientRect(taskbar, &mut client).is_err() {
        return;
    }
    let mut taskbar_screen = RECT::default();
    let _ = GetWindowRect(taskbar, &mut taskbar_screen);
    let width = client.right - client.left;
    let height = (client.bottom - client.top).max(32);

    // Windows 11 exposes the Start and notification-area windows as children
    // of Shell_TrayWnd. Anchor the metrics between them, with safe fallbacks
    // for builds where Explorer does not expose either child.
    let start_class: Vec<u16> = "Start\0".encode_utf16().collect();
    let notify_class: Vec<u16> = "TrayNotifyWnd\0".encode_utf16().collect();
    let tasklist_class: Vec<u16> = "MSTaskListWClass\0".encode_utf16().collect();
    let rebar_class: Vec<u16> = "ReBarWindow32\0".encode_utf16().collect();
    let start = FindWindowExW(
        Some(taskbar),
        None,
        PCWSTR(start_class.as_ptr()),
        PCWSTR::null(),
    )
    .ok();
    let notify = FindWindowExW(
        Some(taskbar),
        None,
        PCWSTR(notify_class.as_ptr()),
        PCWSTR::null(),
    )
    .ok();
    let rebar = FindWindowExW(
        Some(taskbar),
        None,
        PCWSTR(rebar_class.as_ptr()),
        PCWSTR::null(),
    )
    .ok();
    let tasklist = rebar.and_then(|parent| {
        FindWindowExW(
            Some(parent),
            None,
            PCWSTR(tasklist_class.as_ptr()),
            PCWSTR::null(),
        )
        .ok()
    });
    let mut start_rect = RECT::default();
    let mut notify_rect = RECT::default();
    let mut tasklist_rect = RECT::default();
    let start_right = start
        .filter(|h| GetWindowRect(*h, &mut start_rect).is_ok())
        .map(|_| start_rect.right - taskbar_screen.left + 8)
        .unwrap_or(190);
    // The Start button is not the end of the taskbar controls on Windows 11:
    // search and pinned apps live in MSTaskListWClass. Reserve their full
    // rectangle so metrics begin in the free section instead of covering icons.
    let controls_right = tasklist
        .or(rebar)
        .filter(|h| GetWindowRect(*h, &mut tasklist_rect).is_ok())
        .map(|_| tasklist_rect.right - taskbar_screen.left + 8)
        .unwrap_or(start_right);
    let notify_left = notify
        .filter(|h| GetWindowRect(*h, &mut notify_rect).is_ok())
        .map(|_| notify_rect.left - taskbar_screen.left - 8)
        .unwrap_or(width - 250);
    // On current Win11 builds the composition-hosted pinned buttons extend a
    // little past the legacy task-list HWND's reported rectangle. Keep a
    // 180-DPI-unit gutter after that HWND so labels do not cover pinned icons.
    let left = controls_right
        .saturating_add(180)
        .max(start_right)
        .clamp(0, width.saturating_sub(240));
    let right = notify_left.clamp(left + 1, width);
    let child_width = (right - left).max(1);
    if !POSITION_LOGGED.swap(true, Ordering::Relaxed) {
        diagnostic(&format!(
            "taskbar metric bounds: controls_right={controls_right}, notify_left={notify_left}, left={left}, width={child_width}"
        ));
    }

    let _ = SetWindowPos(
        child,
        None,
        left,
        0,
        child_width,
        height,
        SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
    );
}

unsafe extern "system" fn taskbar_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut paint = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut paint);
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let transparent_brush = CreateSolidBrush(COLORREF(1));
            let _ = FillRect(hdc, &rect, transparent_brush);
            let _ = DeleteObject(transparent_brush.into());

            let point_size = f32::from_bits(FONT_SIZE_BITS.load(Ordering::Relaxed));
            let height = -((point_size * 96.0 / 72.0).round() as i32).max(8);
            let face = FONT_NAME
                .lock()
                .map(|s| s.clone())
                .unwrap_or_else(|_| "Segoe UI".to_string());
            let face_wide: Vec<u16> = format!("{face}\0").encode_utf16().collect();
            let font = CreateFontW(
                height,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                VARIABLE_PITCH.0 as u32,
                PCWSTR(face_wide.as_ptr()),
            );
            let old_font = SelectObject(hdc, font.into());
            let rgb = TEXT_RGB.load(Ordering::Relaxed);
            let _ = SetTextColor(hdc, COLORREF(rgb));
            let _ = SetBkMode(hdc, BACKGROUND_MODE(TRANSPARENT.0));

            if let Ok(columns) = DRAW_COLUMNS.lock() {
                if !columns.is_empty() {
                    let spacing = COLUMN_SPACING_PX.load(Ordering::Relaxed) as i32;
                    let widths: Vec<i32> = columns
                        .iter()
                        .map(|(top, bottom)| measure_text(hdc, top).max(measure_text(hdc, bottom)))
                        .collect();
                    let total_width = widths.iter().sum::<i32>()
                        + spacing * widths.len().saturating_sub(1) as i32;
                    let mut x = (rect.right - total_width - 8).max(8);
                    let half = (rect.bottom - rect.top) / 2;
                    for (index, ((top, bottom), width)) in
                        columns.iter().zip(widths.iter()).enumerate()
                    {
                        for (row_index, line) in [top, bottom].iter().enumerate() {
                            let mut wide: Vec<u16> = line.encode_utf16().collect();
                            let mut row = RECT {
                                left: x,
                                top: row_index as i32 * half,
                                right: (x + *width).min(rect.right),
                                bottom: (row_index as i32 + 1) * half,
                            };
                            let _ = DrawTextW(
                                hdc,
                                &mut wide,
                                &mut row,
                                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                            );
                        }
                        x += *width + spacing;
                        let _ = index;
                    }
                }
            }
            let _ = SelectObject(hdc, old_font);
            let _ = DeleteObject(font.into());
            let _ = EndPaint(hwnd, &paint);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn measure_text(hdc: windows::Win32::Graphics::Gdi::HDC, text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let mut bounds = RECT::default();
    let _ = DrawTextW(
        hdc,
        &mut wide,
        &mut bounds,
        DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX,
    );
    (bounds.right - bounds.left).max(0)
}
