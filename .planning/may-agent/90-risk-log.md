# 90 — Risk Log: May 31 Derailers and Mitigations

**Status**: Draft
**Date**: 2026-05-13
**Depends on**: 00-vision.md

## TL;DR

18 days to May 31. Top risks: Rust team buy-in (cultural), Hermes upstream
churn (technical), 邓Sir bandwidth (organizational), Python GIL contention
(performance), and cross-platform sandbox (complexity). Each risk has a
specific mitigation and an escalation trigger.

## Risk Register

### R1: Rust Team Buy-In

| Field | Value |
|-------|-------|
| **Severity** | 🔴 High |
| **Likelihood** | 🟡 Medium |
| **Owner** | Nolan |
| **Status** | 🟡 Monitored |

**Description**: Yifan flagged: "团队大部分人习惯 Python，第一反应会抗拒."
Pushing Rust too hard too fast could alienate the team and slow adoption.

**Mitigation**:
1. Keep Python for agent core — Rust only for runtime, sandbox, and CLI
2. Nolan owns 100% of Rust code initially; team contributes Python as before
3. Write Rust in an accessible style: avoid advanced lifetimes, prefer clone
   over complex borrows in early iterations
4. Ship a working Python-only MVP first, then incrementally replace components
   with Rust — don't start with a blank Rust repo

**Escalation trigger**: If any team member expresses vocal resistance to Rust,
pause Rust expansion and focus on Python EverCore integration. Escalate to
Yifan for team alignment discussion.

### R2: Hermes Upstream Churn

| Field | Value |
|-------|-------|
| **Severity** | 🔴 High |
| **Likelihood** | 🟢 High |
| **Owner** | Nolan |
| **Status** | 🟢 Mitigated |

**Description**: Hermes ships rapidly (v0.13 released May 2026, 13 releases
since v0.2). Aggressive upstream churn could make the fork stale or cause
painful rebase conflicts.

**Mitigation**:
1. Fork at a stable SHA (not a branch). Pin to a known-good commit.
2. Rebase onto upstream quarterly, not weekly. Accept being 1-2 versions
   behind as the price of stability.
