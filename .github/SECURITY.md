# Security Policy

Thank you for helping keep EverOS and its users safe.

## Reporting a Vulnerability

**Do not open a public GitHub issue for a security vulnerability.**

### Preferred: GitHub Security Advisory

If the repository has GitHub Security Advisories enabled, open one at:
`https://github.com/Fearvox/EverOS/security/advisories/new`

### Alternative: Security Tracker Issue Template

For issues that benefit from team visibility and Linear/Slack tracking, use the
**Security Tracker** template at:
`.github/ISSUE_TEMPLATE/security_tracker.yml`

Select it from the issue template picker when opening a new issue, or create
directly via URL:
`https://github.com/Fearvox/EverOS/issues/new?template=security_tracker.yml`

The Security Tracker includes fields for CWE, severity, exposure scope,
affected components, fix summary, evidence, and residual risk. It
auto-applies the `security`, `pr-mirror`, `tracking`, and `urgent` labels,
which trigger immediate routing to Linear `EverMind-Dash` and Slack
`#p-evermind-dash`.

### What to Include

- **Affected component or path** — file, API endpoint, or subsystem.
- **Steps to reproduce** — minimum reproducible case.
- **Impact and severity** — what an attacker gains, under what conditions.
- **Relevant logs, requests, responses, or screenshots.**
- **Suggested fix**, if you have one.

## Supported Versions

| Version | Supported |
|---------|-----------|
| `main` (EverCore `>=0.2.0`) | Active |
| Fork `Fearvox/EverOS` branches | Best-effort |
| `use-cases/` demos | Community-supported, not covered by security SLA |

## In-Scope

- EverCore API, storage, tenant isolation, and memory retrieval behavior.
- Authentication, authorization, or data exposure in `methods/EverCore/`.
- Secret handling in examples, demos, and deployment files.
- Benchmark or use-case code that executes untrusted input.
- CI/CD workflows that handle secrets (`linear-sync.yml`, `sync-upstream.yml`).
- GitHub Actions supply-chain risks in `Fearvox/EverOS` workflows.

## Out of Scope

- Upstream `EverMind-AI/EverOS` security — report those directly to upstream
  maintainers. The fork does not triage upstream's issues.
- Infrastructure-level attacks against GitHub Actions, Linear, or Slack.
- Social engineering or phishing.
- DOS attacks against the public GitHub repo.

## Security Workflow

```
Report → Security Tracker Issue (private or labeled) →
  ├─ linear-sync.yml creates EVE in Linear EverMind-Dash →
  │   └─ Linear → Slack #p-evermind-dash (urgent flag)
  ├─ Triage within 48h (urgent) or 5 business days (standard)
  ├─ Fix developed on fork feature branch
  ├─ Fix promoted upstream via PR to EverMind-AI/EverOS
  └─ Public disclosure after fix lands upstream
```

### Label Routing

| Label | Effect |
|-------|--------|
| `security` | Identifies the issue as security-sensitive |
| `urgent` | Escalates Linear priority to Urgent (1); immediate Slack notification |
| `sync-failed` | Auto-applied if Linear sync errors — check workflow logs |

## Dependabot Alerts

Dependabot scans for vulnerable dependencies across `methods/EverCore/`,
`benchmarks/`, and `use-cases/`. Alerts are visible at:
`https://github.com/Fearvox/EverOS/security/dependabot`

Alert response SLA:
- **Critical**: patch within 7 days
- **High**: patch within 30 days
- **Moderate / Low**: addressed in next regular dependency update cycle

Dependabot PRs should be reviewed for breaking changes before merge, especially
in `use-cases/` where demo lockfiles may pin older versions intentionally.

## Disclosure Timeline

1. **T-0**: Report received, initial triage within 48h.
2. **T-3d**: Confirmation + severity assessment shared with reporter.
3. **T-7d (critical) / T-30d (high)**: Fix developed and tested on fork.
4. **Fix day**: Fix promoted upstream. CVE or advisory filed if applicable.
5. **Fix + 7d**: Public disclosure with full details, credit to reporter.

Maintainers may adjust the timeline for severity, exploitability, or upstream
coordination needs.

## Past Advisories

None at this time. This section will be updated when the first advisory is
published.
