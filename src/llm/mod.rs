mod parser;
use crate::config::Settings;
pub use parser::{LlmParser, LlmPhase};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time,
};

#[cfg(windows)]
// Keep the background console-only LM Studio CLI from flashing a terminal window.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LlmSnapshot {
    pub server_available: bool,
    pub model: Option<String>,
    pub phase: LlmPhase,
    pub progress_pct: Option<f32>,
    pub speed_tps: Option<f32>,
    pub final_speed_tps: Option<f32>,
}

pub fn start(out: Arc<RwLock<LlmSnapshot>>, settings: Settings) {
    std::thread::Builder::new().name("m4-llm-provider".into()).spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        rt.block_on(async move {
            let mut parser = LlmParser::default();
            let api = reqwest::Client::builder().timeout(Duration::from_secs(2)).build().unwrap();
            let base = settings.llm_api.trim_end_matches('/').to_owned();
            let mut inventory_tick = time::interval(Duration::from_secs(3));
            loop {
                let cli = find_lms(&settings.lms_path);
                if let Some(cli) = cli {
                    let mut command = Command::new(cli);
                    command
                        .args(["log", "stream", "--source", "runtime"])
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::null());
                    #[cfg(windows)]
                    command.creation_flags(CREATE_NO_WINDOW);
                    let mut child = match command.spawn() {
                        Ok(p) => p, Err(e) => { tracing::warn!("Unable to start lms runtime stream: {e}"); time::sleep(Duration::from_secs(4)).await; continue; }
                    };
                    if let Some(stdout) = child.stdout.take() {
                        let mut lines = BufReader::new(stdout).lines();
                        loop {
                            tokio::select! {
                                line = lines.next_line() => match line {
                                    Ok(Some(line)) => { parser.process(&line); if let Ok(mut s)=out.write(){*s=parser.snapshot();} }
                                    _ => break,
                                },
                                _ = inventory_tick.tick() => {
                                    let response = api.get(format!("{base}/api/v1/models")).send().await;
                                    let inventory = match response { Ok(r) if r.status().is_success()=>r.json::<serde_json::Value>().await.ok(), _=>None };
                                    let loaded = inventory.is_some();
                                    let models = inventory.as_ref().map(read_loaded_models).unwrap_or_default();
                                    parser.set_inventory(loaded,models);
                                    if let Ok(mut s)=out.write(){*s=parser.snapshot();}
                                }
                            }
                        }
                    }
                    let _ = child.kill().await;
                } else { time::sleep(Duration::from_secs(4)).await; }
            }
        });
    }).expect("spawn llm collector");
}
fn read_loaded_models(root: &serde_json::Value) -> Vec<String> {
    root.get("models")
        .and_then(|m| m.as_array())
        .into_iter()
        .flatten()
        .filter(|m| m.get("type").and_then(|x| x.as_str()) == Some("llm"))
        .filter(|m| {
            m.get("loaded_instances")
                .and_then(|x| x.as_array())
                .is_some_and(|a| !a.is_empty())
        })
        .filter_map(|m| m.get("key").and_then(|x| x.as_str()).map(str::to_owned))
        .collect()
}
fn find_lms(configured: &str) -> Option<std::path::PathBuf> {
    let candidates = [
        configured.to_owned(),
        std::env::var("USERPROFILE")
            .map(|p| format!("{p}\\.lmstudio\\bin\\lms.exe"))
            .unwrap_or_default(),
        "lms.exe".into(),
    ];
    candidates
        .iter()
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .find(|p| p.is_file() || p == std::path::Path::new("lms.exe"))
}
