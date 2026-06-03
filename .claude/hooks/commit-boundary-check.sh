#!/bin/bash
# commit-boundary-check.sh
#
# PreToolUse hook for EverOS. Reads the Claude Code hook payload from stdin,
# self-filters to git-commit / gh-pr-create invocations, and warns when a
# staged change set crosses multiple top-level directories.
#
# Rationale: PR #31 (Raven v2 closure) accidentally bundled 6 independent lanes
# (raven, hermes use-case, skillhub, upstream-return, ci, chores) into one
# 27-commit PR. EverOS convention from that retrospective: one component,
# one PR. This hook is a soft nudge, not a block — cross-cutting work (lint
# sweeps, dependency bumps, .gitignore policy) still proceeds.

set -eu

# Read the hook payload (we don't strictly need to parse it; we just want to
# self-filter and inspect git state). Discard the JSON.
cat >/dev/null 2>&1 || true

# The actual command Claude Code is about to run is exposed via tool input.
# Hook payload format varies across CC versions; we keep this hook scope-safe
# by running unconditionally and only acting when staged changes exist.

# Find the repo root from cwd so the hook works from worktrees too.
repo_root=$(git rev-parse --show-toplevel 2>/dev/null || true)
if [ -z "$repo_root" ]; then
  exit 0
fi

cd "$repo_root"

# What is staged for the next commit?
staged=$(git diff --cached --name-only 2>/dev/null || true)
if [ -z "$staged" ]; then
  exit 0
fi

# Extract the first path segment of each staged file. Filter out hidden
# top-level dirs (.github, .gitignore, .claude) and the root README so a
# legitimate root-doc fix doesn't trip the warning by itself.
top_dirs=$(echo "$staged" \
  | awk -F/ 'NF>1 {print $1} NF==1 {print "_root_"}' \
  | sort -u \
  | grep -Ev '^(\.github|\.claude|_root_)$' || true)

count=$(echo "$top_dirs" | grep -c . || true)

if [ "${count:-0}" -ge 2 ]; then
  cat >&2 <<EOF
⚠ commit-boundary-check: this staged change set crosses $count top-level directories:
$(echo "$top_dirs" | sed 's/^/  - /')

EverOS convention (post-PR-#31 retrospective): one component, one PR.
If these directories are part of the same logical change (e.g., a feature
that genuinely spans methods/ and use-cases/), proceed.
Otherwise, consider:
  git reset HEAD <files-from-other-lanes>
and committing the lanes separately.

This is a warning, not a block. Re-run the commit command to proceed.
EOF
fi

exit 0
