# Raven Run Packet Contract

`v0` contract for a Raven run.

Riven is the operator-facing concept name for this surface. Raven is the
repo-local v0 codename and command namespace kept for compatibility.

Raven is not a marketing page here. This directory defines the packet that a
CLI/TUI can validate, render, and later execute against Hermes/EverOS memory.

## Files

| File | Purpose |
| --- | --- |
| `RIVEN_CONCEPT.md` | Public-safe Riven concept and naming boundary |
| `COMMAND_CONTRACT.md` | Public-safe command/state/gate contract for Raven v0 |
| `schema.json` | Public-safe JSON Schema for a Raven run packet |
| `fixtures/doomsday-run.json` | Sample run packet for the current dogfood lane |
| `../bin/raven-run.mjs` | Local validator and owner-packet renderer |

## Commands

```bash
node ../bin/raven-run.mjs validate fixtures/doomsday-run.json
node ../bin/raven-run.mjs render fixtures/doomsday-run.json
node ../bin/raven-run.mjs verify fixtures/doomsday-run.json
node ../bin/raven-run.mjs summary fixtures/doomsday-run.json
```

## Contract

A Raven run packet records:

- the current goal;
- owners and memory providers;
- independent lanes;
- gates with evidence and blocking status;
- artifacts and next actions.

The computed verdict is conservative:

- `BLOCK` if any blocking gate is `block` or any lane is `block`;
- `FLAG` if any blocking gate is `flag` or `not_run`, or any lane is `flag` or
  `active`;
- `PASS` only when blocking gates and lanes are all pass.

`verify` exits non-zero for `FLAG` or `BLOCK`, so scripts can refuse to call a
packet complete when blocking gates remain open.
