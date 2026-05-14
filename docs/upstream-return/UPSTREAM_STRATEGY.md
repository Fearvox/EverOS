# Upstream Strategy

Live source: upstream inventory fetched on 2026-05-14.

## Current Queue Shape

- Open issues: 52.
- Open PRs: 37.
- PR merge states: 1 `CLEAN`, 4 `BLOCKED`, 32 `DIRTY`.
- The main risk is not lack of patches. The risk is merging stale, duplicated, or old-path patches without resolving maintainer policy.

## Recommended Maintainer Order

1. Triage and merge small current-tree fixes:
   - #211 for #93 if demo smoke passes.
   - #185 for #191 if README conflict is small.
   - #202 for OpenClaw docs if it matches the current plugin path.
2. Resolve high-impact API bugs with one selected PR per bug:
   - #78: pick #138 or request a new narrow PR; close #89/#109.
   - #127: request a focused adapter/fixture fix; #136 is too broad as-is.
   - #131: request a narrow full-episode patch; #132 is too broad as-is.
3. Make maintainer decisions before implementation:
   - delete/reset/cascade semantics (#14/#148);
   - lifecycle/dedup/status/session scope (#95/#143/#27);
   - provider/deployment support (#29/#23/#21/#4/#1);
   - benchmark reproducibility contract (#73/#3/#195/#87).
4. Sweep duplicated cleanup PRs:
   - RRF duplicates: #97/#141/#154.
   - timestamp duplicates: #108/#110/#112/#118.
   - bare-except/code-quality duplicates: #91/#98/#107/#110/#112/#126/#137/#154.
5. Close or rework old-path PRs:
   - `methods/evermemos/...` surfaces should not merge until path relevance is proven.
   - `evermemos-openclaw-plugin/*` should be verified against current package layout even when GitHub reports `CLEAN`.

## Fork Work That Is Worth Doing Locally

- Build a narrow current-tree patch for #191 if #185 remains blocked.
- Build a narrow current-tree OpenClaw docs/fix patch if #202/#128 are stale or path-wrong.
- Prepare answer drafts for repeated question issues so maintainers can close low-code threads quickly.
- Prepare benchmark repro notes, but do not claim result parity without running the exact benchmark path.

## What Not To Do

- Do not merge or push anything to upstream from this fork pass.
- Do not mark issues resolved from title/body alone.
- Do not spend time rebasing every dirty cleanup PR; pick canonical ones first.
- Do not accept old-path patches only because the title matches a live issue.
