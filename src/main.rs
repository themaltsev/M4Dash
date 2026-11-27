#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
mod config;
mod hardware;
mod i18n;
mod llm;
#[cfg(windows)]
mod startup;
#[cfg(windows)]
mod taskbar;
#[cfg(windows)]
mod updater;

#[cfg(windows)]
use anyhow::Result;
#[cfg(windows)]
use std::sync::{atomic::AtomicBool, Arc, RwLock};

#[cfg(windows)]
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("m4_pc_monitor=info".parse()?),
        )
        .with_target(false)
        .init();

    // Read the persisted launch mode before single-instance handling and UAC
    // relaunch so a second launch cannot unexpectedly surface the GUI in tray
    // mode.
    let settings = config::Settings::load()?;
    let (instance_mutex, already_running) = single_instance_mutex()?;
    if already_running {
        if !settings.start_in_tray {
            activate_existing_window();
        }
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(instance_mutex) };
        return Ok(());
    }

    // LibreHardwareMonitor requires an elevated process for Ryzen and board sensors.
    // The initial process releases its singleton before asking Windows for elevation.
    if !unsafe { windows::Win32::UI::Shell::IsUserAnAdmin() }.as_bool() {
        use std::os::windows::ffi::OsStrExt;
        use windows::{
            core::PCWSTR,
            Win32::UI::{
                Shell::ShellExecuteW,
                WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
            },
        };
        let exe: Vec<u16> = std::env::current_exe()?
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let operation: Vec<u16> = "runas\0".encode_utf16().collect();
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(instance_mutex) };
        let show_command = if settings.start_in_tray {
            SW_HIDE
        } else {
            SW_SHOWNORMAL
        };
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR(operation.as_ptr()),
                PCWSTR(exe.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                show_command,
            )
        };
        if result.0 as isize > 32 {
            return Ok(());
        }
        anyhow::bail!("Windows elevation was not granted; CPU temperature sensors require administrator access");
    }

    let snapshot = Arc::new(RwLock::new(hardware::Snapshot::default()));
    let extra = Arc::new(RwLock::new(hardware::ExtraSnapshot::default()));
    let llm_state = Arc::new(RwLock::new(llm::LlmSnapshot::default()));
    let shutdown = Arc::new(AtomicBool::new(false));
    hardware::start(
        snapshot.clone(),
        extra.clone(),
        settings.clone(),
        shutdown.clone(),
    );
    llm::start(llm_state.clone(), settings.clone());

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("M4Dash")
            .with_visible(!settings.start_in_tray)
            .with_active(!settings.start_in_tray)
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([760.0, 520.0])
            .with_icon(std::sync::Arc::new(app::icon_data())),
        ..Default::default()
    };
    let result = eframe::run_native(
        "M4Dash",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(app::MonitorApp::new(
                settings, snapshot, extra, llm_state, shutdown,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()));
    let _ = unsafe { windows::Win32::Foundation::CloseHandle(instance_mutex) };
    result
}

#[cfg(windows)]
fn single_instance_mutex() -> Result<(windows::Win32::Foundation::HANDLE, bool)> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
            System::Threading::CreateMutexW,
        },
    };
    let name: Vec<u16> = "Local\\M4Dash\0".encode_utf16().collect();
    let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr()))? };
    let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    Ok((handle, already_running))
}

#[cfg(windows)]
fn activate_existing_window() {
    use windows::{
        core::PCWSTR,
        Win32::UI::WindowsAndMessaging::{
            FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
        },
    };
    let title: Vec<u16> = "M4Dash\0".encode_utf16().collect();
    if let Ok(window) = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) } {
        unsafe {
            let _ = ShowWindow(window, SW_RESTORE);
            let _ = SetForegroundWindow(window);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    println!("M4Dash is a Windows-only application.");
}
