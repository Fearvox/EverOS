use crate::model::{
    RemoteGate, ResearchGateFact, ResearchLane, ResearchPacket, ResearchSynthesis, Verdict,
};

const REQUIRED_SYNTHESIS_PACKETS: usize = 3;

struct LaneSpec {
    id: &'static str,
    title: &'static str,
    question: &'static str,
    targets: &'static [&'static str],
    output: &'static [&'static str],
    source_refs: &'static [&'static str],
    findings: &'static [&'static str],
    decisions: &'static [&'static str],
    v1_impact: &'static [&'static str],
    risks: &'static [&'static str],
    next: &'static [&'static str],
}

pub fn list_lanes() -> Vec<ResearchLane> {
    lane_specs().iter().map(|lane| lane.view()).collect()
}

pub fn packet_for_lane(lane_id: &str, remote_gates: &[RemoteGate]) -> Option<ResearchPacket> {
    let lane = lane_specs().into_iter().find(|lane| lane.id == lane_id)?;
    let live_gates = live_gate_facts(remote_gates);
    let mut risks = strings(lane.risks);
    for gate in &live_gates {
        if gate.verdict == Verdict::Block {
            risks.push(format!(
                "{} remains BLOCK in live gate evidence; v2 research cannot promote remote readiness.",
                gate.id
            ));
        }
    }

    let verdict = if live_gates.iter().any(|gate| gate.verdict == Verdict::Block) {
        Verdict::Flag
    } else {
        Verdict::Pass
    };

    let mut next = strings(lane.next);
    next.push(
        "Turn this lane into a decision packet before any v1 implementation change.".to_string(),
    );
    next.push(format!(
        "Do not synthesize Raven v2 architecture until at least {REQUIRED_SYNTHESIS_PACKETS} evidence-backed packets exist."
    ));

    Some(ResearchPacket {
        lane_id: lane.id.to_string(),
        lane_title: lane.title.to_string(),
        question: lane.question.to_string(),
        sources: strings(lane.source_refs),
        findings: strings(lane.findings),
        decisions: strings(lane.decisions),
        v1_impact: strings(lane.v1_impact),
        risks,
        next,
        live_gates,
        verdict,
    })
}

pub fn synthesis_readiness(packets: &[ResearchPacket]) -> ResearchSynthesis {
    let packets_ready = packets.len();
    let ready = packets_ready >= REQUIRED_SYNTHESIS_PACKETS;
    ResearchSynthesis {
        verdict: if ready { Verdict::Pass } else { Verdict::Flag },
        packets_ready,
        required_packets: REQUIRED_SYNTHESIS_PACKETS,
        evidence: if ready {
            format!(
                "{packets_ready} evidence-backed packets are available for architecture synthesis."
            )
        } else {
            format!(
                "{packets_ready}/{REQUIRED_SYNTHESIS_PACKETS} evidence-backed packets available."
            )
        },
        decisions: if ready {
            vec![
                "Architecture synthesis may start, but it must still preserve live gate verdicts."
                    .to_string(),
            ]
        } else {
            vec![
                "Hold RAVEN_V2_ARCHITECTURE_PACKET.md until research evidence reaches quorum."
                    .to_string(),
            ]
        },
        risks: vec![
            "Research synthesis without packet quorum becomes prose drift.".to_string(),
            "Remote deploy truth remains owned by DAS-2666/DAS-2669, not the research lane."
                .to_string(),
        ],
        next: if ready {
            vec![
                "Open a bounded architecture synthesis task using the completed packets."
                    .to_string(),
            ]
        } else {
            vec![format!(
                "Collect at least three evidence-backed packets before synthesis ({packets_ready}/{REQUIRED_SYNTHESIS_PACKETS} ready)."
            )]
        },
    }
}

