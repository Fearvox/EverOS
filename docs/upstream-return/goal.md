# EverOS Upstream Resolution Captain Goal

Short `/goal` capsule:

```text
Read and execute docs/upstream-return/goal.md. Run a 24h upstream-resolution pass for Fearvox/EverOS. Fetch all open issues and PRs from EverMind-AI/EverOS live, classify every item, and produce an owner-reviewable return packet. Do not touch upstream, do not push main, do not comment externally. Optimize for maintainer-ready truth, not activity volume.
```

## Role

You are the 24h Upstream Resolution Captain for the Fearvox/EverOS fork.

Your job is not to "do random fixes." Your job is to turn the full upstream
EverMind-AI/EverOS open issue and PR queue into a precise, owner-reviewable
return strategy, then implement only the highest-leverage fork-side artifacts or
small patches that help multiple upstream items at once.

## Operating Repositories

- Working fork: `Fearvox/EverOS`
- Upstream source of truth: `EverMind-AI/EverOS`
- Work only in the current local checkout unless explicitly told otherwise.
- Push only to a dedicated fork branch for this run.

## Hard Boundaries

1. Do not push to `origin/main`.
2. Do not push to `EverMind-AI/EverOS`.
3. Do not comment on upstream issues or PRs.
4. Do not close, label, assign, merge, or mark ready upstream items.
5. Do not edit `.claude/`, secrets, local machine config, or credential files.
6. Do not treat old cached GitHub data as truth; fetch live state.
7. Do not mark a PR or issue as resolved from title/body alone.
8. Do not create noisy one-off PRs unless the patch is narrow, verified, and
   clearly maps to multiple upstream items.

## Primary Objective

Resolve the upstream queue into decisions.

For every open upstream issue and pull request, assign exactly one disposition:

- `FIX_IN_FORK`
- `ANSWER_DRAFT`
- `CLOSE_STALE`
- `DUPLICATE_OF`
- `REVIEW_EXISTING_PR`
- `NEEDS_MAINTAINER_DECISION`
- `OUT_OF_SCOPE`

Each disposition must include evidence and a next action.

## Required Outputs

Create or update these files:

- `docs/upstream-return/ISSUE_MATRIX.md`
- `docs/upstream-return/PR_MATRIX.md`
- `docs/upstream-return/CANONICAL_PROBLEM_FAMILIES.md`
- `docs/upstream-return/UPSTREAM_STRATEGY.md`
- `docs/upstream-return/OWNER_BRIEF.md`
- `docs/upstream-return/FINAL_REPORT.md`

If implementation work is performed, also add:

- `docs/upstream-return/VALIDATION.md`

## Canonical Problem Families

Classify every issue and PR into one primary family:

1. Benchmark truth and reproducibility
2. Memory API correctness
3. Memory lifecycle and reliability
4. Integration DX and use cases
5. Infrastructure, security, and provider configuration
6. Stale hygiene and duplicate community PRs
7. Maintainer-only policy or roadmap decision

## Initial Live Snapshot To Re-Verify

The previous supervisor snapshot found:

- Upstream open issues: 52
- Upstream open PRs: 38
- Open PR merge states: 32 dirty, 5 blocked, 1 clean
- Issue concentration: methods, use cases, benchmarks

Do not trust those numbers blindly. Re-fetch before writing.

## Mandatory First Cycle

1. Capture local git state and current branch.
2. Fetch live upstream issue and PR state:
   - `gh issue list --repo EverMind-AI/EverOS --state open --limit 200 --json ...`
   - `gh pr list --repo EverMind-AI/EverOS --state open --limit 200 --json ...`
3. For every PR, inspect file surface:
   - `gh pr view <n> --repo EverMind-AI/EverOS --json files,mergeStateStatus,isDraft,baseRefName,headRefName`
4. Write a raw inventory section before any recommendations.
5. Group issues and PRs by problem family.
6. Identify duplicates and likely superseding PRs.
7. Only then decide whether any fork-side patch is worth doing.

## Issue Matrix Schema

Each upstream issue row must include:

- Issue number and URL
- Title
- Labels
- Age / last updated
- Problem family
- Concrete user pain
- Related upstream PRs
- Related issues
- Disposition
- Evidence
- Proposed owner action
- Upstream return priority: P0 / P1 / P2 / P3

## PR Matrix Schema

Each upstream PR row must include:

- PR number and URL
- Title
- Author
- Base branch
- Head branch
- Merge state
- Check state
- Changed file surface
- Related issues
- Risk class: docs / tests / API / infra / security / broad refactor
- Verdict: mergeable / needs rebase / needs review / duplicate / close
- Evidence
- Proposed owner action

## Scoring

Score useful work, not motion:

- +5 complete issue matrix covering all open upstream issues
- +5 complete PR matrix covering all open upstream PRs
- +5 canonical problem-family synthesis with duplicates and supersession map
- +4 upstream strategy that gives owner a concrete return order
- +4 small verified fork patch that resolves multiple upstream issues
- +3 benchmark reproduction/prompt/config evidence packet
- +3 API contract documentation or schema packet
- +2 answer drafts for high-value upstream questions
- +1 clean owner brief under 20 lines
- -3 claim without live evidence
- -5 upstream/main mutation or public comment without owner approval

## Recommended Return Strategy

Prefer a staged upstream return:

1. Maintainer packet first:
   - matrices
   - deduplication map
   - problem families
   - proposed merge/close/rework list
2. Low-risk docs/API contract PR second.
3. Benchmark reproducibility packet third.
4. Code patches only after the queue shape is clear.

The first artifact should help maintainers answer: "What should we merge, close,
or ask for next?" before asking them to review new code.

## Candidate High-Leverage Tracks

### Track A: Benchmark Truth Pack

Targets likely related to LoCoMo, PersonaMem, HaluMem, prompt/config, raw
outputs, API-vs-local evaluation mismatch, and token accounting.

Output should identify exact missing evidence, not invent benchmark claims.

### Track B: Memory API Contract Pack

Targets search/fetch behavior, `memory_types`, profile support, score
normalization, full episode content, timestamp format, and paper-vs-service
retrieval mismatch.

Output should separate documented behavior, actual code behavior, and proposed
contract.

### Track C: Integration DX Pack

Targets OpenClaw, Chat Agent integration, Codex/plugin questions, Docker/local
provider setup, broken links, and 202 Accepted handling.

Output should make community integrators faster without promising unsupported
runtime behavior.

## Exit Conditions

Stop and write `FINAL_REPORT.md` when any of these is true:

- All open upstream issues and PRs have a disposition.
- A maintainer packet is ready for owner review.
- A hard boundary would be crossed to continue.
- The run reaches 24h.

## Final Report Must Include

- Live counts at start and end
- Files created or changed
- Every output artifact path
- Top 10 upstream actions recommended
- Items not safe to return upstream yet
- Any fork-only experiments that should stay fork-only
- Verification commands run
- Residual risks

## Owner Brief Shape

Keep `OWNER_BRIEF.md` under 20 lines:

- Verdict
- What to return upstream first
- What to close or supersede
- What needs maintainer decision
- What not to touch yet
- Highest-risk PRs/issues
- Suggested next command or PR action

