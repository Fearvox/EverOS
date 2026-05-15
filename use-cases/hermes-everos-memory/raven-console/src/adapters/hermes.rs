use crate::context::Context;
use crate::model::{HermesChatTranscriptLine, HermesChatTurn, Verdict};
use crate::sanitizer::sanitize_text;
use crate::util::one_line;
use crate::RavenResult;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const MAX_PROMPT_CHARS: usize = 4_000;
const MAX_RESPONSE_CHARS: usize = 8_000;
const MAX_EVIDENCE_CHARS: usize = 1_200;
const HERMES_TURN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Default)]
pub struct HermesOptions {
    pub cwd: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct HermesTurnMeta {
    command: Vec<String>,
    workspace: String,
    runtime: String,
}

pub fn ask(ctx: &Context, prompt: &str) -> RavenResult<HermesChatTurn> {
    ask_with_options(ctx, prompt, HermesOptions::default())
}

pub fn ask_with_options(
    ctx: &Context,
    prompt: &str,
    options: HermesOptions,
) -> RavenResult<HermesChatTurn> {
    let prompt = prompt.trim();
    let runtime = detect_runtime();
    let command = command_label();
    let cwd = resolve_cwd(ctx, options.cwd.as_deref());
    let workspace = cwd
        .as_ref()
        .map(|cwd| workspace_label(ctx, cwd))
        .unwrap_or_else(|err| sanitize_text(err));
    let meta = HermesTurnMeta {
        command,
        workspace,
        runtime,
    };

    if prompt.is_empty() {
        return Ok(HermesChatTurn {
            prompt: String::new(),
            command: meta.command,
            workspace: meta.workspace,
            runtime: meta.runtime,
            verdict: Verdict::Flag,
            exit_code: 0,
            duration_ms: 0,
            response: "Empty prompt.".to_string(),
            evidence: "no Hermes call was made".to_string(),
            transcript: Vec::new(),
        });
    }

    let binary = env::var("RAVEN_HERMES_BIN").unwrap_or_else(|_| "hermes".to_string());
    let bounded_prompt = clamp_chars(prompt, MAX_PROMPT_CHARS);

    let cwd = match cwd {
        Ok(cwd) => cwd,
        Err(err) => {
            return Ok(flag_turn(
                &bounded_prompt,
                meta,
                1,
                0,
                "Hermes cwd is unavailable for this turn.",
                &format!("invalid Hermes cwd: {err}"),
            ))
        }
    };

    let start = Instant::now();
    let mut command = Command::new(binary);
    command
        .arg("-z")
        .arg(build_raven_prompt(
            &bounded_prompt,
            &meta.workspace,
            &meta.runtime,
        ))
        .current_dir(&cwd)
        .env("RAVEN_WORKSPACE_ROOT", &ctx.root)
        .env("RAVEN_OPERATOR_CWD", &cwd)
        .env("RAVEN_HERMES_RUNTIME", &meta.runtime);

    let output = output_with_timeout(&mut command, HERMES_TURN_TIMEOUT);

    match output {
        Ok(TurnOutput::Completed(output)) => {
            let exit_code = output.status.code().unwrap_or(1);
            Ok(turn_from_output(
                &bounded_prompt,
                exit_code,
                &String::from_utf8_lossy(&output.stdout),
                &String::from_utf8_lossy(&output.stderr),
                start.elapsed().as_millis(),
                meta,
            ))
        }
        Ok(TurnOutput::TimedOut(output)) => Ok(turn_from_output(
            &bounded_prompt,
            124,
            &String::from_utf8_lossy(&output.stdout),
            "Hermes turn timed out after 30s.",
            start.elapsed().as_millis(),
            meta,
        )),
        Err(err) => Ok(flag_turn(
            &bounded_prompt,
            meta,
            127,
            start.elapsed().as_millis(),
            "Hermes is unavailable for this turn.",
            &format!("failed to launch Hermes: {err}"),
        )),
    }
}

enum TurnOutput {
    Completed(Output),
    TimedOut(Output),
}

fn output_with_timeout(command: &mut Command, timeout: Duration) -> std::io::Result<TurnOutput> {
    let start = Instant::now();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(TurnOutput::Completed);
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            return child.wait_with_output().map(TurnOutput::TimedOut);
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn build_raven_prompt(prompt: &str, workspace: &str, runtime: &str) -> String {
    format!(
        "You are Hermes inside Raven's local operator console.\n\
Keep the answer concise and operational.\n\
Do not mutate files, remote issues, deploy targets, or credentials unless the operator explicitly asks.\n\
Keep public-surface safety: do not reveal local absolute paths, tokens, private hosts/IPs, or credential paths.\n\
Runtime context: hermes_openai_runtime={runtime}; raven_workspace={workspace}.\n\
If you use terminal tools, operate from the process cwd or from the RAVEN_OPERATOR_CWD/RAVEN_WORKSPACE_ROOT env vars, but do not print those env values.\n\
If Raven gates are discussed, preserve the current truth: DAS-2669 auth-route repair is accepted through DeepSeek/OpenRouter, while DAS-2666 remains blocked until remote env preflight, guarded NixOS test, full smoke, and supervisor PASS exist.\n\n\
Operator prompt:\n{prompt}"
    )
}

pub(crate) fn turn_from_output(
    prompt: &str,
    exit_code: i32,
    stdout: &str,
    stderr: &str,
    duration_ms: u128,
    meta: HermesTurnMeta,
) -> HermesChatTurn {
    let stdout = stdout.trim();
    let stderr = stderr.trim();
    let raw_response = if stdout.is_empty() && !stderr.is_empty() {
        stderr
    } else {
        stdout
    };
    let response = clamp_chars(&sanitize_text(raw_response), MAX_RESPONSE_CHARS);
    let evidence = if exit_code == 0 {
        format!(
            "Hermes oneshot completed in {duration_ms}ms; runtime={}; cwd={}",
            meta.runtime, meta.workspace
        )
    } else if stderr.is_empty() {
        format!(
            "Hermes exited {exit_code} with no stderr; runtime={}; cwd={}",
            meta.runtime, meta.workspace
        )
    } else {
        format!(
            "Hermes exited {exit_code}: {}; runtime={}; cwd={}",
            one_line(stderr),
            meta.runtime,
            meta.workspace
        )
    };
    let response = if response.is_empty() {
        "(no response text)".to_string()
    } else {
        response
    };
    let prompt = sanitize_text(&clamp_chars(prompt, MAX_PROMPT_CHARS));

    HermesChatTurn {
        transcript: vec![
            HermesChatTranscriptLine {
                role: "operator".to_string(),
                content: prompt.clone(),
            },
            HermesChatTranscriptLine {
                role: "assistant".to_string(),
                content: response.clone(),
            },
        ],
        prompt,
        command: meta.command,
        workspace: meta.workspace,
        runtime: meta.runtime,
        verdict: if exit_code == 0 {
            Verdict::Pass
        } else {
            Verdict::Flag
        },
        exit_code,
        duration_ms,
        response,
        evidence: clamp_chars(&sanitize_text(&evidence), MAX_EVIDENCE_CHARS),
    }
}

fn flag_turn(
    prompt: &str,
    meta: HermesTurnMeta,
    exit_code: i32,
    duration_ms: u128,
    response: &str,
    evidence: &str,
) -> HermesChatTurn {
    let prompt = sanitize_text(&clamp_chars(prompt, MAX_PROMPT_CHARS));
    let response = response.to_string();
    HermesChatTurn {
        transcript: vec![
            HermesChatTranscriptLine {
                role: "operator".to_string(),
                content: prompt.clone(),
            },
            HermesChatTranscriptLine {
                role: "assistant".to_string(),
                content: response.clone(),
            },
        ],
        prompt,
        command: meta.command,
        workspace: meta.workspace,
        runtime: meta.runtime,
        verdict: Verdict::Flag,
        exit_code,
        duration_ms,
        response,
        evidence: clamp_chars(&sanitize_text(evidence), MAX_EVIDENCE_CHARS),
    }
}

fn resolve_cwd(ctx: &Context, requested: Option<&Path>) -> Result<PathBuf, String> {
    let cwd = requested.map_or_else(
        || ctx.root.clone(),
        |path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                ctx.root.join(path)
            }
        },
    );

