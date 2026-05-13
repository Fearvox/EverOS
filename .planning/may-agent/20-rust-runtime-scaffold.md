# 20 — Rust Runtime Scaffold: Crate Structure, Traits, Prior Art

**Status**: Draft
**Date**: 2026-05-13
**Depends on**: 10-architecture.md, hermes-recon/architecture.md

## TL;DR

The Rust runtime replaces Hermes's Python CLI (615K LOC), TUI/Gateway layer,
and environment sandboxes with a tokio-based async runtime. It embeds the
Python agent core via PyO3 and provides a seccomp-bpf sandbox for tool
execution. Prior art: Tauri (desktop shell), tokio (async runtime), Burn
(ML framework), candle (inference).

## Crate Map

```
may-agent/                        # workspace root
├── Cargo.toml                    # workspace manifest
├── crates/
│   ├── may-agent-cli/            # binary crate — entry point
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # clap derive, subcommand dispatch
│   │       ├── commands/
│   │       │   ├── run.rs        # `may-agent run`
│   │       │   ├── gateway.rs    # `may-agent gateway`
│   │       │   ├── mcp.rs        # `may-agent mcp`
│   │       │   └── plugin.rs     # `may-agent plugin`
│   │       └── config.rs         # figment loader
│   │
│   ├── may-agent-runtime/        # core runtime library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # re-exports
│   │       ├── agent.rs          # AgentHandle — Python subprocess mgmt
│   │       ├── plugin.rs         # PluginRegistry — WASM + FFI loader
│   │       ├── sandbox.rs        # Sandbox — seccomp/macos/fallback
│   │       ├── gateway.rs        # GatewayService — tower service stack
│   │       ├── memory.rs         # MemoryClient — Evercore HTTP client
│   │       └── telemetry.rs      # tracing spans + metrics
│   │
│   ├── may-agent-ffi/            # PyO3 bridge
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # #[pymodule]
│   │       ├── dispatch.rs       # tool_dispatch()
│   │       ├── sandbox.rs        # sandbox_run()
│   │       └── gateway.rs        # gateway_deliver()
│   │
│   ├── may-agent-desktop/        # Tauri shell (future)
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   └── src/
│   │       └── main.rs
│   │
│   └── may-agent-sandbox/        # Standalone sandbox binary
│       ├── Cargo.toml
│       └── src/
│           └── main.rs           # seccomp worker process
│
├── python/                       # embedded Python (submodule: Hermes fork)
│   └── hermes-agent/             # git submodule at pinned SHA
│
└── tests/
    ├── integration/
    └── sandbox-escape/
```

## Core Traits

### AgentHandle

```rust
/// Manages the lifecycle of the embedded Python agent process.
#[async_trait]
pub trait AgentHandle: Send + Sync {
    /// Start the Python agent subprocess.
    async fn start(&self, config: AgentConfig) -> Result<(), AgentError>;

    /// Send a user message and await response.
    async fn send(&self, msg: UserMessage) -> Result<AgentResponse, AgentError>;

    /// Stop the agent subprocess gracefully.
    async fn stop(&self) -> Result<(), AgentError>;

    /// Check if agent subprocess is alive.
    fn is_alive(&self) -> bool;
}
```

**Implementation**: Spawns `python3 -m hermes_agent --mode headless` as a
tokio-managed subprocess. Communicates via JSON-RPC over stdin/stdout.

### ToolDispatcher

```rust
/// Dispatch a tool call to either the Python agent or a native Rust tool.
#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    /// Resolve which backend handles this tool.
    fn resolve(&self, tool_name: &str) -> ToolBackend;

    /// Execute the tool and return result.
    async fn execute(&self, call: ToolCall) -> Result<ToolResult, ToolError>;

    /// List available tools (merged Python + native).
    fn list_tools(&self) -> Vec<ToolDefinition>;
}

pub enum ToolBackend {
    /// Dispatch to Python agent (most tools).
    Python,
    /// Execute natively in Rust (performance-critical or sandbox-required).
    Native,
    /// Route through Evercore for memory operations.
    Evercore,
}
```

### Sandbox

```rust
/// Isolate tool execution from the host system.
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Execute a command in the sandbox.
    async fn execute(&self, cmd: SandboxCommand) -> Result<SandboxOutput, SandboxError>;

    /// Check if the sandbox is intact (no escape detected).
    fn health_check(&self) -> SandboxHealth;

    /// Get the sandbox backend name for logging.
    fn backend(&self) -> &'static str;
}

pub enum SandboxBackend {
    /// seccomp-bpf (Linux) — syscall-level filtering.
    Seccomp,
    /// macOS Seatbelt — sandbox_init_with_parameters().
    Seatbelt,
    /// Docker container — fallback when kernel sandbox unavailable.
    Docker,
    /// No sandbox — development mode only.
    None,
}
```

### MemoryClient

