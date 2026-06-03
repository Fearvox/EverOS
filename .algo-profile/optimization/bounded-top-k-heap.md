---
algorithm: Bounded Top-K Heap
category: optimization
complexity_time: O(M log K)
complexity_space: O(K)
used_in: docs/superpowers/specs/2026-05-16-hermes-supergrok-nixos-auth-plane-design.md
date: 2026-05-16
---

## Why This Was Chosen

When multiple candidate sources are merged, the system only needs the best K snippets, not a full sort of the entire candidate pool. A bounded min-heap keeps the strongest candidates while avoiding the extra cost of sorting low-value items that will never be shown to Hermes.

## Implementation Notes

Use this only at the merge boundary where candidate sets from collection search, local cache, or memory providers are combined. If the source already returns a stable top-k list, the heap can be skipped; otherwise keep the heap small and enforce K as a hard cap.

## Reference

[Heap / Priority Queue](https://github.com/trekhleb/javascript-algorithms)
