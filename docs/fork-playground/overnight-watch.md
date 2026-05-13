# Overnight Fork Watch

This fork can move fast, but the upstream feed should stay boring and auditable.
The overnight watch is a small GitHub Actions patrol for `Fearvox/EverOS`.

## What It Checks

- `origin/main` drift against `EverMind-AI/EverOS` `upstream/main`.
- Whether the active playground branch exists on the fork:
  `codex-watch-overnight-2026-05-13`.
- Failed, cancelled, or timed-out fork workflow runs in the last 24 hours.
- Upstream and fork pull requests updated in the last 24 hours.

## Tracking Behavior

The workflow prints a public-safe report on every run. If the verdict is `FLAG`,
it opens or updates a GitHub issue labeled:

- `overnight-watch`
- `tracking`
- `pr-mirror`

Issues created by `GITHUB_TOKEN` do not trigger secondary workflows. Because of
that, the watch mirrors the tracking issue to Linear directly when
`LINEAR_API_KEY` is available. The target Linear team/project are:

- Team: `233391d6-ec9e-4aa8-b534-16a221b8119a`
- Project: `39aa3865-345c-4313-9dc0-ab3b509c5d21`

A `FLAG` verdict does not fail the watch workflow by itself. Runtime errors
still fail the workflow, but expected drift or downstream failures are reported
through the tracking issue so the watch does not poison its own next run.

## Manual Run

```bash
REPO_OWNER=Fearvox \
REPO_NAME=EverOS \
WATCH_BRANCH=codex-watch-overnight-2026-05-13 \
OWNER_TIMEZONE=America/Los_Angeles \
LINEAR_TEAM_ID=233391d6-ec9e-4aa8-b534-16a221b8119a \
LINEAR_PROJECT_ID=39aa3865-345c-4313-9dc0-ab3b509c5d21 \
CREATE_TRACKING_ISSUE=false \
node .github/scripts/overnight-watch.mjs
```

Set `CREATE_TRACKING_ISSUE=true` only when you want the local run to mutate
GitHub issues.

## Public-Surface Hygiene

Reports intentionally avoid local absolute paths, host/IP values, token names
beyond the required GitHub secret names, and operator-only commands. They should
be safe to show in Discord or a screen share.