```rust
/// Client for Evercore's REST memory API.
#[async_trait]
pub trait MemoryClient: Send + Sync {
    /// Retrieve relevant memories for a query.
    async fn retrieve(&self, req: RetrieveRequest) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Store a new memory entry.
    async fn store(&self, req: StoreRequest) -> Result<MemoryId, MemoryError>;

    /// Get session info.
    async fn get_session(&self, session_id: &str) -> Result<Session, MemoryError>;

    /// Create a new session.
    async fn create_session(&self, req: CreateSessionRequest) -> Result<SessionId, MemoryError>;
}

pub struct RetrieveRequest {
    pub session_id: String,
    pub query: String,
    pub top_k: usize,
    pub tenant_id: String,
}
```

## Prior Art Analysis

### Tauri — Desktop Shell Model

**Repo**: `tauri-apps/tauri` (MIT, Rust)
**Relevance**: Desktop app shell for May Agent Desktop

| Feature | Tauri approach | May Agent adoption |
|---------|---------------|-------------------|
| Window management | WRY (WebView) | Keep Hermes Web UI in Tauri window |
| IPC | `invoke` + `emit` (JSON) | Replace with agent JSON-RPC |
| Plugin system | Rust-side plugins | Reuse Tauri plugin model for agent plugins |
| Binary size | ~5MB base | Target: < 50MB with Python embed |
| Security | CSP + isolation pattern | Add seccomp sandbox to Tauri's CSP |

### tokio — Async Runtime

**Repo**: `tokio-rs/tokio` (MIT, Rust)
**Relevance**: Foundation for all async I/O in May Agent

| Feature | tokio approach | May Agent adoption |
|---------|---------------|-------------------|
| I/O model | Mio + futures | Standard — all I/O is tokio |
| Task spawning | `tokio::spawn` | Agent sessions are tasks |
| Channels | `mpsc` / `broadcast` | Agent ↔ Gateway communication |
| Timers | `tokio::time` | Session timeouts, heartbeat |
| Signal handling | `tokio::signal` | Graceful shutdown |

### Burn — ML Framework

**Repo**: `tracel-ai/burn` (MIT/Apache 2.0, Rust)
**Relevance**: Optional — for local model inference

| Feature | Burn approach | May Agent adoption |
|---------|---------------|-------------------|
| Model format | Burn native + ONNX import | If local inference needed |
| Backend | WGPU, Candle, NdArray | WGPU for cross-platform |
| Training | Autodiff + optimizers | Not needed (inference only) |
| **Verdict** | — | 🕐 Future — not in May 31 scope |

### candle — Inference

**Repo**: `huggingface/candle` (MIT/Apache 2.0, Rust)
**Relevance**: Optional — for local model inference

| Feature | candle approach | May Agent adoption |
|---------|---------------|-------------------|
| Model loading | safetensors + GGUF | If local LLM needed |
| Quantization | GGML, GPTQ | For edge deployment |
| **Verdict** | — | 🕐 Future — not in May 31 scope |

## Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/may-agent-cli",
    "crates/may-agent-runtime",
    "crates/may-agent-ffi",
    "crates/may-agent-desktop",
    "crates/may-agent-sandbox",
]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
async-trait = "0.1"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/Fearvox/may-agent"
```

## Build Pipeline

```bash
# Development
cargo build --workspace

# With Python embed
cargo build --workspace --features python-embed

# Release (single binary with embedded Python)
cargo build --release --features python-embed,sandbox-seccomp

# Test
cargo test --workspace
cargo test --workspace --features sandbox-seccomp -- --ignored  # sandbox tests

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
```

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| PyO3 version conflicts | Medium | High | Pin PyO3 + Python minor version |
| seccomp too restrictive | Medium | Medium | Allowlist approach, test with Evil Agent Bench |
| Python GIL contention | High | Medium | Subprocess-per-session (no GIL sharing) |
| Cross-compilation pain | Medium | Low | CI matrix: linux-x86_64, macos-arm64 |
| Team Rust learning curve | High | Medium | Nolan owns Rust layer; team stays Python |

## Files to Create (Sprint 1)

1. `Cargo.toml` — workspace manifest
2. `crates/may-agent-cli/` — skeleton with clap
3. `crates/may-agent-runtime/` — AgentHandle + Sandbox + MemoryClient traits
4. `crates/may-agent-ffi/` — minimal PyO3 bridge
5. `crates/may-agent-sandbox/` — seccomp worker binary
6. `.github/workflows/rust-ci.yml` — build + test + lint

## References

- 10-architecture.md — system design with mermaid diagrams
- hermes-recon/architecture.md — Hermes internals
- Tauri: https://github.com/tauri-apps/tauri (MIT)
- tokio: https://github.com/tokio-rs/tokio (MIT)
- Burn: https://github.com/tracel-ai/burn (MIT/Apache 2.0)
- candle: https://github.com/huggingface/candle (MIT/Apache 2.0)
- PyO3: https://github.com/PyO3/pyo3 (MIT/Apache 2.0)
- `CLAUDE_DESKTOP_SANDBOX_SOURCE_TRUTH.md` — macOS Seatbelt reference
