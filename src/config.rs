use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::{collections::BTreeMap, fs, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedUnit {
    #[default]
    MebiBytesPerSecond,
    MegaBitsPerSecond,
}

impl SpeedUnit {
    pub fn label(self) -> &'static str {
        match self {
            Self::MebiBytesPerSecond => "MiB/s",
            Self::MegaBitsPerSecond => "Mb/s",
        }
    }

    pub fn value(self, bytes_per_second: u64) -> f64 {
        match self {
            Self::MebiBytesPerSecond => bytes_per_second as f64 / 1_048_576.0,
            Self::MegaBitsPerSecond => bytes_per_second as f64 * 8.0 / 1_000_000.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Russian,
    English,
    PortugueseBrazil,
    Persian,
    Spanish,
    French,
    Italian,
    Chinese,
}

impl Language {
    pub const ALL: [Self; 8] = [
        Self::Russian,
        Self::English,
        Self::PortugueseBrazil,
        Self::Persian,
        Self::Spanish,
        Self::French,
        Self::Italian,
        Self::Chinese,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::Russian => 0,
            Self::English => 1,
            Self::PortugueseBrazil => 2,
            Self::Persian => 3,
            Self::Spanish => 4,
            Self::French => 5,
            Self::Italian => 6,
            Self::Chinese => 7,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Russian => "🇷🇺 Русский",
            Self::English => "🇬🇧 English",
            Self::PortugueseBrazil => "🇧🇷 Português (Brasil)",
            Self::Persian => "🇮🇷 فارسی",
            Self::Spanish => "🇪🇸 Español",
            Self::French => "🇫🇷 Français",
            Self::Italian => "🇮🇹 Italiano",
            Self::Chinese => "🇨🇳 中文",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutGroup {
    pub first: String,
    pub second: String,
    #[serde(default = "default_group_width")]
    pub width_chars: u16,
}

fn default_group_width() -> u16 {
    28
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub language: Language,
    pub refresh_ms: u64,
    pub llm_api: String,
    pub lms_path: String,
    pub show_taskbar: bool,
    pub start_in_tray: bool,
    pub dark_theme: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_disks: bool,
    pub show_network: bool,
    pub show_llm: bool,
    pub show_motherboard: bool,
    pub disk_visibility: BTreeMap<String, bool>,
    pub network_visibility: BTreeMap<String, bool>,
    pub motherboard_sensor_visibility: BTreeMap<String, bool>,
    pub disk_speed_units: BTreeMap<String, SpeedUnit>,
    pub network_speed_unit: SpeedUnit,
    pub custom_fonts: BTreeMap<String, String>,
    pub custom_names: BTreeMap<String, String>,
    pub taskbar_order: Vec<String>,
    pub layout_groups: Vec<LayoutGroup>,
    pub column_spacing_px: u16,
    pub font_size: f32,
    pub font_family: String,
    pub text_color: [u8; 3],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            language: Language::default(),
            refresh_ms: 200,
            llm_api: "http://127.0.0.1:1234".into(),
            lms_path: String::new(),
            show_taskbar: true,
            start_in_tray: false,
            dark_theme: false,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_disks: true,
            show_network: true,
            show_llm: true,
            show_motherboard: true,
            disk_visibility: BTreeMap::new(),
            network_visibility: BTreeMap::new(),
            motherboard_sensor_visibility: BTreeMap::new(),
            disk_speed_units: BTreeMap::new(),
            network_speed_unit: SpeedUnit::default(),
            custom_fonts: BTreeMap::new(),
            custom_names: BTreeMap::new(),
            taskbar_order: Vec::new(),
            layout_groups: vec![
                LayoutGroup {
                    first: "network:rx".into(),
                    second: "network:tx".into(),
                    width_chars: 14,
                },
                LayoutGroup {
                    first: "cpu".into(),
                    second: "ram".into(),
                    width_chars: 30,
                },
                LayoutGroup {
                    first: "gpu".into(),
                    second: "llm".into(),
                    width_chars: 30,
                },
            ],
            column_spacing_px: 16,
            font_size: 14.0,
            font_family: "Segoe UI".into(),
            text_color: [235, 235, 235],
        }
    }
}

impl Settings {
    pub fn path() -> Result<PathBuf> {
        let exe = std::env::current_exe().context("cannot locate portable executable")?;
        let dir = exe
            .parent()
            .context("portable executable has no parent directory")?;
        Ok(dir.join("settings.json"))
    }
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<Self>(&bytes) {
                Ok(mut s) => {
                    s.schema_version = 1;
                    Ok(s)
                }
                Err(e) => {
                    let corrupt = path.with_extension(format!("corrupt-{}.json", chrono_stamp()));
                    let _ = fs::rename(&path, corrupt);
                    tracing::warn!("Settings damaged; defaults loaded: {e}");
                    Ok(Self::default())
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let settings = Self::default();
                settings.save()?;
                Ok(settings)
            }
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let dir = path.parent().context("settings directory missing")?;
        fs::create_dir_all(dir)?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        #[cfg(windows)]
        {
            use windows::core::PCWSTR;
            use windows::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let from: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
            let to: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            unsafe {
                MoveFileExW(
                    PCWSTR(from.as_ptr()),
                    PCWSTR(to.as_ptr()),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            }
            .ok()
            .context("cannot atomically replace portable settings")?;
        }
        #[cfg(not(windows))]
        fs::rename(tmp, path)?;
        Ok(())
    }
}
fn chrono_stamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
