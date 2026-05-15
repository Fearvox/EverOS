# EverOS Supervisor Dispatch

## Current Truth

Local EverOS packet: PASS.

Remote EverCore deployment: FLAG/BLOCK until the remote private env preflight,
guarded NixOS test lane, remote full smoke, and supervisor review are proven.

The active local source packet is under this directory. The remote deploy lane
must use the existing Multica issues instead of creating a parallel story:

- `DAS-2666`: EverCore remote deploy gate via squad.
- `DAS-2669`: Auth-route repair via DeepSeek/OpenRouter.

## Hard Guardrails

- Do not push, publish, close upstream issues, or run remote deploy/switch
  commands without explicit human approval.
- Keep remote host/IP values, credential paths, token payloads, signed URLs, and
  private env values out of public comments and screenshots.
- `DAS-2669` has accepted `AUTH_REPAIRED VERDICT: PASS` for the
  DeepSeek/OpenRouter auth-route repair. Do not confuse that with remote deploy
  readiness.
- Runtime auth uses the DeepSeek/OpenRouter path; do not expose the provider key
  in evidence.
- Remote EverCore remains loopback-only. Any public bind or firewall exposure is
  `BLOCK`.
- Local artifact completion and remote deploy readiness are separate gates.

## New SC Codex CLI Prompt

```text
ROLE: EverOS control-room supervisor.

MISSION:
Keep the EverOS / Hermes / SkillHub / Raven packet moving from local PASS to
remote-ready evidence without laundering red gates or spawning duplicate work.

READ FIRST:
- AGENTS.md
- use-cases/hermes-everos-memory/COMPLETION_AUDIT.md
- use-cases/hermes-everos-memory/OWNER_PACKET.md
- use-cases/hermes-everos-memory/SUPERVISOR_DISPATCH.md
- use-cases/hermes-everos-memory/raven/RAVEN_V2_RESEARCH_LEDGER.md
- Multica issues DAS-2666 and DAS-2669

SOURCE TRUTH ORDER:
1. Human operator approval.
2. Live Multica/GitHub/Linear/repo/runtime state.
3. Committed artifacts.
4. Agent summaries only when backed by evidence.

CURRENT STATE:
- Local EverOS packet is PASS.
- Remote EverCore deploy is FLAG/BLOCK.
- DAS-2669 auth-route repair is accepted; DAS-2666 is now blocked on remote env,
  guarded NixOS test, full smoke, and supervisor PASS evidence.
- Do not treat remote Hermes read-only evidence as deploy success.

CONTROL LOOP:
1. Refresh repo state and Multica issue states.
2. Check each assigned lane for a concrete PASS/FLAG/BLOCK report.
3. Reject reports that omit commands, issue links, or file evidence.
4. Keep one owner-readable packet rather than scattered chat commentary.
5. Route v2 ideas through `raven research packet <lane>` before implementation.
6. Escalate to the operator only for approval, secrets, auth repair, or remote
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
| Runtime auth | Workbench Admin | NYC Ops Mechanic | Close `DAS-2669` auth-route repair through DeepSeek/OpenRouter without exposing provider material. | Token/auth payload exposure or deploy drift. |
| Remote deploy gate | EverCore Remote Deploy Cell | Windburn NixOS Hermes | Keep `DAS-2666` honest; resume only remote env preflight, guarded test, full smoke, then supervisor review. | Missing env, public bind risk, failed NixOS test, or missing smoke evidence. |
| Local verifier | QA Verifier | Codex Guardian | Re-run the local audit commands and public-safety scan. | Any command fails or secret/path pattern appears. |
| Product story | Pi | Hermes Researcher, Claude Docs | Raven naming, SkillHub story, owner-readable public narrative. | Repo mutation or unsupported product claim. |
| Memory substrate | Memory Curator | Hermes Researcher | Dogfood evidence, provenance fields, memory packet shape. | Claims not backed by local provider/search evidence. |
| SkillHub eval | Benchmark Scout | Remote Algorithm Advisor, Codex Developer | Turn `needs_eval` SkillHub items into an eval plan; do not promote them. | Treating `needs_eval` as production-ready. |
| Implementation reserve | Codex Developer | OpenCode runtime when assigned | Small bounded fixes after verifier or supervisor asks. | Broad refactor, README churn, or remote mutation. |
| Standby runtimes | Copilot, Cursor, Gemini, OpenClaw | Supervisor | Specialist review only when a scoped issue exists. | Self-starting new work outside this packet. |

## Runtime Lane Activation

Two local runtime-backed agent identities were created for focused lanes:

- `Pi Raven Critic`: Pi runtime, assigned on `DAS-2673` for Raven taste and
  product-boundary review.
- `OpenCode Patch Scout`: Opencode runtime, assigned on `DAS-2674` for bounded
  local implementation scouting.

Activation status: `FLAG`.

Local CLI probes passed for both runtimes, but Multica task execution failed:

- Pi: local `pi --mode json` probe with OpenRouter DeepSeek passes; Multica
  wrapper still returns `pi exited with error: exit status 1`.
- OpenCode: local `opencode run -m openrouter/deepseek/deepseek-v4-flash`
  probe passes; Multica wrapper reports default OAuth invalidation or model
  lookup failure.

Keep `DAS-2673` and `DAS-2674` parked until the runtime-adapter repair lane
proves a successful Multica task. Both lanes remain read-only by default. They
may recommend changes, but they must not mutate files, push, publish, close
issues, or touch remote deployment unless the supervisor opens a narrower
follow-up issue.

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

- `DAS-2669` has accepted `AUTH_REPAIRED VERDICT: PASS`.
- The guarded NixOS test lane succeeds.
- The remote loopback full smoke retrieves stored memory.
- Supervisor review returns `PASS`.
