"""Deterministic, human-readable ``message_id`` generation.

Format: ``m_<session_id>_<timestamp_ms>_<idx:03d>_<digest8>``.

Human-readable so logs / debugging / md entries stay greppable. Deterministic
so caller retries (same payload) produce the same ids — pipeline merge
naturally dedupes via the message_id PK in ``unprocessed_buffer``.
"""

from __future__ import annotations

import hashlib

_IDX_PAD = 3  # caller batches are capped at 500 messages (DTO limit), 3 digits cover it


def gen_message_id(
    session_id: str,
    timestamp_ms: int,
    idx: int,
    *,
    role: str,
    sender_id: str,
    text: str,
) -> str:
    """Return ``m_<session_id>_<timestamp_ms>_<idx:03d>_<digest8>``.

    The 8-char digest fingerprints the message identity (role / sender_id /
    text) so two *distinct* messages sharing the same
    ``(session_id, timestamp_ms, idx)`` get *distinct* ids. Without it,
    one-message-per-call streaming (``idx`` always 0) at the same timestamp
    minted colliding ids and the buffer merge silently dropped the second
    message. A genuine retry (identical payload) still reproduces the same
    id, so the ``unprocessed_buffer`` PK dedupe keeps working.
    """
    digest = hashlib.sha1(
        f"{role}\x1f{sender_id}\x1f{text or ''}".encode(), usedforsecurity=False
    ).hexdigest()[:8]
    return f"m_{session_id}_{timestamp_ms}_{idx:0{_IDX_PAD}d}_{digest}"
