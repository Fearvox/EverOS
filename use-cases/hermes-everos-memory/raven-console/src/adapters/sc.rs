use crate::model::{
    ScProviderView, ScReport, ScSessionView, ScStatusView, ScWorktreeView, Verdict,
};
use crate::sanitizer::sanitize_text;
use crate::util::{one_line, truncate};
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const SC_TIMEOUT: Duration = Duration::from_secs(5);

struct ScOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
    duration_ms: u128,
    timed_out: bool,
}

pub fn report() -> ScReport {
    let status = status();
    let providers = providers();
    let sessions = sessions();
    let worktree = worktree();
    let verdict = if status.verdict == Verdict::Block || worktree.verdict == Verdict::Block {
        Verdict::Block
    } else if status.verdict == Verdict::Flag || worktree.verdict == Verdict::Flag {
        Verdict::Flag
    } else {
        Verdict::Pass
    };

    ScReport {
        verdict,
        status,
        providers,
        sessions,
        worktree,
    }
}

pub fn boot_report() -> ScReport {
    ScReport {
        verdict: Verdict::Flag,
        status: ScStatusView {
            verdict: Verdict::Flag,
            ok: false,
            api_version: None,
            app_version: "unknown".to_string(),
            evidence: "TUI boot snapshot skips sc socket calls; press u for live refresh."
                .to_string(),
        },
        providers: Vec::new(),
        sessions: Vec::new(),
        worktree: ScWorktreeView {
            verdict: Verdict::Flag,
            branch: "unknown".to_string(),
            target_branch: "unknown".to_string(),
            dirty: None,
            evidence: "refresh pending".to_string(),
        },
    }
}

