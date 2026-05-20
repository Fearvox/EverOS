# 00 — Vision: May Agent Deliverable

**Status**: Draft
**Date**: 2026-05-13
**Audience**: EverMind internal (Yifan, 邓Sir, 七仔, Nolan)
**Sibling docs**: See INDEX.md for full document map

## TL;DR

Ship a competitive agent by May 31 that stands against OpenClaw and Hermes in
the Chinese enterprise market. Architecture: fork Hermes (MIT) + Rust rewrite
of the runtime layer + Evercore as the memory kernel. Differentiator: Persist
Memory + enterprise ecosystem (企业微信/飞书/Tanka) + benchmark narrative
(EverMem Bench + Evil Agent Bench).

## Vision Statement

The May Agent is **EverMind's first production agent product**. It combines
three proven pieces:

1. **Hermes agent core** (MIT license, 147k★, battle-tested tool dispatch,
   multi-model routing, 20+ messaging platforms)
2. **Rust runtime** (sandbox, plugin loader, CLI — replacing Hermes's 600K+
   lines of Python CLI/TUI code with a fast, safe, single-binary runtime)
3. **Evercore memory kernel** (tenant-scoped long-term memory, multi-tenant
   storage, prompt-optimized recall)

The result: an agent that **remembers across sessions**, runs **safely in
Rust sandboxes**, and deploys **natively into Chinese enterprise chat
ecosystems** (企业微信, 飞书, Tanka).

## Why This Architecture

### The "Fork Hermes" Decision

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| Build from scratch | Full control, no legacy | Won't hit May deadline | ❌ Too slow |
| Wrap Hermes as-is | Fastest path | Python GIL, no Rust sandbox, hard to embed | ❌ No differentiation |
| Fork + rewrite runtime | Proven core, Rust differentiation, embeddable | Integration risk, Rust team buy-in | ✅ Chosen |
| Use OpenClaw instead | Strong community | Not MIT-licensed, different ecosystem | ❌ Licensing risk |

**Decision**: Fork Hermes (MIT) and rewrite the runtime in Rust while keeping
the agent core, tool registry, and model providers intact.

### The "Rust Runtime" Decision

The Hermes `cli.py` is 615K lines. The TUI is a separate React/Ink app. The
environments layer (docker, ssh, modal, etc.) is Python. Replacing all of this
with Rust gives us:

- **Single binary** — no Python venv, no `pip install`, no dependency hell
- **Memory safety** — sandbox enforcement at the type level, not Seatbelt hacks
- **Embeddable** — Tauri shell for desktop, WASM for web, FFI for mobile
- **Performance** — tokio async runtime, zero-copy where possible

### The "Evercore Kernel" Decision

Evercore replaces Hermes's pluggable memory layer (`plugins/memory/`). Instead
of swapping between honcho/mem0/supermemory, the agent gets **one memory
backend that actually works**:

- Tenant-scoped storage (multi-user, multi-project)
- Long-term memory with prompt-optimized retrieval
- EN/ZH bilingual prompt alignment (see `methods/EverCore/src/memory_layer/prompts/`)
- Evaluated on EverMem Bench and EvoAgent Bench

This is 邓Sir's "abstract Memory" suggestion in practice: keep the Hermes
memory plugin interface, swap the implementation.

## Target Market

### Primary: Chinese enterprise AI assistant

- 企业微信 (WeCom) integration via Hermes's existing gateway pattern
- 飞书 (Feishu/Lark) integration — same pattern, different platform adapter
- Tanka integration — Chinese team communication platform
- Persist Memory as the key differentiator — "the agent that remembers your
  project context across sprints"

### Secondary: International agent market

- Competitive with OpenClaw on benchmarks
- Open-source narrative via fork → upstream PR strategy
- Discord community (Nolan's inter-PR marketing play)

## Success Criteria (May 31)

| Criterion | Threshold | Stretch |
|-----------|-----------|---------|
| Agent ships as single binary | `cargo build --release` produces a working binary | Binary < 50MB |
| Memory works across sessions | EverCore stores + retrieves from prior sessions | >80% recall accuracy on EverMem Bench |
| At least 1 enterprise platform | WeCom OR Feishu adapter works end-to-end | Both platforms |
| Sandbox prevents escape | Basic file-write/network egress containment | Passes Evil Agent Bench escape suite |
| Benchmark narrative ready | EverMem + Evil Agent Bench results published | Beats OpenClaw on 2+ axes |

## Non-Goals (May 31)

- Full CLI parity with Hermes (the CLI is 615K lines — we ship the runtime,
  not a clone of `hermes --tui`)
- All 8 memory provider plugins (Evercore replaces them)
- All 20+ messaging platforms (1-2 enterprise platforms + 1 international)
- RL training pipeline (environments/ + rl_cli.py — out of scope)
- Mobile apps (architecture supports it, not shipping May 31)

## Decision Matrix

| Decision | Owner | Date | Status |
|----------|-------|------|--------|
| Fork Hermes (MIT) | 邓Sir + Nolan | 2026-05-12 | ✅ Approved |
| Rust rewrite (not Python fork) | 邓Sir + Nolan | 2026-05-12 | ✅ Approved |
| Evercore as memory kernel | 邓Sir | 2026-05-12 | ✅ Approved |
| Target WeCom + Feishu | Nolan | 2026-05-13 | 📋 Proposed |
| EverMem Bench as primary eval | Nolan | 2026-05-13 | 📋 Proposed |
| Evil Agent Bench for safety | Nolan | 2026-05-13 | 📋 Proposed |
| Tauri for desktop shell | TBD | — | 🕐 Needs research |
| tokio for async runtime | TBD | — | 🕐 Needs research |

## Risk Log (top 5)

1. **Rust team buy-in** — Yifan flagged: "团队大部分人习惯 Python，第一反应会抗拒"
   → Mitigation: keep Python for agent core, Rust only for runtime layer
2. **May 31 deadline** — 18 days from today
   → Mitigation: scope to "single binary + 1 platform + memory works"
3. **Hermes upstream churn** — Hermes ships rapidly (v0.13 as of May 2026)
   → Mitigation: fork at a stable SHA, rebase selectively
4. **邓Sir bandwidth** — "all in 自己的 Agent 品牌上线，可能无暇顾及多"
   → Mitigation: Nolan owns integration; 邓Sir reviews architecture
5. **Evercore API stability** — EverCore is pre-1.0
   → Mitigation: pin to a specific EverCore version, define API contract in 30-evercore-integration-contract.md

## References

- Hermes recon: `.planning/hermes-recon/architecture.md` (this repo)
- EverCore memory manager: `methods/EverCore/src/agentic_layer/memory_manager.py`
- EverCore API: `methods/EverCore/src/infra_layer/adapters/input/api/`
- EverMem Bench: `benchmarks/EverMemBench/`
- EvoAgent Bench: `benchmarks/EvoAgentBench/`
- Fork strategy: `.planning/hermes-recon/architecture.md#fork-strategy`
- Memory context: `~/.claude/projects/.../memory/project_evermind_2026_05.md`
- Hermes upstream: `NousResearch/hermes-agent` (147k★, MIT, SHA: HEAD 2026-05-13)
