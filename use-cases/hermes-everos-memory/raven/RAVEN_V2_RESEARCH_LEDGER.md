# Raven v2 Research Ledger

## North Star

Raven v2 is the next-generation Agents OS console: a native-feeling terminal
operating surface where CCB/CCR/Evensong runtime lineage, EverOS memory, Hermes
skills, and MUW/Superconductor orchestration become one auditable loop.

The goal is not a nicer chat UI. The goal is an operator shell where every
agent action can resolve to memory, state, packet, gate, receipt, or review.

## Parallel Contract

Raven v1 is the build lane. Raven v2 is the research lane.

Research may run at full speed, but it must not block or rewrite v1 unless it
produces a concrete, reviewed implementation packet. V2 findings flow into v1
only through bounded decisions:

- keep;
- revise;
- defer;
- reject;
- open implementation issue.

## Current Truth

- Local EverOS/Hermes/SkillHub/Riven packet is `PASS`.
- Remote EverCore deploy remains `FLAG/BLOCK`.
- `DAS-2666` is the canonical remote deploy gate.
- `DAS-2669` is the Windburn NixOS Codex auth blocker.
- `DAS-2670` is the current control-room dispatch.
- `DAS-2675` tracks Pi/OpenCode Multica runtime-adapter repair.
- Existing Raven v1 build work is dirty in the worktree; do not overwrite it.

## Source Families

### Internal Lineage

- CCB / CCR / Evensong: hackable Claude-Code-like runtime DNA, public evidence
  harness, retrieval benchmarks, Research Vault MCP, and operator handoff
  surface.
- EverOS / EverCore: memory operating layer and multi-tenant memory substrate.
- Hermes: skills, persistent goals, cron/no-agent jobs, gateway, terminal
  backends, and memory/profile model.
- MUW / Superconductor: orchestration, issue gates, runtime control plane,
  review lanes, and bounded fanout.

### External Reference Families

- `yetone/native-feel-skill`
  (`https://github.com/yetone/native-feel-skill`): native-feel discipline,
  decision tree, typed IPC, WebView survival, and ship audit.
- `superagent-ai/grok-cli` (`https://github.com/superagent-ai/grok-cli`):
  OpenTUI energy, remote control, sub-agent UX, and interactive/headless split.
- `openai/codex` (`https://github.com/openai/codex`): smooth terminal REPL
  baseline, local agent flow, release and packaging discipline.
- `claude-code-best/claude-code`
  (`https://github.com/claude-code-best/claude-code`): CCB engineering mine
  for IPC, ACP, remote control, observability, and runtime hacking.
- `NousResearch/hermes-agent` (`https://github.com/NousResearch/hermes-agent`):
  skills, memory, gateway, cron, backend, and portable agent substrate.
- Anthropic Claude Code / Agent SDK / multi-agent research: subagents with
  isolated context, parallel research orchestration, sandbox boundaries, and
  agent harness reuse beyond coding.

## Research Lanes

### Lane 1: Native-Feel TUI/REPL

Question: what makes Raven feel like a native terminal OS surface rather than a
webby text box?

Research targets:

- latency budget;
- keyboard grammar;
- command palette;
- focus and pane stability;
- interrupt/resume semantics;
- scrollback and transcript model;
- hotkey/muscle-memory identity;
- shell/TUI/headless mode split.

Output:

- interaction contract;
- v2 command grammar;
- native-feel audit adapted for terminal agents.

### Lane 2: Runtime DNA Alignment

Question: how do CCB/CCR/Evensong concepts flow into Raven without turning
Raven into a fork dump?

Research targets:

- CLI loop;
- REPL state machine;
- tool approval model;
- pipe/ACP/control-plane concepts;
- telemetry and receipts;
- public handoff/evidence dashboard;
- retrieval benchmark receipts.

Output:

- lineage map;
- implementation boundaries;
- what Raven owns vs what Evensong owns.

### Lane 3: Memory And Skill Substrate

Question: how should Raven make memory, skills, and goals first-class without
becoming a noisy memory browser?

Research targets:

- EverOS memory search/store/status;
- Hermes skills and profiles;
- persistent goals;
- cron/no-agent monitoring receipts;
- provenance fields;
- memory hit explanations.

Output:

- memory pane contract;
- skill registry contract;
- goal/gate model.

### Lane 4: Multi-Agent Orchestration

Question: what is the operator model when many agents are building, reviewing,
and researching at once?

Research targets:

- MUW issue states;
- Superconductor runtimes;
- bounded fanout;
- subagent context isolation;
- task delegation and review packets;
- red-gate routing.

Output:

- control-room state model;
- dispatch grammar;
- review lane protocol.

### Lane 5: Evaluation And Safety

Question: how do we know Raven is making the system more legible rather than
only faster?

Research targets:

- audit trails;
- failure records;
- public-safety scan;
- secret/host/IP redaction;
- benchmark receipt ingestion;
- user-visible truth-state transitions;
- sandbox/permission boundaries.

Output:

- Raven v2 success metrics;
- red-gate invariants;
- public-safe artifact checklist.

## Non-Negotiables

- Do not create a new major repo.
- Do not copy code across incompatible licenses.
- Do not push, publish, deploy, or close upstream issues without explicit
  operator approval.
- Do not expose secrets, private hosts/IPs, credential paths, signed URLs,
  private env values, or local-machine operational details in public artifacts.
- Do not turn research into a pile of summaries. Every research lane must end
  with a decision packet.

## Required Research Packet Shape

```text
RAVEN_V2_RESEARCH_PACKET
LANE:
QUESTION:
SOURCES:
FINDINGS:
DECISIONS:
V1_IMPACT:
RISKS:
NEXT:
VERDICT: PASS | FLAG | BLOCK
```

## First Synthesis Target

Produce `RAVEN_V2_ARCHITECTURE_PACKET.md` only after at least three lanes return
evidence-backed packets. The architecture packet should decide:

- v2 runtime stack;
- TUI/REPL interaction model;
- memory/skill/gate data model;
- what ships in Raven vs remains in Evensong/Hermes/MUW;
- the first v2 implementation slice.
