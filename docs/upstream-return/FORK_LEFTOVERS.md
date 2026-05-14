# Fork Leftovers

As of 2026-05-14, `Fearvox/EverOS` still has a fork-side open PR queue that is
mostly clean at the GitHub merge/check level but not yet resolved into owner
decisions.

## Current Local State

- Local branch: `main`
- Local divergence: `ahead 7` vs `upstream/main`
- Working tree: clean
- This document is for fork cleanup only, not upstream mutation

## Decision Table

| Class | PRs | Recommended action | Why |
| --- | --- | --- | --- |
| Merge now | `#24`, `#27` | Merge after quick owner read | Both are `CLEAN`, have successful docs checks, and are explicit repo-surface improvements rather than speculative backlog work |
| Merge as packet or keep grouped | `#16`-`#22` | Review as one May Agent packet, then merge in order or squash into one curated outcome | The work is coherent, all seven PRs are `CLEAN`, and splitting the review across seven isolated approvals increases owner burden |
| Close after extracting signal | `#26` | Copy any useful review wording into the target thread, then close the PR | This is a draft reply artifact, not durable product/repo state |
| Defer pending explicit dependency policy | `#1`, `#23` | Leave open or close with rationale, but do not merge casually | Both are dependency bumps with no visible validation/check surface in the current fork |
| Triage separately | `#7`-`#15` | Re-check one by one only if they are still intended; otherwise close stale docs/ops drafts aggressively | These are older fork-side sleep-run outputs and are likely superseded by newer queue-shaping work |

## Merge Order

1. `#24` — docs gate repair path
2. `#27` — lead bridge operating contract
3. `#16` — May Agent vision gate
4. `#17`-`#22` — rest of the May Agent packet, preferably reviewed as one batch

## Not Worth Carrying Forward

- `#26` should not remain open as a pseudo-task queue item.
- `#1` and `#23` should not be merged without explicit validation policy.
- The older sleep-run PR tail should not stay open just because it is clean.

## Next Operator Move

Use this sequence:

1. Merge `#24`
2. Merge `#27`
3. Decide whether `#16`-`#22` stay as seven PRs or collapse into one maintainer packet
4. Close `#26`
5. Sweep `#7`-`#15`, `#1`, and `#23` into explicit `close` or `defer`
