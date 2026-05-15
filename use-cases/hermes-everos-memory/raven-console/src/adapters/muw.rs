use crate::constants::{
    ISSUE_ADAPTER_REPAIR, ISSUE_AUTH_BLOCKER, ISSUE_CONTROL_ROOM, ISSUE_LOCAL_VERIFIER,
    ISSUE_MEMORY_WATCH, ISSUE_REMOTE_DEPLOY, WATCHLIST_ISSUES,
};
use crate::model::{AgentView, IssueView, RemoteGate, Verdict};
use crate::sanitizer::sanitize_text;
use crate::util::{one_line, truncate};
use serde_json::Value;
use std::process::Command;

pub fn load_watchlist() -> Vec<IssueView> {
    if Command::new("multica").arg("--version").output().is_err() {
        return WATCHLIST_ISSUES
            .iter()
            .map(|id| fallback_issue(id, "multica CLI unavailable"))
            .collect();
    }

    WATCHLIST_ISSUES.iter().map(|id| load_issue(id)).collect()
}

pub fn remote_gates(issues: &[IssueView]) -> Vec<RemoteGate> {
    let auth_issue = issue(issues, ISSUE_AUTH_BLOCKER);
    let deploy_issue = issue(issues, ISSUE_REMOTE_DEPLOY);
    let adapter_issue = issue(issues, ISSUE_ADAPTER_REPAIR);

    let auth_repaired = auth_issue.map(has_auth_repaired).unwrap_or(false);
    let guarded_nixos = deploy_issue
        .map(|issue| contains_any(issue, &["guarded NixOS test", "nixos-rebuild test"]))
        .unwrap_or(false);
    let remote_full_smoke = deploy_issue
        .map(|issue| {
            contains_any(
                issue,
                &[
                    "remote loopback full smoke",
                    "remote-smoke full",
                    "--mode full",
                ],
            )
        })
        .unwrap_or(false);
    let supervisor_pass = deploy_issue
        .map(|issue| contains_any(issue, &["supervisor PASS", "VERDICT: PASS"]))
        .unwrap_or(false);

    let mut missing = Vec::new();
    if !auth_repaired {
        missing.push("AUTH_REPAIRED on DAS-2669");
    }
    if !guarded_nixos {
        missing.push("guarded NixOS test");
    }
    if !remote_full_smoke {
        missing.push("remote loopback full smoke");
    }
    if !supervisor_pass {
        missing.push("supervisor PASS");
    }

    let adapter_verdict = adapter_issue
        .map(|issue| Verdict::from_packet_word(&issue.status))
        .unwrap_or(Verdict::Flag);

    vec![
        RemoteGate {
            id: ISSUE_AUTH_BLOCKER.to_string(),
            name: "DeepSeek/OpenRouter auth-route repair".to_string(),
            verdict: if auth_repaired {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            blocks_completion: true,
            hard_gate: true,
            evidence: if auth_repaired {
                "AUTH_REPAIRED present in live issue/comment evidence.".to_string()
            } else {
                "AUTH_REPAIRED not present in live issue/comment evidence.".to_string()
            },
            gate_effect: if auth_repaired {
                "Auth block cleared; DAS-2666 still waits on deploy evidence.".to_string()
            } else {
                "Remote deploy lane remains blocked until this passes.".to_string()
            },
        },
        RemoteGate {
            id: ISSUE_REMOTE_DEPLOY.to_string(),
            name: "EverCore remote deploy".to_string(),
            verdict: if missing.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            blocks_completion: true,
            hard_gate: true,
            evidence: if missing.is_empty() {
                "Auth repair, guarded NixOS test, remote loopback full smoke, and supervisor PASS are present.".to_string()
            } else {
                format!("Missing: {}.", missing.join(", "))
            },
            gate_effect: "Overall Raven status may only be FLAG while this remote gate is red."
                .to_string(),
        },
        RemoteGate {
            id: ISSUE_ADAPTER_REPAIR.to_string(),
            name: "Pi/OpenCode adapter repair".to_string(),
            verdict: adapter_verdict,
            blocks_completion: false,
            hard_gate: false,
            evidence: "Adapter repair can unlock Pi/OpenCode lanes but cannot green remote deploy."
                .to_string(),
            gate_effect: "No effect on DAS-2666 remote deploy verdict.".to_string(),
        },
    ]
}

pub fn agent_views(issues: &[IssueView]) -> Vec<AgentView> {
    [
        (
            "Workbench control room",
            ISSUE_CONTROL_ROOM,
            "Track lane truth and owner packet.",
        ),
        (
            "Local verifier",
            ISSUE_LOCAL_VERIFIER,
            "Re-run local Raven and public-safety gates.",
        ),
        (
            "Memory watch",
            ISSUE_MEMORY_WATCH,
            "Keep memory bridge and evidence state visible.",
        ),
        (
            "Auth route repair",
            ISSUE_AUTH_BLOCKER,
            "DeepSeek/OpenRouter auth-route repair; parent deploy proof remains separate.",
        ),
        (
            "EverCore remote deploy",
            ISSUE_REMOTE_DEPLOY,
            "Guarded NixOS test and loopback full smoke only after auth repair.",
        ),
        (
            "Adapter repair",
            ISSUE_ADAPTER_REPAIR,
            "Repair Pi/OpenCode wrapper lanes without changing remote deploy verdict.",
        ),
    ]
    .into_iter()
    .map(|(name, id, scope)| {
        let issue = issue(issues, id);
        AgentView {
            name: name.to_string(),
            issue_id: id.to_string(),
            status: issue
                .map(|issue| issue.status.clone())
                .unwrap_or_else(|| "unavailable".to_string()),
            verdict: issue
                .map(|issue| Verdict::from_packet_word(&issue.status))
                .unwrap_or(Verdict::Flag),
            scope: scope.to_string(),
        }
    })
    .collect()
}

fn load_issue(id: &str) -> IssueView {
    let output = Command::new("multica")
        .arg("issue")
        .arg("get")
        .arg(id)
        .arg("--output")
        .arg("json")
        .output();

    let mut issue = match output {
        Ok(output) if output.status.success() => {
            match serde_json::from_slice::<Value>(&output.stdout) {
                Ok(value) => issue_from_value(id, &value),
                Err(err) => fallback_issue(id, &format!("multica JSON parse failed: {err}")),
            }
        }
        Ok(output) => fallback_issue(id, &format!("multica issue get exited {}", output.status)),
        Err(err) => fallback_issue(id, &err.to_string()),
    };

    if issue.available {
        match load_comments(id) {
            Some(comments) => {
                issue.comments_checked = true;
                let auth_repair_prefix =
                    if id == ISSUE_AUTH_BLOCKER && has_auth_repaired_text(&comments) {
                        "AUTH_REPAIRED VERDICT: PASS "
                    } else {
                        ""
                    };
                issue.evidence_excerpt = sanitize_text(&truncate(
                    &one_line(&format!(
                        "{auth_repair_prefix}{} {}",
                        issue.evidence_excerpt, comments
                    )),
                    900,
                ));
            }
            None => {
                issue.comments_checked = false;
            }
        }
    }

    issue
}

fn load_comments(id: &str) -> Option<String> {
    let output = Command::new("multica")
        .arg("issue")
        .arg("comment")
        .arg("list")
        .arg(id)
        .arg("--output")
        .arg("json")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = serde_json::from_slice::<Value>(&output.stdout).ok()?;
    let mut parts = Vec::new();
    collect_comment_text(&value, &mut parts);
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn collect_comment_text(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_comment_text(item, out);
            }
        }
        Value::Object(map) => {
            for key in ["body", "content", "text", "markdown", "message"] {
                if let Some(text) = map.get(key).and_then(Value::as_str) {
                    out.push(text.to_string());
                }
            }
            for key in ["comments", "items", "data", "nodes"] {
                if let Some(child) = map.get(key) {
                    collect_comment_text(child, out);
                }
            }
        }
        _ => {}
    }
}

