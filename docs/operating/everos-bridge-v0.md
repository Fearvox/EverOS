# EverOS Project Lead Bridge v0 — Workbench Admin Operating Contract

**Source:** [DAS-2660](mention://issue/563b1b34-797d-4904-9c74-4a83ea82fb61) (C1 of 5) under parent packet [DAS-2658](mention://issue/b306ddde-6d5e-4414-af49-5bdff59b6ee8). Supervisor gate [DAS-2659](mention://issue/dee8c53e-c00b-4a20-a4cc-a8da8865299a) PASS (comment 42b14640).
**Version:** v0 — interim. Re-validation required before the 6th EverOS-bound packet or any cadence/automation change.
**Scope:** governs MUW-side intake, routing, and closeout of EverOS-bound work. Does NOT govern upstream EverMind-AI repo, upstream collaborators, or surfaces outside the EverOS project's resource list.

---

## 1. Interim Ownership (v0)
- **Project:** EverOS — id `6a297bc4-a109-49cc-a309-a6d625e2ad51`, workspace `5470ee5d-0791-4713-beb4-fd6a187d6523`.
- **Interim project lead:** Workbench Admin (agent id `5fb626ce-488c-44cd-81c1-0cfb3ea26bce`), assigned by operator decision in [DAS-2658](mention://issue/b306ddde-6d5e-4414-af49-5bdff59b6ee8).
- **Reviewer of record:** Workbench Supervisor (agent id `4e19cffb-1abe-461a-9026-eeb7155668d1`).
- The platform `project.lead_id` remains `null` — this is a **contractual appointment**, not a platform mutation. No `multica project update` is invoked.
- Admin holds the lead only for v0 (first 5 EverOS-bound packets and the cockpit shakedown). Re-appointment or handoff happens via a Supervisor-reviewed packet, not silently.

## 2. Intake Path: MUW → EverOS Cockpit
Triggers that route an issue to the EverOS project:
1. Body / parent / source-link references EverOS, Fearvox/EverOS fork, or an upstream EverMind-AI/EverOS PR/issue.
2. Operator explicitly requests an EverOS lane.
3. A cross-system convergence packet (e.g. [DAS-2658](mention://issue/b306ddde-6d5e-4414-af49-5bdff59b6ee8)) assigns EverOS scope to a child task.

Routing rules:
- Workbench Admin owns first-touch: confirms scope, picks one owner, sets `project_id` to EverOS, sets parent if applicable.
- One owner per issue. `@mention`s only for parallel advice / review / independent research — never for acknowledgement (per CLAUDE.md mention discipline).
- Non-EverOS personal/comms intake stays out of this project. The bridge is bounded to EverOS-bound work only.

Required at create-time (every EverOS-bound issue):
- `INTENT` (single paragraph), `EXPECTED_ARTIFACT`.
- `PROJECT: EverOS`.
- `OWNER` — one Workbench agent.
- `REVIEWER: Workbench Supervisor`.
- `SOURCE_LINK` — parent / packet / upstream URL.
- `GITHUB_ASSOCIATION` block (see §3).

## 3. Required GitHub Association Fields
Every EverOS-bound issue carries a `GITHUB_ASSOCIATION` block. v0 fields:
```
GITHUB_ASSOCIATION
repo_primary: https://github.com/Fearvox/EverOS.git
repo_secondary: https://github.com/EverMind-AI/EverOS.git   # only if upstream-touching
pr: <https URL | none yet>
branch: <branch-name | none yet>
commit: <short SHA | none yet>
github_issue: <https URL | none yet>
association_required_before_pass: yes
```
Rules:
- `repo_primary` is Fearvox/EverOS (fork playground) by default. If the issue is an upstream-only review, `repo_primary` becomes EverMind-AI/EverOS and the body MUST state why.
- "none yet" is acceptable during `in_progress`. It is NOT acceptable in the final PASS comment unless the closeout includes an explicit no-repo-change rationale and Supervisor accepts it on review.
- Fork-side branch convention: `workbench/<DAS-id>-<slug>` so Linear/MUW autolinks resolve cleanly.
- Upstream EverMind-AI/EverOS settings, branches, and PRs are off-limits without separate, explicit human approval. The bridge does not unilaterally mutate upstream.

## 4. Closeout Evidence Requirements
Final closeout comment MUST contain:
- `SOURCE` — this DAS issue + parent packet.
- `LIVE_STATE_CHECKED` — which `multica` reads / repo commands / public URLs were run, with timestamps.
- `GITHUB_ASSOCIATION` — fully populated (repo + branch + commit + PR), OR `no-repo-change rationale: <reason>` (Supervisor must explicitly accept on review).
- `ARTIFACT` — link or inline artifact body.
- `VERIFICATION` — command output, file path, screenshot, link, or `missing-verification: <reason>`.
- `PUBLIC_SAFETY` — explicit confirmation per §6.
- `REMAINING_RISK` — short list of follow-ups or "none".
- `VERDICT` — `PASS` | `FLAG` | `BLOCK`.

Anti-laundering:
- A FLAG or BLOCK from prior comments cannot be silently rewritten to PASS in a later comment. The Supervisor sees the chain.
- "Looks good" or "works for me" without command output, file path, or repo evidence is not PASS. Owner must downgrade their own verdict if evidence is missing.

## 5. Handoff Path to Supervisor Review
1. Owner finishes work, posts the §4 closeout comment, flips issue to `in_review`.
2. A Bounded Supervisor Review Gate run (pattern from DAS-2659) picks the target up; reviewer re-reads evidence before issuing a verdict.
3. **PASS** → issue moves to `done`.
4. **FLAG** → issue stays `in_review`; Admin or owner posts a bounded next-action comment, then re-submits.
5. **BLOCK** → issue moves to `blocked`; Admin escalates to operator with a one-paragraph reason.
6. Admin's role at review time: confirm the handoff path was honored (correct project, correct reviewer, full GITHUB_ASSOCIATION or accepted rationale). Admin does NOT re-grade the owner's technical evidence. Supervisor verdict is authoritative.

## 6. Public-Safety & No-Secret Boundaries
Mandatory in every EverOS-bound artifact:
- No raw secrets, API keys, OAuth tokens, PATs, MCP credentials, or provider tokens.
- No private host/IP values, SSH targets, or tmux session names that map to operator infra.
- No raw transcripts of operator-only chat; no private email, Slack, Gmail, or partner names not already public.
- No raw screenshots that include private surfaces (mailbox, calendar, internal tabs).
- Public artifacts (README/docs in fork) must pass a manual public-surface scan before claiming PASS — redact local paths, operator-only commands, and personal identifiers.
- If a fork-side doc may be pulled upstream, treat it as public-surface from the moment of commit.

Out of scope for the v0 bridge:
- No autopilot reactivation for the EverOS lane.
- No skill mutation (e.g., `workbench-closeout-validator` is not edited from this contract).
- No upstream EverMind-AI repo setting changes.
- No personal/comms intake beyond what this bridge gates.

---

## Evidence (live readback at close time)
- `multica issue get 563b1b34-797d-4904-9c74-4a83ea82fb61 --output json` — this issue, status flipped todo → in_progress, parent confirmed `b306ddde-...`.
- `multica issue get b306ddde-6d5e-4414-af49-5bdff59b6ee8 --output json` — parent packet DAS-2658 read; OPERATOR_DECISIONS_APPLIED confirms Admin as interim EverOS lead v0.
- `multica issue get DAS-2659 --output json` + `multica issue comment list DAS-2659 --output json` — Supervisor PASS comment 42b14640 located and read.
- `multica project list --output json` — EverOS project id `6a297bc4-...`, `lead_id: null`, `resource_count: 2`, `issue_count: 0`, status `in_progress`.
- `multica project resource list 6a297bc4-a109-49cc-a309-a6d625e2ad51 --output json` — resource `fa4f0aa8-...` = `https://github.com/Fearvox/EverOS.git`; resource `8e3d6ac2-...` = `https://github.com/EverMind-AI/EverOS.git`.
- `multica issue get DAS-2661 --output json` — sibling C2 (evidence closeout template) confirmed as separate Markdown-only track; this contract references but does not duplicate.

## GITHUB_ASSOCIATION (this issue)
```
GITHUB_ASSOCIATION
repo_primary: https://github.com/Fearvox/EverOS.git
repo_secondary: https://github.com/EverMind-AI/EverOS.git
pr: https://github.com/Fearvox/EverOS/pull/27
branch: workbench/DAS-2660-everos-bridge-v0
commit: 6636370af46be800d05e63e7812e0435484479da
github_issue: none yet
association_required_before_pass: yes
no-repo-change rationale:
```

## Public-Safety Check (this comment)
No secrets, OAuth tokens, PATs, private host/IPs, SSH/tmux targets, raw transcripts, raw screenshots, or operator-only paths included. UUIDs are workspace-internal identifiers already visible to all workspace members; DAS identifiers and `https://github.com/...` URLs are the public/canonical refs.

## Remaining Risk
- Contract is repo-current on this branch/PR, but not yet merged into `main`; a future agent reading only the default branch will not see it until PR #27 lands.
- No SLA on Supervisor review latency stated in v0; if review stalls, EverOS-bound work pauses at intake.
- `project.lead_id` stays `null` in the platform — the contractual lead is invisible to anyone reading the project model without reading this contract.

## Optional Follow-Ups
1. Merge PR #27 so this contract becomes visible on the fork default branch (`main`).
2. If the operator prefers role-name redaction, land a follow-up that replaces UUIDs with role labels while preserving traceability elsewhere.
3. If the platform later exposes an approved project-lead mutation path, decide whether to mirror the contractual appointment into `project.lead_id`.
4. Extend the contract to template/issue-form glue only in a separate reviewed follow-up; still out of scope for v0.

## Verdict
**PASS** — contract artifact delivered with the six required sections, live-Multica evidence backing every claim, and repo-current branch/commit/PR association populated in this published copy. Remaining risk is now about merge state and future governance follow-ups, not whether the contract has landed in the repo at all.
