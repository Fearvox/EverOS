# 30 — Evercore Integration Contract: API, Wire Format, Failure Modes

**Status**: Draft
**Date**: 2026-05-13
**Depends on**: 10-architecture.md, `methods/EverCore/src/infra_layer/adapters/input/api/`

## TL;DR

The Rust runtime communicates with Evercore via HTTP REST (JSON). This
document specifies the exact API contract, wire format, error codes, and
failure modes that the `MemoryClient` trait implementation must handle.

## API Surface

### Endpoints Used

| Method | Path | Purpose | Caller |
|--------|------|---------|--------|
| `GET` | `/api/v1/health` | Health check | Startup probe |
| `POST` | `/api/v1/memories` | Store personal memory batch | Agent after each session turn |
| `POST` | `/api/v1/memories/group` | Store group memory batch | Gateway for shared context |
| `POST` | `/api/v1/memories/flush` | Trigger boundary detection (personal) | Agent on session end |
| `POST` | `/api/v1/memories/group/flush` | Trigger boundary detection (group) | Gateway periodically |
| `GET` | `/api/v1/memories/search` | Retrieve memories (multi-strategy) | Agent on every user message |
| `POST` | `/api/v1/memories/delete` | Soft delete memory entries | Admin / cleanup |
| `POST` | `/api/v1/groups` | Create or upsert group | Gateway on group chat start |
| `GET` | `/api/v1/groups/{group_id}` | Get group metadata | Gateway |
| `PATCH` | `/api/v1/groups/{group_id}` | Update group metadata | Gateway |

### Endpoints NOT Used (May 31 scope)

- Settings controller (`/api/v1/settings`) — EverCore manages its own config
- Sender controller — message sending is the gateway's job, not EverCore's

## Wire Format

### Store Memory

**Request**: `POST /api/v1/memories`
```json
{
  "tenant_id": "wecom-corp-123",
  "session_id": "sess_abc123",
  "messages": [
    {
      "role": "user",
      "content": "上次讨论的 Rust runtime 方案进展如何？",
      "timestamp": "2026-05-13T07:00:00Z"
    },
    {
      "role": "assistant",
      "content": "tokio 方案已通过，正在写 crate 结构。",
      "timestamp": "2026-05-13T07:00:05Z"
    }
  ],
  "metadata": {
    "platform": "wecom",
    "chat_id": "group_456",
    "agent_version": "0.1.0"
  }
}
```

**Response** (200):
```json
{
  "status": "ok",
  "message_count": 2,
  "boundary_detected": false
}
```

### Retrieve Memory

**Request**: `GET /api/v1/memories/search?tenant_id=wecom-corp-123&session_id=sess_abc123&query=Rust+runtime&strategy=hybrid&top_k=5`

Query parameters:
| Param | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `tenant_id` | string | Yes | — | Tenant scope |
| `session_id` | string | Yes | — | Session scope |
| `query` | string | Yes | — | Search query text |
| `strategy` | string | No | `hybrid` | `keyword`, `vector`, `hybrid`, `rrf`, `agentic` |
| `top_k` | int | No | 5 | Max results |
| `memory_types` | string[] | No | all | Filter: `episode`, `atomic_fact`, `foresight`, `agent_case`, `agent_skill` |

**Response** (200):
```json
{
  "status": "ok",
  "results": [
    {
      "id": "mem_001",
      "type": "episode",
      "content": "团队讨论了 Rust runtime 方案，选择了 tokio + PyO3 架构",
      "relevance": 0.94,
      "created_at": "2026-05-12T15:30:00Z",
      "metadata": {
        "session_id": "sess_prev",
        "participants": ["Nolan", "七仔"]
      }
    },
    {
      "id": "mem_002",
      "type": "atomic_fact",
      "content": "May Agent 使用 Rust + Python 混合架构",
      "relevance": 0.89,
      "created_at": "2026-05-12T15:31:00Z",
      "metadata": {}
    }
  ],
  "strategy_used": "hybrid",
  "search_time_ms": 45
}
```

### Health Check

**Request**: `GET /api/v1/health`

**Response** (200):
```json
{
  "status": "healthy",
  "version": "0.2.0",
  "uptime_seconds": 86400,
  "tenants_active": 3,
  "db_connected": true
}
```

### Error Response (all endpoints)

```json
{
  "status": "error",
  "error_code": "TENANT_NOT_FOUND",
  "message": "Tenant 'wecom-corp-999' does not exist",
  "request_id": "req_abc123"
}
```

## Error Codes

