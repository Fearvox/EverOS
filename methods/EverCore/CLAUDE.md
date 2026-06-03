# methods/EverCore — Local CLAUDE.md

Local-only context for working inside this directory. Root `CLAUDE.md` and
`AGENTS.md` already cover the cross-repo project map and the canonical Quick
Commands — do not duplicate them here.

## What this module is

`memsys` (pyproject name) is the long-term memory operating system for agents.
Multi-tenant, fully async, FastAPI-backed, layered over MongoDB + Elasticsearch

+ Milvus + Redis. Public API surface lives at
`src/infra_layer/adapters/input/api/`.

## Internal layer map

```text
src/
├── core/           cross-cutting infra (DI, tenants, middleware, cache, queue,
│                   lifespan, rate_limit, observation, capability, oxm, lock)
├── memory_layer/   the memory pipeline — LLM, prompts (en/zh), extractors
│                   (memory_extractor, memcell_extractor), profile_indexer,
│                   profile_manager, cluster_manager
├── agentic_layer/  memory_manager.py orchestrates the layers above
├── biz_layer/      business policies on top of memory primitives
├── infra_layer/    HTTP, persistence, vector store, embedding adapters
├── api_specs/      DTOs / request-response contracts
├── service/        service-level wiring
├── migrations/     mongodb + postgresql schema migrations
└── devops_scripts/ sensitive_info scrubbing, milvus_admin, data_fix, i18n
```

Read order for a new task: `agentic_layer/memory_manager.py` → the layer it
touches → `core/` only if you hit a DI / tenant / lifespan question.

## Hard rules in this module

+ **Async everywhere.** No sync I/O in request paths. If a library is sync-only,
  push it to a thread pool via the existing `core/` helpers.
+ **Tenant scoping is not optional.** Every query, write, and cache key must
  carry tenant context resolved through `core/tenants/`. Cross-tenant leakage
  is a P0 bug.
+ **Prompts EN/ZH must stay in lockstep.** `src/memory_layer/prompts/en/` and
  `src/memory_layer/prompts/zh/` are mirrors. Adding a prompt to one without
  the other is a lint failure target.
+ **Public DTOs are a contract.** Files under `src/api_specs/dtos/` are
  consumed by `use-cases/` and external clients. Breaking changes need a
  migration note in `docs/CHANGELOG.md`.

## Working commands (precise — Root CLAUDE.md has the broad strokes)

```bash
# from this directory:
docker compose up -d           # boot mongo + es + milvus + redis (first time)
uv sync                        # install / refresh deps
uv run python src/run.py       # boot the API
make test                      # full pytest run
uv run pytest tests/test_memory_manager_multi_type_search.py -x -vv
                               # single-file iteration with -x stop-on-first-fail
make lint                      # ruff + black + i18n sync check
uv run pyright                 # type check (config in pyrightconfig.json)
```

## Common gotchas

+ Milvus standalone takes ~30s to become healthy. `docker compose ps` will
  show "starting" — wait for "healthy" before `python src/run.py`.
+ `env.template` defaults to OpenRouter → `x-ai/grok-4-fast`. Local runs that
  hit the actual LLM need a real `LLM_API_KEY` (OpenRouter or DeepSeek key).
+ The 202 Accepted path in `SimpleMemoryManager` is the async-ingest contract
  — do not collapse it to 200. See `tests/test_simple_memory_manager.py`.
+ Multi-type search (recall + extract) has hybrid dedup logic in
  `agentic_layer/memory_manager.py` — `test_memory_manager_multi_type_search.py`
  pins the invariant.

## Cross-directory contract

Things outside `methods/EverCore/` that depend on this module:

+ `use-cases/hermes-everos-memory/` mounts EverCore as the memory provider via
  the public HTTP API. Changing routes under `src/infra_layer/adapters/input/api/`
  needs a heads-up in that use case.
+ `benchmarks/EverMemBench/` exercises the recall + extract paths. Schema
  changes in DTOs require regenerating any frozen benchmark inputs.
+ `methods/EverCore/examples/openclaw-plugin/` is the JS plugin reference; the
  `engine.js` / `types.js` contract mirrors the Python DTOs.

## What does NOT belong here

+ New cross-cutting frameworks (auth plane, retrieval cache, MCP server, etc.).
  Those go to their own top-level lane and consume EverCore through the public
  API. Do not bolt them into `src/`.
+ Repository-wide planning state. `.planning/`, `.goal/`, and `.remember/` are
  root-level. Subdirectory CLAUDE.md files stay focused on this module only.
