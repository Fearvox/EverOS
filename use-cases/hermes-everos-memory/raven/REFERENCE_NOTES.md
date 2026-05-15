# Raven Console Reference Notes

Reference scan date: 2026-05-15.

No source code was copied or vendored. These notes record license posture and
portable UX/architecture lessons only.

## yetone/native-feel-skill

- License: MIT, verified from `LICENSE` in the shallow clone.
- Useful lesson: optimize for identity and operator muscle memory. Raven should
  keep a stable command shape and fast repeated verbs instead of exposing every
  internal subsystem as a new surface.
- Copied code: none.

## superagent-ai/grok-cli

- License: MIT, verified from `LICENSE` in the shallow clone.
- Useful lesson: split interactive and headless flows cleanly. Raven v0 keeps
  `repl`/`tui` interactive entrypoints while preserving scriptable commands
  like `status`, `packet show`, `memory search`, and `run verify`.
- Useful lesson: surface sub-agent and verification state as first-class
  operator data, but do not pretend failed runtimes are healthy.
- Copied code: none.

## openai/codex

- License: Apache-2.0, verified from `LICENSE` in the shallow clone.
- Useful lesson: a Rust CLI multitool can own command routing while a TUI is a
  separate operator shell. Raven v0 follows that split with a Rust command core
  and an ANSI-only first TUI screen.
- Useful lesson: local sandbox/deploy policy belongs in visible status, not in
  hidden assumptions.
- Copied code: none.

## claude-code-best/claude-code

- License: unresolved in the shallow checkout. The README advertises a GitHub
  license badge, but no `LICENSE` file was present in the cloned tree.
- Useful lesson: slash commands and provider-login surfaces are familiar to
  operators, but license uncertainty means this repo was used only for broad
  product-pattern inspiration.
- Copied code: none.

## NousResearch/hermes-agent

- License: MIT, verified from `LICENSE` in the shallow clone.
- Useful lesson: keep CLI, messaging, providers, memory, and skill systems as
  distinct adapter layers. Raven should be the console over those layers, not a
  replacement provider or another agent runtime.
- Useful lesson: slash command routing, provider status, and memory search are
  the right primitives for v0.
- Copied code: none.
