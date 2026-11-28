mod parser;
use crate::config::{LlmProvider, Settings};
pub use parser::{LlmParser, LlmPhase};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
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

pub fn start(out: Arc<RwLock<LlmSnapshot>>, settings: Arc<RwLock<Settings>>) {
    std::thread::Builder::new().name("m4-llm-provider".into()).spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        rt.block_on(async move {
            let mut parser = LlmParser::default();
            // These endpoints are local services; never route loopback traffic through a
            // desktop HTTP proxy inherited from the environment.
            let api = reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();
            let lm_snapshot = Arc::new(RwLock::new(LlmSnapshot::default()));
            let strata_snapshot: Arc<RwLock<Option<(Instant, LlmSnapshot)>>> = Arc::new(RwLock::new(None));
            let poll_api = api.clone();
            let poll_lm = lm_snapshot.clone();
            let poll_strata = strata_snapshot.clone();
            let poll_out = out.clone();
            let poll_settings = settings.clone();
            tokio::spawn(async move {
                let mut tick = time::interval(Duration::from_secs(1));
                let mut previous: Option<(usize, Instant)> = None;
                loop {
                    tick.tick().await;
                    let now = Instant::now();
                    let current = read_settings(&poll_settings);
                    let snapshot = if current.llm_provider != LlmProvider::LmStudio {
                        let base = current.strata_api.trim_end_matches('/');
                        match poll_api.get(format!("{base}/api/stats")).send().await {
                            Ok(r) if r.status().is_success() => r.json::<RustingStrataStats>().await.ok()
                                .map(|stats| strata_snapshot_from_stats(stats, &mut previous, now)),
                            // Older RustingStrata builds expose the OpenAI-compatible model
                            // inventory but not /api/stats. Still show the loaded model name;
                            // live TPS is only available from /api/stats.
                            _ => match poll_api.get(format!("{base}/v1/models")).send().await {
                                Ok(r) if r.status().is_success() => r
                                    .json::<serde_json::Value>()
                                    .await
                                    .ok()
                                    .and_then(strata_snapshot_from_models),
                                _ => None,
                            },
                        }
                    } else {
                        previous = None;
                        None
                    };
                    if let Ok(mut state) = poll_strata.write() {
                        *state = snapshot.map(|s| (now, s));
                    }
                    publish_selected(&poll_out, &poll_lm, &poll_strata, &poll_settings);
                }
            });
            let mut inventory_tick = time::interval(Duration::from_secs(3));
            let mut mode_tick = time::interval(Duration::from_secs(1));
            loop {
                let current = read_settings(&settings);
                if current.llm_provider == LlmProvider::RustingStrata {
                    time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
                let cli = find_lms(&current.lms_path);
                if let Some(cli) = cli {
                    let mut command = Command::new(cli);
                    command
                        .args(["log", "stream", "--source", "runtime"])
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::null())
                        // The collector owns this long-running CLI process; never orphan it when
                        // the app closes or its runtime is shut down.
                        .kill_on_drop(true);
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
                                    Ok(Some(line)) => { parser.process(&line); if let Ok(mut s)=lm_snapshot.write(){*s=parser.snapshot();} publish_selected(&out, &lm_snapshot, &strata_snapshot, &settings); }
                                    _ => break,
                                },
                                _ = inventory_tick.tick() => {
                                    if read_settings(&settings).llm_provider == LlmProvider::RustingStrata { break; }
                                    let base = read_settings(&settings).llm_api.trim_end_matches('/').to_owned();
                                    let response = api.get(format!("{base}/api/v1/models")).send().await;
                                    let inventory = match response { Ok(r) if r.status().is_success()=>r.json::<serde_json::Value>().await.ok(), _=>None };
                                    let loaded = inventory.is_some();
                                    let models = inventory.as_ref().map(read_loaded_models).unwrap_or_default();
                                    parser.set_inventory(loaded,models);
                                    if let Ok(mut s)=lm_snapshot.write(){*s=parser.snapshot();}
                                    publish_selected(&out, &lm_snapshot, &strata_snapshot, &settings);
                                },
                                _ = mode_tick.tick() => {
                                    if read_settings(&settings).llm_provider == LlmProvider::RustingStrata { break; }
                                },
                            }
                        }
                    }
                    let _ = child.kill().await;
                } else {
                    tokio::select! {
                        _ = inventory_tick.tick() => {
                            if read_settings(&settings).llm_provider == LlmProvider::RustingStrata { continue; }
                            let base = read_settings(&settings).llm_api.trim_end_matches('/').to_owned();
                            let response = api.get(format!("{base}/api/v1/models")).send().await;
                            let inventory = match response { Ok(r) if r.status().is_success()=>r.json::<serde_json::Value>().await.ok(), _=>None };
                            let loaded = inventory.is_some();
                            let models = inventory.as_ref().map(read_loaded_models).unwrap_or_default();
                            parser.set_inventory(loaded, models);
                            if let Ok(mut s)=lm_snapshot.write(){*s=parser.snapshot();}
                            publish_selected(&out, &lm_snapshot, &strata_snapshot, &settings);
                        },
                        _ = time::sleep(Duration::from_secs(1)) => {}
                    }
                }
            }
        });
    }).expect("spawn llm collector");
}

