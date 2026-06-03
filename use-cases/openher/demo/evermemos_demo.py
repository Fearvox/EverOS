#!/usr/bin/env python3
"""
OpenHer × EverCore Integration Demo

Demonstrates how EverCore provides long-term memory to the
AI Being persona engine. Shows session context loading, memory
storage, search, and relationship vector evolution.

Usage:
    # With EverCore Cloud
    export EVERMEMOS_BASE_URL=https://api.evermind.ai/v1
    export EVERMEMOS_API_KEY=your_key
    python demo/evermemos_demo.py

    # With self-hosted EverCore
    export EVERMEMOS_BASE_URL=http://localhost:1995
    python demo/evermemos_demo.py

Note: the rewritten EverCore HTTP API mounts every route under the
absolute prefix ``/api/v1/memory`` (see
``src/everos/entrypoints/api/routes/``). EVERMEMOS_BASE_URL should point
at the host root (no ``/api/v1`` suffix); a trailing ``/api/v1`` is
stripped automatically for backward compatibility.
"""

import asyncio
import os
import sys
import json
import time
from typing import Optional

# ──────────────────────────────────────────────
# EverCore Client (minimal standalone version)
# ──────────────────────────────────────────────

try:
    import httpx
except ImportError:
    print("❌ httpx not installed. Run: pip install httpx")
    sys.exit(1)


class EverCoreClient:
    """Minimal EverCore client for demo purposes."""

    def __init__(self, base_url: str, api_key: str = ""):
        # Routes are mounted at the absolute prefix "/api/v1/memory", so the
        # client builds full paths from the host root. Strip a trailing
        # "/api/v1" (or "/api/v1/") so legacy env values keep working.
        root = base_url.rstrip("/")
        for suffix in ("/api/v1/memory", "/api/v1"):
            if root.endswith(suffix):
                root = root[: -len(suffix)]
                break
        self.base_url = root.rstrip("/")
        self.api_key = api_key
        self._client = httpx.AsyncClient(timeout=30.0)
        self.available = bool(base_url)

    async def _headers(self) -> dict:
        h = {"Content-Type": "application/json"}
        if self.api_key:
            h["Authorization"] = f"Bearer {self.api_key}"
        return h

    async def health_check(self) -> bool:
        """Check if EverCore is reachable (GET /health)."""
        try:
            resp = await self._client.get(
                f"{self.base_url}/health", headers=await self._headers()
            )
            return resp.status_code == 200
        except Exception:
            return False

    async def store_turn(
        self,
        user_id: str,
        persona_id: str,
        persona_name: str,
        user_name: str,
        group_id: str,
        user_message: str,
        agent_reply: str,
    ) -> dict:
        """Store a conversation turn as memory.

        POST /api/v1/memory/add — body is a ``MemorizeAddRequest``:
        ``session_id`` (the demo's group_id namespace), ``app_id`` /
        ``project_id`` scope, and ``messages[]`` of
        ``{sender_id, sender_name, role, timestamp(ms), content}``.
        The contract requires ``timestamp`` to be a Unix epoch in
        **milliseconds** (int > 0) and ``role`` to be one of
        ``user | assistant | tool``.
        """
        now_ms = int(time.time() * 1000)
        messages = [
            {
                "sender_id": user_id,
                "sender_name": user_name,
                "role": "user",
                "timestamp": now_ms,
                "content": user_message,
            },
            {
                "sender_id": persona_id,
                "sender_name": persona_name,
                "role": "assistant",
                "timestamp": now_ms,
                "content": agent_reply,
            },
        ]
        resp = await self._client.post(
            f"{self.base_url}/api/v1/memory/add",
            json={
                "session_id": group_id,
                "app_id": "openher",
                "project_id": "demo",
                "messages": messages,
            },
            headers=await self._headers(),
        )
        return resp.json() if resp.status_code == 200 else {"error": resp.text}

    async def search(
        self,
        query: str,
        user_id: str,
        group_id: str,
        top_k: int = 5,
    ) -> dict:
        """Search for relevant memories.

        POST /api/v1/memory/search — body is a ``SearchRequest``: the owner
        is identified by ``user_id`` XOR ``agent_id`` (exactly one), the
        method enum is ``hybrid``, ``top_k`` must be -1 or 1..100, and
        ``include_profile`` pulls the accumulated profile alongside hits.
        The demo's ``group_id`` namespace maps to ``app_id`` / ``project_id``
        scope (same scope used on the /add side).
        """
        resp = await self._client.post(
            f"{self.base_url}/api/v1/memory/search",
            json={
                "user_id": user_id,
                "query": query,
                "app_id": "openher",
                "project_id": "demo",
                "method": "hybrid",
                "top_k": top_k,
                "include_profile": True,
            },
            headers=await self._headers(),
        )
        return resp.json() if resp.status_code == 200 else {"error": resp.text}

    async def get_user_profile(self, user_id: str) -> dict:
        """Get the accumulated user profile.

        There is no dedicated ``/users/{id}/profile`` route in the
        rewritten API. Profiles are listed via POST /api/v1/memory/get with
        ``memory_type="profile"`` (user-owned only). This returns the inner
        ``profile_data`` dict of the first profile row, or ``{}`` if none.
        """
        resp = await self._client.post(
            f"{self.base_url}/api/v1/memory/get",
            json={
                "user_id": user_id,
                "memory_type": "profile",
                "app_id": "openher",
                "project_id": "demo",
            },
            headers=await self._headers(),
        )
        if resp.status_code != 200:
            return {}
        profiles = resp.json().get("data", {}).get("profiles", [])
        if not profiles:
            return {}
        return profiles[0].get("profile_data", {})

    async def close(self):
        await self._client.aclose()


