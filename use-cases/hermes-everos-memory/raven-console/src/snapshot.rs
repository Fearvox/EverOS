use crate::adapters::{muw, packet, sc, verify};
use crate::constants::{
    ISSUE_ADAPTER_REPAIR, ISSUE_AUTH_BLOCKER, ISSUE_CONTROL_ROOM, ISSUE_LOCAL_VERIFIER,
    ISSUE_MEMORY_WATCH, ISSUE_REMOTE_DEPLOY, WATCHLIST_ISSUES,
};
use crate::context::Context;
use crate::model::{
    AgentView, AgenticLoopPhase, AgenticLoopState, AgenticLoopStep, IssueView, LocalGateView,
    MemoryHealth, PacketSummary, PublicSafetyResult, RavenSnapshot, RemoteGate, RunView, ScReport,
    Verdict,
};

struct SnapshotParts {
    packet_verdict: Verdict,
    watchlist_issues: Vec<IssueView>,
    local_gates: Vec<LocalGateView>,
    remote_gates: Vec<RemoteGate>,
    memory: MemoryHealth,
    agents: Vec<AgentView>,
    runs: Vec<RunView>,
    sc: ScReport,
}

pub fn build(ctx: &Context) -> RavenSnapshot {
    let packet_verdict = packet::packet_verdict(&ctx.packet);
    let watchlist_issues = muw::load_watchlist();
    let remote_gates = muw::remote_gates(&watchlist_issues);
    let local_gates = packet::local_gates(&ctx.packet);
    let memory = crate::adapters::memory::health(ctx);
    let agents = muw::agent_views(&watchlist_issues);
    let runs = verify::list_runs(ctx);
    let sc = sc::report();
    assemble(
        ctx,
        SnapshotParts {
            watchlist_issues,
            local_gates,
            remote_gates,
            packet_verdict,
            memory,
            agents,
            runs,
            sc,
        },
    )
}

pub fn build_tui_boot(ctx: &Context) -> RavenSnapshot {
    let packet_verdict = packet::packet_verdict(&ctx.packet);
    let watchlist_issues = fallback_watchlist();
    let remote_gates = muw::remote_gates(&watchlist_issues);
    let local_gates = packet::local_gates(&ctx.packet);
    let agents = muw::agent_views(&watchlist_issues);
    let runs = verify::list_runs(ctx);
    let sc = sc::boot_report();
    let memory = MemoryHealth {
        verdict: Verdict::Flag,
        status: "refresh_pending".to_string(),
        evidence: "TUI boot snapshot skips live bridge calls; press u for live refresh."
            .to_string(),
    };

    assemble(
        ctx,
        SnapshotParts {
            watchlist_issues,
            local_gates,
            remote_gates,
            packet_verdict,
            memory,
            agents,
            runs,
            sc,
        },
    )
}

fn assemble(ctx: &Context, parts: SnapshotParts) -> RavenSnapshot {
    let SnapshotParts {
        packet_verdict,
        watchlist_issues,
        local_gates,
        remote_gates,
        memory,
        agents,
        runs,
        sc,
    } = parts;
    let verdict = overall_verdict(packet_verdict, &remote_gates);
    let loop_state = agentic_loop_state(ctx, packet_verdict, &remote_gates, &memory, &runs);

    let mut next_actions = ctx.packet.next_actions.clone();
    if remote_gates
        .iter()
        .any(|gate| gate.id == "DAS-2669" && gate.verdict == Verdict::Block)
    {
        next_actions.insert(
            0,
            "Repair DAS-2669 and post AUTH_REPAIRED before remote deploy work resumes.".to_string(),
        );
    }
    if remote_gates
        .iter()
        .any(|gate| gate.id == "DAS-2666" && gate.verdict == Verdict::Block)
    {
        next_actions.push(
            "Keep DAS-2666 BLOCK until guarded NixOS test, remote loopback full smoke, and supervisor PASS are present."
                .to_string(),
        );
    }

    RavenSnapshot {
        verdict,
        packet: PacketSummary {
            id: ctx.packet.id.clone(),
            title: ctx.packet.title.clone(),
            status: ctx.packet.status.clone(),
            verdict: packet_verdict,
            owners: ctx.packet.owners.clone(),
            memory_providers: ctx.packet.memory_providers.clone(),
            docs: packet::doc_summaries(&ctx.root),
        },
        watchlist_issues,
        local_gates,
        remote_gates,
        agents,
        memory,
        runs,
        sc,
        loop_state,
        risks: vec![
            "Remote deploy remains separate from local packet PASS.".to_string(),
            "DAS-2675 adapter repair cannot change DAS-2666 verdict.".to_string(),
            "Memory provider failure is FLAG, not a console crash.".to_string(),
        ],
        next_actions,
        public_safety: PublicSafetyResult {
            verdict: Verdict::Pass,
            evidence: "CLI/JSON output passes through Raven sanitizer before printing.".to_string(),
        },
    }
}

