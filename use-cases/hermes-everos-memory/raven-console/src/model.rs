use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    Pass,
    Flag,
    Block,
}

impl Verdict {
    pub fn from_packet_word(value: &str) -> Self {
        let lower = value.to_ascii_lowercase();
        if lower.contains("block") || lower.contains("failed") {
            Self::Block
        } else if lower.contains("pass")
            || lower.contains("done")
            || lower.contains("closed")
            || lower.contains("complete")
        {
            Self::Pass
        } else {
            Self::Flag
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Flag => "FLAG",
            Self::Block => "BLOCK",
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AgenticLoopPhase {
    Capture,
    Plan,
    Act,
    Observe,
    Verify,
    Receipt,
}

impl AgenticLoopPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "CAPTURE",
            Self::Plan => "PLAN",
            Self::Act => "ACT",
            Self::Observe => "OBSERVE",
            Self::Verify => "VERIFY",
            Self::Receipt => "RECEIPT",
        }
    }
}

impl fmt::Display for AgenticLoopPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunPacket {
    pub id: String,
    pub title: String,
    pub goal: String,
    pub status: String,
    pub owners: Vec<String>,
    pub memory_providers: Vec<String>,
    pub lanes: Vec<Lane>,
    pub gates: Vec<Gate>,
    pub artifacts: Vec<Artifact>,
    pub evidence_refs: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lane {
    pub id: String,
    pub owner: String,
    pub scope: String,
    pub mutation_policy: String,
    pub verdict: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Gate {
    pub id: String,
    pub name: String,
    pub status: String,
    pub command: Option<String>,
    pub evidence: String,
    pub blocks_completion: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,
    pub purpose: String,
    pub public_safe: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PacketSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub verdict: Verdict,
    pub owners: Vec<String>,
    pub memory_providers: Vec<String>,
    pub docs: Vec<DocSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocSummary {
    pub path: String,
    pub verdict: Verdict,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LocalGateView {
    pub id: String,
    pub name: String,
    pub verdict: Verdict,
    pub command: String,
    pub evidence: String,
    pub blocks_completion: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct IssueView {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub updated_at: String,
    pub available: bool,
    pub source: String,
    pub comments_checked: bool,
    pub evidence_excerpt: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RemoteGate {
    pub id: String,
    pub name: String,
    pub verdict: Verdict,
    pub blocks_completion: bool,
    pub hard_gate: bool,
    pub evidence: String,
    pub gate_effect: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentView {
    pub name: String,
    pub issue_id: String,
    pub status: String,
    pub verdict: Verdict,
    pub scope: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemoryHealth {
    pub verdict: Verdict,
    pub status: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemorySearchResult {
    pub query: String,
    pub verdict: Verdict,
    pub evidence: String,
    pub result: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HermesChatTranscriptLine {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct HermesChatTurn {
    pub prompt: String,
    pub command: Vec<String>,
    pub workspace: String,
    pub runtime: String,
    pub verdict: Verdict,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub response: String,
    pub evidence: String,
    pub transcript: Vec<HermesChatTranscriptLine>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunView {
    pub id: String,
    pub command: String,
    pub verdict: Verdict,
    pub source: String,
    pub evidence: String,
    pub receipt_path: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgenticLoopStep {
    pub phase: AgenticLoopPhase,
    pub label: String,
    pub verdict: Verdict,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgenticLoopState {
    pub verdict: Verdict,
    pub objective: String,
    pub active_phase: AgenticLoopPhase,
    pub mode: String,
    pub mutation_policy: String,
    pub allowed_actions: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub evidence_required: Vec<String>,
    pub output_contract: String,
    pub steps: Vec<AgenticLoopStep>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScStatusView {
    pub verdict: Verdict,
    pub ok: bool,
    pub api_version: Option<u64>,
    pub app_version: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScProviderView {
    pub provider_key: String,
    pub display_name: String,
    pub enabled: bool,
    pub model_count: usize,
    pub reasoning_efforts: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScSessionView {
    pub thread_id: String,
    pub provider_key: String,
    pub title: String,
    pub model: String,
    pub reasoning_effort: String,
    pub active_turn: bool,
    pub closed: bool,
    pub branch: String,
    pub worktree: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScWorktreeView {
    pub verdict: Verdict,
    pub branch: String,
    pub target_branch: String,
    pub dirty: Option<bool>,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScReport {
    pub verdict: Verdict,
    pub status: ScStatusView,
    pub providers: Vec<ScProviderView>,
    pub sessions: Vec<ScSessionView>,
    pub worktree: ScWorktreeView,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicSafetyResult {
    pub verdict: Verdict,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RavenSnapshot {
    pub verdict: Verdict,
    pub packet: PacketSummary,
    pub watchlist_issues: Vec<IssueView>,
    pub local_gates: Vec<LocalGateView>,
    pub remote_gates: Vec<RemoteGate>,
    pub agents: Vec<AgentView>,
    pub memory: MemoryHealth,
    pub runs: Vec<RunView>,
    pub sc: ScReport,
    pub loop_state: AgenticLoopState,
    pub risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub public_safety: PublicSafetyResult,
}

#[derive(Clone, Debug, Serialize)]
pub struct GateEffect {
    pub gate_id: String,
    pub before: String,
    pub after: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RavenReceipt {
    pub id: String,
    pub command: Vec<String>,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub verdict: Verdict,
    pub evidence_excerpt: String,
    pub gate_effects: Vec<GateEffect>,
    pub public_safety: PublicSafetyResult,
}

#[derive(Clone, Debug, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub verdict: Verdict,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DoctorReport {
    pub verdict: Verdict,
    pub checks: Vec<DoctorCheck>,
    pub next: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NativeAuditItem {
    pub category: String,
    pub verdict: Verdict,
    pub evidence: String,
    pub hard_failure: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct NativeAuditReport {
    pub verdict: Verdict,
    pub items: Vec<NativeAuditItem>,
    pub blocks_pass_on: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResearchLane {
    pub id: String,
    pub title: String,
    pub question: String,
    pub targets: Vec<String>,
    pub output: Vec<String>,
    pub source_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResearchGateFact {
    pub id: String,
    pub verdict: Verdict,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResearchPacket {
    pub lane_id: String,
    pub lane_title: String,
    pub question: String,
    pub sources: Vec<String>,
    pub findings: Vec<String>,
    pub decisions: Vec<String>,
    pub v1_impact: Vec<String>,
    pub risks: Vec<String>,
    pub next: Vec<String>,
    pub live_gates: Vec<ResearchGateFact>,
    pub verdict: Verdict,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResearchSynthesis {
    pub verdict: Verdict,
    pub packets_ready: usize,
    pub required_packets: usize,
    pub evidence: String,
    pub decisions: Vec<String>,
    pub risks: Vec<String>,
    pub next: Vec<String>,
}
