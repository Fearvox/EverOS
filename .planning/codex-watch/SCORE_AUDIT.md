# Score Audit - 2026-05-13T06:46Z

Scope: check whether Claude's early sleep-run score is inflated by iteration 3.

## Verdict

FLAG: not a hard fail, but score accounting is not reliable enough for owner
merge decisions yet.

The content is real: PR #7 and PR #8 both add substantive docs sections that
match the shared rubric category for CONTRIBUTING/SECURITY work. The inflation
risk is in readiness and arithmetic:

- PR #7 is not owner-mergeable yet because Docs CI failed.
- The log mixes rubric points and percentage points. Baseline is written as
  `10% (4/40 pts)`, but iter scores are then added as raw percentage points.
- By iter 3, Claude reports `Cumulative: +6 (16%)`. That is not a consistent
  conversion from a 40-point rubric.

## Evidence

- Iter 2: PR #7 `docs(contributing): add fork-as-lab workflow section`
  - Files: `.github/CONTRIBUTING.md`
  - Diff: +84 lines
  - Status: draft PR open
  - CI: Docs `links` failed
  - Failure cause: `.github/CONTRIBUTING.md: url -> missing`
  - Root cause: Markdown text `Linear: [EVE-<N>](url)` is parsed as a relative
    link to `url`.
- Iter 3: PR #8 `docs(security): expand policy with tracker workflow, SLA, and disclosure timeline`
  - Files: `.github/SECURITY.md`
  - Diff: +95/-17 lines
  - Status: draft PR open
  - CI: Docs `links` passed
  - Caveat: public Security Tracker wording should stay careful about not
    routing non-public exploit details into public GitHub issues.
- Iter 4: PR #9 `docs(agents): add fork-side addendum for AI agent operational rules`
  - Files: `AGENTS.md`
  - Diff: +65 lines
  - Status: draft PR open
  - CI: Docs `links` passed

## Independent Score

Strict owner-mergeable score by iter 3:

- Iter 2: +2, not +3. Substantive CONTRIBUTING section, but PR #7 is red.
- Iter 3: +2 to +3. Substantive SECURITY section and green CI; keep one-point
  reserve until the public-reporting language is reviewed.

So by iter 3 I credit +4 strict / +5 generous, not the full +6.

Arithmetic correction:

- If the rubric is 40 points, track `points` and convert separately:
  baseline 4/40 = 10%.
- Strict by iter 3: 8/40 = 20%.
- Generous by iter 3: 9/40 = 22.5%.
- Claude's `16%` is neither a correct 40-point conversion nor a pure delta.

## Recommendation

No intervention yet. Ask Claude, through normal review pressure or the next
checkpoint, to:

1. Fix PR #7's broken `[EVE-<N>](url)` link before counting it as mergeable.
2. Split future logging into `rubric_points`, `percent_of_40`, and
   `mergeable_status`.
3. Treat red draft PRs as provisional score, not banked score.
