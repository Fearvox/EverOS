# Hermes SuperGrok NixOS Goal

Short `/goal` capsule:

```text
Read and execute docs/superpowers/specs/2026-05-16-hermes-supergrok-nixos-auth-plane-design.md as the source of truth. Turn Hermes SuperGrok OAuth, xAI collection sync, and NixOS host control into a clean three-plane implementation with content-addressed delta sync, bounded top-k retrieval, and a local retrieval cache. Keep auth boundaries strict, preserve the existing EverOS memory provider, and prove the remote lane with live smokes before calling it done.
```

## Role

You are the implementation captain for the Hermes SuperGrok on NixOS lane.

Your job is not to redesign the auth model again. Your job is to turn the approved spec into a working remote lane that can:

- log into Hermes with SuperGrok OAuth,
- refresh the xAI knowledge bundle from NixOS,
- retrieve context through Hermes hooks and plugin boundaries,
- and keep the management key, OAuth state, and host control separate.

## Starting State

The current repo already has:

- a committed design spec at `docs/superpowers/specs/2026-05-16-hermes-supergrok-nixos-auth-plane-design.md`;
- a populated xAI `Windburn` collection with the repo knowledge bundle;
- an existing EverOS memory provider path under `use-cases/hermes-everos-memory`;
- a remote EverCore/NixOS packet under `use-cases/hermes-everos-memory/deploy/nixos/`;
- an `.algo-profile/` history for the content-addressed sync, bounded top-k merge, and retrieval cache choices.

Do not treat any of those as optional context. They are the baseline.

## Hard Boundaries

1. Do not mix SuperGrok OAuth with xAI collection management credentials.
2. Do not expose raw tokens, private paths, or host/IP details in model-visible context.
3. Do not replace the existing EverOS memory provider.
4. Do not turn the knowledge sync path into a browser-driven manual workflow.
5. Do not claim incremental sync unless a no-op delta path is proven on an unchanged source tree.
6. Do not claim retrieval is ready unless cache hit, cache miss, and stale-bundle behavior are all tested.
7. Do not touch unrelated workspace junk in `.goal/`, `.kilo/`, `.playwright-mcp/`, or local run output.

## Primary Objective

Deliver a remote-first Hermes knowledge lane on NixOS where:

- Hermes session auth is handled by SuperGrok OAuth,
- xAI knowledge uploads are handled by a collection-scoped management key on the host,
- the sync job is content-addressed and delta-aware,
- retrieval uses a local cache plus bounded top-k merge,
- and every plane can fail independently without collapsing the others.

## Required Outputs

The implementation should produce:

- a NixOS-hosted sync service and timer for the knowledge bundle,
- a manifest/delta engine that skips unchanged documents,
- a Hermes-facing retrieval layer that injects concise, provenanced context,
- cache and receipt artifacts for sync and retrieval,
- validation scripts or smokes that prove session, sync, cache, delta, and red-gate behavior,
- and any small docs updates needed to keep the operator flow legible.

## Phase Plan

### Phase 0 - Live State Verification

Confirm the real starting conditions before any edits:

- Hermes xAI OAuth login is still available locally,
- the `Windburn` collection is present and readable,
- the NixOS remote packet still matches the intended host shape,
- the knowledge bundle source roots are the ones we want to sync,
- and the current workspace is not carrying a hidden breakage in the relevant paths.

Gate: no implementation until the live state matches the plan.

### Phase 1 - Knowledge Sync Plane

Build the host-owned sync path first:

- normalize the approved source roots,
- walk the source tree once,
- sanitize and hash each document,
- write a manifest row per path with content hash and upload state,
- diff the new manifest against the last successful manifest,
- upload only added or changed documents in stable path order,
- tombstone deletions explicitly,
- and publish the new manifest pointer only after the upload succeeds.

This path should be `O(N)` on the first build and `O(Δ)` on refresh when the tree is unchanged or only lightly changed.

Gate: an unchanged tree must produce a no-op diff and skip upload.

### Phase 2 - Retrieval Plane

Build the Hermes-facing retrieval path on top of the sync plane:

- add a short-lived SQLite retrieval cache,
- key it by collection id, bundle hash, normalized query hash, top_k, and stable filter serialization,
- inject only top-k snippets with provenance,
- and use a bounded min-heap when multiple candidate sources must be merged.

This keeps repeated turns cheap and keeps the read path from degenerating into full sorts or repeated collection lookups.

Gate: repeated queries against the same bundle should hit cache, and merged candidate lists should preserve only the strongest `K` items without a full resort.

### Phase 3 - Hooks and Safety

Wire the policy layer around the retrieval path:

- `pre_tool_call` blocks mis-scoped or dangerous calls,
- `pre_llm_call` injects the retrieved context and current health state,
- `transform_tool_result` redacts secrets, paths, and oversized output,
- `post_tool_call` records a compact receipt with tool, duration, status, and collection revision.

Keep `execute_code` limited to mechanical packaging and validation work.

Gate: no secret or private-path material appears in model-visible output or receipts.

### Phase 4 - Validation and Proof

Prove each plane independently:

- session smoke: Hermes can log into xAI with SuperGrok OAuth and start a turn,
- sync smoke: the host can build, upload, and refresh the `Windburn` collection,
- cache smoke: repeated reads reuse the retrieval cache when the bundle hash is unchanged,
- delta smoke: unchanged sources produce a no-op manifest diff,
- top-k smoke: merged candidates stay bounded at `K`,
- failure smoke: missing secrets, expired auth, and retrieval timeouts degrade cleanly.

Gate: no plane can be marked PASS from inference alone.

## Decision Order

When trade-offs appear, prefer this order:

1. Strict auth separation.
2. Remote host reliability.
3. Incremental sync efficiency.
4. Retrieval latency.
5. Cosmetic cleanup.

## Exit Conditions

Stop when all of the following are true:

- the NixOS sync plane works end to end,
- the Hermes retrieval path works end to end,
- the cache and delta gates pass,
- the failure cases degrade cleanly,
- and the implementation is small enough that the operator can reason about the trust boundaries in one pass.

## Final Deliverable

When this goal is complete, the repo should have a single, truthful story:

- SuperGrok OAuth runs the Hermes session,
- the host-managed xAI key refreshes the collection,
- Hermes hooks handle context injection and redaction,
- and the EverOS memory provider remains the durable local memory layer.
