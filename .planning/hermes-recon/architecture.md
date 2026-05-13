# Hermes Agent Architecture Recon

**Source**: `NousResearch/hermes-agent` (147k ★, MIT)
**Clone**: `/tmp/hermes-recon` (depth=1, 2026-05-13T06:58Z)
**Version**: HEAD as of 2026-05-13
**Repo size**: ~100MB (shallow clone)

## Architecture Overview

Hermes is a Python agent framework with a plugin system, tool registry,
multi-provider model backend, and 20+ messaging platform adapters. The
core agent loop (`run_agent.py`) is ~12K lines; the CLI (`cli.py`) is
~615K lines. Total Python surface is massive — a full Rust rewrite of
the runtime (per 邓Sir's plan) is a major undertaking.

```
┌─────────────────────────────────────────────┐
│  Entry Points                                │
│  cli.py (TUI)  run_agent.py (headless)       │
│  batch_runner.py  rl_cli.py  mcp_serve.py    │
├─────────────────────────────────────────────┤
│  Core Agent Loop                              │
│  AIAgent class (run_agent.py, ~12K LOC)      │
│  - Prompt construction                        │
│  - Model provider dispatch                    │
│  - Tool call parsing + execution              │
│  - Compression (trajectory_compressor.py)     │
├─────────────────────────────────────────────┤
│  Tool System                                  │
│  tools/registry.py → ToolEntry               │
│  model_tools.py → handle_function_call()      │
│  80+ tool files auto-discovered               │
├──────────────┬──────────────────────────────┤
│   Plugins     │   Environments               │
│  memory (8)   │  local, docker, ssh,         │
│  models (6+)  │  modal, daytona,             │
│  context_eng  │  singularity                 │
│  kanban       │                              │
│  observability│                              │
└──────────────┴──────────────────────────────┘
```

## Key Components

### 1. Tool Dispatch Interface

**File**: `tools/registry.py:ToolEntry`

```python
class ToolEntry:
    __slots__ = (
        "name", "toolset", "schema", "handler", "check_fn",
        "requires_env", "is_async", "description", "emoji",
        "max_result_size_chars", "dynamic_schema_overrides",
    )
```

Tool files call `registry.register()` at module level. `discover_builtin_tools()`
uses AST parsing to find modules with register calls, then imports them lazily.
This is a **self-registering pattern** — no central tool list, no manual sync.

Evercore integration seam: The registry is the natural injection point. An
Evercore tool adapter would register tools that proxy through Evercore's API
layer instead of calling local handler functions.

### 2. Memory Abstraction

**File**: `plugins/memory/__init__.py`

Memory providers implement a `MemoryProvider` ABC. Eight implementations ship:

| Provider | File | Description |
|----------|------|-------------|
| honcho | `plugins/memory/honcho/` | Conversation memory |
| mem0 | `plugins/memory/mem0/` | mem0ai integration |
| supermemory | `plugins/memory/supermemory/` | Supermemory vector DB |
| retaindb | `plugins/memory/retaindb/` | RetainDB |
| openviking | `plugins/memory/openviking/` | OpenViking |
| byterover | `plugins/memory/byterover/` | ByteRover |
| hindsight | `plugins/memory/hindsight/` | Hindsight |
| holographic | `plugins/memory/holographic/` | Holographic memory |

Only ONE provider active at a time, selected via `config.yaml` → `memory.provider`.
Providers are discovered by scanning for `register_memory_provider` or
`MemoryProvider` in `__init__.py` source.

**Evercore integration seam**: Replace the MemoryProvider ABC with an Evercore
adapter. The adapter would use the same config-driven selection (`memory.provider: evercore`)
but route all memory operations through EverCore's REST API (or direct import).
This is 邓Sir's "abstract Memory" suggestion — keep the Hermes memory plugin
interface but swap the backend.

### 3. MCP Integration

**File**: `mcp_serve.py` (~32K LOC)

Hermes can run as an MCP server, exposing its tools to MCP clients (Claude
Desktop, VS Code, etc.). The MCP server wraps Hermes's tool registry and
exposes tool definitions via the MCP protocol.

**Evercore integration seam**: EverCore could register as an MCP client that
consumes Hermes's tool definitions, or Hermes's MCP server could be extended
to proxy tools through Evercore's memory layer before execution.

### 4. Sandbox Model

**File**: `environments/`

Hermes supports multiple execution backends for tool sandboxing:

| Environment | Status | Description |
|-------------|--------|-------------|
| local | Default | Direct host execution (no sandbox) |
| docker | Active | Containerized execution |
| ssh | Active | Remote execution via SSH |
| modal | Active | Modal.com serverless |
| daytona | Active | Daytona dev environments |
| singularity | Active | HPC containers |

**Evercore integration seam**: The Rust runtime would need to implement at
least the local + docker backends. Tauri/Burn/tokio are the prior-art
references for sandbox primitives in Rust.

### 5. Plugin System

**File**: `plugins/`

Plugin categories and count:
- `memory/` (8 providers) — memory backends
- `model-providers/` (6+) — inference backends (openrouter, anthropic, gmi, etc.)
- `context_engine/` — context augmentation
- `kanban/` — multi-agent board dispatcher
- `observability/` — metrics/traces/logs
- `image_gen/` — image generation providers
- `platforms/` — messaging platform adapters (in `gateway/platforms/`)

Plugins are discovered via filesystem scanning. User-installed plugins live
in `$HERMES_HOME/plugins/`. Bundled plugins take precedence on name collisions.

### 6. Session Store

**File**: `hermes_state.py` (~127K LOC)

SQLite-based session database with FTS5 full-text search. Stores conversation
history, tool call results, and agent state. Schema includes `sessions`,
`messages`, `tool_calls`, `tool_results`, `memories` tables.

**Evercore integration seam**: Replace the SQLite session store with EverCore's
tenant-scoped memory storage. Sessions become EverCore tenants; messages and
tool results become memory entries.

## Integration Seams Summary

| Seam | Hermes Component | Evercore Counterpart | Effort |
|------|-----------------|---------------------|--------|
| Memory backend | `plugins/memory/` ABC | EverCore memory_manager.py | Medium |
| Session store | `hermes_state.py` SQLite | EverCore tenant storage | High |
| Tool registry | `tools/registry.py` | EverCore API tool adapter | Low |
| Context engine | `plugins/context_engine/` | EverCore context retrieval | Medium |
| Plugin discovery | Filesystem scan | Config-driven plugin loading | Low |
| Environment sandbox | `environments/` backends | Rust sandbox (tokio/Tauri) | High |

## Fork Strategy

Per the May plan (project_evermind_2026_05 memory):
1. **Fork Hermes** (MIT license, no legal barrier)
2. **Rust rewrite of runtime** — environments, sandbox, plugin loader, CLI
3. **Evercore kernel integration** — memory backend, session store, context engine
4. **Keep Hermes GUI plugin** — the TUI/web interface stays as Hermes's
   presentation layer
5. **EverMem Bench + Evil Agent Bench** — benchmark narrative for the fork

## Prior Art for Rust Runtime

Per the May plan proposal:
- **Tauri** — desktop app shell (Hermes Desktop replacement)
- **Burn** — ML framework (model provider integration)
- **tokio** — async runtime (agent loop, tool dispatch)
- **candle** — inference (optional, for local model support)

## Key Metrics

- ~12K LOC core agent loop (`run_agent.py`)
- ~615K LOC CLI (`cli.py`) — primarily UI code, not runtime
- 80+ tool modules in `tools/`
- 8 memory provider implementations
- 20+ messaging platform adapters
- ~17K tests across ~900 files

## Files Referenced

- `/tmp/hermes-recon/AGENTS.md` — project structure guide
- `/tmp/hermes-recon/tools/registry.py` — ToolEntry definition, `discover_builtin_tools()`
- `/tmp/hermes-recon/plugins/memory/__init__.py` — MemoryProvider ABC, provider discovery
- `/tmp/hermes-recon/mcp_serve.py` — MCP server (~32K LOC)
- `/tmp/hermes-recon/environments/` — sandbox backends
- `/tmp/hermes-recon/hermes_state.py` — SQLite session store
- `/tmp/hermes-recon/run_agent.py` — AIAgent class (~12K LOC)
- `/tmp/hermes-recon/model_tools.py` — tool orchestration

## References in Existing Codebase

- `methods/EverCore/src/agentic_layer/memory_manager.py` — core memory manager
- `methods/EverCore/src/infra_layer/adapters/input/api/` — REST API controllers
- `methods/EverCore/docs/` — setup, usage, architecture docs
- `benchmarks/EverMemBench/` — memory quality evaluation
- `benchmarks/EvoAgentBench/` — agent self-evolution evaluation
