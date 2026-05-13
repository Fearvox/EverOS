# May Agent — Document Index

**Status**: Active
**Date**: 2026-05-13
**Owner**: Nolan + 七仔 + 邓Sir (reviewer)

## TL;DR

Six architecture documents define the May Agent: strategy (00), system design (10),
Rust runtime scaffold (20), EverCore integration contract (30), benchmark strategy
(40), and risk log (90). This index provides a reading map, cross-document reference
matrix, and status dashboard.

## Document Manifest

| # | File | Lines | PR | Subject |
|---|------|-------|-----|---------|
| 00 | `00-vision.md` | 141 | #16 | Strategy, decision matrix, success criteria |
| 10 | `10-architecture.md` | 264 | #17 | System design, mermaid diagrams, data flow |
| 20 | `20-rust-runtime-scaffold.md` | 305 | #18 | Crate structure, async traits, build pipeline |
| 30 | `30-evercore-integration-contract.md` | 266 | #19 | API surface, wire format, error codes |
| 40 | `40-benchmark-strategy.md` | 185 | #20 | 3-benchmark plan, success criteria, publication |
| 90 | `90-risk-log.md` | 216 | #21 | 7 risks, mitigations, escalation triggers |
| — | `INDEX.md` | this | #22 | Document guide and cross-reference |

**Total**: 1,377 lines across 6 core documents.

## Reading Paths

### Path A — Executive (30 min)

For Yifan, 邓Sir, and stakeholders who need the strategic picture without
implementation detail.

1. `00-vision.md` — what we're building, and why now
2. `90-risk-log.md` — what could go wrong, and our plan for each risk
3. `00-vision.md` success criteria table (bottom of doc) — how we measure May 31

Skip: 10, 20, 30, 40 (delegate to Nolan/七仔).

### Path B — Developer (2 hours)

For Nolan and 七仔 implementing the Rust runtime and EverCore integration.

1. `00-vision.md` — strategic context
2. `10-architecture.md` — system design and data flow (read the mermaid diagrams first)
3. `20-rust-runtime-scaffold.md` — crate map, trait signatures, build order
4. `30-evercore-integration-contract.md` — API endpoints, error codes, wire format
5. `90-risk-log.md` — R2 (Hermes churn), R4 (GIL), R5 (sandbox)

Reference `40-benchmark-strategy.md` when building benchmark integration in Sprint 3.

### Path C — Reviewer (60 min)

For 邓Sir or Yifan doing architecture review.

1. `00-vision.md` decision matrix (8 choices) — validate assumptions
2. `10-architecture.md` decision matrices (4) — validate technical choices
3. `90-risk-log.md` — validate risk ratings and escalation triggers
4. Spot-check `20-rust-runtime-scaffold.md` trait signatures
5. Spot-check `30-evercore-integration-contract.md` endpoint mapping

## Cross-Document Reference Matrix

| Topic | 00 | 10 | 20 | 30 | 40 | 90 |
|-------|----|----|----|----|----|-----|
| Fork Hermes decision | ● | → | | | | |
| Rust runtime rationale | ● | ● | ● | | | |
| EverCore memory kernel | ● | ● | | ● | | |
| seccomp sandbox | | ● | ● | | | R5 |
| WeCom / Feishu | ● | ● | | | | |
| Crate structure | | | ● | | | |
| PyO3 FFI | | ● | ● | | | |
| ToolDispatcher trait | | | ● | | | |
| AgentHandle trait | | | ● | | | |
| MemoryClient trait | | | ● | ● | | |
| REST API endpoints | | | | ● | | |
| Tenant model | | | | ● | | |
| Circuit breaker | | | | ● | | |
| Error codes (8) | | | | ● | | |
| EverMem Bench | | | | | ● | |
| EvoAgent Bench | | | | | ● | |
| Evil Agent Bench | | | | | ● | |
| arXiv publication | | | | | ● | |
| Rust team buy-in | | | | | | R1 |
| Hermes upstream churn | | | | | | R2 |
| 邓Sir bandwidth | | | | | | R3 |
| Python GIL | | | | | | R4 |
| Cross-platform sandbox | | | | | | R5 |
| May 31 deadline | ● | | | | | R6 |

● = primary source, → = secondary reference

## Key Decisions (with locations)

| ID | Decision | Doc | Section |
|----|----------|-----|---------|
| D1 | Fork Hermes, don't build from scratch | 00 | Decision Matrix |
| D2 | Rust runtime + Python agent core (hybrid) | 10 | Decision Matrix 1 |
| D3 | EverCore memory backend (not Postgres direct) | 10 | Decision Matrix 2 |
| D4 | seccomp-bpf primary, Docker fallback, macOS native later | 10 | Decision Matrix 3 |
| D5 | WeCom P0, Feishu P0, Slack P1, Discord P2 | 10 | Decision Matrix 4 |
| D6 | 5-crate structure (cli, runtime, ffi, desktop, sandbox) | 20 | Crate Map |
| D7 | Subprocess-per-session model for GIL isolation | 20 | Risk Matrix |
| D8 | Circuit breaker with 30s timeout, 3 retries | 30 | Failure Modes |
| D9 | 3-benchmark strategy, Evil Agent Bench as differentiator | 40 | Benchmark Plan |
| D10 | Scope to MVP for May 31 — binary + 1 platform + 1 benchmark | 00 | Success Criteria |

## Document Relationships

```
00-vision.md (strategy anchor)
    ├── 10-architecture.md (design realization)
    │   ├── 20-rust-runtime-scaffold.md (implementation detail)
    │   └── 30-evercore-integration-contract.md (API contract)
    ├── 40-benchmark-strategy.md (validation plan)
    └── 90-risk-log.md (derailer tracking)
```

## Gap Analysis

| Area | Status | Doc | Action |
|------|--------|-----|--------|
| Cargo.toml (actual) | Not started | 20 | Sprint 1 deliverable |
| PyO3 FFI bindings | Not started | 20, 30 | Sprint 2 deliverable |
| EverCore Docker compose | Exists | 30 | Needs tenant-id header change |
| seccomp profile (BPF) | Not started | 10, 20 | Sprint 2 deliverable |
| macOS Seatbelt profile | Not started | 10 | Post-May-31 |
| Windows sandbox | Out of scope | 90 | Per R5 |
| WeCom adapter | Not started | 10 | Sprint 3 deliverable |
| Feishu adapter | Not started | 10 | Sprint 3 deliverable |
| EverMem Bench run | Exists | 40 | Run against May Agent |
| EvoAgent Bench run | Exists | 40 | Run against May Agent |
| Evil Agent Bench | Not started | 40 | Sprint 3 deliverable |

## Status Dashboard

| Doc | Draft | Reviewed | Linked from INDEX |
|-----|-------|----------|-------------------|
| 00-vision | ✅ | ⬜ 邓Sir | ✅ |
| 10-architecture | ✅ | ⬜ 邓Sir | ✅ |
| 20-rust-runtime-scaffold | ✅ | ⬜ | ✅ |
| 30-evercore-integration-contract | ✅ | ⬜ | ✅ |
| 40-benchmark-strategy | ✅ | ⬜ | ✅ |
| 90-risk-log | ✅ | ⬜ | ✅ |

## References

- Hermes recon: `.planning/hermes-recon/architecture.md`
- EverCore entry: `methods/EverCore/src/run.py`
- EverCore controllers: `methods/EverCore/src/infra_layer/adapters/input/api/`
- CLAUDE_DESKTOP_SANDBOX_SOURCE_TRUTH.md (macOS sandbox forensics)
- Sleep run log: `.planning/SLEEP_LOG.md`
