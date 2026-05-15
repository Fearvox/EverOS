# Raven Concept Packet v0

## Verdict

PASS for the Raven concept artifact.

Raven is the operator-facing name and the repo-local implementation namespace
for the memory-backed execution lane.

## Naming Contract

| Name | Role | Status |
| --- | --- | --- |
| Raven | operator surface, CLI, packet schema, fixtures, and SkillHub install target | implemented v0 surface |

Use Raven everywhere. Do not introduce a second product/internal name unless a
future migration plan explicitly changes the namespace.

## Product Thesis

Raven is not a chat transcript viewer and not a generic dashboard. It is a
memory-backed operator surface for focused agent work:

- capture one goal;
- recall prior decisions and red gates before execution;
- split work into bounded lanes;
- preserve mutation policy per lane;
- verify blocking gates with commands or explicit evidence;
- export an owner-readable packet.

## First Run Shape

The first Raven run is the Doomsday EverOS lane:

1. Raven concept exploration through the Raven command contract.
2. EverMe SkillHub MVP packet and read-only mock API.
3. Hermes/EverOS provider dogfood with store, search, recall, and real Hermes
   profile verification.

## Interface Wedge

The minimal useful UI is command-grade:

```text
raven capture <goal>
raven memory search <query>
raven lane list
raven gate verify
raven export
```

For v0, these map to the existing `raven-run` validator/renderer and the
`raven/fixtures/doomsday-run.json` packet.

## Guardrails

- Do not expose raw call transcript content.
- Do not publish private paths, host/IP values, screenshots, tokens, or
  credential paths.
- Do not treat remote NixOS deploy as complete until the deploy smoke passes on
  the remote loopback service.
- Do not widen Raven into a new major repo before the packet contract earns it.

## Current Evidence

- `raven/COMMAND_CONTRACT.md` defines the v0 command/state/gate contract.
- `raven/fixtures/doomsday-run.json` records the first focused run.
- `bin/raven-run.mjs verify raven/fixtures/doomsday-run.json` computes the
  packet verdict and fails non-zero for open blocking gates.
- `OWNER_PACKET.md` separates local packet PASS from remote deploy FLAG.