pub fn packet_markdown(packet: &ResearchPacket) -> String {
    let mut out = Vec::new();
    out.push("RAVEN_V2_RESEARCH_PACKET".to_string());
    out.push(format!("LANE: {} / {}", packet.lane_id, packet.lane_title));
    out.push(format!("QUESTION: {}", packet.question));
    push_list(&mut out, "SOURCES", &packet.sources);
    push_list(&mut out, "FINDINGS", &packet.findings);
    push_list(&mut out, "DECISIONS", &packet.decisions);
    push_list(&mut out, "V1_IMPACT", &packet.v1_impact);
    push_list(&mut out, "RISKS", &packet.risks);
    push_list(&mut out, "NEXT", &packet.next);
    out.push("LIVE_GATES:".to_string());
    for gate in &packet.live_gates {
        out.push(format!("- {}: {} {}", gate.id, gate.verdict, gate.evidence));
    }
    out.push(format!("VERDICT: {}", packet.verdict));
    out.join("\n")
}

pub fn synthesis_markdown(synthesis: &ResearchSynthesis) -> String {
    let mut out = Vec::new();
    out.push("RAVEN_V2_SYNTHESIS_READINESS".to_string());
    out.push(format!("VERDICT: {}", synthesis.verdict));
    out.push(format!(
        "PACKETS: {}/{}",
        synthesis.packets_ready, synthesis.required_packets
    ));
    out.push(format!("EVIDENCE: {}", synthesis.evidence));
    push_list(&mut out, "DECISIONS", &synthesis.decisions);
    push_list(&mut out, "RISKS", &synthesis.risks);
    push_list(&mut out, "NEXT", &synthesis.next);
    out.join("\n")
}

fn push_list(out: &mut Vec<String>, title: &str, values: &[String]) {
    out.push(format!("{title}:"));
    for value in values {
        out.push(format!("- {value}"));
    }
}

impl LaneSpec {
    fn view(&self) -> ResearchLane {
        ResearchLane {
            id: self.id.to_string(),
            title: self.title.to_string(),
            question: self.question.to_string(),
            targets: strings(self.targets),
            output: strings(self.output),
            source_refs: strings(self.source_refs),
        }
    }
}