    if cwd.is_dir() {
        Ok(cwd)
    } else {
        Err(cwd.to_string_lossy().to_string())
    }
}

fn workspace_label(ctx: &Context, cwd: &Path) -> String {
    if cwd == ctx.root {
        return "case-root".to_string();
    }

    if let Ok(relative) = cwd.strip_prefix(&ctx.root) {
        let relative = relative.to_string_lossy().replace('\\', "/");
        return format!("case-root/{relative}");
    }

    sanitize_text(&cwd.to_string_lossy())
}

fn command_label() -> Vec<String> {
    let binary = env::var("RAVEN_HERMES_BIN").unwrap_or_else(|_| "hermes".to_string());
    let label = Path::new(&binary)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&binary);
    vec![
        sanitize_text(label),
        "-z".to_string(),
        "[raven-prompt]".to_string(),
    ]
}

fn detect_runtime() -> String {
    if let Ok(runtime) = env::var("RAVEN_HERMES_RUNTIME") {
        return sanitize_text(runtime.trim());
    }

    let Some(home) = env::var_os("HOME") else {
        return "unknown".to_string();
    };
    let config = PathBuf::from(home).join(".hermes/config.yaml");
    let Ok(text) = fs::read_to_string(config) else {
        return "unknown".to_string();
    };

    text.lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("openai_runtime:")
                .map(|value| value.trim().trim_matches('"').trim_matches('\''))
        })
        .filter(|value| !value.is_empty())
        .map(sanitize_text)
        .unwrap_or_else(|| "unknown".to_string())
}

