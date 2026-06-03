---
algorithm: LRU Retrieval Cache
category: structures
complexity_time: O(1)
complexity_space: O(capacity)
used_in: docs/superpowers/specs/2026-05-16-hermes-supergrok-nixos-auth-plane-design.md
date: 2026-05-16
---

## Why This Was Chosen

Repeated Hermes queries against the same collection bundle should reuse prior retrieval results instead of hitting the collection on every turn. An LRU-style cache gives bounded memory with constant-time average lookup and eviction, which matches the read-heavy, hot-query pattern of the remote NixOS lane.

## Implementation Notes

Cache keys should include collection id, bundle hash, normalized query hash, top_k, and filter serialization so different auth contexts do not collide. A TTL layer should sit on top of the LRU policy so stale entries disappear even if the bundle hash does not change.

## Reference

[Data Structures Reference](https://github.com/trekhleb/javascript-algorithms)
