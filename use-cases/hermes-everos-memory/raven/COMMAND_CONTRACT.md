# Raven Command Contract v0

Raven is the operator surface for memory-backed agent work. It is not a
dashboard first and not a marketing page. It is a command contract that turns a
goal, memory substrate, lanes, gates, and evidence into an owner-readable run
packet.

Naming note: Riven is the product/concept name for this operator surface. Raven
is the current repo-local CLI and packet namespace. Keep the Raven namespace
until a migration plan preserves existing verifiers and SkillHub install
targets.

## Shape

Raven v0 ships as a thin CLI/TUI contract over local files and Hermes/EverOS
memory.

It owns:

- run packet validation;
- memory recall before work starts;
- lane and mutation-policy visibility;
- gate execution and verdict calculation;
- owner packet export.

It does not own:

- final public storytelling;
- broad repo orchestration outside the packet;
- upstream mutation without explicit approval;
- secrets, host details, or private machine topology.

## Commands

| Command | Input | Output | Gate |
| --- | --- | --- | --- |
| `raven init` | repo root | `.raven/packet.json` seed | no secrets in seed |
| `raven memory search <query>` | query text | bounded memory refs | EverOS provider available |
| `raven run <packet>` | run packet | updated packet + iteration log | mutation policy honored |
| `raven lane list` | run packet | lane table | owners/scopes visible |
| `raven gate verify` | run packet | PASS/FLAG/BLOCK table | blocking gates cannot be skipped |
| `raven export` | run packet | owner packet markdown | public-safety scan clean |

## Run State

Raven treats the run packet as the source of operational state:

```ts
type RavenRunState =
  | "captured"
  | "dispatching"
  | "executing"
  | "reviewing"
  | "done"
  | "blocked";
```

State transitions:

1. `captured` after goal and packet exist.
2. `dispatching` after lanes and mutation policies are assigned.
3. `executing` while local code/docs/tests are changing.
4. `reviewing` after work stops and evidence is being checked.
5. `done` only when every blocking gate is `pass`.
6. `blocked` when a blocking gate needs human or external action.

## Memory Behavior

Before execution Raven asks memory for:

- prior owner decisions;
- known red gates;
- current memory-provider health;
- relevant run packets or closeouts.

After execution Raven writes:

- changed files;
- verification commands and verdicts;
- unresolved risks;
- one next action.

The memory loop is proof-backed only when a unique marker can be stored and
searched back through EverOS.

## Gate Semantics

`PASS` means the specific requirement was tested at the scope it claims.

`FLAG` means the path is usable but not fully proven, stale, or missing an
external observation.

`BLOCK` means a required gate failed or needs human approval before continuing.

Raven must not upgrade `FLAG` to `PASS` because nearby tests passed.

## First Artifact

The first Raven artifact is the owner packet rendered from
`raven/fixtures/doomsday-run.json`.

Current proof command:

```bash
node bin/raven-run.mjs render raven/fixtures/doomsday-run.json
```

Current local gate verifier:

```bash
node bin/raven-run.mjs verify raven/fixtures/doomsday-run.json
```

This is the repo-local equivalent of:

```bash
raven gate verify --packet raven/fixtures/doomsday-run.json
```

It exits non-zero for `FLAG` or `BLOCK`.
