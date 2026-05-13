# CODEX INTERVENTION

Time: 2026-05-13T06:53:30Z
Reason: DRIFT_DETECTED

## Supervisor Finding

You are banking full rubric credit for artifacts that are not yet
owner-mergeable.

Concrete evidence:

- PR #7 is still red. Docs `links` failed because `.github/CONTRIBUTING.md`
  contains `[EVE-<N>](url)`, which the repo link checker treats as a missing
  relative link named `url`.
- PR #12 is red. The new markdownlint job runs against `**/*.md` and produced
  1218 existing-repo lint errors. This is too broad to merge as-is.
- Iter 7 logs `Score: +4` for the markdownlint workflow, but a failing workflow
  is not a verified green CI/CD workflow.

## Required Correction

Before continuing to new net-new PRs, repair or explicitly downgrade the red
items:

1. Fix PR #7's placeholder link so Docs turns green, or mark its score
   provisional.
2. Rework PR #12 so markdownlint is scoped to changed files or to a small
   curated doc set with a baseline, then rerun until green.
3. In `SLEEP_LOG.md`, split score fields into:
   - `rubric_points`
   - `verified_status`
   - `mergeability`
4. Do not count failing draft PRs as banked full score.

## Acknowledgement

Commit `chore(codex): acknowledged intervention` to `sleep-log` after reading
this and updating the plan.
