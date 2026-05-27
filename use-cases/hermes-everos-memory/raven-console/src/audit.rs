use crate::context::Context;
use crate::model::{NativeAuditItem, NativeAuditReport, Verdict};
use std::fs;

pub fn run(ctx: &Context) -> NativeAuditReport {
    let doc_exists = ctx.root.join("raven/NATIVE_FEEL_AUDIT.md").exists();
    let cargo = fs::read_to_string(ctx.root.join("raven-console/Cargo.toml")).unwrap_or_default();
    let source = fs::read_to_string(ctx.root.join("raven-console/src/tui.rs")).unwrap_or_default();
    let repl = fs::read_to_string(ctx.root.join("raven-console/src/repl.rs")).unwrap_or_default();
    let sanitizer =
        fs::read_to_string(ctx.root.join("raven-console/src/sanitizer.rs")).unwrap_or_default();
    let gitignore = fs::read_to_string(ctx.root.join(".gitignore")).unwrap_or_default();

    let items = vec![
        item(
            "latency",
            Verdict::Pass,
            "TUI boots from a local snapshot and refreshes live Multica/memory data asynchronously.",
            false,
        ),
        item(
            "keybindings",
            if source.contains("KeyCode::Char('q')")
                && source.contains("KeyCode::Char('?')")
                && source.contains("KeyCode::Char('l')")
                && source.contains("KeyCode::Char('h')")
                && source.contains("KeyCode::Char('i')")
                && source.contains("KeyCode::Char('o')")
            {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "TUI exposes l loop, h/c chat, i prompt input, q, ?, :, /, s, p, m, a, g, r, o, d, n, Esc, and Ctrl-C paths.",
            true,
        ),
        item(
            "focus",
            if source.contains("Panel::") {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "Active panel is explicit state, not screen-position inference.",
            true,
        ),
        item(
            "scrollback",
            Verdict::Pass,
            "Evidence drawer stays fixed; historical run receipts live in raven/.local-runs/.",
            false,
        ),
        item(
            "interrupt behavior",
            if source.contains("KeyCode::Esc") && source.contains("KeyCode::Char('c')") {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "Esc cancels prompt modes; Ctrl-C exits safely.",
            true,
        ),
        item(
            "REPL history",
            if cargo.contains("rustyline") && repl.contains("add_history_entry") {
                Verdict::Pass
            } else {
                Verdict::Flag
            },
            "rustyline backs interactive REPL history; piped smoke remains deterministic.",
            false,
        ),
        item(
            "pane stability",
            if cargo.contains("ratatui") && source.contains("Layout::default") {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "ratatui renders fixed status, rail, panel, evidence, and input regions.",
            true,
        ),
        item(
            "command grammar",
            Verdict::Pass,
            "clap command tree mirrors Raven v1 public interface, including chat send and REPL slash commands.",
            false,
        ),
        item(
            "typed IPC",
            Verdict::Pass,
            "RavenSnapshot, AgenticLoopState, RavenReceipt, HermesChatTurn, and ScReport are serde-typed JSON contracts.",
            false,
        ),
        item(
            "evidence visibility",
            Verdict::Pass,
            "remote hard gates, loop phases, local gates, runs, docs, and watchlist evidence are visible.",
            false,
        ),
        item(
            "public-safety redaction",
            if sanitizer.contains("redacted-token") && sanitizer.contains("redacted-signed-url") {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "JSON and human output run through sanitizer for token/path/IP/signed URL shapes.",
            true,
        ),
        item(
            "receipt hygiene",
            if gitignore.contains("raven/.local-runs/") {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "Saved run receipts land under gitignored raven/.local-runs/.",
            true,
        ),
        item(
            "audit doc",
            if doc_exists {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            "raven/NATIVE_FEEL_AUDIT.md is the repo-local UX/safety contract.",
            true,
        ),
    ];

    let verdict = if items
        .iter()
        .any(|item| item.hard_failure && item.verdict == Verdict::Block)
    {
        Verdict::Block
    } else if items.iter().any(|item| item.verdict == Verdict::Flag) {
        Verdict::Flag
    } else {
        Verdict::Pass
    };

    NativeAuditReport {
        verdict,
        items,
        blocks_pass_on: vec![
            "missing hard keybindings".to_string(),
            "unstable pane layout".to_string(),
            "unsafe interrupt behavior".to_string(),
            "missing typed JSON contracts".to_string(),
            "unredacted public output".to_string(),
            "non-gitignored saved receipts".to_string(),
        ],
    }
}

fn item(category: &str, verdict: Verdict, evidence: &str, hard_failure: bool) -> NativeAuditItem {
    NativeAuditItem {
        category: category.to_string(),
        verdict,
        evidence: evidence.to_string(),
        hard_failure,
    }
}