| Code | HTTP Status | Retry? | Meaning |
|------|-------------|--------|---------|
| `TENANT_NOT_FOUND` | 404 | No | Tenant ID not registered |
| `SESSION_NOT_FOUND` | 404 | No | Session ID not found |
| `INVALID_REQUEST` | 400 | No | Malformed JSON or missing required field |
| `MEMORY_STORE_FAILED` | 500 | Yes | Database write error |
| `MEMORY_RETRIEVE_FAILED` | 500 | Yes | Vector search error |
| `EMBEDDING_FAILED` | 500 | Yes | Embedding model unavailable |
| `RATE_LIMITED` | 429 | Yes (backoff) | Too many requests |
| `SERVICE_UNAVAILABLE` | 503 | Yes (backoff) | EverCore is down or starting |

## Failure Mode Handling

### Circuit Breaker

```rust
// Pseudocode — the MemoryClient implementation must include:
struct EvercoreClient {
    http: reqwest::Client,
    base_url: String,
    circuit_breaker: CircuitBreaker,
}

impl EvercoreClient {
    async fn request(&self, req: Request) -> Result<Response, MemoryError> {
        if self.circuit_breaker.is_open() {
            return Err(MemoryError::CircuitOpen);
        }

        match self.http.execute(req).await {
            Ok(resp) if resp.status().is_success() => {
                self.circuit_breaker.record_success();
                Ok(resp)
            }
            Ok(resp) if resp.status() == 429 => {
                self.circuit_breaker.record_failure();
                Err(MemoryError::RateLimited)
            }
            Ok(resp) if resp.status().is_server_error() => {
                self.circuit_breaker.record_failure();
                Err(MemoryError::ServerError(resp.status()))
            }
            Ok(resp) => {
                Err(MemoryError::ClientError(resp.status()))
            }
            Err(e) if e.is_timeout() => {
                self.circuit_breaker.record_failure();
                Err(MemoryError::Timeout)
            }
            Err(e) => {
                Err(MemoryError::Network(e))
            }
        }
    }
}
```

Circuit breaker config:
- Failure threshold: 5 errors in 60s window
- Half-open timeout: 30s
- Max retries: 3 with exponential backoff (1s, 2s, 4s)

### Graceful Degradation

When EverCore is unreachable:
1. **Store path**: Buffer messages in local SQLite (`hermes_state.py` already has
   session storage). Replay when EverCore comes back.
2. **Retrieve path**: Fall back to local session history (last N messages from
   current session). This gives degraded but functional memory.
3. **Health path**: Log warning, continue startup (agent works without memory).

### Consistency Model

- **At-least-once delivery**: Store requests are retried until acknowledged.
  Duplicate detection via idempotency key (`X-Idempotency-Key` header).
- **Eventually consistent retrieval**: Newly stored memories may take up to
  the flush interval (default: end of session) to appear in search results.
  The `flush` endpoint forces immediate indexing.

## Tenant Model

Each messaging platform group/chat maps to an EverCore tenant:

| Platform | Tenant ID Pattern | Session ID Pattern |
|----------|------------------|-------------------|
| WeCom (企业微信) | `wecom-{corp_id}` | `wecom-{corp_id}-{chat_id}` |
| Feishu (飞书) | `feishu-{tenant_key}` | `feishu-{tenant_key}-{chat_id}` |
| Slack | `slack-{team_id}` | `slack-{team_id}-{channel_id}` |
| Discord | `discord-{guild_id}` | `discord-{guild_id}-{channel_id}` |
| Direct/CLI | `evermind-{user_id}` | `evermind-{user_id}-{session_id}` |

Tenant creation: implicit on first `POST /api/v1/memories` with a new `tenant_id`.
No explicit tenant registration required (EverCore creates on demand).

## Integration Test Plan

```bash
# 1. Health check
curl http://localhost:1995/api/v1/health

# 2. Store memory
curl -X POST http://localhost:1995/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{"tenant_id":"test-001","session_id":"sess-001","messages":[{"role":"user","content":"Hello","timestamp":"2026-05-13T00:00:00Z"}]}'

# 3. Retrieve memory
curl "http://localhost:1995/api/v1/memories/search?tenant_id=test-001&session_id=sess-001&query=Hello&top_k=3"

# 4. Circuit breaker test (stop EverCore, send 6 requests, verify circuit opens)

# 5. Idempotency test (send same request twice, verify no duplicate memories)
```

## References

- `methods/EverCore/src/infra_layer/adapters/input/api/memory/memory_controller.py` — POST /api/v1/memories
- `methods/EverCore/src/infra_layer/adapters/input/api/memory/memory_search_controller.py` — GET /api/v1/memories/search
- `methods/EverCore/src/infra_layer/adapters/input/api/memory/group_controller.py` — Group CRUD
- `methods/EverCore/src/infra_layer/adapters/input/api/health/` — Health endpoint
- `methods/EverCore/src/agentic_layer/memory_manager.py` — Core memory manager
- 10-architecture.md — Memory bridge section
- 20-rust-runtime-scaffold.md — `MemoryClient` trait definition