fn clamp_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let mut output = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        output.push_str(" ...[truncated]");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{turn_from_output, HermesTurnMeta};
    use crate::model::Verdict;

    fn meta() -> HermesTurnMeta {
        HermesTurnMeta {
            command: vec!["hermes".to_string(), "-z".to_string()],
            workspace: "case-root".to_string(),
            runtime: "codex_app_server".to_string(),
        }
    }

    #[test]
    fn successful_turn_sanitizes_output() {
        let turn = turn_from_output(
            "inspect status",
            0,
            "ready from /Users/alice/work and token sk-proj-abcdefghijklmnopqrstuvwxyz123456",
            "",
            42,
            meta(),
        );

        assert_eq!(turn.verdict, Verdict::Pass);
        assert_eq!(turn.workspace, "case-root");
        assert_eq!(turn.runtime, "codex_app_server");
        assert!(!turn.response.contains("/Users/alice"));
        assert!(!turn.response.contains("sk-proj-"));
        assert!(turn.response.contains("[redacted-path]"));
        assert!(turn.response.contains("[redacted-token]"));
        assert_eq!(turn.transcript.len(), 2);
    }

    #[test]
    fn failed_turn_is_flag_and_sanitizes_stderr() {
        let turn = turn_from_output(
            "ask",
            2,
            "",
            "failed on 127.0.0.1:8080 with token=secret-value",
            7,
            meta(),
        );

        assert_eq!(turn.verdict, Verdict::Flag);
        assert_eq!(turn.exit_code, 2);
        assert!(!turn.evidence.contains("127.0.0.1"));
        assert!(!turn.evidence.contains("secret-value"));
        assert!(turn.evidence.contains("[redacted-ip]"));
        assert!(turn.evidence.contains("token=[redacted-secret]"));
    }
}
