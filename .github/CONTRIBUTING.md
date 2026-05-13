# Contributing to EverOS

Thanks for helping improve EverOS. This repository brings together architecture
methods, benchmarks, and use cases for long-term memory in self-evolving agents,
so there are several useful ways to contribute.

## Ways to Contribute

- Improve or extend an architecture method in `methods/`.
- Add benchmark tasks, adapters, or reproducibility notes in `benchmarks/`.
- Add a memory-enabled app, demo, or integration in `use-cases/`.
- Fix documentation, examples, setup steps, or broken links.
- Report bugs with clear reproduction steps and environment details.

## Development Setup

Most core development happens in EverCore:

```bash
git clone https://github.com/EverMind-AI/EverOS.git
cd EverOS/methods/EverCore

docker compose up -d
uv sync
cp env.template .env
uv run python src/run.py
```

Verify the server:

```bash
curl http://localhost:1995/health
```

## Common Commands

```bash
cd methods/EverCore
make test                     # Run tests
make lint                     # Run formatting and i18n checks
uv sync --group evaluation    # Install evaluation dependencies
```

## Pull Request Checklist

Before opening a PR, please check:

- The change is scoped to the relevant area: `methods/`, `benchmarks/`, or
  `use-cases/`.
- Setup or behavior changes are documented.
- Tests or manual verification are included when relevant.
- No secrets, `.env` files, generated build output, or dependency folders are
  committed.
- Active relative links in Markdown files resolve.

## Use-Case Contributions

Use cases should be easy for a new developer to inspect and run. Each use case
should include:

- A README with what it does, how to run it, and what memory feature it shows.
- A small `.env.example` when configuration is required.
- No committed images, build output, dependency folders, or secrets.

Images should be hosted with GitHub user attachments or another external asset
URL instead of committed to the repository.

## Fork-as-Lab Workflow

`Fearvox/EverOS` is a development fork of `EverMind-AI/EverOS`. All experimental
work happens on the fork before selective promotion upstream.

### Staying Current with Upstream

The fork auto-rebases onto upstream `main` every 6 hours via
`sync-upstream.yml`. This replays fork-only commits (templates, workflows, docs)
on top of the latest upstream. If you're working on a feature branch:

```bash
# Rebase your branch onto the latest fork main
git fetch origin
git rebase origin/main
```

If the auto-rebase encounters a conflict, it aborts and opens a tracking issue.
Manual resolution:

```bash
git checkout main
git pull upstream main --rebase
# resolve conflicts, then:
git push origin main --force-with-lease
```

### Branch Strategy

| Branch pattern | Purpose | Lifetime |
|---------------|---------|----------|
| `sleep-iter-*-*` | Automated overnight runs | Feature branch, merged or closed |
| `codex-watch-*` | Codex co-agent patrol | Isolated worktree, never touch |
| `feature/*` | Human-driven features | Feature branch → PR to origin/main |
| `sleep-log` | Overnight run audit log | Persistent tracking branch |

### Label Conventions

| Label | Color | Use on |
|-------|-------|--------|
| `pr-mirror` | `#0E8A16` | Issues that mirror an upstream PR — triggers Linear sync |
| `tracking` | `#5319E7` | Long-lived tracking issues |
| `security` | `#B60205` | Security advisories or security-relevant PRs |
| `urgent` | `#D93F0B` | High-priority; escalates in Linear |
| `sync-failed` | `#D93F0B` | Auto-applied when Linear sync fails for an issue |

### Issue Templates

Use the template picker when opening an issue. The two fork-specific templates:

- **PR Tracker** (`pr_tracker.yml`) — track an upstream PR for Linear/Slack
  visibility. Requires `pr_number`, `pr_url`, `author`, `area`, `scope`, and
  `evidence`. Applies `pr-mirror` + `tracking` labels.
- **Security Tracker** (`security_tracker.yml`) — track a security advisory.
  Adds `security` + `urgent` labels on top of the PR tracker labelset.

Both templates auto-trigger `linear-sync.yml`, which creates a corresponding
Linear issue in the `EverMind-Dash` project and comments back with the EVE
identifier.

### Linear Sync

Issues labeled `pr-mirror` are mirrored to Linear's `EverMind-Dash` project
automatically. The sync is one-way (GitHub → Linear). The bot comments
`🔗 Linear: [EVE-<N>](url)` on success.

If the bot adds a `sync-failed` label, check the workflow run logs at
`https://github.com/Fearvox/EverOS/actions/workflows/linear-sync.yml`.

### Promoting to Upstream

When a fork change is ready for `EverMind-AI/EverOS`:

```bash
gh pr create --repo EverMind-AI/EverOS \
  --base main \
  --head Fearvox:main \
  --title "feat: description" --body "..."
```

Templates and workflows committed to the fork are replayed on top of upstream
during every rebase cycle. They never conflict unless upstream adds same-named
files — low probability, handled by auto-rebase conflict detection.

## Style Notes

- Follow existing patterns before adding new abstractions.
- EverCore I/O is async; use `await`.
- EverCore is multi-tenant; keep data tenant-scoped.
- Keep prompt changes aligned across
  `methods/EverCore/src/memory_layer/prompts/en/` and
  `methods/EverCore/src/memory_layer/prompts/zh/` when applicable.

## Community

Please keep discussions respectful, constructive, and welcoming. See
`CODE_OF_CONDUCT.md` for expectations.

By contributing, you agree that your contributions are licensed under the
Apache License 2.0.