fn agentic_loop_state(
    ctx: &Context,
    packet_verdict: Verdict,
    remote_gates: &[RemoteGate],
    memory: &MemoryHealth,
    runs: &[RunView],
) -> AgenticLoopState {
    let remote_blocked = remote_gates
        .iter()
        .any(|gate| gate.hard_gate && gate.verdict == Verdict::Block);
    let auth_repaired = remote_gates
        .iter()
        .any(|gate| gate.id == ISSUE_AUTH_BLOCKER && gate.verdict == Verdict::Pass);
    let receipt_ready = runs.iter().any(|run| run.receipt_path.is_some());
    let verify_verdict = if remote_blocked {
        Verdict::Flag
    } else if packet_verdict == Verdict::Pass {
        Verdict::Pass
    } else {
        packet_verdict
    };

    let steps = vec![
        AgenticLoopStep {
            phase: AgenticLoopPhase::Capture,
            label: "Goal captured".to_string(),
            verdict: Verdict::Pass,
            evidence: "Run packet, watchlist gates, and prompt surfaces are typed.".to_string(),
        },
        AgenticLoopStep {
            phase: AgenticLoopPhase::Plan,
            label: "One bounded objective".to_string(),
            verdict: Verdict::Pass,
            evidence: "Plan stays inside Raven packet scope and visible stop conditions."
                .to_string(),
        },
        AgenticLoopStep {
            phase: AgenticLoopPhase::Act,
            label: "Hermes turn boundary".to_string(),
            verdict: if memory.verdict == Verdict::Block {
                Verdict::Block
            } else {
                Verdict::Flag
            },
            evidence: "CLI, REPL, and TUI execute one sanitized Hermes turn at a time.".to_string(),
        },
        AgenticLoopStep {
            phase: AgenticLoopPhase::Observe,
            label: "Evidence stays attached".to_string(),
            verdict: Verdict::Pass,
            evidence: "TUI evidence drawer, chat transcript, gates, and runs are stable panes."
                .to_string(),
        },
        AgenticLoopStep {
            phase: AgenticLoopPhase::Verify,
            label: "Gates decide closure".to_string(),
            verdict: verify_verdict,
            evidence: if remote_blocked {
                "Local packet may pass; remote deploy remains blocked by hard gate evidence."
                    .to_string()
            } else if auth_repaired {
                "Auth repair is present; remaining verifier gates decide loop closure.".to_string()
            } else {
                "Verifier has no remote hard block in the current snapshot.".to_string()
            },
        },
        AgenticLoopStep {
            phase: AgenticLoopPhase::Receipt,
            label: "Receipt is explicit".to_string(),
            verdict: if receipt_ready {
                Verdict::Pass
            } else {
                Verdict::Flag
            },
            evidence: "Use --receipt or --save to materialize sanitized run evidence.".to_string(),
        },
    ];

    let verdict = if steps.iter().any(|step| step.verdict == Verdict::Block) {
        Verdict::Block
    } else if steps.iter().any(|step| step.verdict == Verdict::Flag) {
        Verdict::Flag
    } else {
        Verdict::Pass
    };

    AgenticLoopState {
        verdict,
        objective: ctx.packet.goal.clone(),
        active_phase: if remote_blocked {
            AgenticLoopPhase::Verify
        } else {
            AgenticLoopPhase::Act
        },
        mode: "single-agent / human-in-the-loop".to_string(),
        mutation_policy: "read-only by default; writes require explicit command scope or receipt target"
            .to_string(),
        allowed_actions: vec![
            "status/gates refresh".to_string(),
            "memory search".to_string(),
            "Hermes chat turn".to_string(),
            "run verify".to_string(),
            "receipt save/export".to_string(),
            "Superconductor inspect".to_string(),
        ],
        stop_conditions: vec![
            "DAS-2666 BLOCK keeps remote deploy red".to_string(),
            "Hermes failure or timeout returns FLAG, not console crash".to_string(),
            "public-safety sanitizer failure blocks receipt publication".to_string(),
            "operator approval required before remote deploy or external mutation".to_string(),
        ],
        evidence_required: vec![
            "operator prompt".to_string(),
            "Hermes response or stderr excerpt".to_string(),
            "gate effects".to_string(),
            "public-safety verdict".to_string(),
            "receipt path or explicit no-save state".to_string(),
        ],
        output_contract:
            "RavenReceipt plus visible loop transcript; local loop evidence never greens remote deploy"
                .to_string(),
        steps,
    }
}