pub fn status() -> ScStatusView {
    let output = run_sc(&["status", "--json"]);
    if output.exit_code != 0 || output.timed_out {
        return ScStatusView {
            verdict: if output.timed_out {
                Verdict::Block
            } else {
                Verdict::Flag
            },
            ok: false,
            api_version: None,
            app_version: "unknown".to_string(),
            evidence: output_evidence("sc status", &output),
        };
    }

    let Ok(value) = serde_json::from_str::<Value>(&output.stdout) else {
        return ScStatusView {
            verdict: Verdict::Flag,
            ok: false,
            api_version: None,
            app_version: "unknown".to_string(),
            evidence: "sc status returned non-json output".to_string(),
        };
    };

    let ok = value.get("ok").and_then(Value::as_bool).unwrap_or(false);
    let api_version = value.get("api_version").and_then(Value::as_u64);
    let app_version = value
        .get("app_version")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    ScStatusView {
        verdict: if ok { Verdict::Pass } else { Verdict::Flag },
        ok,
        api_version,
        evidence: sanitize_text(&format!(
            "sc socket responded in {}ms; api={}; app={}",
            output.duration_ms,
            api_version
                .map(|version| version.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            app_version
        )),
        app_version,
    }
}

pub fn providers() -> Vec<ScProviderView> {
    let output = run_sc(&["chat", "providers", "--json"]);
    if output.exit_code != 0 || output.timed_out {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(&output.stdout) else {
        return Vec::new();
    };

    value
        .get("providers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|provider| ScProviderView {
            provider_key: string_field(provider, "provider_key"),
            display_name: string_field(provider, "display_name"),
            enabled: provider
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            model_count: provider
                .get("models")
                .and_then(Value::as_array)
                .map(|models| models.len())
                .unwrap_or(0),
            reasoning_efforts: provider
                .get("supported_reasoning_efforts")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect(),
        })
        .collect()
}

pub fn sessions() -> Vec<ScSessionView> {
    let output = run_sc(&["chat", "list", "--json"]);
    if output.exit_code != 0 || output.timed_out {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(&output.stdout) else {
        return Vec::new();
    };

    value
        .get("sessions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|session| ScSessionView {
            thread_id: string_field(session, "thread_id"),
            provider_key: string_field(session, "provider_key"),
            title: string_field(session, "title"),
            model: string_field(session, "model"),
            reasoning_effort: string_field(session, "reasoning_effort"),
            active_turn: session
                .get("active_turn")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            closed: session
                .get("closed")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            branch: string_field(session, "branch"),
            worktree: public_worktree_label(&string_field(session, "worktree_path")),
        })
        .collect()
}

pub fn worktree() -> ScWorktreeView {
    let output = run_sc(&["worktree", "status", "--json"]);
    if output.exit_code != 0 || output.timed_out {
        return ScWorktreeView {
            verdict: if output.timed_out {
                Verdict::Block
            } else {
                Verdict::Flag
            },
            branch: "unknown".to_string(),
            target_branch: "unknown".to_string(),
            dirty: None,
            evidence: output_evidence("sc worktree status", &output),
        };
    }

    let Ok(value) = serde_json::from_str::<Value>(&output.stdout) else {
        return ScWorktreeView {
            verdict: Verdict::Flag,
            branch: "unknown".to_string(),
            target_branch: "unknown".to_string(),
            dirty: None,
            evidence: "sc worktree status returned non-json output".to_string(),
        };
    };

    let branch = first_string(&value, &["branch", "current_branch", "head_branch"]);
    let target_branch = first_string(&value, &["target_branch", "base_branch"]);
    let dirty = value
        .get("dirty")
        .or_else(|| value.get("has_uncommitted_changes"))
        .and_then(Value::as_bool);

    ScWorktreeView {
        verdict: Verdict::Pass,
        branch: branch.unwrap_or_else(|| "unknown".to_string()),
        target_branch: target_branch.unwrap_or_else(|| "unknown".to_string()),
        dirty,
        evidence: format!("sc worktree status completed in {}ms", output.duration_ms),
    }
}

fn run_sc(args: &[&str]) -> ScOutput {
    let start = Instant::now();
    let mut child = match Command::new(sc_binary())
        .args(args)
        .current_dir(sc_cwd())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            return ScOutput {
                exit_code: 127,
                stdout: String::new(),
                stderr: err.to_string(),
                duration_ms: start.elapsed().as_millis(),
                timed_out: false,
            }
        }
    };

    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) if start.elapsed() >= SC_TIMEOUT => {
                let _ = child.kill();
                let output = child.wait_with_output().ok();
                return ScOutput {
                    exit_code: 124,
                    stdout: output
                        .as_ref()
                        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
                        .unwrap_or_default(),
                    stderr: output
                        .as_ref()
                        .map(|output| String::from_utf8_lossy(&output.stderr).to_string())
                        .unwrap_or_else(|| "sc command timed out".to_string()),
                    duration_ms: start.elapsed().as_millis(),
                    timed_out: true,
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(err) => {
                let _ = child.kill();
                return ScOutput {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: err.to_string(),
                    duration_ms: start.elapsed().as_millis(),
                    timed_out: false,
                };
            }
        }
    }

    match child.wait_with_output() {
        Ok(output) => ScOutput {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms: start.elapsed().as_millis(),
            timed_out: false,
        },
        Err(err) => ScOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: err.to_string(),
            duration_ms: start.elapsed().as_millis(),
            timed_out: false,
        },
    }
}

fn sc_binary() -> PathBuf {
    if let Ok(path) = env::var("RAVEN_SC_BIN") {
        return PathBuf::from(path);
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".superconductor/bin/sc"))
        .unwrap_or_else(|| PathBuf::from("sc"))
}

fn sc_cwd() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        if candidate.join(".git").exists() {
            return candidate.to_path_buf();
        }
    }
    cwd
}

fn output_evidence(label: &str, output: &ScOutput) -> String {
    if output.timed_out {
        return format!("{label} timed out after {}ms", output.duration_ms);
    }

    let text = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };
    sanitize_text(&format!(
        "{label} exited {} in {}ms: {}",
        output.exit_code,
        output.duration_ms,
        truncate(&one_line(text), 240)
    ))
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn public_worktree_label(path: &str) -> String {
    let sanitized = sanitize_text(path);
    if sanitized == path {
        return sanitized;
    }

    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("worktree")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::public_worktree_label;

    #[test]
    fn worktree_label_avoids_absolute_paths() {
        assert_eq!(public_worktree_label("/Users/alice/EverOS/"), "EverOS");
    }
}