fn strata_snapshot_from_models(root: serde_json::Value) -> Option<LlmSnapshot> {
    let model = root
        .get("data")?
        .as_array()?
        .iter()
        .find_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))?
        .trim()
        .to_owned();
    (!model.is_empty()).then_some(LlmSnapshot {
        server_available: true,
        model: Some(model),
        phase: LlmPhase::Idle,
        progress_pct: None,
        speed_tps: None,
        final_speed_tps: None,
    })
}

fn read_settings(settings: &Arc<RwLock<Settings>>) -> Settings {
    settings.read().map(|s| s.clone()).unwrap_or_default()
}

#[derive(Deserialize)]
struct RustingStrataStats {
    model: Option<String>,
    #[serde(default)]
    busy: usize,
    #[serde(default)]
    live_tokens: usize,
    #[serde(default)]
    recent: Vec<RustingStrataRequest>,
}

#[derive(Deserialize)]
struct RustingStrataRequest {
    model: String,
    completion_tokens: usize,
    decode_ms: f64,
}

fn strata_snapshot_from_stats(
    stats: RustingStrataStats,
    previous: &mut Option<(usize, Instant)>,
    now: Instant,
) -> LlmSnapshot {
    let model_loaded = stats.model.is_some();
    let model = stats.model.filter(|m| !m.trim().is_empty()).or_else(|| {
        stats
            .recent
            .last()
            .map(|r| r.model.clone())
            .filter(|m| !m.trim().is_empty())
    });
    let final_speed_tps = stats
        .recent
        .last()
        .and_then(|r| {
            (r.decode_ms > 0.0).then_some(r.completion_tokens as f32 * 1000.0 / r.decode_ms as f32)
        })
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 10000.0);
    let speed_tps = if stats.busy > 0 {
        let result = previous
            .and_then(|(tokens, at)| {
                let elapsed = now.duration_since(at).as_secs_f32();
                (stats.live_tokens >= tokens && elapsed > 0.0)
                    .then_some((stats.live_tokens - tokens) as f32 / elapsed)
            })
            .filter(|v| v.is_finite() && *v > 0.0 && *v < 10000.0);
        *previous = Some((stats.live_tokens, now));
        result
    } else {
        *previous = None;
        None
    };
    LlmSnapshot {
        server_available: true,
        model,
        phase: if stats.busy > 0 {
            if speed_tps.is_some() {
                LlmPhase::Tg
            } else {
                LlmPhase::Preparing
            }
        } else if model_loaded || final_speed_tps.is_some() {
            LlmPhase::Idle
        } else {
            LlmPhase::NoModel
        },
        progress_pct: None,
        speed_tps,
        final_speed_tps,
    }
}

fn publish_selected(
    out: &Arc<RwLock<LlmSnapshot>>,
    lm: &Arc<RwLock<LlmSnapshot>>,
    strata: &Arc<RwLock<Option<(Instant, LlmSnapshot)>>>,
    settings: &Arc<RwLock<Settings>>,
) {
    let preferred = strata.read().ok().and_then(|s| {
        s.as_ref()
            .filter(|(at, _)| at.elapsed() < Duration::from_secs(3))
            .map(|(_, snapshot)| snapshot.clone())
    });
    let fallback = lm.read().ok().map(|s| s.clone()).unwrap_or_default();
    let current = read_settings(settings);
    let selected = match current.llm_provider {
        LlmProvider::Automatic => preferred.unwrap_or(fallback),
        LlmProvider::LmStudio => fallback,
        LlmProvider::RustingStrata => preferred.unwrap_or(LlmSnapshot {
            server_available: false,
            model: None,
            phase: LlmPhase::Offline,
            progress_pct: None,
            speed_tps: None,
            final_speed_tps: None,
        }),
    };
    if let Ok(mut current) = out.write() {
        *current = selected;
    }
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
