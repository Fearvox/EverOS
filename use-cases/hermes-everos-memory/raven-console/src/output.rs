use crate::model::{
    DoctorReport, HermesChatTurn, MemorySearchResult, NativeAuditReport, RavenReceipt,
    RavenSnapshot, ResearchLane, ResearchPacket, ResearchSynthesis, ScProviderView, ScReport,
    ScSessionView, ScStatusView, ScWorktreeView, Verdict,
};
use crate::sanitizer::{sanitize_json, sanitize_text};
use crate::util::one_line;
use crate::RavenResult;
use std::io::{self, Write};

pub fn json<T: serde::Serialize>(value: &T) -> RavenResult<()> {
    let safe = sanitize_json(value)?;
    println!("{}", serde_json::to_string_pretty(&safe)?);
    Ok(())
}

pub fn status(snapshot: &RavenSnapshot) {
    line("RAVEN_STATUS");
    line(&format!("VERDICT: {}", snapshot.verdict));
    line(&format!(
        "LOCAL_PACKET: {} ({})",
        snapshot.packet.verdict, snapshot.packet.title
    ));
    let remote = hard_remote_verdict(snapshot);
    line(&format!(
        "REMOTE_EVERCORE: {remote} (DAS-2666 and DAS-2669 are hard gates)"
    ));
    line(&format!(
        "MEMORY: {} ({})",
        snapshot.memory.verdict, snapshot.memory.status
    ));
    line("");
    line("WATCHLIST:");
    for issue in &snapshot.watchlist_issues {
        line(&format!(
            "- {}: {} [{}] source={} comments={}",
            issue.id,
            issue.title,
            issue.status,
            issue.source,
            if issue.comments_checked {
                "checked"
            } else {
                "not_checked"
            }
        ));
    }
    line("");
    line("NEXT:");
    for action in &snapshot.next_actions {
        line(&format!("- {action}"));
    }
}

pub fn packet(snapshot: &RavenSnapshot) {
    line("RAVEN_PACKET");
    line(&format!("VERDICT: {}", snapshot.packet.verdict));
    line(&format!("ID: {}", snapshot.packet.id));
    line(&format!("TITLE: {}", snapshot.packet.title));
    line(&format!("STATUS: {}", snapshot.packet.status));
    line(&format!("OWNERS: {}", snapshot.packet.owners.join(", ")));
    line(&format!(
        "MEMORY_PROVIDERS: {}",
        snapshot.packet.memory_providers.join(", ")
    ));
    line("");
    line("SOURCE_SUMMARIES:");
    for doc in &snapshot.packet.docs {
        line(&format!("- {}: {} {}", doc.path, doc.verdict, doc.evidence));
    }
}

pub fn packet_export_markdown(snapshot: &RavenSnapshot) -> String {
    let mut output = Vec::new();
    output.push(format!("# {}", snapshot.packet.title));
    output.push(String::new());
    output.push(format!("VERDICT: {}", snapshot.verdict));
    output.push(format!("Packet: {}", snapshot.packet.id));
    output.push(format!("Status: {}", snapshot.packet.status));
    output.push(String::new());
    output.push("## Gates".to_string());
    output.push(String::new());
    for gate in &snapshot.remote_gates {
        output.push(format!(
            "- {} / {}: {} - {}",
            gate.id, gate.name, gate.verdict, gate.evidence
        ));
    }
    for gate in &snapshot.local_gates {
        output.push(format!(
            "- {} / {}: {} - {}",
            gate.id, gate.name, gate.verdict, gate.evidence
        ));
    }
    output.push(String::new());
    output.push("## Next".to_string());
    output.push(String::new());
    for action in &snapshot.next_actions {
        output.push(format!("- {action}"));
    }
    sanitize_text(&output.join("\n"))
}

pub fn gates(snapshot: &RavenSnapshot) {
    line("RAVEN_GATES");
    line(&format!("VERDICT: {}", hard_remote_verdict(snapshot)));
    line("");
    line("REMOTE_HARD_GATES:");
    for gate in &snapshot.remote_gates {
        line(&format!(
            "- {} / {}: {} blocks={} hard={} evidence={} effect={}",
            gate.id,
            gate.name,
            gate.verdict,
            gate.blocks_completion,
            gate.hard_gate,
            gate.evidence,
            gate.gate_effect
        ));
    }
    line("");
    line("LOCAL_PACKET_GATES:");
    for gate in &snapshot.local_gates {
        line(&format!(
            "- {} / {}: {} blocks={} command={} evidence={}",
            gate.id, gate.name, gate.verdict, gate.blocks_completion, gate.command, gate.evidence
        ));
    }
    line("");
    line("STOP_CONDITIONS:");
    if snapshot
        .remote_gates
        .iter()
        .any(|gate| gate.id == "DAS-2669" && gate.verdict == Verdict::Block)
    {
        line("- no AUTH_REPAIRED on DAS-2669");
    }
    line("- missing guarded NixOS test");
    line("- missing remote loopback full smoke");
    line("- missing supervisor PASS");
    line("- public bind/firewall exposure or unredacted private evidence");
}

