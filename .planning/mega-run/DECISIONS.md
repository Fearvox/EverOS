# Mega Run Decisions

## 2026-05-13T09:09:45Z - Start From Origin Main

Decision: create and work from `mega-24h-curator-2026-05-13` based on `origin/main`.

Reason: current `sleep-log` branch is ahead of `origin/main` by 32 commits. The runbook requires a clean run branch and forbids main/upstream writes. Keeping `sleep-log` untouched preserves prior audit state.

## 2026-05-13T09:09:45Z - Treat #12 As Workflow-Scope Bug

Decision: do not fix 1218 markdownlint findings repo-wide. Repair the workflow so markdownlint only checks changed Markdown files or an explicit baseline.

Reason: runbook explicitly says not to lint 1000+ legacy errors. A repo-wide formatting pass would be high-noise and low owner value.

## 2026-05-13T09:20:39Z - Pin Markdownlint Noise Boundary

Decision: disable `MD060/table-column-style` in `.markdownlint.json` while keeping substantive Markdown rules enabled.

Reason: local `markdownlint` v0.40.0 reports table style noise that the May Agent workflow did not target. The runbook asks for lightweight docs hygiene and forbids broad legacy formatting churn.