fn issue_from_value(id: &str, value: &Value) -> IssueView {
    let identifier = string_field(value, &["identifier", "id", "key"]).unwrap_or(id);
    let title =
        string_field(value, &["title", "name", "summary"]).unwrap_or_else(|| fallback_title(id));
    let status = string_field(
        value,
        &["status", "state", "workflow_state", "workflowStatus"],
    )
    .unwrap_or("unknown");
    let priority = string_field(value, &["priority"]).unwrap_or("unknown");
    let updated_at =
        string_field(value, &["updated_at", "updatedAt", "updated"]).unwrap_or("unknown");
    let description = string_field(value, &["description", "body", "content"]).unwrap_or("");

    IssueView {
        id: identifier.to_string(),
        title: sanitize_text(title),
        status: sanitize_text(status),
        priority: sanitize_text(priority),
        updated_at: sanitize_text(updated_at),
        available: true,
        source: "live".to_string(),
        comments_checked: false,
        evidence_excerpt: sanitize_text(&truncate(
            &one_line(&format!("{title} {status} {description}")),
            900,
        )),
    }
}

fn fallback_issue(id: &str, reason: &str) -> IssueView {
    IssueView {
        id: id.to_string(),
        title: fallback_title(id).to_string(),
        status: if id == ISSUE_REMOTE_DEPLOY || id == ISSUE_AUTH_BLOCKER {
            "blocked".to_string()
        } else {
            "unavailable".to_string()
        },
        priority: "unknown".to_string(),
        updated_at: "unknown".to_string(),
        available: false,
        source: "fallback".to_string(),
        comments_checked: false,
        evidence_excerpt: sanitize_text(reason),
    }
}

fn fallback_title(id: &str) -> &'static str {
    match id {
        ISSUE_REMOTE_DEPLOY => "EverCore remote deploy gate",
        ISSUE_AUTH_BLOCKER => "Repair Windburn NixOS Codex runtime auth",
        ISSUE_CONTROL_ROOM => "Raven control-room watch",
        ISSUE_LOCAL_VERIFIER => "Raven local verifier watch",
        ISSUE_MEMORY_WATCH => "Raven memory evidence watch",
        ISSUE_ADAPTER_REPAIR => "Pi/OpenCode adapter repair",
        _ => "Unknown watch issue",
    }
}

