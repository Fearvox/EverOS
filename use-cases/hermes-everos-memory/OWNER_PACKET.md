# Hermes EverOS Memory Owner Packet

## Verdict

PASS for the local Raven, EverMe SkillHub, and Hermes/EverOS dogfood packet.

FLAG remains for remote NixOS deployment. The deploy packet is ready for review,
but EverCore is not yet proven active on the remote loopback service.

## What Shipped

- Hermes `everos` memory-provider shim with search, store, health, flush,
  prefetch, sync, and auto-flush behavior.
- Raven run packet contract, command contract, renderer, and gate verifier.
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
cd use-cases/hermes-everos-memory && just dogfood-smoke provider-only
cd use-cases/hermes-everos-memory && just skillhub-api-smoke
cd use-cases/hermes-everos-memory && just skillhub-import-sample
cd use-cases/hermes-everos-memory && just skillhub-views skillhub/fixtures/evoagentbench-musician-life-event.json
cd use-cases/hermes-everos-memory && just raven-sample
cd use-cases/hermes-everos-memory && just raven-render
cd use-cases/hermes-everos-memory && just raven-verify
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

## Guardrails Preserved

- No new major repo.
- No push or external publish.
- No private host/IP/token/credential path in public artifacts.
- No final EverMe UI claim before product/design-system constraints.
- Red remote deploy gate remains red.

## Next Action

Stage the EverCore NixOS module into the Windburn workhorse lane with the
private env file already present on the host, then run `nixos-rebuild test`.
