# EverOS Supervisor Dispatch

## Current Truth

Local EverOS packet: PASS.

Remote EverCore deployment: FLAG/BLOCK until the remote auth repair and guarded
NixOS test lane are proven.

The active local source packet is under this directory. The remote deploy lane
must use the existing Multica issues instead of creating a parallel story:

- `DAS-2666`: EverCore remote deploy gate via squad.
- `DAS-2669`: Repair Windburn NixOS Codex runtime auth.

## Hard Guardrails

- Do not push, publish, close upstream issues, or run remote deploy/switch
  commands without explicit human approval.
- Keep remote host/IP values, credential paths, token payloads, signed URLs, and
  private env values out of public comments and screenshots.
- Do not retry Windburn NixOS Codex deploy work until `DAS-2669` posts
  `AUTH_REPAIRED` from a successful read-only proof task.
- Remote EverCore remains loopback-only. Any public bind or firewall exposure is
  `BLOCK`.
- Local artifact completion and remote deploy readiness are separate gates.

## New SC Codex CLI Prompt

```text
ROLE: EverOS control-room supervisor.

MISSION:
Keep the EverOS / Hermes / SkillHub / Riven packet moving from local PASS to
remote-ready evidence without laundering red gates or spawning duplicate work.

READ FIRST:
- AGENTS.md
- use-cases/hermes-everos-memory/COMPLETION_AUDIT.md
- use-cases/hermes-everos-memory/OWNER_PACKET.md
- use-cases/hermes-everos-memory/SUPERVISOR_DISPATCH.md
- Multica issues DAS-2666 and DAS-2669

SOURCE TRUTH ORDER:
1. Human operator approval.
2. Live Multica/GitHub/Linear/repo/runtime state.
3. Committed artifacts.
4. Agent summaries only when backed by evidence.

CURRENT STATE:
- Local EverOS packet is PASS.
- Remote EverCore deploy is FLAG/BLOCK.
- DAS-2669 auth repair is the blocker before Windburn NixOS Codex can be used.
- Do not treat remote Hermes read-only evidence as deploy success.

CONTROL LOOP:
1. Refresh repo state and Multica issue states.
2. Check each assigned lane for a concrete PASS/FLAG/BLOCK report.
3. Reject reports that omit commands, issue links, or file evidence.
4. Keep one owner-readable packet rather than scattered chat commentary.
5. Escalate to the operator only for approval, secrets, auth repair, or remote
   mutation decisions.

OUTPUT SHAPE:
VERDICT: PASS | FLAG | BLOCK
EVIDENCE:
CHANGES:
RISKS:
NEXT:
```

## Multica Dispatch Map

| Lane | Lead | Support | Scope | Stop Condition |
| --- | --- | --- | --- | --- |
| Control room | Workbench Supervisor | Workbench Synthesizer | Track all lanes and produce one owner packet. | Any lane reports success without evidence. |
| Runtime auth | Workbench Admin | NYC Ops Mechanic | Repair `DAS-2669`; prove Windburn NixOS Codex can do a read-only task. | Token/auth payload exposure or deploy drift. |
| Remote deploy gate | EverCore Remote Deploy Cell | Windburn NixOS Hermes | Keep `DAS-2666` honest; read-only preflight until `AUTH_REPAIRED`. | Missing env, public bind risk, failed NixOS test, or Codex auth still broken. |
| Local verifier | QA Verifier | Codex Guardian | Re-run the local audit commands and public-safety scan. | Any command fails or secret/path pattern appears. |
| Product story | Pi | Hermes Researcher, Claude Docs | Riven naming, SkillHub story, owner-readable public narrative. | Repo mutation or unsupported product claim. |
| Memory substrate | Memory Curator | Hermes Researcher | Dogfood evidence, provenance fields, memory packet shape. | Claims not backed by local provider/search evidence. |
| SkillHub eval | Benchmark Scout | Remote Algorithm Advisor, Codex Developer | Turn `needs_eval` SkillHub items into an eval plan; do not promote them. | Treating `needs_eval` as production-ready. |
| Implementation reserve | Codex Developer | OpenCode runtime when assigned | Small bounded fixes after verifier or supervisor asks. | Broad refactor, README churn, or remote mutation. |
| Standby runtimes | Copilot, Cursor, Gemini, OpenClaw | Supervisor | Specialist review only when a scoped issue exists. | Self-starting new work outside this packet. |

## Required Reports

Each active lane must end with:

```text
VERDICT:
EVIDENCE:
CHANGES:
RISKS:
NEXT:
```

No lane may mark the remote deploy path `PASS` until all of these are true:

- `DAS-2669` has `AUTH_REPAIRED`.
- The guarded NixOS test lane succeeds.
- The remote loopback full smoke retrieves stored memory.
- Supervisor review returns `PASS`.

