# AGENTS.md

This repository is organized around the same reader journey as the top-level
README:

1. **Use cases** show what persistent memory enables in real products and
   workflows.
2. **Quick Start** gets EverCore running locally.
3. **Architecture methods** document the memory systems included in EverOS.
4. **Benchmarks** and **Evaluation** show how to measure and reproduce results.

## Project Map

- `methods/EverCore/` - long-term memory operating system for agents.
- `methods/HyperMem/` - hypergraph-based hierarchical memory architecture.
- `benchmarks/EverMemBench/` - memory quality evaluation.
- `benchmarks/EvoAgentBench/` - agent self-evolution evaluation.
- `use-cases/` - apps, demos, and integrations built on top of the memory layer.

## README Guidance

- Keep the top-level README flow smooth: overview, use cases, quick start,
  architecture methods, benchmarks, evaluation, citations, community.
- Avoid repeating the three-part project framing after the overview. Later
  sections should act as catalogues or action paths.
- Use repository-relative links in the README, and verify that active relative
  links resolve before finishing.
- Keep commented-out README blocks out unless they are intentionally preserved
  for a near-term restoration.

## Open-Source DX Guidance

- Keep root uncluttered. Prefer community files in `.github/`:
  `.github/CONTRIBUTING.md`, `.github/CODE_OF_CONDUCT.md`,
  `.github/SECURITY.md`, issue templates, and the pull request template.
- Treat `CITATION.cff` as optional. Add it only if the project wants GitHub's
  "Cite this repository" affordance at the cost of one extra root file.
- Favor clear run paths, small examples, and explicit verification commands.
- Make contribution paths obvious for architecture methods, benchmarks, docs,
  and use cases.
- Treat broken links, stale setup commands, missing `.env.example` files, and
  unclear issue templates as developer-experience bugs.
- Keep `.github/workflows/docs.yml` lightweight and dependency-free so docs
  hygiene is easy to trust.

## Quick Commands

```bash
cd methods/EverCore
docker compose up -d          # Start infrastructure
uv sync                       # Install dependencies
uv run python src/run.py      # Run application
make test                     # Run tests
make lint                     # Run formatting/i18n checks
uv run pyright                # Type check, if pyright is installed
```

## Key Entry Points

- `methods/EverCore/src/run.py` - EverCore application entry.
- `methods/EverCore/src/agentic_layer/memory_manager.py` - core memory manager.
- `methods/EverCore/src/infra_layer/adapters/input/api/` - REST API controllers.
- `methods/EverCore/docs/` - EverCore setup, usage, and architecture docs.
- `methods/EverCore/evaluation/` - EverCore evaluation runner and reports.

## Development Notes

- All I/O is async; use `await`.
- EverCore is multi-tenant; data must remain tenant-scoped.
- Prompts live in `methods/EverCore/src/memory_layer/prompts/` with EN/ZH
  variants.
- Prefer existing repo patterns and component boundaries before adding new
  abstractions.

## Review guidelines

- GitHub Copilot, Codex, and other review agents should follow
  `.github/copilot-instructions.md`.
- Start PR reviews with the MUW block:
  `VERDICT: PASS / FLAG / BLOCK`, `VERDICT_SUMMARY:`, and `EVIDENCE:`.
- Do not mark a PR `PASS` from author summary alone; inspect the actual diff,
  linked issue, and available checks first.
- Report actionable findings first, ordered by severity, with file/path,
  evidence, impact, and fix guidance.

## Fork-Side Addendum

This fork is a development lab for `EverMind-AI/EverOS`. Agents operating here
must preserve the fork boundary and keep source-truth routing explicit.

### Fork Identity

| Remote | Repo | Role |
|---|---|---|
| `origin` | `Fearvox/EverOS` | Writeable fork for experiments, mirrors, docs, and review packets. |
| `upstream` | `EverMind-AI/EverOS` | Read-only source project unless the human owner explicitly approves an upstream return. |

### GitHub CLI Rule

Always pass `--repo Fearvox/EverOS` to `gh` commands that mutate fork state.
Do not rely on GitHub CLI default-target heuristics when a command can write.

### Branch Conventions

| Prefix | Owner | Rule |
|---|---|---|
| `sleep-iter-*` | Overnight agent lanes | Draft PR first; merge, revise, or close after live checks. |
| `codex-watch-*` | Codex watch lanes | Treat as isolated unless explicitly assigned. |
| `sleep-log` | Overnight audit trail | Only the owning sleep lane may force-push. |
| `feature/*` | Human developers | Leave untouched unless the owner assigns it. |

### Planning Directory

`.planning/` is agent workspace material. Keep it out of public PRs unless the
owner explicitly wants a curated evidence packet in repo history.

Expected sleep-lane artifacts include:

- `SLEEP_LOG.md` for per-iteration notes.
- `HEARTBEAT.txt` for the current iteration marker.
- `baseline/` for pre-run snapshots.
- `CODEX_INTERVENTION.md` for co-agent interrupts.
- `WAKEUP_REPORT.md` for end-of-run summaries.

### Labels Agents May Apply

| Label | When |
|---|---|
| `tracking` | Opening a long-lived watch or catalog issue. |
| `sync-failed` | Workflow error handling, normally applied by automation. |
| `pr-mirror` | Issue-template or sync-created PR mirrors; do not add casually. |

Agents must not remove labels applied by humans or CI.

### Hard Bans

1. Never push to `upstream`.
2. Never push to `origin/main` directly; use feature branch plus draft PR.
3. Never modify `.claude/` or `settings.json` unless explicitly assigned.
4. Never send Slack, Linear, Discord, or email updates without explicit owner
   approval.
5. Never delete files larger than 1 KB without a backup tag or owner approval.
6. Never force-push outside `sleep-log` and your own feature branches.
7. Never touch `codex-watch-*` branches unless explicitly assigned.
8. Never exceed 60 GitHub API requests in a 60-second window.

### CI/CD Awareness

- `.github/workflows/sync-upstream.yml` syncs the fork with upstream. Agent
  branches should refresh from `origin/main` after sync cycles.
- `.github/workflows/linear-sync.yml` handles `pr-mirror`-labeled issues.
  Manual label changes can create Linear mirror noise.
- `.github/workflows/overnight-watch.yml` powers the sleep-lane patrol. Do not
  edit it during an active sleep run unless the owner assigns that work.

## Development Notes

- All I/O is async; use `await`.
- EverCore is multi-tenant; data must remain tenant-scoped.
- Prompts live in `methods/EverCore/src/memory_layer/prompts/` with EN/ZH
  variants.
- Prefer existing repo patterns and component boundaries before adding new
  abstractions.

## GitHub Agent Review Contract

- GitHub Copilot, Codex, and other review agents should follow
  `.github/copilot-instructions.md`.
- Start PR reviews with the MUW block:
  `VERDICT: PASS / FLAG / BLOCK`, `VERDICT_SUMMARY:`, and `EVIDENCE:`.
- Do not mark a PR `PASS` from author summary alone; inspect the actual diff,
  linked issue, and available checks first.
- Report actionable findings first, ordered by severity, with file/path,
  evidence, impact, and fix guidance.
