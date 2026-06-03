# use-cases/hermes-everos-memory — Local CLAUDE.md

Local-only context. Root `CLAUDE.md` and `AGENTS.md` cover the cross-repo map.

## What this use case is

Hermes `MemoryProvider` integration that mounts **EverCore** (HTTP API at
`http://127.0.0.1:1995` by default) as the memory backend for Hermes sessions.
Covers prefetch (pre-turn recall), `sync_turn` (post-turn persistence) with
auto-flush, and explicit memory tools (search, store, health, flush).

This is also the staging ground for the **Hermes SuperGrok NixOS** lane (see
`docs/superpowers/specs/2026-05-16-hermes-supergrok-nixos-auth-plane-design.md`).

## Internal map

```text
__init__.py          thin Hermes interface shim (Python class entry)
bin/
  everos-memory.mjs    operator/dev CLI (Node/Bun) — health/search/sync-smoke
  skillhub-packet.mjs  SkillHub fixture validator
  skillhub-mock-api.mjs SkillHub mock API server
  raven-run.mjs        Raven run packet validate/render
  mock-openai-compatible.mjs  Mock OpenAI-compatible server
scripts/
  install-local.sh     installs provider into Hermes profile (no activation)
  skillhub-api-smoke.sh  HTTP smoke against SkillHub mock
deploy/
  nixos/               remote workhorse deploy packet (DEPLOY_PACKET.md,
                       README.md, evercore-remote-workhorse.nix)
skillhub/fixtures/     read-only views + install-packet fixtures
raven/fixtures/        doomsday + dogfood run fixtures
plugin.yaml            Hermes plugin manifest
package.json           Node scripts: health / search / sync-smoke /
                       skillhub:* / raven:* / mock-openai:* / test
justfile               just-runner shortcuts
```

## Hard rules

- **EverCore lifecycle is not our problem.** This package does not start
  EverCore. The expectation is documented in `README.md`: bring EverCore up
  first with `cd methods/EverCore && uv run python src/run.py --host 127.0.0.1
  --port 1995`.
- **Configuration is env-var driven only.** No hard-coded URLs or user IDs.
  See `EVEROS_*` vars in `README.md`. Defaults stay loopback-friendly.
- **Remote deploy stays loopback-bound by default.** `deploy/nixos/` keeps
  EverCore on `127.0.0.1`; CCR / external clients reach it through reverse
  proxy, not direct binding.
- **One component, one PR.** This lane is about to expand into Hermes
  SuperGrok + NixOS sync service + retrieval plugin + `everos-ops-mcp`.
  Each of those is a separate PR. **No multi-component PRs.** See the
  commit-boundary hook in root `.claude/`.

## Working commands

```bash
# from this directory:
npm run health        # ping EverCore at EVEROS_API_BASE_URL
npm run search        # smoke a search call
npm run sync-smoke    # round-trip sync_turn

# SkillHub mock:
npm run skillhub:serve     # boot mock API
npm run skillhub:check     # validate config-only
npm run skillhub:sample    # validate fixture
npm run skillhub:smoke     # HTTP smoke

# Raven:
npm run raven:sample       # validate doomsday-run fixture
npm run raven:render       # render fixture to terminal

# Self-test:
npm test                   # everos-memory self-test

# Install into Hermes (no activation):
bash scripts/install-local.sh
```

## Common gotchas

- **EverCore must be reachable.** `EVEROS_API_BASE_URL=http://127.0.0.1:1995`
  is the default. If EverCore is on a remote host, set this explicitly — do
  not assume tunnels.
- **`EVEROS_AUTO_FLUSH=1` and `EVEROS_SYNC_INLINE=1` are CLI-friendly
  defaults.** They make recall immediately searchable, at the cost of an
  extra round trip. Production / long-running session may want them `0`.
- **`memory_types` is comma-separated.** Default is
  `episodic_memory,profile`. Adding a third type means EverCore must support
  it on the search method.

## Cross-directory contract

- **Consumes** `methods/EverCore/` through its HTTP API only. No Python imports.
- **Surfaces** to Hermes through `__init__.py` (Python provider class) +
  `plugin.yaml` (manifest). Hermes loads the class at startup.
- **Does not** depend on `methods/HyperMem/` or `benchmarks/`.
- **Future Hermes SuperGrok lane** will add: NixOS sync service, retrieval
  plugin, `everos-ops-mcp`. Those will live here under
  `deploy/`, a new `plugin/` subtree, and a new `ops-mcp/` subtree
  respectively — each landing in its own PR.

## What does NOT belong here

- EverCore feature changes — those are in `methods/EverCore/`.
- New memory architectures — those are in `methods/`.
- Repo-wide planning state. `.planning/`, `.goal/`, `.remember/` stay at root.
- Multi-component PRs that mix sync + plugin + MCP + docs. **Always split.**
