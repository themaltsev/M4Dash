#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
mod config;
mod hardware;
mod llm;
#[cfg(windows)]
mod startup;
#[cfg(windows)]
mod taskbar;

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

    let (instance_mutex, already_running) = single_instance_mutex()?;
    if already_running {
        activate_existing_window();
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(instance_mutex) };
        return Ok(());
    }

    let settings = config::Settings::load()?;
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
            .with_title("M4 PC Monitor")
            .with_visible(true)
            .with_active(!settings.start_in_tray)
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([760.0, 520.0])
            .with_icon(std::sync::Arc::new(app::icon_data())),
        ..Default::default()
    };
    let result = eframe::run_native(
        "M4 PC Monitor",
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
    use std::hash::{Hash, Hasher};
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
            System::Threading::CreateMutexW,
        },
    };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::env::current_exe()?.hash(&mut hasher);
    let name: Vec<u16> = format!("Local\\M4PCMonitor-{:016x}\0", hasher.finish())
        .encode_utf16()
        .collect();
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
    let title: Vec<u16> = "M4 PC Monitor\0".encode_utf16().collect();
    if let Ok(window) = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) } {
        unsafe {
            let _ = ShowWindow(window, SW_RESTORE);
            let _ = SetForegroundWindow(window);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    println!("M4 PC Monitor is a Windows-only application.");
}