# ──────────────────────────────────────────────
# Relationship Vector (from EverCore session)
# ──────────────────────────────────────────────

def compute_relationship_vector(profile_data: dict) -> dict:
    """
    Extract 4D relationship vector from EverCore profile data.

    These 4 dimensions expand the persona engine's neural network
    from 8D to 12D input, allowing it to differentiate behavior
    between strangers and old friends.
    """
    return {
        "relationship_depth": min(1.0, profile_data.get("interaction_count", 0) / 50),
        "emotional_valence": profile_data.get("sentiment_avg", 0.0),
        "trust_level": min(1.0, profile_data.get("trust_score", 0.0)),
        "pending_foresight": 1.0 if profile_data.get("foresight") else 0.0,
    }


def apply_relationship_ema(
    prior: dict,
    delta: dict,
    conversation_depth: float,
    prev_ema: Optional[dict] = None,
) -> dict:
    """
    Semi-emergent relationship update (Step 2.5 of ChatAgent lifecycle).

    Blends EverCore prior with LLM-judged delta through EMA:
      - alpha modulated by conversation depth (deeper = trust LLM more)
      - Clips to valid ranges
      - Preserves momentum through prev_ema
    """
    if prev_ema is None:
        prev_ema = dict(prior)

    alpha = max(0.15, min(0.65, 0.15 + 0.5 * conversation_depth))

    ema = {}
    for k in prior:
        lo = -1.0 if k == "emotional_valence" else 0.0
        posterior = max(lo, min(1.0, prior[k] + delta.get(k, 0.0)))
        prev = prev_ema.get(k, prior[k])
        ema[k] = round(alpha * posterior + (1 - alpha) * prev, 4)

    return ema


# ──────────────────────────────────────────────
# Demo
# ──────────────────────────────────────────────

