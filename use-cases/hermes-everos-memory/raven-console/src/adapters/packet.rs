use crate::model::{DocSummary, LocalGateView, RunPacket, Verdict};
use crate::sanitizer::sanitize_text;
use crate::util::{one_line, truncate};
use crate::RavenResult;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

pub fn read_packet(path: &Path) -> RavenResult<RunPacket> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

pub fn packet_verdict(packet: &RunPacket) -> Verdict {
    if packet
        .lanes
        .iter()
        .any(|lane| lane.verdict.eq_ignore_ascii_case("block"))
        || packet
            .gates
            .iter()
            .any(|gate| gate.blocks_completion && gate.status.eq_ignore_ascii_case("block"))
    {
        return Verdict::Block;
    }

    if packet.lanes.iter().any(|lane| {
        matches!(
            lane.verdict.to_ascii_lowercase().as_str(),
            "flag" | "active"
        )
    }) || packet.gates.iter().any(|gate| {
        gate.blocks_completion
            && matches!(
                gate.status.to_ascii_lowercase().as_str(),
                "flag" | "not_run"
            )
    }) {
        return Verdict::Flag;
    }

    Verdict::Pass
}

pub fn local_gates(packet: &RunPacket) -> Vec<LocalGateView> {
    packet
        .gates
        .iter()
        .map(|gate| LocalGateView {
            id: gate.id.clone(),
            name: gate.name.clone(),
            verdict: Verdict::from_packet_word(&gate.status),
            command: gate.command.clone().unwrap_or_else(|| "manual".to_string()),
            evidence: sanitize_text(&one_line(&gate.evidence)),
            blocks_completion: gate.blocks_completion,
        })
        .collect()
}

pub fn doc_summaries(root: &Path) -> Vec<DocSummary> {
    [
        "COMPLETION_AUDIT.md",
        "OWNER_PACKET.md",
        "SUPERVISOR_DISPATCH.md",
        "raven/NATIVE_FEEL_AUDIT.md",
    ]
    .into_iter()
    .map(|path| doc_summary(root, path))
    .collect()
}

fn doc_summary(root: &Path, relative: &str) -> DocSummary {
    let path = root.join(relative);
    match fs::read_to_string(&path) {
        Ok(text) => {
            let title = text
                .lines()
                .next()
                .unwrap_or(relative)
                .trim_start_matches("# ")
                .to_string();
            let line = text
                .lines()
                .find(|line| {
                    line.contains("PASS") || line.contains("FLAG") || line.contains("BLOCK")
                })
                .unwrap_or("verdict not found");
            DocSummary {
                path: relative.to_string(),
                verdict: Verdict::from_packet_word(line),
                evidence: sanitize_text(&truncate(&format!("{title}; {}", one_line(line)), 260)),
            }
        }
        Err(err) => DocSummary {
            path: relative.to_string(),
            verdict: Verdict::Block,
            evidence: format!("read failed: {err}"),
        },
    }
}
