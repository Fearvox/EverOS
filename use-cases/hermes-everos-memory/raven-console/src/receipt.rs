use crate::adapters::verify::VerifyResult;
use crate::constants::{ISSUE_ADAPTER_REPAIR, ISSUE_REMOTE_DEPLOY, RUNS_DIR};
use crate::context::Context;
use crate::model::{GateEffect, HermesChatTurn, PublicSafetyResult, RavenReceipt, Verdict};
use crate::sanitizer::{public_safety_verdict, sanitize_text};
use crate::util::{one_line, run_id, truncate};
use crate::RavenResult;
use std::fs;
use std::path::PathBuf;

pub fn from_verify(result: &VerifyResult) -> RavenReceipt {
    let evidence = sanitize_text(&truncate(
        &one_line(&format!("{} {}", result.stdout, result.stderr)),
        900,
    ));
    let safety = if public_safety_verdict(&evidence) {
        PublicSafetyResult {
            verdict: Verdict::Pass,
            evidence: "receipt evidence excerpt is sanitized.".to_string(),
        }
    } else {
        PublicSafetyResult {
            verdict: Verdict::Block,
            evidence: "receipt evidence still contains sensitive-looking material.".to_string(),
        }
    };

    RavenReceipt {
        id: run_id("raven-verify"),
        command: result.command.clone(),
        exit_code: result.exit_code,
        duration_ms: result.duration_ms,
        verdict: result.verdict,
        evidence_excerpt: evidence,
        gate_effects: vec![
            GateEffect {
                gate_id: "local-packet".to_string(),
                before: "configured".to_string(),
                after: result.verdict.to_string(),
                note: "Local Raven packet verifier executed through bin/raven-run.mjs.".to_string(),
            },
            GateEffect {
                gate_id: ISSUE_REMOTE_DEPLOY.to_string(),
                before: "BLOCK unless live remote evidence proves every hard gate".to_string(),
                after: "unchanged".to_string(),
                note: "run verify is local-only and cannot green remote deploy.".to_string(),
            },
            GateEffect {
                gate_id: ISSUE_ADAPTER_REPAIR.to_string(),
                before: "watch".to_string(),
                after: "unchanged".to_string(),
                note: "adapter repair evidence has no effect on DAS-2666.".to_string(),
            },
        ],
        public_safety: safety,
    }
}

pub fn from_chat(turn: &HermesChatTurn) -> RavenReceipt {
    let evidence = sanitize_text(&truncate(
        &one_line(&format!(
            "runtime={} cwd={} evidence={} response={}",
            turn.runtime, turn.workspace, turn.evidence, turn.response
        )),
        900,
    ));
    let safety = if public_safety_verdict(&evidence)
        && public_safety_verdict(&turn.prompt)
        && public_safety_verdict(&turn.response)
    {
        PublicSafetyResult {
            verdict: Verdict::Pass,
            evidence: "chat prompt, response, and evidence are sanitized.".to_string(),
        }
    } else {
        PublicSafetyResult {
            verdict: Verdict::Block,
            evidence: "chat transcript still contains sensitive-looking material.".to_string(),
        }
    };

    RavenReceipt {
        id: run_id("raven-chat"),
        command: turn.command.clone(),
        exit_code: turn.exit_code,
        duration_ms: turn.duration_ms,
        verdict: turn.verdict,
        evidence_excerpt: evidence,
        gate_effects: vec![
            GateEffect {
                gate_id: "hermes-chat".to_string(),
                before: "requested".to_string(),
                after: turn.verdict.to_string(),
                note: "Hermes dialogue executed through the shared Raven adapter.".to_string(),
            },
            GateEffect {
                gate_id: ISSUE_REMOTE_DEPLOY.to_string(),
                before: "BLOCK unless live remote evidence proves every hard gate".to_string(),
                after: "unchanged".to_string(),
                note: "chat receipts cannot green remote deploy.".to_string(),
            },
        ],
        public_safety: safety,
    }
}

pub fn save_receipt(
    ctx: &Context,
    receipt: &RavenReceipt,
    target: Option<&str>,
) -> RavenResult<PathBuf> {
    let path = match target {
        Some(path) => PathBuf::from(path),
        None => ctx.root.join(RUNS_DIR).join(format!("{}.json", receipt.id)),
    };

    let absolute = if path.is_absolute() {
        path
    } else {
        ctx.root.join(path)
    };

    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(receipt)?;
    fs::write(&absolute, format!("{text}\n"))?;
    Ok(absolute)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{HermesChatTranscriptLine, HermesChatTurn};

    #[test]
    fn chat_receipt_preserves_remote_deploy_boundary() {
        let turn = HermesChatTurn {
            prompt: "status".to_string(),
            command: vec![
                "hermes".to_string(),
                "-z".to_string(),
                "[raven-prompt]".to_string(),
            ],
            workspace: "case-root".to_string(),
            runtime: "codex_app_server".to_string(),
            verdict: Verdict::Pass,
            exit_code: 0,
            duration_ms: 11,
            response: "ready".to_string(),
            evidence: "Hermes oneshot completed".to_string(),
            transcript: vec![HermesChatTranscriptLine {
                role: "assistant".to_string(),
                content: "ready".to_string(),
            }],
        };

        let receipt = from_chat(&turn);

        assert_eq!(receipt.verdict, Verdict::Pass);
        assert!(receipt
            .gate_effects
            .iter()
            .any(|effect| effect.gate_id == ISSUE_REMOTE_DEPLOY && effect.after == "unchanged"));
        assert_eq!(receipt.public_safety.verdict, Verdict::Pass);
    }
}
