# Canonical Problem Families

Live source: upstream issue/PR inventory fetched on 2026-05-14.

## 1. Memory API correctness and documentation

Representative issues: #191, #78, #58, #46, #45, #34, #131.

Open PRs: #185, #196, #138, #109, #89, #132.

Maintainer move: choose one canonical v1 API docs patch and one canonical multi-type search patch. Close or rework the old-path/broad PRs after the choice.

## 2. Memory lifecycle semantics

Representative issues: #148, #143, #133, #101, #95, #27, #14.

Open PRs: #129, #106.

Maintainer move: decide cascade delete, reset, dedup, expiry, status metadata, and session scoping as API contracts before accepting implementation churn.

## 3. Benchmark reproducibility and evaluation DX

Representative issues: #195, #127, #88, #87, #73, #56, #41, #31, #22, #3.

Open PRs: #136, #115.

Maintainer move: publish a versioned repro matrix with expected runtime/cost, dataset/version pins, and tolerated metric deltas. Fix #127 with a narrow adapter patch.

## 4. OpenClaw and external integration DX

Representative issues: #193, #177, #150, #139, #93, #57, #52, #15, #11.

Open PRs: #211, #202, #189, #128, #86.

Maintainer move: keep current-tree OpenClaw docs/fixes; avoid merging legacy `methods/evermemos` or `evermemos-openclaw-plugin` paths without a path relevance check.

## 5. Provider and deployment configuration

Representative issues: #29, #23, #21, #9, #6, #4, #2, #1.

Open PRs: #206, #157, #144, #90.

Maintainer move: define supported provider matrix and official Docker/dependency boundary. Security/env fixes can proceed once DX migration is explicit.

## 6. Broad cleanup PR backlog

Representative issues: #50, #48 plus code-quality-only PRs without linked user reports.

Open PRs: #154, #141, #137, #126, #118, #113, #112, #110, #108, #107, #98, #97, #91.

Maintainer move: stop reviewing these as independent random cleanup. Pick one narrow bug-linked patch per family, then close duplicates.
