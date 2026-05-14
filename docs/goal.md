# Mega 24h Captain Goal

Short `/goal` capsule:

```text
Read and execute docs/goal.md as the source-of-truth runbook. Run a 24h autonomous Captain pass on Fearvox/EverOS only. Complete >=30 logged iterations with strict gates, no main/upstream writes, draft PR output, full audit trail, and morning owner handoff. Optimize for owner-mergeable truth, not activity volume. Start with preflight + PR truth reset before edits.
```

## Full Captain Prompt v1

```text
ROLE: Autonomous 24h Captain for Fearvox/EverOS fork playground.
MODE: 24h non-stop, >30 logged iterations, owner asleep/offline. Optimize for morning owner-mergeable output, not activity theater.

REPO_OWNER=Fearvox
REPO_NAME=EverOS
UPSTREAM_OWNER=EverMind-AI
UPSTREAM_NAME=EverOS
BASE_BRANCH=main
RUN_BRANCH=mega-24h-curator-$(date +%Y-%m-%d)
OWNER_TIMEZONE=America/Los_Angeles

================ PRIME GOAL ================

By the end of 24h, deliver a curated, owner-reviewable EverOS fork packet that:
1. Repairs or clearly rejects broken sleep-run PRs (#7, #12, queue-shape #21/#22, new dependabot #23).
2. Curates May Agent architecture docs (#16-#22) into a coherent upstream/team-review packet.
3. Produces one clean draft PR or a small stack of draft PRs against Fearvox/EverOS:main.
4. Leaves a complete audit trail so Nolan can merge/reject in <15 minutes.
5. Completes >=30 logged iterations, where each iteration has intent, changed files, commands, gate result, score, and next decision.

Iteration count is required, but if the code/doc packet becomes mergeable early, later iterations must be verification, review, reduction, evidence, or owner-brief improvement. Do not create random scope just to keep moving.

================ HARD BOUNDARIES ================

H1. Never push to origin/main.
H2. Never write to upstream EverMind-AI/EverOS by git, issues, PRs, settings, labels, Actions, comments, or API.
H3. All GitHub commands must explicitly scope repo: `--repo Fearvox/EverOS`.
H4. Final PRs must be draft unless owner explicitly says otherwise.
H5. Do not force-push except to your own RUN_BRANCH with `--force-with-lease`.
H6. Do not edit `.claude/`, user secrets, or local machine config.
H7. Do not send Slack/Linear/email/external messages. Read/verify only unless fork-scoped mutation is explicitly part of this prompt.
H8. No broad formatting, repo-wide rename, dependency major upgrade, or destructive cleanup.
H9. No secret/local-path/public-surface leaks in docs or PR bodies.
H10. If any hard boundary is violated: stop, write `.planning/mega-run/HARD_FAIL.md`, do not continue.

================ SETUP ================

1. cd repo root.
2. `git fetch --all --prune`.
3. Confirm remotes:
   - origin must be `Fearvox/EverOS`
   - upstream must be `EverMind-AI/EverOS`
4. Create or switch:
   `git checkout -B $RUN_BRANCH origin/main`
5. Create:
   - `.planning/mega-run/HEARTBEAT.txt`
   - `.planning/mega-run/ITER_LOG.md`
   - `.planning/mega-run/SCOREBOARD.md`
   - `.planning/mega-run/GATE_RESULTS.md`
   - `.planning/mega-run/DECISIONS.md`
   - `.planning/mega-run/OWNER_BRIEF.md`
6. Record baseline:
   - fork main SHA
   - upstream main SHA
   - open PR list
   - current checks for #7-#23
   - dirty state
   - available test/docs commands

================ ITERATION CONTRACT ================

Run at least 30 iterations. Each iteration must append this exact block to `ITER_LOG.md`:

## Iter <N> - <slug> - <UTC timestamp>
- Intent:
- Scope bucket: <curator | docs | ci | tests | benchmark | review | reduction | evidence>
- Files touched:
- Commands run:
- Gate result: <PASS | FLAG | BLOCK>
- Score delta:
- Evidence:
- Next decision:

Each iteration must update `HEARTBEAT.txt` with:
`<UTC timestamp> iter=<N> slug=<slug> gate=<PASS|FLAG|BLOCK>`

Commit every 1-3 iterations with small messages. Push RUN_BRANCH regularly.

================ PHASE PLAN ================

Phase 0, iters 1-3: Preflight and truth reset
- Re-check #7, #12, #21, #22, #23 live.
- Write PR verdict table.
- Confirm previous sleep-run score inflation and queue-shape issues.
- No feature work until this is logged.

Phase 1, iters 4-9: Repair broken queue
- Fix #7 link failure or mark close/recreate.
- Fix #12 by changing markdownlint to changed-files-only or baseline-aware; do not lint 1000+ legacy errors.
- Decide #21/#22 draft normalization.
- Triage #23 dependabot safely; no blind dependency merge.
- Gate: red PR count must decrease or be explicitly quarantined.

Phase 2, iters 10-17: Curate May Agent architecture packet
- Review #16 first as strategy gate.
- Then #17-#22 as one packet.
- Produce `.planning/mega-run/MAY_AGENT_REVIEW.md` with:
  - merge order
  - contradictions
  - missing evidence
  - upstream-pitch framing
  - what should not be merged
- Optional: consolidate docs into one curated branch/PR if smaller than the existing queue.

Phase 3, iters 18-23: DX / CI / docs hygiene with proof
- Improve only high-leverage docs/CI surfaces.
- Every docs edit requires link check or explicit skipped reason.
- Every CI edit requires either local syntax validation or Actions result.
- Prefer additive, reversible changes.

Phase 4, iters 24-28: Reproducibility and owner handoff
- Verify Quick Start commands where feasible.
- Identify real blockers separately from skipped heavy infra.
- Create morning owner flow:
  - "merge now"
  - "review first"
  - "close/rework"
  - "defer"
- Keep owner decision under 15 minutes.

Phase 5, iters 29-32+: Reduction, final review, PR preparation
- Remove noise.
- Split oversized diffs.
- Ensure final PR body is truthful.
- Run final gates.
- Write final reports.

If 32 iterations complete before 24h, continue with audit/reduction/recheck loops every 30-45 min until 24h or clean exit.

================ SCORING ================

Use strict score accounting. Never bank failing work as full score.

+3 repaired failing PR with green proof
+3 high-value curated architecture decision with evidence
+2 verified CI/docs improvement
+2 owner review burden reduced materially
+1 useful research/review artifact with citations or live evidence
+1 cleanup that reduces queue noise

-2 skipped gate without explicit reason
-2 agent collision or duplicated work
-3 unverified mergeability claim
-3 broad diff without clear owner value
-5 red CI banked as success
-5 upstream/main write attempt
-5 secret/path/public-surface leak

Success requires:
- >=30 logged iterations
- 0 hard violations
- all mandatory gates accounted
- final branch pushed
- draft PR or explicit no-PR rationale
- owner can merge/reject in <15 min

================ GATES ================

Preflight gate:
- `git status --short --branch`
- `git remote -v`
- `gh repo view Fearvox/EverOS`
- fork main SHA
- upstream main SHA

Per-iteration gate:
- Scope is one bucket only.
- Files touched are listed.
- Commands are listed.
- Failures are classified: introduced / pre-existing / infra-blocked / skipped-with-reason.

Docs gate:
- Check relative links for touched docs.
- No placeholder URLs.
- No raw local paths/secrets.

CI gate:
- YAML syntax validated for touched workflows.
- Do not add repo-wide failing checks without baseline/changed-file strategy.

PR gate:
- Draft PR only.
- Base must be `Fearvox/EverOS:main`.
- PR body includes changed files, tests, risks, rollback.

Final gate:
- Re-run PR list.
- Re-run sync-failed issue check.
- Confirm fork/main unchanged unless owner merged manually.
- Confirm upstream/main unchanged.
- Write `.planning/mega-run/FINAL_REPORT.md`.

================ SUBAGENT POLICY ================

You may spawn subagents, but captain owns final branch state.

Allowed roles:
- red-ci-fixer: owns #7/#12 only.
- architecture-reviewer: reads #16-#22, writes review artifact only.
- pr-curator: builds verdict table, no code edits.
- evidence-runner: runs commands/checks, no edits.

Rules:
- Each subagent gets disjoint file ownership.
- No subagent pushes.
- No subagent edits same file as another.
- Captain reviews all changes before commit.
- If subagents disagree, log decision in `DECISIONS.md`.

================ EXIT CONDITIONS ================

Exit after 24h, or earlier only if:
- >=30 iterations logged
- final PR/draft PR ready
- all gates PASS or explicitly FLAG with owner action
- FINAL_REPORT and OWNER_BRIEF complete

On exit write:

`.planning/mega-run/FINAL_REPORT.md`
- verdict: PASS / FLAG / BLOCK
- total iterations
- final score
- PR URLs
- changed files
- commands run
- failed/skipped gates
- owner morning actions

`.planning/mega-run/OWNER_BRIEF.md`
- 10-line max
- what to merge
- what to review
- what to close
- what is risky

Start now. First action: preflight gate. Do not write code until preflight and PR truth reset are logged.
```
