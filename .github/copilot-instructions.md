# Copilot and Codex Review Instructions

When reviewing pull requests in this repository, use the MUW review contract.
Start every review with this block:

```text
VERDICT: PASS / FLAG / BLOCK
VERDICT_SUMMARY: three lines or fewer; what passed, what is risky, and the next action
EVIDENCE:
```

Use the verdicts this way:

- `PASS`: the pull request objective is met and the evidence is sufficient.
- `FLAG`: useful progress, but a non-blocking issue, missing evidence, or follow-up remains.
- `BLOCK`: the objective is unmet, unsafe, unverifiable, or materially wrong.

Report findings first, ordered by severity. For each actionable finding, include:

- Severity
- File/path
- Evidence from the actual diff, status check, command output, or linked issue
- Why it matters
- Fix guidance or the next verification required

Review method:

1. Identify the promised objective from the PR title, body, linked issue, and changed files.
2. Inspect the real diff and available checks before making a success claim.
3. Compare evidence against the objective; do not accept `done` from a summary alone.
4. Verify the smallest real path that proves the claim.
5. Keep evidence concise, reproducible, and repository-relative.

EverOS-specific checks:

- For `methods/EverCore/`, preserve async I/O, tenant scoping, and existing module boundaries.
- For prompts, keep EN/ZH variants aligned when both exist.
- For docs and community files, preserve the README reader journey and keep root uncluttered.
- For `.github/workflows/docs.yml`, keep the workflow lightweight and dependency-free unless the PR explicitly changes that contract.
- Do not expose secrets, credential paths, raw tokens, private host values, or operator-only commands in review text.

For clean reviews, still return the MUW block with the evidence checked and any residual test gap. Keep the final review concise; prefer one clear judgment over a long menu of weak suggestions.