pub fn research_lanes(lanes: &[ResearchLane]) {
    line("RAVEN_V2_RESEARCH_LANES");
    line("VERDICT: FLAG");
    line("EVIDENCE: lanes are bounded by RAVEN_V2_RESEARCH_LEDGER.md; packets must be live-gate calibrated.");
    for lane in lanes {
        line(&format!(
            "- {} / {}: {}",
            lane.id, lane.title, lane.question
        ));
    }
    line("NEXT: raven research packet <lane>");
}

pub fn research_packet(packet: &ResearchPacket) {
    line("RAVEN_V2_RESEARCH_PACKET");
    line(&format!("LANE: {} / {}", packet.lane_id, packet.lane_title));
    line(&format!("VERDICT: {}", packet.verdict));
    line(&format!("QUESTION: {}", packet.question));
    line("FINDINGS:");
    for finding in &packet.findings {
        line(&format!("- {finding}"));
    }
    line("DECISIONS:");
    for decision in &packet.decisions {
        line(&format!("- {decision}"));
    }
    line("LIVE_GATES:");
    for gate in &packet.live_gates {
        line(&format!(
            "- {}: {} {}",
            gate.id, gate.verdict, gate.evidence
        ));
    }
    line("NEXT:");
    for action in &packet.next {
        line(&format!("- {action}"));
    }
}

pub fn research_synthesis(synthesis: &ResearchSynthesis) {
    line("RAVEN_V2_SYNTHESIS_READINESS");
    line(&format!("VERDICT: {}", synthesis.verdict));
    line(&format!(
        "PACKETS: {}/{}",
        synthesis.packets_ready, synthesis.required_packets
    ));
    line(&format!("EVIDENCE: {}", synthesis.evidence));
    line("NEXT:");
    for action in &synthesis.next {
        line(&format!("- {action}"));
    }
}

pub fn agents(snapshot: &RavenSnapshot) {
    line("RAVEN_AGENTS");
    line("VERDICT: FLAG");
    for agent in &snapshot.agents {
        line(&format!(
            "- {}: {} {} ({}) scope={}",
            agent.name, agent.verdict, agent.status, agent.issue_id, agent.scope
        ));
    }
}

pub fn runs(snapshot: &RavenSnapshot) {
    line("RAVEN_RUNS");
    line(&format!(
        "VERDICT: {}",
        if snapshot
            .runs
            .iter()
            .any(|run| run.verdict == Verdict::Block)
        {
            Verdict::Block
        } else if snapshot.runs.iter().any(|run| run.verdict == Verdict::Flag) {
            Verdict::Flag
        } else {
            Verdict::Pass
        }
    ));
    for run in &snapshot.runs {
        let receipt = run
            .receipt_path
            .as_ref()
            .map(|path| format!(" receipt={path}"))
            .unwrap_or_default();
        line(&format!(
            "- {}: {} source={} command={}{} evidence={}",
            run.id, run.verdict, run.source, run.command, receipt, run.evidence
        ));
    }
}

