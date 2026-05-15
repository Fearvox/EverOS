# Doomsday EverOS Completion Audit

## Verdict

PASS for the focused EverOS execution lane.

The requested artifacts are present, public-safe, and verified:

- Riven concept exploration;
- EverMe SkillHub MVP design and implementation plan;
- Hermes/EverOS dogfood memory-provider integration artifacts;
- owner-readable packet and verifiers.

Remote NixOS deployment remains a separate follow-on `FLAG`; it is not counted
as local artifact completion.

## Requirement Matrix

| Requirement | Evidence | Verdict |
| --- | --- | --- |
| Turn the source call into one focused lane | `raven/fixtures/doomsday-run.json` records one run with three bounded lanes and no open blocking gates | PASS |
| Ship Riven concept exploration | `raven/RIVEN_CONCEPT.md` defines thesis, naming boundary, interface wedge, guardrails, and current evidence | PASS |
| Preserve Raven compatibility | `raven/COMMAND_CONTRACT.md`, `raven/schema.json`, and `bin/raven-run.mjs` keep the v0 namespace working | PASS |
| Ship EverMe SkillHub MVP plan | `skillhub/MVP_IMPLEMENTATION_PLAN.md` defines product contract, five MVP views, API contract, data additions, sequence, and gates | PASS |
| Ship SkillHub implementation slice | `skillhub/schema.json`, fixtures, `bin/skillhub-packet.mjs`, and `bin/skillhub-mock-api.mjs` validate/render/import/serve packets | PASS |
| Ship Hermes/EverOS plugin artifacts | `__init__.py`, `plugin.yaml`, `scripts/install-local.sh`, and `bin/everos-memory.mjs` implement and install the provider shim | PASS |
| Prove provider load | `just provider-load` | PASS |
| Prove SkillHub API | `just skillhub-api-smoke` | PASS |
| Prove real SkillHub import | `just skillhub-import-sample` plus `just skillhub-views skillhub/fixtures/evoagentbench-musician-life-event.json` | PASS |
| Prove Raven/Riven packet | `node bin/raven-run.mjs summary raven/fixtures/doomsday-run.json` and `just raven-verify` | PASS |
| Prove full memory loop | `just dogfood-smoke full` with a fresh user id | PASS |
| Prove real Hermes profile path | `hermes -z` storing a unique public marker, then `node bin/everos-memory.mjs search "$MARKER"` | PASS |
| Avoid widening scope | no new major repo; artifacts stay under `use-cases/hermes-everos-memory/` | PASS |
| Avoid private operational details | public-safety scan over owner packet, Riven/Raven docs, run packet, and SkillHub docs returns no matches | PASS |

## Commands

```bash
cd use-cases/hermes-everos-memory
bash -n scripts/*.sh deploy/nixos/scripts/*.sh
for f in bin/*.mjs; do node --check "$f"; done
node bin/raven-run.mjs summary raven/fixtures/doomsday-run.json
just provider-load
just dogfood-smoke provider-only
just skillhub-api-smoke
just skillhub-import-sample
just skillhub-views skillhub/fixtures/evoagentbench-musician-life-event.json
just raven-sample
just raven-render
just raven-verify
just mock-openai-check
EVEROS_USER_ID="verify-riven-$(date +%s)" EVEROS_SEARCH_METHOD=hybrid EVEROS_MEMORY_TYPES=episodic_memory,raw_message,profile,agent_memory just dogfood-smoke full
MARKER="RIVEN_DOGFOOD_VERIFY_$(date +%s)" && hermes -z "Use the EverOS memory tool to store exactly this public verification marker: ${MARKER}." && node bin/everos-memory.mjs search "$MARKER"
```

Repo-root checks:

```bash
git diff --check -- use-cases/hermes-everos-memory
rg -n -i -f <operator-local-public-safety-patterns> use-cases/hermes-everos-memory/OWNER_PACKET.md use-cases/hermes-everos-memory/raven use-cases/hermes-everos-memory/skillhub
```

## Residual Risks

- Remote NixOS deployment remains `FLAG` until the module is applied and the
  remote `--mode full` smoke passes.
- Raven-to-Riven command rename is intentionally deferred; current v0 keeps
  `raven-run` to avoid breaking existing packet and SkillHub contracts.
- SkillHub write routes remain proposed until EverMe backend constraints are
  available.
