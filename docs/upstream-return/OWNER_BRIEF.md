# Owner Brief

1. Live upstream queue on 2026-05-14: 52 open issues, 37 open PRs.
2. Targeted 2026-05-15 recheck: #191/#93/#78 are still open; #185/#211 are `BLOCKED`; #89/#109/#138 are `DIRTY`.
3. First returnable slice is ready locally: #191 README POST search example, #93 202 Accepted demo handling, #78 multi-memory-type retrieval.
4. The slice should go upstream as one narrow current-tree PR, not as rebases of #185/#211/#89/#109/#138.
5. Required verification for that PR: targeted pytest, black check, `git diff --check`, and reviewer pass on API contract.
6. Next small-review candidate after this slice is #202 for OpenClaw docs if it matches the current plugin path.
7. #127 is important, but #136 is too broad; request a focused filename mismatch patch with repro.
8. #131 likewise needs a narrow full-episode patch; #132 is too broad.
9. OpenClaw fixes must be checked against current paths; several PRs still touch legacy `methods/evermemos` or plugin-root paths.
10. Delete/reset/cascade memory semantics (#14/#148) need an owner API decision before code.
11. Benchmark reproducibility issues (#73/#3/#195/#87) need an official matrix, not isolated replies.
12. Provider/deployment requests (#29/#23/#21/#4/#1) should become a supported-provider decision.
13. Most cleanup PRs are duplicates; pick one narrow bug-linked cleanup path and close the rest.