pub fn sc_report(report: &ScReport) {
    line("RAVEN_SC");
    line(&format!("VERDICT: {}", report.verdict));
    line(&format!(
        "STATUS: {} ok={} api={} app={} evidence={}",
        report.status.verdict,
        report.status.ok,
        report
            .status
            .api_version
            .map(|version| version.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        report.status.app_version,
        report.status.evidence
    ));
    sc_worktree(&report.worktree);
    sc_sessions(&report.sessions);
    sc_providers(&report.providers);
}

pub fn sc_status(status: &ScStatusView) {
    line("RAVEN_SC_STATUS");
    line(&format!("VERDICT: {}", status.verdict));
    line(&format!("OK: {}", status.ok));
    line(&format!(
        "API_VERSION: {}",
        status
            .api_version
            .map(|version| version.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    ));
    line(&format!("APP_VERSION: {}", status.app_version));
    line(&format!("EVIDENCE: {}", status.evidence));
}

pub fn sc_sessions(sessions: &[ScSessionView]) {
    line("RAVEN_SC_SESSIONS");
    line(&format!("COUNT: {}", sessions.len()));
    for session in sessions.iter().take(12) {
        line(&format!(
            "- {} provider={} model={} reasoning={} active={} closed={} branch={} worktree={} title={}",
            session.thread_id,
            session.provider_key,
            session.model,
            session.reasoning_effort,
            session.active_turn,
            session.closed,
            session.branch,
            session.worktree,
            session.title
        ));
    }
}

pub fn sc_providers(providers: &[ScProviderView]) {
    line("RAVEN_SC_PROVIDERS");
    line(&format!("COUNT: {}", providers.len()));
    for provider in providers.iter().take(16) {
        line(&format!(
            "- {} enabled={} models={} reasoning={} display={}",
            provider.provider_key,
            provider.enabled,
            provider.model_count,
            provider.reasoning_efforts.join("/"),
            provider.display_name
        ));
    }
}

pub fn sc_worktree(worktree: &ScWorktreeView) {
    line("RAVEN_SC_WORKTREE");
    line(&format!("VERDICT: {}", worktree.verdict));
    line(&format!("BRANCH: {}", worktree.branch));
    line(&format!("TARGET_BRANCH: {}", worktree.target_branch));
    line(&format!(
        "DIRTY: {}",
        worktree
            .dirty
            .map(|dirty| dirty.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    ));
    line(&format!("EVIDENCE: {}", worktree.evidence));
}

pub fn memory_health(snapshot: &RavenSnapshot) {
    line("RAVEN_MEMORY_HEALTH");
    line(&format!("VERDICT: {}", snapshot.memory.verdict));
    line(&format!("STATUS: {}", snapshot.memory.status));
    line(&format!("EVIDENCE: {}", snapshot.memory.evidence));
}

pub fn memory_search(result: &MemorySearchResult) {
    line("RAVEN_MEMORY_SEARCH");
    line(&format!("VERDICT: {}", result.verdict));
    line(&format!("QUERY: {}", result.query));
    line(&format!("EVIDENCE: {}", result.evidence));
}

pub fn chat_turn(turn: &HermesChatTurn) {
    line("RAVEN_CHAT");
    line(&format!("VERDICT: {}", turn.verdict));
    line(&format!("EXIT_CODE: {}", turn.exit_code));
    line(&format!("DURATION_MS: {}", turn.duration_ms));
    line(&format!("RUNTIME: {}", turn.runtime));
    line(&format!("WORKSPACE: {}", turn.workspace));
    line(&format!("EVIDENCE: {}", turn.evidence));
    line("ASSISTANT:");
    for raw in turn.response.lines().take(80) {
        println!("{}", sanitize_text(raw));
    }
}

pub fn verify_human(receipt: &RavenReceipt) {
    line("RAVEN_RUN_VERIFY");
    line(&format!("VERDICT: {}", receipt.verdict));
    line(&format!("EXIT_CODE: {}", receipt.exit_code));
    line(&format!("DURATION_MS: {}", receipt.duration_ms));
    line(&format!("EVIDENCE: {}", receipt.evidence_excerpt));
    line("GATE_EFFECTS:");
    for effect in &receipt.gate_effects {
        line(&format!(
            "- {}: {} -> {} ({})",
            effect.gate_id, effect.before, effect.after, effect.note
        ));
    }
    line(&format!(
        "PUBLIC_SAFETY: {} {}",
        receipt.public_safety.verdict, receipt.public_safety.evidence
    ));
}

pub fn doctor(report: &DoctorReport) {
    line("RAVEN_DOCTOR");
    line(&format!("VERDICT: {}", report.verdict));
    for check in &report.checks {
        line(&format!(
            "- {}: {} {}",
            check.name, check.verdict, check.evidence
        ));
    }
    line(&format!("NEXT: {}", report.next));
}

pub fn native_audit(report: &NativeAuditReport) {
    line("RAVEN_NATIVE_AUDIT");
    line(&format!("VERDICT: {}", report.verdict));
    for item in &report.items {
        line(&format!(
            "- {}: {} hard_failure={} evidence={}",
            item.category, item.verdict, item.hard_failure, item.evidence
        ));
    }
    line(&format!(
        "BLOCKS_PASS_ON: {}",
        report.blocks_pass_on.join(", ")
    ));
}

pub fn write_text(target: Option<&str>, text: &str) -> RavenResult<()> {
    match target {
        Some("-") | None => {
            println!("{}", sanitize_text(text).trim_end());
            Ok(())
        }
        Some(path) => {
            std::fs::write(path, format!("{}\n", sanitize_text(text).trim_end()))?;
            line(&format!("WROTE: {path}"));
            Ok(())
        }
    }
}

pub fn flush_stdout() -> RavenResult<()> {
    io::stdout().flush()?;
    Ok(())
}

pub fn line(text: &str) {
    println!("{}", sanitize_text(&one_line_preserving_blank(text)));
}

fn hard_remote_verdict(snapshot: &RavenSnapshot) -> Verdict {
    if snapshot
        .remote_gates
        .iter()
        .any(|gate| gate.hard_gate && gate.verdict == Verdict::Block)
    {
        Verdict::Block
    } else if snapshot
        .remote_gates
        .iter()
        .any(|gate| gate.verdict == Verdict::Flag)
    {
        Verdict::Flag
    } else {
        Verdict::Pass
    }
}

fn one_line_preserving_blank(text: &str) -> String {
    if text.is_empty() {
        String::new()
    } else {
        one_line(text)
    }
}
