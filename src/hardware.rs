use crate::config::Settings;
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use sysinfo::{Disks, Networks, System};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub cpu_name: String,
    pub cpu_usage: f32,
    pub cpu_frequency_mhz: u64,
    pub cpu_threads: usize,
    pub cpu_temperature_c: Option<f32>,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub gpu_name: Option<String>,
    pub gpu_utilization_pct: Option<u32>,
    pub gpu_temp_c: Option<u32>,
    pub vram_used_mib: Option<u32>,
    pub vram_total_mib: Option<u32>,
    pub disks: Vec<DiskMetric>,
    pub networks: Vec<NetworkMetric>,
    pub sampled_unix_ms: u128,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraSnapshot {
    pub cpu_temperature_c: Option<f32>,
    pub cpu_temperature_sensor: Option<String>,
    pub motherboard_name: Option<String>,
    pub motherboard_sensors: Vec<BoardSensor>,
    pub provider_error: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardSensor {
    pub name: String,
    pub kind: String,
    pub value: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiskMetric {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub total_bytes: u64,
    pub read_bytes_per_sec: u64,
    pub written_bytes_per_sec: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkMetric {
    pub name: String,
    pub received_bytes_per_sec: u64,
    pub sent_bytes_per_sec: u64,
}

pub fn start(
    out: Arc<RwLock<Snapshot>>,
    extra: Arc<RwLock<ExtraSnapshot>>,
    settings: Settings,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
) {
    start_lhm_provider(extra, shutdown);
    thread::Builder::new()
        .name("m4-hardware-monitor".into())
        .spawn(move || {
            let mut sys = System::new_all();
            let mut disks = Disks::new_with_refreshed_list();
            let mut networks = Networks::new_with_refreshed_list();
            let mut nvml = nvml_wrapper::Nvml::init().ok();
            let mut last_sample = std::time::Instant::now();
            loop {
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                disks.refresh(true);
                networks.refresh(true);
                let elapsed_secs = last_sample.elapsed().as_secs_f64().max(0.001);
                last_sample = std::time::Instant::now();
                let cpu_name = sys
                    .cpus()
                    .first()
                    .map(|c| c.brand().to_string())
                    .unwrap_or_else(|| "CPU".into());
                let cpu_usage = sys.global_cpu_usage();
                let cpu_frequency_mhz = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
                let cpu_threads = sys.cpus().len();
                // sysinfo/WMI ACPI readings cannot be identified as CPU package/Tctl, so never label them as CPU temperature.
                let cpu_temperature_c: Option<f32> = None;
                let gpu = nvml
                    .as_mut()
                    .and_then(|n| n.device_by_index(0).ok())
                    .map(|d| {
                        let name = d.name().unwrap_or_else(|_| "NVIDIA GPU".into());
                        let util = d.utilization_rates().ok().map(|u| u.gpu);
                        let temp = d
                            .temperature(
                                nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu,
                            )
                            .ok();
                        let mem = d.memory_info().ok();
                        (
                            name,
                            util,
                            temp,
                            mem.as_ref().map(|m| (m.used / 1048576) as u32),
                            mem.map(|m| (m.total / 1048576) as u32),
                        )
                    });
                if nvml.is_some() && gpu.is_none() {
                    nvml = None;
                }
                let snapshot = Snapshot {
                    cpu_name,
                    cpu_usage,
                    cpu_frequency_mhz,
                    cpu_threads,
                    cpu_temperature_c,
                    ram_used_bytes: sys.used_memory(),
                    ram_total_bytes: sys.total_memory(),
                    gpu_name: gpu.as_ref().map(|g| g.0.clone()),
                    gpu_utilization_pct: gpu.as_ref().and_then(|g| g.1),
                    gpu_temp_c: gpu.as_ref().and_then(|g| g.2),
                    vram_used_mib: gpu.as_ref().and_then(|g| g.3),
                    vram_total_mib: gpu.as_ref().and_then(|g| g.4),
                    disks: disks
                        .iter()
                        .map(|d| {
                            let usage = d.usage();
                            DiskMetric {
                                id: format!(
                                    "{}|{}",
                                    d.mount_point().to_string_lossy(),
                                    d.name().to_string_lossy()
                                ),
                                name: d.name().to_string_lossy().into_owned(),
                                kind: format!("{:?}", d.kind()),
                                total_bytes: d.total_space(),
                                read_bytes_per_sec: (usage.read_bytes as f64 / elapsed_secs) as u64,
                                written_bytes_per_sec: (usage.written_bytes as f64 / elapsed_secs)
                                    as u64,
                            }
                        })
                        .collect(),
                    networks: networks
                        .iter()
                        .map(|(name, n)| NetworkMetric {
                            name: name.clone(),
                            received_bytes_per_sec: (n.received() as f64 / elapsed_secs) as u64,
                            sent_bytes_per_sec: (n.transmitted() as f64 / elapsed_secs) as u64,
                        })
                        .collect(),
                    sampled_unix_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                };
                if let Ok(mut target) = out.write() {
                    *target = snapshot;
                }
                thread::sleep(Duration::from_millis(settings.refresh_ms.max(500)));
            }
        })
        .expect("spawn hardware collector");
}

fn start_lhm_provider(
    out: Arc<RwLock<ExtraSnapshot>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
) {
    thread::Builder::new()
        .name("m4-lhm-hardware-provider".into())
        .spawn(move || {
            use std::os::windows::process::CommandExt;
            use std::sync::atomic::Ordering;

            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let Some(dir) = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(ToOwned::to_owned))
            else {
                return;
            };
            let script = dir.join("hardware-sensors.ps1");
            let library = dir.join("LibreHardwareMonitorLib.dll");
            if !script.is_file() || !library.is_file() {
                if let Ok(mut sample) = out.write() {
                    sample.provider_error = Some("LHM sensor provider files are missing".into());
                }
                return;
            }

            let mut child = match Command::new("powershell.exe")
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ])
                .arg(script)
                .arg("-LhmPath")
                .arg(library)
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => child,
                Err(error) => {
                    if let Ok(mut sample) = out.write() {
                        sample.provider_error =
                            Some(format!("Unable to start sensor provider: {error}"));
                    }
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if shutdown.load(Ordering::Acquire) {
                        break;
                    }
                    match line {
                        Ok(line) if !line.trim().is_empty() => {
                            match serde_json::from_str::<ExtraSnapshot>(&line) {
                                Ok(sample) => {
                                    if let Ok(mut target) = out.write() {
                                        *target = sample;
                                    }
                                }
                                Err(error) => {
                                    tracing::debug!("Invalid LHM sensor sample: {error}");
                                    if let Ok(mut sample) = out.write() {
                                        sample.provider_error =
                                            Some(format!("Invalid LHM sample: {error}"));
                                    }
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(error) => {
                            tracing::warn!("LHM sensor provider stream failed: {error}");
                            if let Ok(mut sample) = out.write() {
                                sample.provider_error = Some(format!("LHM stream failed: {error}"));
                            }
                            break;
                        }
                    }
                }
            }
            let _ = child.kill();
            let _ = child.wait();
        })
        .expect("spawn LHM hardware provider");
}
