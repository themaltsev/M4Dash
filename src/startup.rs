//! Per-user Windows logon setting for the portable build.
use anyhow::{Context, Result};
use std::{iter::once, path::PathBuf};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
            RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE,
            REG_OPTION_NON_VOLATILE, REG_SZ,
        },
    },
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "M4PCMonitor";

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(once(0)).collect()
}

pub fn is_enabled() -> bool {
    let key_name = wide(RUN_KEY);
    let value_name = wide(VALUE_NAME);
    let mut key = HKEY::default();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if status.is_err() {
        return false;
    }
    let mut size = 0u32;
    let query = unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            None,
            Some(&mut size),
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    query.is_ok()
}

pub fn set_enabled(enabled: bool) -> Result<()> {
    let key_name = wide(RUN_KEY);
    let value_name = wide(VALUE_NAME);
    let mut key = HKEY::default();
    let mut disposition = Default::default();
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_name.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            Some(&mut disposition),
        )
    };
    status
        .ok()
        .context("cannot open current-user Run registry key")?;

    let result = if enabled {
        let current_executable: PathBuf =
            std::env::current_exe().context("cannot locate portable executable")?;
        let executable = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|local_app_data| {
                let legacy = local_app_data.join("Programs").join("M4 PC Monitor");
                if current_executable.parent().is_some_and(|parent| {
                    parent
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&legacy.to_string_lossy())
                }) {
                    local_app_data.join("Programs").join("M4Dash.exe")
                } else {
                    current_executable.clone()
                }
            })
            .unwrap_or(current_executable);
        let command = format!("\"{}\" --background", executable.display());
        let mut data: Vec<u16> = command.encode_utf16().collect();
        data.push(0);
        let bytes =
            unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), data.len() * 2) };
        unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), None, REG_SZ, Some(bytes)) }
            .ok()
            .context("cannot enable M4Dash at sign-in")
    } else {
        let status = unsafe { RegDeleteValueW(key, PCWSTR(value_name.as_ptr())) };
        if status == ERROR_FILE_NOT_FOUND {
            Ok(())
        } else {
            status.ok().context("cannot disable M4Dash at sign-in")
        }
    };
    let _ = unsafe { RegCloseKey(key) };
    result
}
