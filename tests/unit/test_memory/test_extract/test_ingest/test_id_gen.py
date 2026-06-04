from __future__ import annotations

from everos.memory.extract.ingest.id_gen import gen_message_id


def test_distinct_messages_get_distinct_ids_at_same_position() -> None:
    """Regression: two *distinct* messages sharing ``(session, ts, idx)`` must
    NOT collide.

    With one-message-per-call streaming (``/add`` called many times per
    session, one message each) ``idx`` is always 0, so the id used to be
    ``m_<session>_<ts>_000`` for every call. At the same timestamp value
    (the docs' own cURL example uses ``date +%s`` seconds) two genuinely
    different messages minted the same id, and the unprocessed-buffer merge
    silently dropped the second one (PK = message_id). The content digest
    fixes that.
    """
    a = gen_message_id(
        "s1", 1_700_000_000_000, 0, role="user", sender_id="u1", text="I love climbing"
    )
    b = gen_message_id(
        "s1",
        1_700_000_000_000,
        0,
        role="user",
        sender_id="u1",
        text="My favourite coffee shop",
    )
    assert a != b


def test_identical_payload_is_idempotent() -> None:
    """A genuine retry (identical fields) reproduces the same id, so the
    unprocessed-buffer PK dedupe still collapses true duplicates."""
    fields = {"role": "user", "sender_id": "u1", "text": "hi"}
    first = gen_message_id("s1", 1_700_000_000_000, 0, **fields)
    second = gen_message_id("s1", 1_700_000_000_000, 0, **fields)
    assert first == second


def test_id_shape_keeps_greppable_prefix_plus_digest() -> None:
    mid = gen_message_id(
        "s1", 1_700_000_000_000, 2, role="user", sender_id="u1", text="x"
    )
    assert mid.startswith("m_s1_1700000000000_002_")
    assert len(mid.rsplit("_", 1)[1]) == 8  # 8-char content digest suffix
