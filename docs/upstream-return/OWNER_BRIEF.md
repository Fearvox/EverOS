# Owner Brief

1. Live upstream queue on 2026-05-14: 52 open issues, 37 open PRs.
2. Only one open PR is GitHub-clean: #128, but its file path looks legacy and still needs path relevance review.
3. Four PRs are blocked: #211, #206, #202, #185.
4. The highest-value small-review candidates are #211 (#93), #185 (#191), and #202 (#150/#139).
5. #78 should have one canonical multi-memory-type fix; close #89/#109 after choosing #138 or requesting a new narrow patch.
6. #127 is important, but #136 is too broad; request a focused filename mismatch patch with repro.
7. #131 likewise needs a narrow full-episode patch; #132 is too broad.
8. OpenClaw fixes must be checked against current paths; several PRs still touch legacy `methods/evermemos` or plugin-root paths.
9. Delete/reset/cascade memory semantics (#14/#148) need an owner API decision before code.
10. Benchmark reproducibility issues (#73/#3/#195/#87) need an official matrix, not isolated replies.
11. Provider/deployment requests (#29/#23/#21/#4/#1) should become a supported-provider decision.
12. Most cleanup PRs are duplicates; pick one narrow bug-linked cleanup path and close the rest.
13. Recommended next maintainer action: review #211, #185, #202, then publish the duplicate/stale PR closeout policy.