fn string_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    let object = value.as_object()?;
    for key in keys {
        if let Some(text) = object.get(*key).and_then(Value::as_str) {
            return Some(text);
        }
        if let Some(inner) = object.get(*key).and_then(Value::as_object) {
            if let Some(text) = inner.get("name").and_then(Value::as_str) {
                return Some(text);
            }
            if let Some(text) = inner.get("title").and_then(Value::as_str) {
                return Some(text);
            }
        }
    }
    None
}

fn issue<'a>(issues: &'a [IssueView], id: &str) -> Option<&'a IssueView> {
    issues.iter().find(|issue| issue.id == id)
}

fn has_auth_repaired(issue: &IssueView) -> bool {
    has_auth_repaired_text(&issue.evidence_excerpt)
}

fn has_auth_repaired_text(text: &str) -> bool {
    let evidence = text.to_ascii_uppercase();
    evidence.contains("AUTH_REPAIRED")
        && (evidence.contains("VERDICT: PASS")
            || evidence.contains("AUTH_REPAIRED: PASS")
            || evidence.contains("AUTH_REPAIR_PROOF: PASS"))
}

fn contains_any(issue: &IssueView, needles: &[&str]) -> bool {
    let haystack = issue.evidence_excerpt.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| haystack.contains(&needle.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::remote_gates;
    use crate::constants::{ISSUE_ADAPTER_REPAIR, ISSUE_AUTH_BLOCKER, ISSUE_REMOTE_DEPLOY};
    use crate::model::{IssueView, Verdict};

    #[test]
    fn missing_auth_repaired_keeps_remote_gates_blocked() {
        let gates = remote_gates(&[
            issue(
                ISSUE_AUTH_BLOCKER,
                "blocked",
                "read-only proof still failing",
            ),
            issue(
                ISSUE_REMOTE_DEPLOY,
                "blocked",
                "guarded NixOS test remote loopback full smoke supervisor PASS",
            ),
        ]);

        assert_eq!(gate(&gates, ISSUE_AUTH_BLOCKER), Verdict::Block);
        assert_eq!(gate(&gates, ISSUE_REMOTE_DEPLOY), Verdict::Block);
    }

    #[test]
    fn deploy_needs_every_hard_evidence_marker() {
        let gates = remote_gates(&[
            issue(ISSUE_AUTH_BLOCKER, "closed", "AUTH_REPAIRED VERDICT: PASS"),
            issue(ISSUE_REMOTE_DEPLOY, "blocked", "guarded NixOS test only"),
        ]);

        assert_eq!(gate(&gates, ISSUE_REMOTE_DEPLOY), Verdict::Block);
    }

    #[test]
    fn future_auth_repair_mentions_do_not_pass_auth_gate() {
        let gates = remote_gates(&[
            issue(
                ISSUE_AUTH_BLOCKER,
                "blocked",
                "VERDICT: FLAG post AUTH_REPAIRED only after proof succeeds",
            ),
            issue(
                ISSUE_REMOTE_DEPLOY,
                "blocked",
                "guarded NixOS test remote loopback full smoke supervisor PASS",
            ),
        ]);

        assert_eq!(gate(&gates, ISSUE_AUTH_BLOCKER), Verdict::Block);
        assert_eq!(gate(&gates, ISSUE_REMOTE_DEPLOY), Verdict::Block);
    }

    #[test]
    fn auth_repair_detector_handles_marker_after_stale_prefix() {
        let stale_prefix = "VERDICT: BLOCK old refresh token failure ".repeat(80);
        let evidence = format!("{stale_prefix} AUTH_REPAIRED VERDICT: PASS");

        assert!(super::has_auth_repaired_text(&evidence));
    }

    #[test]
    fn adapter_repair_pass_does_not_green_remote_deploy() {
        let gates = remote_gates(&[
            issue(ISSUE_AUTH_BLOCKER, "blocked", "runtime auth still broken"),
            issue(ISSUE_REMOTE_DEPLOY, "blocked", "waiting on auth"),
            issue(ISSUE_ADAPTER_REPAIR, "closed", "adapter PASS"),
        ]);

        assert_eq!(gate(&gates, ISSUE_ADAPTER_REPAIR), Verdict::Pass);
        assert_eq!(gate(&gates, ISSUE_REMOTE_DEPLOY), Verdict::Block);
    }

    fn issue(id: &str, status: &str, evidence: &str) -> IssueView {
        IssueView {
            id: id.to_string(),
            title: id.to_string(),
            status: status.to_string(),
            priority: "unknown".to_string(),
            updated_at: "unknown".to_string(),
            available: true,
            source: "test".to_string(),
            comments_checked: true,
            evidence_excerpt: evidence.to_string(),
        }
    }

    fn gate(gates: &[crate::model::RemoteGate], id: &str) -> Verdict {
        gates
            .iter()
            .find(|gate| gate.id == id)
            .map(|gate| gate.verdict)
            .unwrap()
    }
}
