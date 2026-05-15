# PR Packet: Memory API Onboarding Stability

Date: 2026-05-15.

## Target Issues

- #191: README memory search API example is outdated.
- #93: demo reports "Storage failed" when the API returns HTTP 202 Accepted.
- #78: search only uses `memory_types[0]` and silently ignores later requested
  memory types.

## Scope

This is one narrow current-tree slice:

- `README.md`
- `methods/EverCore/demo/utils/simple_memory_manager.py`
- `methods/EverCore/src/agentic_layer/memory_manager.py`
- `methods/EverCore/tests/test_memory_manager_multi_type_search.py`
- `methods/EverCore/tests/test_simple_memory_manager.py`

The upstream PR should not include Raven/deploy work, OpenClaw work, benchmark
adapter fixes, provider policy, or delete/reset semantics.

## Proposed PR Summary

- Update the README search example to use `POST /api/v1/memories/search`.
- Treat HTTP 202 Accepted as successful background extraction in
  `SimpleMemoryManager.store()`.
- Query all requested non-profile memory types in keyword/vector retrieval
  instead of only `memory_types[0]`.
- Preserve same backend ids from different memory collections during hybrid
  dedupe by using `(memory_type,id)`.
- Add focused regression tests for multi-type retrieval and 202 Accepted demo
  behavior.

## Verification

Commands run:

```bash
cd methods/EverCore
PYTHONPATH=src uv run pytest tests/test_memory_manager_multi_type_search.py tests/test_simple_memory_manager.py
uv run black --check src/agentic_layer/memory_manager.py demo/utils/simple_memory_manager.py tests/test_memory_manager_multi_type_search.py tests/test_simple_memory_manager.py
cd ../..
git diff --check -- README.md methods/EverCore docs/upstream-return
```

Result: PASS.

Notes:

- `PYTHONPATH=src` is required for direct targeted pytest invocation from this
  checkout.
- `tests/test_memory_controller.py` collected zero pytest tests and was not used
  as verification evidence.

## Live Upstream State Rechecked

On 2026-05-15:

- #191, #93, and #78 were still open.
- #185 and #211 were still `BLOCKED`.
- #89, #109, and #138 were still `DIRTY`.

## Reviewer Questions

- Does the README POST example match the public search-controller contract?
- Should the #78 fix remain limited to non-profile `MemoryManager` retrieval,
  leaving profile/raw-message orchestration to the higher-level search service?
- Is `(memory_type,id)` the right dedupe key for cross-collection hybrid hits?
