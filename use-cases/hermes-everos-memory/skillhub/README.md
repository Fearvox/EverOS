# SkillHub Packet Contract

`v0` packet contract for EverMe SkillHub dogfooding.

This is not the final EverMe UI. It is the portable skill object that Raven,
Hermes, and EverCore can exchange before the cloud/product API is finalized.

## Files

| File | Purpose |
| --- | --- |
| `schema.json` | Public-safe JSON Schema for one SkillHub packet |
| `fixtures/raven-skillhub-sample.json` | Sample packet used by smoke tests |
| `fixtures/evoagentbench-musician-life-event.json` | Real `SKILL.md` import fixture |
| `../bin/skillhub-packet.mjs` | Local exporter/validator helper |
| `../bin/skillhub-mock-api.mjs` | Read-only HTTP adapter for Raven/Hermes dogfood |
| `MVP_IMPLEMENTATION_PLAN.md` | Public-safe MVP view/API/implementation contract |

## Contract

A SkillHub packet represents one skill plus enough provenance for an agent
runtime to decide whether it can install or use it.

Required fields:

- `id`
- `name`
- `summary`
- `visibility`
- `status`
- `version`
- `source`
- `domains`
- `install_targets`
- `evidence_refs`
- `body_markdown`

## Local Commands

Validate the sample:

```bash
node ../bin/skillhub-packet.mjs validate fixtures/raven-skillhub-sample.json
node ../bin/skillhub-packet.mjs render fixtures/raven-skillhub-sample.json
node ../bin/skillhub-packet.mjs views fixtures/raven-skillhub-sample.json
node ../bin/skillhub-mock-api.mjs --check
```

Export an existing `SKILL.md` file:

```bash
node ../bin/skillhub-packet.mjs from-skill \
  ../../benchmarks/EvoAgentBench/src/skill_evolution/evermemos/skills_sample/MUSICIAN/musician_life_event/SKILL.md
```

The imported fixture is checked with:

```bash
node ../bin/skillhub-packet.mjs validate fixtures/evoagentbench-musician-life-event.json
node ../bin/skillhub-packet.mjs views fixtures/evoagentbench-musician-life-event.json
```

## Dogfood Path

1. EverCore extracts or stores a skill.
2. SkillHub exports it as this packet.
3. Hermes/Raven imports the packet as an installable skill or memory-backed
   runtime hint.
4. Evaluation evidence updates `evidence_refs`, `eval_score`, and `status`.

## Mock API

The mock API is a local read-only adapter over packet JSON files. It exists so
Raven, Hermes, or EverCore clients can prove their integration contract before
the final EverMe backend exists.

```bash
node ../bin/skillhub-mock-api.mjs --port 18765
```

Smoke the routes:

```bash
../scripts/skillhub-api-smoke.sh
```

Routes:

- `GET /health`
- `GET /skills`
- `GET /skills?target=hermes`
- `GET /skills/:id`
- `GET /skills/:id/render`
- `POST /skills/validate`
