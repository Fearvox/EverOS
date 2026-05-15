use crate::constants::RUNS_DIR;
use crate::context::Context;
use crate::model::{RunView, Verdict};
use crate::sanitizer::{sanitize_json, sanitize_text};
use crate::util::{one_line, path_for_display, truncate};
use serde_json::Value;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Instant;

pub struct VerifyResult {
    pub command: Vec<String>,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub verdict: Verdict,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_verify(ctx: &Context) -> VerifyResult {
    let command = vec![
        "node".to_string(),
        "bin/raven-run.mjs".to_string(),
        "verify".to_string(),
        "raven/fixtures/doomsday-run.json".to_string(),
    ];

    let started = Instant::now();
    let output = Command::new("node")
        .arg("bin/raven-run.mjs")
        .arg("verify")
        .arg("raven/fixtures/doomsday-run.json")
        .current_dir(&ctx.root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let duration_ms = started.elapsed().as_millis();

    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(1);
            let verdict = match exit_code {
                0 => Verdict::Pass,
                2 => Verdict::Block,
                _ => Verdict::Flag,
            };
            VerifyResult {
                command,
                exit_code,
                duration_ms,
                verdict,
                stdout: sanitize_text(&String::from_utf8_lossy(&output.stdout)),
                stderr: sanitize_text(&String::from_utf8_lossy(&output.stderr)),
            }
        }
        Err(err) => VerifyResult {
            command,
            exit_code: 1,
            duration_ms,
            verdict: Verdict::Flag,
            stdout: String::new(),
            stderr: sanitize_text(&format!("failed to spawn verifier: {err}")),
        },
    }
}

pub fn list_runs(ctx: &Context) -> Vec<RunView> {
    let dir = ctx.root.join(RUNS_DIR);
    let mut saved = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|item| item.to_str()) != Some("json") {
                continue;
            }
            if let Ok(text) = fs::read_to_string(&path) {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    let safe = sanitize_json(&value).unwrap_or(Value::Null);
                    saved.push(RunView {
                        id: safe
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("saved-receipt")
                            .to_string(),
                        command: safe
                            .get("command")
                            .map(|value| one_line(&value.to_string()))
                            .unwrap_or_else(|| "unknown".to_string()),
                        verdict: safe
                            .get("verdict")
                            .and_then(Value::as_str)
                            .map(Verdict::from_packet_word)
                            .unwrap_or(Verdict::Flag),
                        source: "saved-receipt".to_string(),
                        evidence: safe
                            .get("evidence_excerpt")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .unwrap_or_else(|| "receipt present".to_string()),
                        receipt_path: Some(path_for_display(&path)),
                    });
                }
            }
        }
    }

    if !saved.is_empty() {
        saved.sort_by(|left, right| left.id.cmp(&right.id));
        return saved;
    }

    ctx.packet
        .gates
        .iter()
        .map(|gate| RunView {
            id: gate.id.clone(),
            command: gate.command.clone().unwrap_or_else(|| "manual".to_string()),
            verdict: Verdict::from_packet_word(&gate.status),
            source: "configured-gate".to_string(),
            evidence: sanitize_text(&truncate(&one_line(&gate.evidence), 260)),
            receipt_path: None,
        })
        .collect()
}