async def main():
    base_url = os.getenv("EVERMEMOS_BASE_URL", "")
    api_key = os.getenv("EVERMEMOS_API_KEY", "")

    if not base_url:
        print("=" * 60)
        print("OpenHer × EverCore Integration Demo")
        print("=" * 60)
        print()
        print("⚠️  EVERMEMOS_BASE_URL not set.")
        print()
        print("To run this demo, set up EverCore:")
        print()
        print("  Option A — Cloud:")
        print("    export EVERMEMOS_BASE_URL=https://api.evermind.ai/v1")
        print("    export EVERMEMOS_API_KEY=your_key")
        print()
        print("  Option B — Self-hosted:")
        print("    cd vendor/EverCore && docker compose up -d")
        print("    uv run python src/run.py")
        print("    export EVERMEMOS_BASE_URL=http://localhost:1995")
        print()
        print("Get your API key: https://console.evermind.ai/")
        print()
        print("Running in simulation mode...\n")
        await demo_simulation()
        return

    client = EverCoreClient(base_url, api_key)

    print("=" * 60)
    print("OpenHer × EverCore Integration Demo")
    print("=" * 60)
    print(f"\n📡 EverCore: {base_url}")

    # Health check
    healthy = await client.health_check()
    if not healthy:
        print("❌ EverCore is not reachable. Check your URL and try again.")
        await client.close()
        return
    print("✅ EverCore is healthy\n")

    # ── Demo conversation ──
    user_id = "demo_user"
    persona_id = "luna"
    persona_name = "Luna (陆暖)"
    user_name = "Demo User"
    group_id = f"{persona_id}__{user_id}"

    conversations = [
        ("My name is Alex, I'm a software engineer", "Nice to meet you Alex! What kind of software do you work on?"),
        ("I love hiking in the mountains on weekends", "That sounds wonderful! There's something about being up high that makes everything else feel small."),
        ("I drink my coffee black, no sugar", "Noted! A purist. I respect that."),
    ]

    print("📝 Storing conversation memories...\n")
    for user_msg, agent_reply in conversations:
        result = await client.store_turn(
            user_id=user_id,
            persona_id=persona_id,
            persona_name=persona_name,
            user_name=user_name,
            group_id=group_id,
            user_message=user_msg,
            agent_reply=agent_reply,
        )
        status = "✅" if "error" not in result else "❌"
        print(f"  {status} User: \"{user_msg[:50]}...\"")

    # Wait for indexing
    print("\n⏳ Waiting for memory indexing (3s)...")
    await asyncio.sleep(3)

    # Search
    print("\n🔍 Searching for relevant memories...\n")
    queries = [
        "What does Alex like to do on weekends?",
        "How does Alex take their coffee?",
        "What is Alex's occupation?",
    ]

    for query in queries:
        result = await client.search(
            query=query,
            user_id=user_id,
            group_id=group_id,
        )
        # New envelope: {"request_id": ..., "data": {"episodes": [...],
        # "profiles": [...], ...}}. Episode hits carry a "summary" plus
        # nested "atomic_facts"; surface whichever is present.
        episodes = result.get("data", {}).get("episodes", [])
        print(f"  Q: \"{query}\"")
        if episodes:
            for ep in episodes[:2]:
                facts = ep.get("atomic_facts", [])
                snippet = (
                    facts[0]["content"]
                    if facts
                    else ep.get("summary") or ep.get("episode", "")
                )
                print(f"     → {str(snippet)[:100]}")
        elif "error" in result:
            print(f"     → (error: {str(result['error'])[:80]})")
        else:
            print("     → (no results yet — indexing may still be in progress)")
        print()

    # Relationship vector
    print("📊 Relationship Vector Evolution:\n")
    prior = {"relationship_depth": 0.0, "emotional_valence": 0.0, "trust_level": 0.0, "pending_foresight": 0.0}
    deltas = [
        {"relationship_depth": 0.1, "emotional_valence": 0.2, "trust_level": 0.05},
        {"relationship_depth": 0.05, "emotional_valence": 0.1, "trust_level": 0.1},
        {"relationship_depth": 0.08, "emotional_valence": 0.15, "trust_level": 0.12},
    ]

    ema = None
    for i, delta in enumerate(deltas):
        ema = apply_relationship_ema(prior, delta, conversation_depth=0.2 * (i + 1), prev_ema=ema)
        print(f"  Turn {i+1}: depth={ema['relationship_depth']:.3f} "
              f"valence={ema['emotional_valence']:.3f} "
              f"trust={ema['trust_level']:.3f}")
        prior = ema

    print(f"\n  → After 3 turns: no longer a stranger (depth={ema['relationship_depth']:.3f})")
    print(f"  → Neural network now produces warmer, more familiar behavioral signals\n")

    await client.close()
    print("✅ Demo complete!")


