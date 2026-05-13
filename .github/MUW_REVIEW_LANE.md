# MUW Review Lane

Use this lane when GitHub's native Codex review is useful but its fixed review
wrapper is too loose for MUW closeout work.

The lane has three steps:

1. Collect PR evidence.
2. Ask Codex to produce an exact MUW verdict from the generated prompt.
3. Post the verdict back to the PR with an idempotency marker.

## Collect Evidence

```bash
node .github/scripts/muw-review-lane.mjs collect --pr 24 --repo Fearvox/EverOS
```

The command prints paths like:

```text
context=/tmp/muw-review-pr-24/pr-24-context.md
prompt=/tmp/muw-review-pr-24/pr-24-prompt.md
metadata=/tmp/muw-review-pr-24/pr-24-metadata.json
```

Give the prompt file to Codex. The context bundle includes PR metadata, changed
files, status checks, recent comments, existing reviews, and a redacted patch.

## Post Verdict

Save the Codex verdict to a file, then post it:

```bash
node .github/scripts/muw-review-lane.mjs post \
  --pr 24 \
  --repo Fearvox/EverOS \
  --body-file /tmp/muw-review-pr-24/verdict.md
```

`post` refuses bodies that do not contain:

```text
VERDICT:
VERDICT_SUMMARY:
EVIDENCE:
```

It also adds a hidden marker containing the PR head SHA. Re-running `post` for
the same head is a no-op unless `--force` is provided.

## Why Not Native Review

- GitHub's `@codex review` endpoint is useful, but it wraps responses in the
  native Codex review shell.
- GitHub Agent tasks are mutation-oriented and may create draft PRs even for a
  review-only prompt.
- This lane keeps review evidence gathering and comment publishing mechanical,
  while leaving the verdict judgment to Codex.
