# Riven Concept Packet v0

## Verdict

PASS for the Riven concept artifact.

Riven is the operator-facing concept name for the memory-backed execution lane.
The current repo-local implementation remains under the `raven` directory and
`raven-run` command for v0 compatibility.

## Naming Boundary

| Name | Role | Status |
| --- | --- | --- |
| Riven | product/concept name for the operator surface | concept artifact |
| Raven | current CLI, packet schema, fixtures, and SkillHub install target | implemented v0 surface |

Do not rename files, commands, or install targets until there is a migration
plan. The present contract treats Raven as the working codename for Riven v0.

## Product Thesis

Riven is not a chat transcript viewer and not a generic dashboard. It is a
memory-backed operator surface for focused agent work:

- capture one goal;
- recall prior decisions and red gates before execution;
- split work into bounded lanes;
- preserve mutation policy per lane;
- verify blocking gates with commands or explicit evidence;
- export an owner-readable packet.

## First Run Shape

The first Riven run is the Doomsday EverOS lane:

1. Riven concept exploration through the Raven command contract.
2. EverMe SkillHub MVP packet and read-only mock API.
3. Hermes/EverOS provider dogfood with store, search, recall, and real Hermes
   profile verification.

## Interface Wedge

The minimal useful UI is command-grade:

```text
riven capture <goal>
riven memory search <query>
riven lane list
riven gate verify
riven export
```

For v0, these map to the existing `raven-run` validator/renderer and the
`raven/fixtures/doomsday-run.json` packet.

## Guardrails

- Do not expose raw call transcript content.
- Do not publish private paths, host/IP values, screenshots, tokens, or
  credential paths.
- Do not treat remote NixOS deploy as complete until the deploy smoke passes on
  the remote loopback service.
- Do not widen Riven into a new major repo before the packet contract earns it.

## Current Evidence

- `raven/COMMAND_CONTRACT.md` defines the v0 command/state/gate contract.
- `raven/fixtures/doomsday-run.json` records the first focused run.
- `bin/raven-run.mjs verify raven/fixtures/doomsday-run.json` computes the
  packet verdict and fails non-zero for open blocking gates.
- `OWNER_PACKET.md` separates local packet PASS from remote deploy FLAG.
