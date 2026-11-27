use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub refresh_ms: u64,
    pub llm_api: String,
    pub lms_path: String,
    pub show_taskbar: bool,
    pub dark_theme: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_disks: bool,
    pub show_network: bool,
    pub show_llm: bool,
    pub font_size: f32,
    pub text_color: [u8; 3],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            refresh_ms: 200,
            llm_api: "http://127.0.0.1:1234".into(),
            lms_path: String::new(),
            show_taskbar: true,
            dark_theme: true,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_disks: true,
            show_network: true,
            show_llm: true,
            font_size: 14.0,
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
