# Raven Command Contract v1

Raven is the operator surface for memory-backed agent work. It is not a
dashboard first and not a marketing page. It is a command contract that turns a
goal, memory substrate, lanes, gates, and evidence into an owner-readable run
packet.

Naming note: Raven is the product, internal, CLI, packet, and fixture namespace.
Keep one name across docs and code unless a future migration plan explicitly
changes it.

## Shape

Raven v1 ships as a Rust CLI, Hermes-backed chat command, slash-command REPL,
and ratatui TUI contract over local files, Hermes/EverOS memory, Multica watch
issues, and local verifier receipts.

It owns:

- run packet validation;
- memory health/search before work starts;
- lane and mutation-policy visibility;
- single-agent loop visibility;
- gate visibility and conservative verdict calculation;
- sanitized JSON snapshot and receipt output;
- owner packet export;
- shared Hermes chat adapter across CLI, REPL, and TUI;
- native-feel and public-safety audit.

It does not own:

- final public storytelling;
- broad repo orchestration outside the packet;
- upstream mutation without explicit approval;
- secrets, host details, or private machine topology.

## Commands

| Command | Input | Output | Gate |
| --- | --- | --- | --- |
| `raven status [--json]` | live local files and watch issues | `RavenSnapshot` or compact status | local `PASS` plus remote `BLOCK` renders overall `FLAG` |
| `raven tui` | terminal | ratatui console with status, rail, active panel, evidence drawer, input line | `RAVEN_TUI_ONCE=1` must render deterministic smoke output |
| `raven repl` | slash commands | same handlers as CLI | piped smoke stays deterministic |
| `raven chat send [--cwd <path>] [--json] [--receipt <path\|->] [--save] <prompt>` | bounded prompt text | sanitized `HermesChatTurn` or `RavenReceipt` | Hermes failure is `FLAG`, not UI crash; chat receipts cannot green remote deploy |
| `raven loop [status] [--json]` | packet + gate + run state | `AgenticLoopState` or compact loop contract | loop closure is `FLAG` while remote hard gates or missing receipts remain |
| `raven packet show [--json]` | local packet/docs | packet summary | source docs resolve |
| `raven packet export [--output <path\|->]` | snapshot | sanitized owner packet markdown | public-safety sanitizer clean |
| `raven memory health [--json]` | EverOS bridge | health verdict | provider failure is `FLAG`, not crash |
| `raven memory search <query> [--json]` | query text | bounded memory refs | empty query is `FLAG` |
| `raven agents list [--json]` | Multica watch issues | agent/watch table | unavailable Multica falls back to `FLAG` |
| `raven gates [--json]` | packet + watch evidence | local and remote gate table | hard gates cannot be skipped |
| `raven research lanes [--json]` | `RAVEN_V2_RESEARCH_LEDGER.md` | bounded research lane list | every lane must end in a packet |
| `raven research packet <lane> [--json] [--output <path\|->]` | research ledger + live remote gates | `RavenResearchPacket` | live `DAS-2666/2669` red gates force `FLAG` context |
| `raven research synthesize [--json] [--output <path\|->]` | completed research packets | synthesis readiness report | less than three packets stays `FLAG`; no architecture packet |
| `raven runs list [--json]` | saved receipts or packet gates | run/receipt table | receipts read from gitignored local dir |
| `raven sc [all\|status\|sessions\|providers\|worktree] [--json]` | Superconductor socket via thin CLI | `ScReport` or focused view | unavailable socket or merge-base failure is `FLAG`, never a crash |
| `raven run verify [--receipt <path\|->] [--save]` | local run packet | `RavenReceipt` or human output | local verifier cannot green remote deploy |
| `raven doctor [--json]` | toolchain/files/bridge | dependency report | missing hard local dependency blocks |
| `raven native-audit [--json]` | source + audit doc | UX/safety gate report | hard UX/safety failure blocks `PASS` |

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

## Single Agentic Loop Behavior

Raven keeps a visible single-agent loop above the raw chat transcript:

```text
CAPTURE -> PLAN -> ACT -> OBSERVE -> VERIFY -> RECEIPT
```

The loop is intentionally human-in-the-loop. It captures one bounded objective,
routes one Hermes turn, keeps observations attached to the evidence drawer, then
lets gates and receipts decide closure. This is the missing bridge between
`chat` and `runs`: a prompt can produce useful local evidence, but it cannot
silently mutate gate state.

Loop invariants:

- `raven loop` prints the typed `AgenticLoopState`;
- `/loop` in the REPL maps to the same handler;
- `l` in the TUI opens the loop panel, and `i` from that panel starts one
  Hermes prompt;
- chat turns add live loop breadcrumbs for capture, act, observe, and verify;
- receipts are explicit through `--receipt` or `--save`;
- remote deploy remains red until `DAS-2666` hard evidence passes.

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

## Hermes Chat Behavior

The chat surface is a shared adapter, not a second TUI-only path:

- `raven chat send <prompt>` executes a single Hermes oneshot turn;
- `raven repl` accepts `/chat <prompt>`, `/hermes <prompt>`, and bare text as
  Hermes dialogue;
- `raven tui` exposes a Hermes chat panel with background execution so prompt
  submission does not block redraw, keyboard handling, or gate visibility.

The adapter injects an explicit Raven working directory into the Hermes process
and labels it as `case-root` or `case-root/<relative>` in public output. It also
records the detected Hermes runtime, so Codex app-server turns can be separated
from legacy `auto` turns during review.

Every prompt, response, stderr excerpt, transcript line, receipt excerpt, and
JSON field goes through Raven's public-safety sanitizer before display or save.
`--receipt -` prints a sanitized `RavenReceipt`; `--save` writes one under the
gitignored local runs directory.

## Superconductor Behavior

Raven treats Superconductor as the conductor plane, not as a mutation authority:

- `raven sc status` checks whether the local Superconductor socket responds;
- `raven sc sessions` lists active chat sessions with provider/model/branch;
- `raven sc providers` summarizes provider availability without dumping every
  model into the TUI;
- `raven sc worktree` reports target/base status and preserves merge-base
  failures as `FLAG`.

The adapter has a short timeout and only performs read-only calls. It does not
spawn sessions, select tabs, cancel turns, close sessions, or rewrite the
Superconductor target branch.

## Research Behavior

Raven v2 research is structured as packets, not freeform notes:

- `raven research lanes` lists the five bounded research lanes from
  `RAVEN_V2_RESEARCH_LEDGER.md`;
- `raven research packet <lane>` renders one lane into the required packet
  shape with live hard-gate evidence attached;
- `raven research synthesize` only reports readiness until at least three
  evidence-backed packets exist.

Research packets may recommend v1 implementation slices, but they cannot mark
remote deploy ready or bypass `DAS-2666` / `DAS-2669`.

## Gate Semantics

`PASS` means the specific requirement was tested at the scope it claims.

`FLAG` means the path is usable but not fully proven, stale, or missing an
external observation.

`BLOCK` means a required gate failed or needs human approval before continuing.

Raven must not upgrade `FLAG` to `PASS` because nearby tests passed.

Remote EverCore deploy has extra hard rules:

- `DAS-2669` must expose `AUTH_REPAIRED VERDICT: PASS` before the auth block is
  considered repaired;
- `DAS-2666` may not render `PASS` unless auth repair, guarded NixOS test,
  remote loopback full smoke, and supervisor `PASS` are all present;
- `DAS-2675` can repair Pi/OpenCode adapter lanes, but never changes remote
  deploy verdict.

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
raven run verify
```

It exits non-zero for `FLAG` or `BLOCK`.
