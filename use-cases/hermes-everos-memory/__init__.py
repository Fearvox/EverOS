"""EverOS memory provider for Hermes.

This file is intentionally a thin Python shim because Hermes memory providers
are loaded as Python classes. The provider talks to EverCore over HTTP and keeps
all behavior best-effort so memory outages never block Hermes turns.
"""

from __future__ import annotations

import json
import os
import threading
import time
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional

from agent.memory_provider import MemoryProvider
from tools.registry import tool_error

DEFAULT_BASE_URL = "http://127.0.0.1:1995"
DEFAULT_MEMORY_TYPES = ["episodic_memory", "profile"]


SEARCH_SCHEMA = {
    "name": "everos_search",
    "description": (
        "Search EverOS/EverCore memory. Use for past session context, user/project "
        "preferences, prior decisions, and agent trajectory recall."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "query": {"type": "string", "description": "Search query."},
            "top_k": {"type": "integer", "description": "Max results, default 5."},
            "memory_types": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Memory types: episodic_memory, profile, agent_memory, raw_message.",
            },
            "method": {
                "type": "string",
                "enum": ["keyword", "vector", "hybrid", "agentic"],
                "description": "Retrieval method, default hybrid.",
            },
        },
        "required": ["query"],
    },
}

STORE_SCHEMA = {
    "name": "everos_store",
    "description": "Store an explicit durable fact or note into EverOS memory.",
    "parameters": {
        "type": "object",
        "properties": {
            "content": {"type": "string", "description": "Memory content to store."},
            "role": {
                "type": "string",
                "enum": ["user", "assistant"],
                "description": "Role attribution, default user.",
            },
        },
        "required": ["content"],
    },
}

HEALTH_SCHEMA = {
    "name": "everos_health",
    "description": "Check whether local EverCore is reachable.",
    "parameters": {"type": "object", "properties": {}, "required": []},
}

FLUSH_SCHEMA = {
    "name": "everos_flush",
    "description": "Flush buffered agent messages for the current EverOS user/session.",
    "parameters": {"type": "object", "properties": {}, "required": []},
}


class EverOSClient:
    def __init__(self, base_url: str, timeout: float = 10.0):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def request(self, method: str, path: str, body: Optional[dict] = None) -> dict:
        data = None
        headers = {"Content-Type": "application/json"}
        if body is not None:
            data = json.dumps(body).encode("utf-8")
        req = urllib.request.Request(
            self.base_url + path,
            data=data,
            headers=headers,
            method=method,
        )
        with urllib.request.urlopen(req, timeout=self.timeout) as response:  # noqa: S310
            raw = response.read().decode("utf-8")
        if not raw:
            return {}
        return json.loads(raw)

    def health(self) -> dict:
        return self.request("GET", "/health")

    def search(
        self,
        *,
        query: str,
        user_id: str,
        method: str,
        memory_types: List[str],
        top_k: int,
    ) -> dict:
        return self.request(
            "POST",
            "/api/v1/memories/search",
            {
                "query": query,
                "method": method,
                "memory_types": memory_types,
                "top_k": top_k,
                "filters": {"user_id": user_id},
            },
        )

    def add_agent_messages(
        self,
        *,
        user_id: str,
        session_id: str,
        messages: List[dict],
    ) -> dict:
        return self.request(
            "POST",
            "/api/v1/memories/agent",
            {
                "user_id": user_id,
                "session_id": session_id,
                "messages": messages,
            },
        )

    def flush_agent(self, *, user_id: str, session_id: str) -> dict:
        return self.request(
            "POST",
            "/api/v1/memories/agent/flush",
            {"user_id": user_id, "session_id": session_id},
        )


