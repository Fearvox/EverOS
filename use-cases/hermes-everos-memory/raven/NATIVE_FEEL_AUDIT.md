# Raven Native Feel Audit

## Verdict

PASS for the v1 local terminal console contract.

This audit is Raven-specific. It borrows the discipline of native-feeling CLI
tools without copying any external reference implementation.

## Categories

| Category | Gate | Current Evidence | Verdict |
| --- | --- | --- | --- |
| Latency | Commands must return usable `PASS/FLAG/BLOCK` state without crashing when bridges are absent. | Memory and Multica adapters degrade to `FLAG` or fallback watch state. | PASS |
| Keybindings | A TUI operator can move without memorizing long commands. | `h`/`c` chat, `i` prompt input, `?`, `:`, `/`, `s`, `p`, `m`, `a`, `g`, `r`, `o` Superconductor, `d`, `n`, `q`, `Esc`, and `Ctrl-C` are handled. | PASS |
| Focus | The active panel is explicit state. | Panels are `Status`, `Packet`, `Chat`, `Memory`, `Agents`, `Gates`, `Runs`, `Doctor`, `NativeAudit`, and `Help`. | PASS |
| Scrollback | Evidence remains visible without layout churn. | The evidence drawer stays fixed; deep historical receipts live in `raven/.local-runs/`. | PASS |
| Interrupt behavior | Interrupts must exit or cancel cleanly. | `Esc` cancels prompt modes; `Ctrl-C` exits the TUI loop. | PASS |
| REPL history | Interactive command recall should feel local-native. | `rustyline` backs the interactive REPL; piped input stays deterministic for smoke tests. | PASS |
| Pane stability | Dynamic data cannot resize the command surface unpredictably. | `ratatui` uses fixed status, rail, evidence, and input regions around a flexible active panel. | PASS |
| Command grammar | CLI and REPL commands share the same operator vocabulary. | Slash commands map to status, packet, chat, memory, agents, gates, runs, doctor, audit, and quit handlers. | PASS |
| Typed IPC | Machine output is typed and redacted. | `RavenSnapshot`, `RavenReceipt`, `HermesChatTurn`, and `ScReport` are serialized through the sanitizer before JSON printing. | PASS |
| Evidence visibility | Hard gates and receipts are first-class. | DAS-2666, DAS-2669, local packet gates, saved receipts, and configured verification commands render directly. | PASS |
| Public-safety redaction | Public output must not expose private paths, hosts/IPs, tokens, credential paths, or signed URLs. | Human and JSON output pass through the sanitizer; receipts store sanitized excerpts. | PASS |

## Hard PASS Blockers

`raven native-audit` must refuse `PASS` when any hard category fails:

- missing keybindings for chat/input/quit/help/palette/search/status/gates/runs/audit;
- missing stable TUI panes;
- unsafe interrupt behavior;
- missing typed JSON snapshot or receipt contracts;
- unredacted public output;
- saved receipts not ignored by git.
