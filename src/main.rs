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
use std::sync::{Arc, RwLock};

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
    let llm_state = Arc::new(RwLock::new(llm::LlmSnapshot::default()));
    hardware::start(snapshot.clone(), settings.clone());
    llm::start(llm_state.clone(), settings.clone());

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("M4 PC Monitor")
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
                settings, snapshot, llm_state,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[cfg(not(windows))]
fn main() {
    println!("M4 PC Monitor is a Windows-only application.");
}
