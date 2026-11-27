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
    Russian,
    #[default]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn is_dark(self, system_is_dark: bool) -> bool {
        match self {
            Self::System => system_is_dark,
            Self::Light => false,
            Self::Dark => true,
        }
    }
}

#[cfg(windows)]
pub fn system_prefers_dark_theme() -> bool {
    use windows::{
        core::PCWSTR,
        Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
            REG_VALUE_TYPE,
        },
    };

    let key_name: Vec<u16> = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let value_name: Vec<u16> = "AppsUseLightTheme".encode_utf16().chain(Some(0)).collect();
    let mut key = HKEY::default();
    if unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            KEY_QUERY_VALUE,
            &mut key,
        )
    }
    .is_err()
    {
        return false;
    }

    let mut value = 1u32;
    let mut value_size = std::mem::size_of::<u32>() as u32;
    let mut value_type = REG_VALUE_TYPE(0);
    let queried = unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some((&mut value as *mut u32).cast()),
            Some(&mut value_size),
        )
    }
    .is_ok();
    let _ = unsafe { RegCloseKey(key) };
    queried && value_size >= std::mem::size_of::<u32>() as u32 && value == 0
}

#[cfg(windows)]
pub fn system_accent_color() -> Option<[u8; 3]> {
    use windows::{
        core::PCWSTR,
        Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
            REG_VALUE_TYPE,
        },
    };

    let key_name: Vec<u16> = r"Software\Microsoft\Windows\DWM"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut key = HKEY::default();
    if unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            KEY_QUERY_VALUE,
            &mut key,
        )
    }
    .is_err()
    {
        return None;
    }

    // AccentColor tracks the user's selected Windows accent. Older Windows
    // builds may only expose ColorizationColor, so try it as a fallback.
    let mut color = None;
    for value in ["AccentColor", "ColorizationColor"] {
        let value_name: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
        let mut raw = 0u32;
        let mut value_size = std::mem::size_of::<u32>() as u32;
        let mut value_type = REG_VALUE_TYPE(0);
        let queried = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(value_name.as_ptr()),
                None,
                Some(&mut value_type),
                Some((&mut raw as *mut u32).cast()),
                Some(&mut value_size),
            )
        }
        .is_ok();
        if queried && value_size >= std::mem::size_of::<u32>() as u32 {
            // These registry values are stored as AARRGGBB.
            color = Some([
                ((raw >> 16) & 0xff) as u8,
                ((raw >> 8) & 0xff) as u8,
                (raw & 0xff) as u8,
            ]);
            break;
        }
    }
    let _ = unsafe { RegCloseKey(key) };
    color
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
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default, rename = "dark_theme", skip_serializing)]
    pub legacy_dark_theme: Option<bool>,
    pub refresh_ms: u64,
    pub llm_api: String,
    pub lms_path: String,
    pub show_taskbar: bool,
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
            schema_version: 2,
            language: Language::default(),
            theme_mode: ThemeMode::System,
            legacy_dark_theme: None,
            refresh_ms: 200,
            llm_api: "http://127.0.0.1:1234".into(),
            lms_path: String::new(),
            show_taskbar: true,
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
    pub fn data_dir() -> Result<PathBuf> {
        let exe = std::env::current_exe().context("cannot locate portable executable")?;
        let dir = exe
            .parent()
            .context("portable executable has no parent directory")?;
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        if let Some(local_app_data) = local_app_data {
            let legacy = local_app_data.join("Programs").join("M4 PC Monitor");
            if dir
                .to_string_lossy()
                .eq_ignore_ascii_case(&legacy.to_string_lossy())
            {
                return Ok(local_app_data.join("Programs").join("M4Dash"));
            }
        }
        Ok(dir.to_path_buf())
    }

    pub fn path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("settings.json"))
    }
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<Self>(&bytes) {
                Ok(mut s) => {
                    if let Some(was_dark) = s.legacy_dark_theme.take() {
                        // Older settings only represented a dark-mode checkbox.
                        // Preserve an explicit dark choice; unchecked now follows system.
                        s.theme_mode = if was_dark {
                            ThemeMode::Dark
                        } else {
                            ThemeMode::System
                        };
                    }
                    s.schema_version = 2;
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
