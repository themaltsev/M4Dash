//! GitHub release checking and confirmation-gated Windows self-update.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use windows::{
    core::PCWSTR,
    Win32::UI::WindowsAndMessaging::{
        MessageBoxW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_TOPMOST,
        MB_YESNO,
    },
};

static UPDATE_READY: AtomicBool = AtomicBool::new(false);
const REPOSITORY: &str = "themaltsev/m4-pc-monitor";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn start_check() {
    let _ = thread::Builder::new()
        .name("m4-update-check".into())
        .spawn(|| {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    tracing::warn!("Could not start update check: {error}");
                    return;
                }
            };
            if let Err(error) = runtime.block_on(check_and_offer()) {
                tracing::warn!("Update check failed: {error:#}");
            }
        });
}

pub fn update_ready() -> bool {
    UPDATE_READY.swap(false, Ordering::AcqRel)
}

async fn check_and_offer() -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("M4-PC-Monitor/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .build()?;
    let release: Release = client
        .get(format!(
            "https://api.github.com/repos/{REPOSITORY}/releases/latest"
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let latest = release.tag_name.trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");
    if compare_versions(latest, current) <= 0 {
        return Ok(());
    }

    let installer_name = format!("M4-PC-Monitor-Setup-{latest}-x64.exe");
    let installer = release
        .assets
        .iter()
        .find(|asset| asset.name == installer_name)
        .ok_or_else(|| anyhow::anyhow!("latest release has no Windows setup asset"))?;
    let language = crate::config::Settings::load()
        .map(|settings| settings.language)
        .unwrap_or_default();
    if !confirm_update(language, latest) {
        return Ok(());
    }

    let temp_dir = std::env::temp_dir();
    let setup_path = temp_dir.join(format!(
        "M4PCMonitor-Setup-{latest}-{}.exe",
        std::process::id()
    ));
    let bytes = client
        .get(&installer.browser_download_url)
        .timeout(Duration::from_secs(300))
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let checksum_name = format!("{installer_name}.sha256");
    if let Some(checksum) = release
        .assets
        .iter()
        .find(|asset| asset.name == checksum_name)
    {
        let expected = client
            .get(&checksum.browser_download_url)
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if expected.len() != 64 || expected != actual {
            show_error(
                language,
                crate::i18n::tr(language, "Проверка SHA-256 установщика не пройдена"),
            );
            anyhow::bail!("setup SHA-256 verification failed");
        }
    }
    if bytes.len() < 64 * 1024 || !bytes.starts_with(b"MZ") {
        show_error(
            language,
            crate::i18n::tr(
                language,
                "Файл установщика повреждён или имеет неверный формат",
            ),
        );
        anyhow::bail!("downloaded setup is not a valid Windows executable");
    }
    fs::write(&setup_path, &bytes)?;

    let executable = std::env::current_exe()?;
    let install_dir = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("application directory is unavailable"))?;
    if let Err(error) = launch_installer_after_exit(&setup_path, &executable, install_dir) {
        let _ = fs::remove_file(&setup_path);
        show_error(
            language,
            &format!(
                "{}: {error}",
                crate::i18n::tr(language, "Не удалось запустить установщик")
            ),
        );
        return Err(error);
    }
    UPDATE_READY.store(true, Ordering::Release);
    Ok(())
}

fn compare_versions(left: &str, right: &str) -> i32 {
    let parse = |value: &str| -> Vec<u64> {
        value
            .split('.')
            .map(|part| part.split('-').next().unwrap_or(part).parse().unwrap_or(0))
            .collect()
    };
    let left = parse(left);
    let right = parse(right);
    for index in 0..left.len().max(right.len()) {
        let a = *left.get(index).unwrap_or(&0);
        let b = *right.get(index).unwrap_or(&0);
        match a.cmp(&b) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => {}
        }
    }
    0
}

fn confirm_update(language: crate::config::Language, version: &str) -> bool {
    let message = crate::i18n::tr_format(
        language,
        "Доступна новая версия {}. Скачать и установить её?",
        &[version.to_owned()],
    );
    message_box(
        &message,
        crate::i18n::tr(language, "Обновление M4 PC Monitor"),
        MB_YESNO | MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST,
    ) == IDYES
}

fn show_error(language: crate::config::Language, message: &str) {
    message_box(
        message,
        crate::i18n::tr(language, "Ошибка обновления"),
        MB_OK | MB_ICONERROR | MB_SETFOREGROUND | MB_TOPMOST,
    );
}

fn message_box(
    message: &str,
    title: &str,
    flags: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT {
    let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(message.as_ptr()),
            PCWSTR(title.as_ptr()),
            flags,
        )
    }
}

fn ps_quote(value: &Path) -> String {
    value.to_string_lossy().replace('\'', "''")
}

fn launch_installer_after_exit(
    setup: &Path,
    executable: &Path,
    install_dir: &Path,
) -> anyhow::Result<()> {
    let script =
        std::env::temp_dir().join(format!("M4PCMonitor-Update-{}.ps1", std::process::id()));
    let script_contents = format!(
        r#"$ErrorActionPreference = 'Stop'
$pidToWait = {pid}
$setup = '{setup}'
$exe = '{exe}'
$installDir = '{install_dir}'
Add-Type -AssemblyName System.Windows.Forms
while (Get-Process -Id $pidToWait -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 350 }}
try {{
  $result = Start-Process -FilePath $setup -ArgumentList @('/S', "/D=$installDir") -Wait -PassThru
  if ($result.ExitCode -ne 0) {{ throw "Installer exited with code $($result.ExitCode)" }}
  Start-Process -FilePath $exe
}} catch {{
  [void][System.Windows.Forms.MessageBox]::Show("M4 PC Monitor update failed: $($_.Exception.Message)", 'M4 PC Monitor', 'OK', 'Error')
  Start-Process -FilePath $exe -ErrorAction SilentlyContinue
}}
Remove-Item -LiteralPath $setup -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
"#,
        pid = std::process::id(),
        setup = ps_quote(setup),
        exe = ps_quote(executable),
        install_dir = ps_quote(install_dir),
    );
    fs::write(&script, script_contents)?;
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
        ])
        .arg(&script)
        .spawn()?;
    Ok(())
}