3. Upstream PRs (Nolan's existing contributor status) keep the fork's
   changes aligned with upstream's direction
4. Hermes is MIT — no legal pressure to stay current

**Escalation trigger**: If a critical security fix lands in Hermes upstream
and the fork is >2 versions behind, prioritize a rebase sprint.

### R3: 邓Sir Bandwidth

| Field | Value |
|-------|-------|
| **Severity** | 🟡 Medium |
| **Likelihood** | 🟢 High |
| **Owner** | Nolan |
| **Status** | 🟢 Mitigated |

**Description**: 邓Sir "all in 自己的 Agent 品牌上线，可能无暇顾及多."
The May Agent's architecture reviewer may have limited availability.

**Mitigation**:
1. Nolan owns architecture decisions; 邓Sir reviews, doesn't drive
2. Write self-contained architecture docs (this document set) so review
   is fast — 邓Sir can read in 30 min and give thumbs up/down
3. Weekly async sync: one Markdown doc, not a meeting
4. Fallback: Yifan can approve architecture if 邓Sir is unavailable

**Escalation trigger**: If 邓Sir is unavailable for >2 weeks and a critical
architecture decision is blocked, escalate to Yifan for decision authority.

### R4: Python GIL Contention

| Field | Value |
|-------|-------|
| **Severity** | 🟡 Medium |
| **Likelihood** | 🟡 Medium |
| **Owner** | Nolan |
| **Status** | 🟢 Mitigated |

**Description**: The Python agent core runs under the GIL. If the Rust
runtime calls into Python from multiple tokio tasks, GIL contention could
bottleneck throughput.

**Mitigation**:
1. **Subprocess-per-session model**: each agent session is a separate Python
   subprocess (not thread). No GIL sharing across sessions.
2. **PyO3 `allow_threads`**: release the GIL during Rust-native operations
   (HTTP calls, sandbox execution, file I/O)
3. **Bounded concurrency**: max 10 concurrent Python subprocesses. Beyond
   that, queue in Rust.
4. **If GIL becomes the bottleneck**: move tool dispatch to Rust. The
   `ToolDispatcher::resolve()` → `ToolBackend::Native` path already supports
   this without changing the agent core.

**Escalation trigger**: If single-session latency exceeds 500ms p95 due to
GIL contention, prioritize moving one hot tool to Rust native dispatch.

### R5: Cross-Platform Sandbox

| Field | Value |
|-------|-------|
| **Severity** | 🟡 Medium |
| **Likelihood** | 🟢 High |
| **Owner** | Nolan |
| **Status** | 🟡 Monitored |

**Description**: seccomp-bpf only works on Linux. macOS needs Seatbelt
(`sandbox_init_with_parameters`). Windows has no kernel sandbox. Cross-platform
sandbox is a multi-month effort — not realistic for May 31.

**Mitigation**:
1. **May 31**: Linux seccomp + macOS Docker fallback. No Windows.
2. **macOS native sandbox**: use `sandbox_init` from libsystem (documented in
   `CLAUDE_DESKTOP_SANDBOX_SOURCE_TRUTH.md`). Lower priority than seccomp.
3. **Docker fallback**: reuses Hermes's existing Docker environment code.
   Always available, just slower.
4. **Accept the gap**: Evil Agent Bench only tests Linux initially.

**Escalation trigger**: If a customer demands Windows support, escalate to
Yifan for prioritization — Windows sandbox is not in the May 31 scope.

### R6: May 31 Deadline

| Field | Value |
|-------|-------|
| **Severity** | 🔴 High |
| **Likelihood** | 🟢 High |
| **Owner** | Yifan + Nolan |
| **Status** | 🟢 Mitigated |

**Description**: 18 days from today. The full vision (Rust runtime + seccomp
sandbox + WeCom + Feishu + 2 benchmarks + desktop shell) is not realistic.
Scope must be cut.

**Mitigation**:
1. **Scope to MVP**: single binary + 1 platform (WeCom) + memory works +
   1 benchmark result. Everything else is stretch.
2. **Prioritize ruthlessly**: if it's not in the success criteria table
   (00-vision.md), it's not required for May 31.
3. **Ship incrementally**: binary first, platform second, benchmarks third.
   Each is independently demonstrable.
4. **Communicate early**: if May 31 is at risk by May 20, tell Yifan
   immediately. Don't hero-code to the deadline silently.

**Escalation trigger**: If by May 20 the binary doesn't build end-to-end,
escalate to Yifan for scope renegotiation.

### R7: Hermes Community Reaction

| Field | Value |
|-------|-------|
| **Severity** | 🟢 Low |
| **Likelihood** | 🟢 Low |
| **Owner** | Nolan |
| **Status** | 🟢 Monitored |

**Description**: The Hermes community could perceive the fork as competitive
rather than collaborative, damaging Nolan's contributor standing.

**Mitigation**:
1. **"Respectful fork" pattern**: continue upstream PRs for non-differentiating
   improvements (bug fixes, tool improvements, model provider updates)
2. **Keep Evercore integration fork-only initially** — don't push memory
   abstraction changes upstream until proven and stable
3. **Nolan's 2 merged PRs establish good faith** — the community knows him
   as a contributor, not a competitor
4. **MIT license means no legal risk** — the fork is perfectly legitimate

**Escalation trigger**: If a Hermes maintainer expresses concern publicly,
Nolan should engage directly and explain the fork-as-lab model.

## Risk Heatmap

```
Likelihood
  High │  R2 ●     R3 ●     R5 ●     R6 ●
       │
Medium │  R1 ●     R4 ●
       │
   Low │  R7 ●
       │
       └──────────────────────────────────
         Low      Medium     High
                   Severity
```

## Weekly Review Cadence

| Week | Date | Focus | Owner |
|------|------|-------|-------|
| W1 | May 13-17 | Architecture docs + Rust skeleton | Nolan |
| W2 | May 18-24 | Python-Rust FFI + Evercore integration | Nolan + 七仔 |
| W3 | May 25-31 | Platform adapter + benchmarks + ship | All |

Each Friday: review risk register, update statuses, escalate if needed.

## References

- 00-vision.md — success criteria and risk log (top 5)
- `CLAUDE_DESKTOP_SANDBOX_SOURCE_TRUTH.md` — macOS sandbox forensics
- `~/.claude/projects/.../memory/project_evermind_2026_05.md` — team context
