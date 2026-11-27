use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmPhase {
    Loading,
    Preparing,
    Pp,
    Tg,
    Unloading,
    Idle,
    Offline,
    Unknown,
    #[default]
    NoModel,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEvent {
    pub phase: LlmPhase,
    pub model: Option<String>,
    pub progress_pct: Option<f32>,
    pub speed_tps: Option<f32>,
    pub slot: Option<String>,
    pub final_sample: bool,
}
#[derive(Default)]
pub struct LlmParser {
    available: bool,
    loaded_model: Option<String>,
    last_model: Option<String>,
    slots: HashMap<String, RuntimeEvent>,
    lifecycle: Option<RuntimeEvent>,
    final_rates: HashMap<String, f32>,
    last_event: Option<Instant>,
}
impl LlmParser {
    pub fn process(&mut self, line: &str) -> Option<RuntimeEvent> {
        let mut event = parse_runtime_line(line)?;
        if event.model.is_none() {
            event.model = self.loaded_model.clone();
        }
        if let (Some(found), Some(canonical)) = (&event.model, &self.loaded_model) {
            if normalize_model(found) == normalize_model(canonical) {
                event.model = Some(canonical.clone());
            }
        }
        if matches!(
            event.phase,
            LlmPhase::Loading
                | LlmPhase::Preparing
                | LlmPhase::Pp
                | LlmPhase::Tg
                | LlmPhase::Unloading
        ) {
            self.last_event = Some(Instant::now());
        }
        let slot = event.slot.clone().unwrap_or_else(|| "anonymous".into());
        match event.phase {
            LlmPhase::Loading | LlmPhase::Preparing | LlmPhase::Unloading => {
                self.lifecycle = Some(event.clone())
            }
            LlmPhase::Pp | LlmPhase::Tg => {
                self.lifecycle = Some(event.clone());
                self.slots.insert(slot, event.clone());
            }
            LlmPhase::Idle => {
                self.slots.remove(&slot);
                if let Some(m) = &event.model {
                    self.last_model = Some(m.clone());
                }
                if event.final_sample {
                    if let (Some(m), Some(v)) = (&event.model, event.speed_tps) {
                        self.final_rates.insert(normalize_model(m), v);
                    }
                }
                if self.slots.is_empty() {
                    self.lifecycle = None;
                }
            }
            _ => {}
        }
        Some(event)
    }
    pub fn set_inventory(&mut self, available: bool, loaded_models: Vec<String>) {
        self.available = available;
        self.loaded_model = if loaded_models.len() == 1 {
            loaded_models.into_iter().next()
        } else {
            None
        };
    }
    pub fn snapshot(&self) -> super::LlmSnapshot {
        let current = self
            .slots
            .values()
            .max_by_key(|e| if e.phase == LlmPhase::Tg { 2 } else { 1 })
            .or(self.lifecycle.as_ref());
        let phase = if let Some(e) = current.filter(|_| {
            self.available
                || self
                    .last_event
                    .is_some_and(|t| t.elapsed() < Duration::from_secs(8))
        }) {
            e.phase
        } else if !self.available {
            LlmPhase::Offline
        } else if self.loaded_model.is_some() {
            LlmPhase::Idle
        } else {
            LlmPhase::NoModel
        };
        let model = current
            .and_then(|e| e.model.clone())
            .or_else(|| self.loaded_model.clone())
            .or_else(|| self.last_model.clone());
        let rate = current.and_then(|e| e.speed_tps);
        let final_speed_tps = model
            .as_ref()
            .and_then(|m| self.final_rates.get(&normalize_model(m)).copied());
        super::LlmSnapshot {
            server_available: self.available,
            model,
            phase,
            progress_pct: current.and_then(|e| e.progress_pct),
            speed_tps: rate,
            final_speed_tps,
        }
    }
}
pub fn parse_runtime_line(line: &str) -> Option<RuntimeEvent> {
    let phase=if Regex::new(r"(?i)\b(unloading|unload(?:ing)?\s+model)\b").ok()?.is_match(line){LlmPhase::Unloading}
    else if Regex::new(r"(?i)\b(loading|load(?:ing)?\s+model)\b").ok()?.is_match(line){LlmPhase::Loading}
    else if Regex::new(r"(?i)\bpreparing\s+(request|prompt|context|kv|generation)\b|\btokeniz(ing|ation)\b|\b(request|task)\b.*\b(started|accepted|queued)\b").ok()?.is_match(line){LlmPhase::Preparing}
    else if Regex::new(r"(?i)\b(prompt processing|processing prompt)\b").ok()?.is_match(line){LlmPhase::Pp}
    else if Regex::new(r"(?i)\btg_3s\s*=").ok()?.is_match(line){LlmPhase::Tg}
    else if Regex::new(r"(?i)\|\s*eval time\s*=").ok()?.is_match(line){LlmPhase::Idle}
    else if Regex::new(r"(?i)\b(slot\b.*\b(release|free|reset|idle)|request\b.*\b(cancel|abort|finished|completed|done))").ok()?.is_match(line){LlmPhase::Idle}else{return None};
    let model = Regex::new(r#"(?i)\bmodel\s*[=:]\s*["']?([A-Za-z0-9][A-Za-z0-9._/-]{2,255})"#)
        .ok()?
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_owned()));
    let slot = Regex::new(r"(?i)\bslot(?:[_\s-]*id)?\s*(?:[#:= >-]\s*)?(\d+)\b")
        .ok()?
        .captures(line)
        .and_then(|c| c.get(1).map(|m| format!("slot:{}", m.as_str())));
    let pct = Regex::new(r"(?i)progress\s*=\s*(\d+(?:\.\d+)?)|(\d+(?:\.\d+)?)\s*%")
        .ok()?
        .captures(line)
        .and_then(|c| c.get(1).or_else(|| c.get(2))?.as_str().parse::<f32>().ok())
        .filter(|p| *p >= 0.0 && *p <= 100.0);
    let progress_pct = if phase == LlmPhase::Loading || phase == LlmPhase::Pp {
        pct.map(|p| {
            if p <= 1.0 && !line.contains('%') {
                p * 100.0
            } else {
                p
            }
        })
    } else {
        None
    };
    let speed = Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:tokens?\s+per\s+second|t/s)")
        .ok()?
        .captures(line)
        .and_then(|c| c.get(1)?.as_str().parse::<f32>().ok())
        .filter(|v| *v > 0.0 && *v < 10000.0);
    Some(RuntimeEvent {
        phase,
        model,
        progress_pct,
        speed_tps: speed,
        slot,
        final_sample: phase == LlmPhase::Idle && line.to_ascii_lowercase().contains("eval time"),
    })
}
pub fn normalize_model(m: &str) -> String {
    let mut s = m.trim().to_ascii_lowercase();
    for ext in [".gguf", ".safetensors", ".bin"] {
        s = s.strip_suffix(ext).unwrap_or(&s).to_owned();
    }
    for q in ["-q4_k_m", "-q5_k_m", "-q8_0", "-f16", "-distill"] {
        if s.ends_with(q) {
            s.truncate(s.len() - q.len());
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_actual_runtime_formats() {
        let e=parse_runtime_line("model=qwen3.8-9b-distill slot=0 prompt processing progress = 0.97 | 1942.1 tokens per second").unwrap();
        assert_eq!(e.phase, LlmPhase::Pp);
        assert_eq!(e.progress_pct, Some(97.));
        assert_eq!(e.speed_tps, Some(1942.1));
    }
    #[test]
    fn processing_start_without_rate_stays_pp() {
        let e = parse_runtime_line("model=qwen3.8-4b-distill slot=0 Processing prompt 0%").unwrap();
        assert_eq!(e.phase, LlmPhase::Pp);
        assert_eq!(e.progress_pct, Some(0.));
        assert_eq!(e.speed_tps, None);
    }
    #[test]
    fn same_model_slots_do_not_become_multi() {
        let mut p = LlmParser::default();
        p.set_inventory(true, vec!["qwen3.8-9b-distill".into()]);
        p.process("model=qwen3.8-9b-distill slot=0 tg_3s = 43.76 t/s");
        p.process("model=qwen3.8-9b-Q4_K_M slot=1 tg_3s = 41.25 t/s");
        assert_eq!(
            normalize_model("qwen3.8-9b-Q4_K_M"),
            normalize_model("qwen3.8-9b-distill")
        );
        assert_eq!(p.snapshot().model.as_deref(), Some("qwen3.8-9b-distill"));
    }
    #[test]
    fn preparation_and_prompt_start_are_active_without_offline_gap() {
        let mut p = LlmParser::default();
        p.set_inventory(true, vec!["qwen3.8-4b-distill".into()]);
        p.process("model=qwen3.8-4b-distill slot=0 request started");
        assert_eq!(p.snapshot().phase, LlmPhase::Preparing);
        p.process("model=qwen3.8-4b-distill slot=0 Processing prompt 0%");
        assert_eq!(p.snapshot().phase, LlmPhase::Pp);
        assert_eq!(p.snapshot().progress_pct, Some(0.));
    }
    #[test]
    fn final_speed_survives_slot_release() {
        let mut p = LlmParser::default();
        p.set_inventory(true, vec!["qwen3.8-9b-distill".into()]);
        p.process("model=qwen3.8-9b-distill slot=0 tg_3s = 43.76 t/s");
        p.process("model=qwen3.8-9b-distill slot=0 | eval time = 1.2 s | 44.10 tokens per second");
        let s = p.snapshot();
        assert_eq!(s.phase, LlmPhase::Idle);
        assert_eq!(s.final_speed_tps, Some(44.10));
    }
}
