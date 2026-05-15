# Raven Run Packet Contract

`v0` contract for a Raven run.

Raven is the operator-facing concept name and the repo-local command namespace.

Raven is not a marketing page here. This directory defines the packet that a
CLI/TUI can validate, render, and later execute against Hermes/EverOS memory.

## Files

| File | Purpose |
| --- | --- |
| `RAVEN_CONCEPT.md` | Public-safe Raven concept and naming contract |
| `COMMAND_CONTRACT.md` | Public-safe command/state/gate contract for Raven v1 |
| `REFERENCE_NOTES.md` | Public-safe reference scan and license notes |
| `schema.json` | Public-safe JSON Schema for a Raven run packet |
| `fixtures/doomsday-run.json` | Sample run packet for the current dogfood lane |
| `../raven-console/` | Rust Raven v1 CLI/REPL/TUI implementation |
| `../bin/raven` | Local shell wrapper for the Rust console |
| `../bin/raven-run.mjs` | Local validator and owner-packet renderer |

## Commands

```bash
node ../bin/raven-run.mjs validate fixtures/doomsday-run.json
node ../bin/raven-run.mjs render fixtures/doomsday-run.json
node ../bin/raven-run.mjs verify fixtures/doomsday-run.json
node ../bin/raven-run.mjs summary fixtures/doomsday-run.json
```

Console entrypoints:

```bash
bin/raven --help
bin/raven status
bin/raven status --json
bin/raven packet show
bin/raven packet export --output -
bin/raven loop
bin/raven loop --json
bin/raven chat send "summarize current hard gates"
bin/raven chat send --receipt - "summarize current hard gates"
bin/raven chat send --save "summarize current hard gates"
bin/raven memory health
bin/raven memory search "operator gate"
bin/raven agents list
bin/raven gates
bin/raven research lanes
bin/raven research packet native-feel
bin/raven research synthesize
bin/raven runs list
bin/raven sc
bin/raven sc sessions
bin/raven sc worktree
bin/raven run verify
bin/raven run verify --receipt -
bin/raven native-audit
bin/raven repl
RAVEN_TUI_ONCE=1 bin/raven tui
```

Just targets:

```bash
just raven-status
just raven-packet
just raven-loop
just raven-gates
just raven-agents
just raven-research-lanes
just raven-research-packet-smoke
just raven-research-synthesis
just raven-doctor
just raven-native-audit
just raven-runs
just raven-sc
just raven-sc-status
just raven-sc-sessions
just raven-sc-providers
just raven-sc-worktree
just raven-run-verify
just raven-chat-smoke
just raven-chat-receipt-smoke
just raven-repl-smoke
just raven-tui-smoke
just raven-console-check
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

The v1 console keeps local packet truth and remote deploy truth separate:

- local packet `PASS` plus remote hard gate `BLOCK` renders overall `FLAG`, not
  `PASS`;
- `DAS-2669` exposes `AUTH_REPAIRED VERDICT: PASS` for the accepted
  DeepSeek/OpenRouter auth-route repair; that clears only the auth block;
- `DAS-2666` cannot render `PASS` until auth repair, guarded NixOS test, remote
  loopback full smoke, and supervisor `PASS` are all present;
- `DAS-2675` can repair adapter lanes but has no effect on the remote deploy
  verdict.

Hermes dialogue is shared across surfaces: `raven chat send`, bare text or
`/chat` inside `raven repl`, and the `h` panel inside `raven tui` all use the
same sanitized adapter. The adapter records the public-safe Raven workspace
label, detected Hermes runtime, command shape, and sanitized transcript. Chat
receipts can be printed with `--receipt -` or saved with `--save`; they never
change remote deploy gate state.

The single-agent loop is also explicit: `raven loop`, `/loop`, and the TUI `l`
panel expose capture, plan, act, observe, verify, and receipt phases. Prompt
turns add live loop breadcrumbs, but gate closure still requires verifier and
remote hard-gate evidence.

Superconductor state is visible through `raven sc`. The adapter is read-only,
times out quickly, and turns socket or merge-base failures into `FLAG` evidence
instead of blocking the Raven console.
