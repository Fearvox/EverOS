# Hermes EverOS Memory Provider

Hermes memory-provider integration for EverOS.

This use case makes EverCore available to Hermes through Hermes' native
`MemoryProvider` lifecycle:

- pre-turn recall with `prefetch`
- post-turn persistence with `sync_turn` and local auto-flush
- explicit memory tools for search, store, health, and flush
- compression/delegation hooks reserved for the next pass

The provider is intentionally split:

- `__init__.py` is a thin Hermes interface shim. Hermes memory providers are
  loaded as Python classes, so this file stays small and dependency-free.
- `bin/everos-memory.mjs` is the operator/dev CLI used by Bun or Node for
  direct smoke tests.
- `scripts/install-local.sh` installs the provider into the active Hermes
  profile without activating it.
- `OWNER_PACKET.md` summarizes the current PASS/FLAG evidence for review.

## Status

`v0` targets local EverCore at `http://127.0.0.1:1995`.

It does not start EverCore for you. Bring EverCore up first:

```bash
cd methods/EverCore
docker compose up -d
uv sync
uv run python src/run.py --host 127.0.0.1 --port 1995
```

Remote NixOS/workhorse deployment packet:

- `deploy/nixos/DEPLOY_PACKET.md`
- `deploy/nixos/README.md`
- `deploy/nixos/evercore-remote-workhorse.nix`

The remote packet keeps EverCore bound to loopback by default and treats CCR as
a client/review lane, not the stateful memory host.

## Configure

Environment variables:

| Variable | Default | Description |
| --- | --- | --- |
| `EVEROS_API_BASE_URL` | `http://127.0.0.1:1995` | EverCore API base URL |
| `EVEROS_USER_ID` | `hermes-user` | User scope for personal/agent memory |
| `EVEROS_AGENT_ID` | `hermes` | Sender id for assistant turns |
| `EVEROS_SEARCH_METHOD` | `hybrid` | EverCore search method |
| `EVEROS_MEMORY_TYPES` | `episodic_memory,profile` | Search memory types |
| `EVEROS_TOP_K` | `5` | Number of memories to recall |
| `EVEROS_AUTO_FLUSH` | `1` | Flush agent memory after writes so recall is immediately searchable |
| `EVEROS_SYNC_INLINE` | `1` | Write/flush synchronously for CLI sessions that exit immediately |

## Local Smoke

```bash
just provider-load
just health
just search "Hermes memory provider"
just sync-smoke
just dogfood-smoke
```

Equivalent without `just`:

```bash
bun run bin/everos-memory.mjs health
bun run bin/everos-memory.mjs search "Hermes memory provider"
bun run bin/everos-memory.mjs sync-smoke
```

Node 18+ also works:

```bash
node bin/everos-memory.mjs health
```

`just provider-load` is offline and only verifies that Hermes can discover and
load the provider from a temporary profile.

`just dogfood-smoke` is also offline by default and verifies the Python provider
itself. When EverCore is running, use:

```bash
just dogfood-smoke health
just dogfood-smoke full
```

Remote health smoke:

```bash
just remote-smoke
just remote-smoke full
```

SkillHub packet dogfood:

```bash
just skillhub-sample
just skillhub-render
just skillhub-api-check
just skillhub-api-smoke
just skillhub-api
just skillhub-from-skill ../../benchmarks/EvoAgentBench/src/skill_evolution/evermemos/skills_sample/MUSICIAN/musician_life_event/SKILL.md
```

Raven run packet dogfood:

```bash
just raven-sample
just raven-render
```

Local model-free EverCore dogfood helper:

```bash
just mock-openai-check
just mock-openai
```

Point `LLM_BASE_URL`, `OPENROUTER_BASE_URL`, `VECTORIZE_BASE_URL`, and
`RERANK_BASE_URL` at the local mock when you need to verify the EverCore
store/extract/index/search loop without touching external model providers.

## Install Into Hermes

```bash
scripts/install-local.sh
hermes config set memory.provider everos
hermes plugins enable everos
```

Or use the interactive selector:

```bash
hermes memory setup
```

Run:

```bash
hermes memory status
```

Expected active status:

```text
Provider: everos
Plugin: installed
Status: available
```

## Safety Notes

- This provider is local-first and does not require an API key.
- It does not print raw memory payloads during install.
- It uses Hermes profile-scoped config through environment variables and
  `plugins.everos-memory` config keys.
- If EverCore is down, the provider reports unavailable and Hermes should
  continue without external memory.

## v0 Contract

Hermes calls:

- `initialize(session_id, hermes_home, platform, agent_identity, user_id, ...)`
- `prefetch(query, session_id=...)`
- `sync_turn(user_content, assistant_content, session_id=...)`
- `get_tool_schemas()`
- `handle_tool_call(name, args)`

EverCore calls:

- `GET /health`
- `POST /api/v1/memories/search`
- `POST /api/v1/memories/agent`
- `POST /api/v1/memories/agent/flush`