async def demo_simulation():
    """Run demo in simulation mode (no EverCore connection)."""
    print("📊 Simulating Relationship Vector Evolution:\n")
    print("   This shows how the 4D EverCore relationship vector")
    print("   deepens over multiple conversation turns.\n")

    prior = {"relationship_depth": 0.0, "emotional_valence": 0.0, "trust_level": 0.0, "pending_foresight": 0.0}

    # Simulate 10 turns of conversation
    simulated_deltas = [
        (0.3, {"relationship_depth": 0.10, "emotional_valence": 0.15, "trust_level": 0.05}),
        (0.4, {"relationship_depth": 0.08, "emotional_valence": 0.10, "trust_level": 0.08}),
        (0.5, {"relationship_depth": 0.05, "emotional_valence": 0.20, "trust_level": 0.12}),
        (0.6, {"relationship_depth": 0.06, "emotional_valence": -0.10, "trust_level": 0.03}),
        (0.7, {"relationship_depth": 0.04, "emotional_valence": 0.08, "trust_level": 0.10}),
        (0.7, {"relationship_depth": 0.03, "emotional_valence": 0.12, "trust_level": 0.08}),
        (0.8, {"relationship_depth": 0.02, "emotional_valence": 0.05, "trust_level": 0.06}),
        (0.8, {"relationship_depth": 0.03, "emotional_valence": 0.10, "trust_level": 0.05}),
        (0.9, {"relationship_depth": 0.01, "emotional_valence": 0.08, "trust_level": 0.04}),
        (0.9, {"relationship_depth": 0.02, "emotional_valence": 0.06, "trust_level": 0.03}),
    ]

    ema = None
    for i, (depth, delta) in enumerate(simulated_deltas, 1):
        alpha = max(0.15, min(0.65, 0.15 + 0.5 * depth))
        ema = apply_relationship_ema(prior, delta, conversation_depth=depth, prev_ema=ema)
        bar_d = "█" * int(ema["relationship_depth"] * 20)
        bar_v = "█" * int(max(0, ema["emotional_valence"]) * 20)
        bar_t = "█" * int(ema["trust_level"] * 20)
        print(f"  Turn {i:2d} (α={alpha:.2f}): "
              f"depth={ema['relationship_depth']:.3f} {bar_d}")
        print(f"                     "
              f"valence={ema['emotional_valence']:+.3f} {bar_v}")
        print(f"                     "
              f"trust={ema['trust_level']:.3f} {bar_t}")
        print()
        prior = ema

    print("  ──────────────────────────────────")
    print(f"  Final state: depth={ema['relationship_depth']:.3f}, "
          f"valence={ema['emotional_valence']:+.3f}, "
          f"trust={ema['trust_level']:.3f}")
    print()
    print("  Turn 4 shows a negative emotional event (valence delta = -0.10),")
    print("  but the EMA smoothing prevents overreaction — the relationship")
    print("  continues to deepen because trust was already building.")
    print()
    print("  This vector feeds into the 25D neural network input,")
    print("  producing different behavioral signals for strangers vs. friends:")
    print("  - Higher warmth, vulnerability, and initiative for trusted users")
    print("  - More guarded, formal signals for new users")
    print()
    print("✅ Simulation complete!")


if __name__ == "__main__":
    asyncio.run(main())
