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
            .with_active(true)
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([760.0, 520.0])
            .with_icon(std::sync::Arc::new(app::icon_data())),
        ..Default::default()
    };
    eframe::run_native(
        "M4 PC Monitor",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(app::MonitorApp::new(
                settings, snapshot, extra, llm_state, shutdown,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[cfg(not(windows))]
fn main() {
    println!("M4 PC Monitor is a Windows-only application.");
}
