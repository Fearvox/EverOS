# Hermes EverOS Memory Owner Packet

## Verdict

PASS for the local Raven, EverMe SkillHub, and Hermes/EverOS dogfood
packet.

FLAG remains for remote NixOS deployment. The deploy packet is ready for review,
but EverCore is not yet proven active on the remote loopback service.

## What Shipped

- Hermes `everos` memory-provider shim with search, store, health, flush,
  prefetch, sync, and auto-flush behavior.
- Raven concept packet and naming contract, implemented through the Raven
  command namespace.
- Raven run packet contract, command contract, renderer, and gate verifier.
- Raven v1 local console: Rust CLI, REPL, and ratatui TUI entrypoints that
  expose typed status, packet, gates, agents, memory, runs, receipts, native
  audit, and local verification without mutating remote state.
- Raven Hermes chat bridge: `raven chat send`, bare-text/`/chat` REPL turns,
  and the TUI Hermes panel share one sanitized adapter; TUI execution runs in
  the background so redraw and key handling remain live.
- Raven v2 research harness: `raven research lanes`, `raven research packet
  <lane>`, and `raven research synthesize` keep v2 work as live-gate-calibrated
  decision packets instead of freeform research prose.
- EverMe SkillHub packet schema, MVP view plan, renderer, read-only mock API,
  API-backed views/install-packet routes, and one real EvoAgentBench `SKILL.md`
  import fixture.
- NixOS/workhorse deploy packet, compose file, module draft, env example, and
  remote smoke script.
- Local mock OpenAI-compatible server for model-free EverCore dogfood.

## Verification

Current local PASS verification set:

```bash
bash -n use-cases/hermes-everos-memory/scripts/*.sh use-cases/hermes-everos-memory/deploy/nixos/scripts/*.sh
cd use-cases/hermes-everos-memory && for f in bin/*.mjs; do node --check "$f"; done
git diff --check -- use-cases/hermes-everos-memory
cd use-cases/hermes-everos-memory && just provider-load
cd use-cases/hermes-everos-memory && just deepseek-auth-preflight
cd use-cases/hermes-everos-memory && just dogfood-smoke provider-only
cd use-cases/hermes-everos-memory && just skillhub-api-smoke
cd use-cases/hermes-everos-memory && just skillhub-import-sample
cd use-cases/hermes-everos-memory && just skillhub-views skillhub/fixtures/evoagentbench-musician-life-event.json
cd use-cases/hermes-everos-memory && just raven-sample
cd use-cases/hermes-everos-memory && just raven-render
cd use-cases/hermes-everos-memory && just raven-verify
cd use-cases/hermes-everos-memory && just raven-console-check
cd use-cases/hermes-everos-memory && just raven-status
cd use-cases/hermes-everos-memory && bin/raven status --json
cd use-cases/hermes-everos-memory && just raven-research-lanes
cd use-cases/hermes-everos-memory && just raven-research-packet-smoke
cd use-cases/hermes-everos-memory && just raven-research-synthesis
cd use-cases/hermes-everos-memory && RAVEN_HERMES_BIN=/bin/echo bin/raven chat send raven chat smoke
cd use-cases/hermes-everos-memory && RAVEN_HERMES_BIN=/bin/echo bin/raven --json chat send "check raven chat redaction fixture"
cd use-cases/hermes-everos-memory && just raven-run-verify
cd use-cases/hermes-everos-memory && bin/raven run verify --receipt -
cd use-cases/hermes-everos-memory && just raven-repl-smoke
cd use-cases/hermes-everos-memory && just raven-tui-smoke
cd use-cases/hermes-everos-memory && just raven-native-audit
cd use-cases/hermes-everos-memory && just raven-runs
cd use-cases/hermes-everos-memory && just mock-openai-check
cd use-cases/hermes-everos-memory && EVEROS_USER_ID="verify-raven-$(date +%s)" EVEROS_SEARCH_METHOD=hybrid EVEROS_MEMORY_TYPES=episodic_memory,raw_message,profile,agent_memory just dogfood-smoke full
cd use-cases/hermes-everos-memory && MARKER="RAVEN_DOGFOOD_VERIFY_$(date +%s)" && hermes -z "Use the EverOS memory tool to store exactly this public verification marker: ${MARKER}." && node bin/everos-memory.mjs search "$MARKER"
```

Hermes profile verification:

```bash
hermes memory status
```

Expected status:

```text
Provider: everos
Plugin: installed
Status: available
```

Remote deploy verification remains separate:

```bash
cd use-cases/hermes-everos-memory && just remote-smoke full
```

Treat that command as `FLAG` until the NixOS module is applied and EverCore is
running on the remote loopback service.

## Remote Disposition

Read-only workhorse probe:

- NixOS host is reachable.
- System state is running.
- Failed systemd unit count was zero during the dry-run probe.
- EverCore service/timer are inactive.
- Remote loopback health at the EverCore API port is unavailable.

Remote deploy remains `FLAG` until the EverCore module is staged into the
workhorse configuration, `nixos-rebuild test` passes, and the remote
`--mode full` smoke passes on-host.

Live MUW calibration on 2026-05-15: `DAS-2669` is unblocked for the
DeepSeek/OpenRouter auth-route repair with `AUTH_REPAIRED VERDICT: PASS`.
`DAS-2666` remains `BLOCK` because remote private env preflight, guarded NixOS
test, remote loopback full smoke, and supervisor `PASS` are still missing.

## Guardrails Preserved

- No new major repo.
- No push or external publish.
- No private host/IP/token/credential path in public artifacts.
- No final EverMe UI claim before product/design-system constraints.
- Red remote deploy gate remains red.
- Raven console keeps remote deploy actions read-only/visible; it does not run
  `nixos-rebuild`, `switch`, publish, push, or close issues.
- `DAS-2669` auth-route repair is accepted through the DeepSeek/OpenRouter
  path; it does not by itself green remote deploy readiness.
- Remote LLM auth is pinned to DeepSeek through OpenRouter, and the preflight
  checks that shape without printing provider keys.
- `DAS-2666` remains `BLOCK` until auth repair, guarded NixOS test, remote
  loopback full smoke, and supervisor `PASS` are all present.
- `DAS-2675` can repair Pi/OpenCode adapter lanes but cannot green the remote
  deploy verdict.

## Next Action

Resume `DAS-2666` from the DeepSeek/OpenRouter auth path: run the remote private
env preflight with `--require-key`, then the guarded `nixos-rebuild test`, then
the remote loopback full-smoke sequence, and only then request supervisor
review.