fn live_gate_facts(remote_gates: &[RemoteGate]) -> Vec<ResearchGateFact> {
    remote_gates
        .iter()
        .filter(|gate| gate.hard_gate || matches!(gate.id.as_str(), "DAS-2666" | "DAS-2669"))
        .map(|gate| ResearchGateFact {
            id: gate.id.clone(),
            verdict: gate.verdict,
            evidence: gate.evidence.clone(),
        })
        .collect()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn lane_specs() -> Vec<LaneSpec> {
    vec![
        LaneSpec {
            id: "native-feel",
            title: "Native-Feel TUI/REPL",
            question: "What makes Raven feel like a native terminal OS surface rather than a webby text box?",
            targets: &[
                "latency budget",
                "keyboard grammar",
                "focus and pane stability",
                "interrupt/resume semantics",
                "scrollback and transcript model",
            ],
            output: &[
                "interaction contract",
                "v2 command grammar",
                "native-feel audit adapted for terminal agents",
            ],
            source_refs: &[
                "raven/RAVEN_V2_RESEARCH_LEDGER.md#lane-1-native-feel-tuirepl",
                "raven/NATIVE_FEEL_AUDIT.md",
                "raven/COMMAND_CONTRACT.md#hermes-chat-behavior",
            ],
            findings: &[
                "Raven v1 already separates fast boot rendering from live refresh, which is the right latency shape for v2.",
                "The CCR-level target is a stable REPL/TUI split: shell, slash REPL, and TUI chat must share one command grammar.",
                "Native feel depends on interrupt behavior and pane stability as much as visual density.",
            ],
            decisions: &[
                "Keep Rust ratatui for v1; research richer v2 terminal runtimes only through decision packets.",
                "Treat chat transcript, gate evidence, and command output as typed state, not painted strings.",
            ],
            v1_impact: &[
                "Add harness checks before changing TUI layout again.",
                "Keep Hermes chat as shared adapter across CLI, REPL, and TUI.",
            ],
            risks: &[
                "A prettier TUI can hide stale gate truth if refresh and evidence panes are not explicit.",
            ],
            next: &[
                "Measure cold boot, first paint, and chat submit latency with deterministic smoke output.",
            ],
        },
        LaneSpec {
            id: "runtime-dna",
            title: "Runtime DNA Alignment",
            question: "How do CCB/CCR/Evensong concepts flow into Raven without turning Raven into a fork dump?",
            targets: &[
                "CLI loop",
                "REPL state machine",
                "tool approval model",
                "ACP/control-plane concepts",
                "telemetry and receipts",
            ],
            output: &[
                "lineage map",
                "implementation boundaries",
                "Raven-owned versus Hermes/MUW-owned responsibilities",
            ],
            source_refs: &[
                "raven/RAVEN_V2_RESEARCH_LEDGER.md#lane-2-runtime-dna-alignment",
                "raven/COMMAND_CONTRACT.md#shape",
                "OWNER_PACKET.md",
            ],
            findings: &[
                "Raven owns operator-visible truth state; Hermes owns provider dialogue; MUW owns live issue gates.",
                "Receipts are the bridge between interactive UX and reviewable evidence.",
            ],
            decisions: &[
                "Do not vendor external runtime code into v1.",
                "Represent borrowed runtime ideas as boundaries and tests before implementation.",
            ],
            v1_impact: &[
                "Keep adapters thin and read-only unless a command explicitly writes a receipt.",
            ],
            risks: &[
                "Fork-dump research would widen scope and make v1 harder to verify.",
            ],
            next: &[
                "Create a lineage map packet that marks keep/revise/defer/reject per runtime idea.",
            ],
        },
        LaneSpec {
            id: "memory-skill",
            title: "Memory And Skill Substrate",
            question: "How should Raven make memory, skills, and goals first-class without becoming a noisy memory browser?",
            targets: &[
                "EverOS memory search/store/status",
                "Hermes skills and profiles",
                "persistent goals",
                "provenance fields",
                "memory hit explanations",
            ],
            output: &[
                "memory pane contract",
                "skill registry contract",
                "goal/gate model",
            ],
            source_refs: &[
                "raven/RAVEN_V2_RESEARCH_LEDGER.md#lane-3-memory-and-skill-substrate",
                "skillhub/MVP_IMPLEMENTATION_PLAN.md",
                "skillhub/schema.json",
                "DAS-2672",
            ],
            findings: &[
                "DAS-2672 already separated production-ready local facts from needs_eval SkillHub items.",
                "Skill promotion needs eval evidence; packet existence alone is not a skill-quality claim.",
            ],
            decisions: &[
                "Build eval harness before adding richer SkillHub fields.",
                "Keep memory provider failure as FLAG, not a console crash.",
            ],
            v1_impact: &[
                "Use v1 research packets to select the next SkillHub implementation issue.",
            ],
            risks: &[
                "Memory browsing without provenance can look powerful while weakening trust.",
            ],
            next: &[
                "Draft the SkillHub eval harness packet before mutating skill fixtures.",
            ],
        },
        LaneSpec {
            id: "orchestration",
            title: "Multi-Agent Orchestration",
            question: "What is the operator model when many agents are building, reviewing, and researching at once?",
            targets: &[
                "MUW issue states",
                "bounded fanout",
                "subagent context isolation",
                "task delegation and review packets",
                "red-gate routing",
            ],
            output: &[
                "control-room state model",
                "dispatch grammar",
                "review lane protocol",
            ],
            source_refs: &[
                "raven/RAVEN_V2_RESEARCH_LEDGER.md#lane-4-multi-agent-orchestration",
                "SUPERVISOR_DISPATCH.md",
                "DAS-2670",
                "DAS-2666",
                "DAS-2669",
            ],
            findings: &[
                "Current control room truth is issue-led: local PASS and remote BLOCK must remain separate.",
                "Raven should route work by gate state before spawning or assigning broader fanout.",
            ],
            decisions: &[
                "Remote deploy stays owned by DAS-2666; auth repair stays owned by DAS-2669.",
                "Adapter repair lanes cannot change remote deploy verdicts.",
            ],
            v1_impact: &[
                "Research commands should display live MUW blockers before next implementation suggestions.",
            ],
            risks: &[
                "Without live issue calibration, v2 planning can launder stale local PASS into remote readiness.",
            ],
            next: &[
                "Define dispatch grammar for assigning research packets without opening mutation lanes prematurely.",
            ],
        },
        LaneSpec {
            id: "evaluation-safety",
            title: "Evaluation And Safety",
            question: "How do we know Raven is making the system more legible rather than only faster?",
            targets: &[
                "audit trails",
                "failure records",
                "public-safety scan",
                "secret/host/IP redaction",
                "benchmark receipt ingestion",
                "truth-state transitions",
            ],
            output: &[
                "Raven v2 success metrics",
                "red-gate invariants",
                "public-safe artifact checklist",
            ],
            source_refs: &[
                "raven/RAVEN_V2_RESEARCH_LEDGER.md#lane-5-evaluation-and-safety",
                "raven/NATIVE_FEEL_AUDIT.md",
                "raven/COMMAND_CONTRACT.md#gate-semantics",
            ],
            findings: &[
                "Raven already sanitizes JSON/text output; v2 needs receipt-level proof that redaction remains intact.",
                "Success metrics must include truth-state preservation, not just command latency.",
            ],
            decisions: &[
                "Public-safety failures block PASS for native audit and research packet promotion.",
                "Architecture synthesis must preserve hard red gates in its first page.",
            ],
            v1_impact: &[
                "Add research packet smoke tests to prevent prose-only v2 output.",
            ],
            risks: &[
                "Safety claims are easy to overstate if screenshots or markdown include raw operational details.",
            ],
            next: &[
                "Add a public-safety scan target for research packet exports.",
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{list_lanes, packet_for_lane, synthesis_readiness};
    use crate::model::{RemoteGate, Verdict};

    fn blocked_auth_gate() -> Vec<RemoteGate> {
        vec![RemoteGate {
            id: "DAS-2669".to_string(),
            name: "runtime auth".to_string(),
            verdict: Verdict::Block,
            blocks_completion: true,
            hard_gate: true,
            evidence: "AUTH_REPAIRED missing in live issue/comment evidence".to_string(),
            gate_effect: "blocks remote deploy readiness".to_string(),
        }]
    }

    #[test]
    fn lists_five_research_lanes() {
        let lanes = list_lanes();

        assert_eq!(lanes.len(), 5);
        assert_eq!(lanes[0].id, "native-feel");
        assert!(lanes.iter().any(|lane| lane.id == "evaluation-safety"));
    }

    #[test]
    fn packet_preserves_live_remote_blockers() {
        let packet = packet_for_lane("native-feel", &blocked_auth_gate()).unwrap();

        assert_eq!(packet.verdict, Verdict::Flag);
        assert_eq!(packet.lane_id, "native-feel");
        assert!(packet
            .risks
            .iter()
            .any(|risk| risk.contains("DAS-2669") && risk.contains("BLOCK")));
        assert!(packet
            .next
            .iter()
            .any(|action| action.contains("decision packet")));
    }

    #[test]
    fn synthesis_stays_flag_until_three_packets() {
        let synthesis = synthesis_readiness(&[]);

        assert_eq!(synthesis.verdict, Verdict::Flag);
        assert!(synthesis
            .next
            .iter()
            .any(|action| action.contains("three evidence-backed packets")));
    }
}