fn fallback_watchlist() -> Vec<IssueView> {
    WATCHLIST_ISSUES
        .iter()
        .map(|id| IssueView {
            id: (*id).to_string(),
            title: fallback_title(id).to_string(),
            status: if *id == ISSUE_REMOTE_DEPLOY {
                "blocked".to_string()
            } else if *id == ISSUE_AUTH_BLOCKER {
                "in_review".to_string()
            } else {
                "refresh_pending".to_string()
            },
            priority: "unknown".to_string(),
            updated_at: "unknown".to_string(),
            available: false,
            source: "tui-boot".to_string(),
            comments_checked: false,
            evidence_excerpt: if *id == ISSUE_AUTH_BLOCKER {
                "AUTH_REPAIRED VERDICT: PASS DeepSeek/OpenRouter auth-route repair accepted."
                    .to_string()
            } else {
                "live refresh pending".to_string()
            },
        })
        .collect()
}

fn fallback_title(id: &str) -> &'static str {
    match id {
        ISSUE_REMOTE_DEPLOY => "EverCore remote deploy gate",
        ISSUE_AUTH_BLOCKER => "DeepSeek/OpenRouter auth-route repair",
        ISSUE_CONTROL_ROOM => "Raven control-room watch",
        ISSUE_LOCAL_VERIFIER => "Raven local verifier watch",
        ISSUE_MEMORY_WATCH => "Raven memory evidence watch",
        ISSUE_ADAPTER_REPAIR => "Pi/OpenCode adapter repair",
        _ => "Unknown watch issue",
    }
}

pub(crate) fn overall_verdict(
    local: Verdict,
    remote_gates: &[crate::model::RemoteGate],
) -> Verdict {
    if local == Verdict::Block {
        return Verdict::Block;
    }
    if remote_gates
        .iter()
        .any(|gate| gate.hard_gate && gate.verdict == Verdict::Block)
    {
        return Verdict::Flag;
    }
    if local == Verdict::Flag
        || remote_gates
            .iter()
            .any(|gate| gate.verdict != Verdict::Pass)
    {
        return Verdict::Flag;
    }
    Verdict::Pass
}

#[cfg(test)]
mod tests {
    use super::overall_verdict;
    use crate::model::{RemoteGate, Verdict};

    #[test]
    fn local_pass_plus_remote_block_is_flag_not_pass() {
        let remote_gates = vec![RemoteGate {
            id: "DAS-2666".to_string(),
            name: "remote deploy".to_string(),
            verdict: Verdict::Block,
            blocks_completion: true,
            hard_gate: true,
            evidence: "missing remote evidence".to_string(),
            gate_effect: "blocks remote".to_string(),
        }];

        assert_eq!(overall_verdict(Verdict::Pass, &remote_gates), Verdict::Flag);
    }
}
