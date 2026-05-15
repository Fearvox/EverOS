use crate::context::Context;
use crate::model::{MemoryHealth, MemorySearchResult, Verdict};
use crate::sanitizer::{sanitize_text, sanitize_value};
use crate::util::{one_line, truncate};
use serde_json::Value;
use std::process::{Command, Stdio};

pub fn health(ctx: &Context) -> MemoryHealth {
    let output = Command::new("node")
        .arg("bin/everos-memory.mjs")
        .arg("health")
        .current_dir(&ctx.root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let status = serde_json::from_str::<Value>(&stdout)
                .ok()
                .and_then(|value| {
                    value
                        .get("status")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| {
                            value
                                .get("data")
                                .and_then(|data| data.get("status"))
                                .and_then(Value::as_str)
                                .map(str::to_string)
                        })
                })
                .unwrap_or_else(|| "available".to_string());
            MemoryHealth {
                verdict: Verdict::Pass,
                status: sanitize_text(&status),
                evidence: sanitize_text(&truncate(&one_line(&stdout), 260)),
            }
        }
        Ok(output) => MemoryHealth {
            verdict: Verdict::Flag,
            status: "unavailable".to_string(),
            evidence: sanitize_text(&format!(
                "everos-memory health exited {}; {}",
                output.status,
                one_line(&String::from_utf8_lossy(&output.stderr))
            )),
        },
        Err(err) => MemoryHealth {
            verdict: Verdict::Flag,
            status: "unavailable".to_string(),
            evidence: sanitize_text(&format!("memory bridge unavailable: {err}")),
        },
    }
}

pub fn search(ctx: &Context, query: &str) -> MemorySearchResult {
    if query.trim().is_empty() {
        return MemorySearchResult {
            query: String::new(),
            verdict: Verdict::Flag,
            evidence: "no query supplied".to_string(),
            result: None,
        };
    }

    let output = Command::new("node")
        .arg("bin/everos-memory.mjs")
        .arg("search")
        .arg(query)
        .current_dir(&ctx.root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = serde_json::from_str::<Value>(&stdout)
                .ok()
                .map(sanitize_value);
            MemorySearchResult {
                query: sanitize_text(query),
                verdict: Verdict::Pass,
                evidence: sanitize_text(&truncate(&one_line(&stdout), 500)),
                result,
            }
        }
        Ok(output) => MemorySearchResult {
            query: sanitize_text(query),
            verdict: Verdict::Flag,
            evidence: sanitize_text(&format!(
                "everos-memory search exited {}; {}",
                output.status,
                one_line(&String::from_utf8_lossy(&output.stderr))
            )),
            result: None,
        },
        Err(err) => MemorySearchResult {
            query: sanitize_text(query),
            verdict: Verdict::Flag,
            evidence: sanitize_text(&format!("memory bridge unavailable: {err}")),
            result: None,
        },
    }
}