class EverOSMemoryProvider(MemoryProvider):
    def __init__(self):
        self._base_url = os.environ.get("EVEROS_API_BASE_URL", DEFAULT_BASE_URL)
        self._user_id = os.environ.get("EVEROS_USER_ID", "hermes-user")
        self._agent_id = os.environ.get("EVEROS_AGENT_ID", "hermes")
        self._search_method = os.environ.get("EVEROS_SEARCH_METHOD", "hybrid")
        self._top_k = int(os.environ.get("EVEROS_TOP_K", "5"))
        self._auto_flush = os.environ.get("EVEROS_AUTO_FLUSH", "1").lower() not in {
            "0",
            "false",
            "no",
        }
        self._sync_inline = os.environ.get("EVEROS_SYNC_INLINE", "1").lower() not in {
            "0",
            "false",
            "no",
        }
        self._memory_types = self._parse_memory_types()
        self._client = EverOSClient(self._base_url)
        self._session_id = ""
        self._prefetch_result = ""
        self._prefetch_lock = threading.Lock()
        self._prefetch_thread: Optional[threading.Thread] = None
        self._sync_thread: Optional[threading.Thread] = None
        self._consecutive_failures = 0
        self._breaker_open_until = 0.0

    @property
    def name(self) -> str:
        return "everos"

    def is_available(self) -> bool:
        try:
            status = self._client.health()
            return status.get("status") in {"healthy", "ok"} or bool(status)
        except Exception:
            return False

    def initialize(self, session_id: str, **kwargs) -> None:
        self._session_id = session_id
        self._user_id = (
            kwargs.get("user_id")
            or os.environ.get("EVEROS_USER_ID")
            or self._user_id
        )
        identity = kwargs.get("agent_identity") or os.environ.get("EVEROS_AGENT_ID")
        if identity:
            self._agent_id = f"hermes-{identity}"

    def system_prompt_block(self) -> str:
        return (
            "# EverOS Memory\n"
            "Active. Use EverOS for durable cross-session recall. "
            "Use everos_search for explicit lookup and everos_store for durable facts."
        )

    def prefetch(self, query: str, *, session_id: str = "") -> str:
        if self._prefetch_thread and self._prefetch_thread.is_alive():
            self._prefetch_thread.join(timeout=2.0)
        with self._prefetch_lock:
            result = self._prefetch_result
            self._prefetch_result = ""
        if result:
            return result
        if not query:
            return ""
        # First turn after startup has no warmed result yet. Do a small direct
        # recall so enabling the provider is immediately visible.
        try:
            return self._format_prefetch(
                self._search(query=query, top_k=min(self._top_k, 3))
            )
        except Exception:
            self._record_failure()
            return ""

    def queue_prefetch(self, query: str, *, session_id: str = "") -> None:
        if not query or self._is_breaker_open():
            return

        def run() -> None:
            try:
                formatted = self._format_prefetch(self._search(query=query))
                with self._prefetch_lock:
                    self._prefetch_result = formatted
                self._record_success()
            except Exception:
                self._record_failure()

        self._prefetch_thread = threading.Thread(
            target=run, daemon=True, name="everos-prefetch"
        )
        self._prefetch_thread.start()

    def sync_turn(
        self, user_content: str, assistant_content: str, *, session_id: str = ""
    ) -> None:
        if self._is_breaker_open():
            return
        effective_session = session_id or self._session_id or f"hermes-{int(time.time())}"
        now = int(time.time() * 1000)

        def run() -> None:
            try:
                self._client.add_agent_messages(
                    user_id=self._user_id,
                    session_id=effective_session,
                    messages=[
                        {
                            "role": "user",
                            "sender_id": self._user_id,
                            "timestamp": now,
                            "content": user_content,
                        },
                        {
                            "role": "assistant",
                            "sender_id": self._agent_id,
                            "timestamp": now + 1,
                            "content": assistant_content,
                        },
                    ],
                )
                self._flush_session(effective_session)
                self._record_success()
            except Exception:
                self._record_failure()

        if self._sync_inline:
            run()
            return
        if self._sync_thread and self._sync_thread.is_alive():
            self._sync_thread.join(timeout=2.0)
        self._sync_thread = threading.Thread(
            target=run, daemon=True, name="everos-sync"
        )
        self._sync_thread.start()

    def get_tool_schemas(self) -> List[Dict[str, Any]]:
        return [SEARCH_SCHEMA, STORE_SCHEMA, HEALTH_SCHEMA, FLUSH_SCHEMA]

    def handle_tool_call(self, tool_name: str, args: Dict[str, Any], **kwargs) -> str:
        try:
            if tool_name == "everos_health":
                return json.dumps({"result": self._client.health()}, ensure_ascii=False)
            if tool_name == "everos_search":
                query = args.get("query", "")
                if not query:
                    return tool_error("Missing required parameter: query")
                data = self._search(
                    query=query,
                    top_k=int(args.get("top_k") or self._top_k),
                    method=args.get("method") or self._search_method,
                    memory_types=args.get("memory_types") or self._memory_types,
                )
                return json.dumps(self._compact_search_result(data), ensure_ascii=False)
            if tool_name == "everos_store":
                content = args.get("content", "")
                if not content:
                    return tool_error("Missing required parameter: content")
                role = args.get("role") if args.get("role") in {"user", "assistant"} else "user"
                sender_id = self._agent_id if role == "assistant" else self._user_id
                session_id = self._session_id or f"hermes-{int(time.time())}"
                data = self._client.add_agent_messages(
                    user_id=self._user_id,
                    session_id=session_id,
                    messages=[
                        {
                            "role": role,
                            "sender_id": sender_id,
                            "timestamp": int(time.time() * 1000),
                            "content": content,
                        }
                    ],
                )
                self._flush_session(session_id)
                return json.dumps({"result": "stored", "data": data.get("data")}, ensure_ascii=False)
            if tool_name == "everos_flush":
                data = self._client.flush_agent(
                    user_id=self._user_id,
                    session_id=self._session_id or "",
                )
                return json.dumps({"result": "flushed", "data": data.get("data")}, ensure_ascii=False)
        except urllib.error.URLError as exc:
            self._record_failure()
            return tool_error(f"EverOS unavailable: {exc}")
        except Exception as exc:
            self._record_failure()
            return tool_error(f"EverOS tool failed: {exc}")
        return tool_error(f"Unknown EverOS tool: {tool_name}")

    def on_pre_compress(self, messages: List[Dict[str, Any]]) -> str:
        if not messages:
            return ""
        tail = []
        for item in messages[-12:]:
            role = item.get("role", "")
            content = item.get("content", "")
            if isinstance(content, str) and content.strip():
                tail.append(f"{role}: {content[:1200]}")
        if not tail:
            return ""
        try:
            query = "\n".join(tail)[-3000:]
            return self._format_prefetch(self._search(query=query, top_k=5))
        except Exception:
            return ""

    def on_memory_write(
        self,
        action: str,
        target: str,
        content: str,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> None:
        if action not in {"add", "replace"} or not content:
            return
        note = f"[Hermes built-in memory:{target}] {content}"
        self.sync_turn(note, "Stored in Hermes built-in memory.", session_id=self._session_id)

    def on_delegation(
        self, task: str, result: str, *, child_session_id: str = "", **kwargs
    ) -> None:
        if not task and not result:
            return
        content = f"Delegated task: {task}\n\nResult: {result}"
        self.sync_turn(content, "Delegation observation recorded.", session_id=self._session_id)

    def on_session_end(self, messages: List[Dict[str, Any]]) -> None:
        if self._sync_thread and self._sync_thread.is_alive():
            self._sync_thread.join(timeout=3.0)
        self._flush_session(self._session_id)
        del messages

    def shutdown(self) -> None:
        for thread in (self._prefetch_thread, self._sync_thread):
            if thread and thread.is_alive():
                thread.join(timeout=3.0)
        self._flush_session(self._session_id)

    def get_config_schema(self) -> List[Dict[str, Any]]:
        return [
            {"key": "base_url", "description": "EverCore base URL", "default": DEFAULT_BASE_URL},
            {"key": "user_id", "description": "EverOS user id", "default": "hermes-user"},
            {"key": "agent_id", "description": "EverOS agent id", "default": "hermes"},
            {"key": "search_method", "description": "Search method", "default": "hybrid", "choices": ["keyword", "vector", "hybrid", "agentic"]},
            {"key": "top_k", "description": "Prefetch result count", "default": "5"},
        ]

    def save_config(self, values: Dict[str, Any], hermes_home: str) -> None:
        # Hermes' generic setup persists memory.provider. Runtime config stays
        # env-first for now so profiles, cron, and gateway can scope separately.
        del values, hermes_home

    def _search(
        self,
        *,
        query: str,
        top_k: Optional[int] = None,
        method: Optional[str] = None,
        memory_types: Optional[List[str]] = None,
    ) -> dict:
        return self._client.search(
            query=query,
            user_id=self._user_id,
            method=method or self._search_method,
            memory_types=memory_types or self._memory_types,
            top_k=top_k or self._top_k,
        )

    def _flush_session(self, session_id: str) -> None:
        if not self._auto_flush or not session_id:
            return
        self._client.flush_agent(user_id=self._user_id, session_id=session_id)

    def _format_prefetch(self, data: dict) -> str:
        compact = self._compact_search_result(data)
        rows = compact.get("memories", [])
        if not rows:
            return ""
        lines = ["## EverOS Memory"]
        for item in rows[: self._top_k]:
            title = item.get("subject") or item.get("type") or "memory"
            text = item.get("text") or ""
            score = item.get("score")
            prefix = f"- [{score:.2f}] " if isinstance(score, (int, float)) else "- "
            lines.append(f"{prefix}{title}: {text[:600]}")
        return "\n".join(lines)

    def _compact_search_result(self, data: dict) -> dict:
        payload = data.get("data") or {}
        memories = []
        for ep in payload.get("episodes") or []:
            memories.append(
                {
                    "type": "episodic_memory",
                    "subject": ep.get("subject") or "",
                    "text": ep.get("summary") or ep.get("episode") or "",
                    "score": ep.get("score"),
                    "session_id": ep.get("session_id"),
                }
            )
        for profile in payload.get("profiles") or []:
            memories.append(
                {
                    "type": "profile",
                    "subject": "profile",
                    "text": json.dumps(profile.get("profile_data") or {}, ensure_ascii=False),
                    "score": profile.get("score"),
                }
            )
        agent_memory = payload.get("agent_memory") or {}
        for case in agent_memory.get("cases") or []:
            memories.append(
                {
                    "type": "agent_case",
                    "subject": case.get("task_intent") or "agent case",
                    "text": case.get("approach") or "",
                    "score": case.get("score"),
                    "session_id": case.get("session_id"),
                }
            )
        for skill in agent_memory.get("skills") or []:
            memories.append(
                {
                    "type": "agent_skill",
                    "subject": skill.get("name") or "agent skill",
                    "text": skill.get("description") or skill.get("content") or "",
                    "score": skill.get("score"),
                }
            )
        return {"count": len(memories), "memories": memories}

    def _parse_memory_types(self) -> List[str]:
        raw = os.environ.get("EVEROS_MEMORY_TYPES", "")
        if not raw:
            return list(DEFAULT_MEMORY_TYPES)
        return [item.strip() for item in raw.split(",") if item.strip()]

    def _is_breaker_open(self) -> bool:
        if self._consecutive_failures < 5:
            return False
        if time.monotonic() >= self._breaker_open_until:
            self._consecutive_failures = 0
            return False
        return True

    def _record_success(self) -> None:
        self._consecutive_failures = 0

    def _record_failure(self) -> None:
        self._consecutive_failures += 1
        if self._consecutive_failures >= 5:
            self._breaker_open_until = time.monotonic() + 120


def register(ctx) -> None:
    ctx.register_memory_provider(EverOSMemoryProvider())
