#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const config = {
  repoOwner: process.env.REPO_OWNER || "Fearvox",
  repoName: process.env.REPO_NAME || "EverOS",
  upstreamRepo: process.env.UPSTREAM_REPO || "EverMind-AI/EverOS",
  watchBranch:
    process.env.WATCH_BRANCH || "codex-watch-overnight-2026-05-13",
  ownerTimezone: process.env.OWNER_TIMEZONE || "America/Los_Angeles",
  linearTeamId:
    process.env.LINEAR_TEAM_ID || "233391d6-ec9e-4aa8-b534-16a221b8119a",
  linearProjectId:
    process.env.LINEAR_PROJECT_ID || "39aa3865-345c-4313-9dc0-ab3b509c5d21",
  createTrackingIssue:
    (process.env.CREATE_TRACKING_ISSUE || "false").toLowerCase() === "true",
  githubToken: process.env.GITHUB_TOKEN || process.env.GH_TOKEN || "",
  linearApiKey: process.env.LINEAR_API_KEY || "",
};

const repoSlug = `${config.repoOwner}/${config.repoName}`;
const recentWindowHours = Number(process.env.WATCH_WINDOW_HOURS || 24);
const since = new Date(Date.now() - recentWindowHours * 60 * 60 * 1000);
const now = new Date();

function run(command, args, options = {}) {
  const result = execFileSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", options.inheritStderr ? "inherit" : "pipe"],
    env: {
      ...process.env,
      GH_TOKEN: config.githubToken || process.env.GH_TOKEN,
    },
  });
  return result.trim();
}

function tryRun(command, args, fallback = "") {
  try {
    return run(command, args);
  } catch (error) {
    return fallback;
  }
}

function ghJson(endpoint) {
  const output = run("gh", [
    "api",
    "-H",
    "Accept: application/vnd.github+json",
    endpoint,
  ]);
  return JSON.parse(output);
}

function ensureRemote(name, url) {
  const remotes = tryRun("git", ["remote"]).split("\n").filter(Boolean);
  if (!remotes.includes(name)) {
    run("git", ["remote", "add", name, url]);
  }
}

function fetchGitState() {
  ensureRemote("upstream", `https://github.com/${config.upstreamRepo}.git`);
  tryRun("git", ["fetch", "origin", "--prune"], "");
  tryRun("git", ["fetch", "upstream", "main", "--prune"], "");

  const forkHead = tryRun("git", ["rev-parse", "--short", "origin/main"], "unknown");
  const upstreamHead = tryRun(
    "git",
    ["rev-parse", "--short", "upstream/main"],
    "unknown",
  );
  const counts = tryRun(
    "git",
    ["rev-list", "--left-right", "--count", "origin/main...upstream/main"],
    "0\t0",
  )
    .split(/\s+/)
    .map((value) => Number(value));

  const watchBranchSha = tryRun(
    "git",
    ["ls-remote", "--heads", "origin", config.watchBranch],
    "",
  )
    .split(/\s+/)[0]
    ?.slice(0, 12);

  return {
    forkHead,
    upstreamHead,
    forkAhead: counts[0] || 0,
    forkBehind: counts[1] || 0,
    watchBranchSha: watchBranchSha || "",
  };
}

function fetchGitHubState() {
  const runs = ghJson(
    `/repos/${repoSlug}/actions/runs?per_page=50&status=completed`,
  ).workflow_runs || [];
  const failedRuns = runs
    .filter((runInfo) => new Date(runInfo.created_at) >= since)
    .filter((runInfo) =>
      ["failure", "cancelled", "timed_out"].includes(runInfo.conclusion),
    )
    .slice(0, 10);

  const upstreamPulls = ghJson(
    `/repos/${config.upstreamRepo}/pulls?state=open&sort=updated&direction=desc&per_page=30`,
  )
    .filter((pull) => new Date(pull.updated_at) >= since)
    .slice(0, 10);

  const forkPulls = ghJson(
    `/repos/${repoSlug}/pulls?state=open&sort=updated&direction=desc&per_page=30`,
  )
    .filter((pull) => new Date(pull.updated_at) >= since)
    .slice(0, 10);

  return { failedRuns, upstreamPulls, forkPulls };
}

function lineForPull(pull) {
  return `- #${pull.number} ${pull.title} (${pull.user.login}, updated ${pull.updated_at})`;
}

function lineForRun(runInfo) {
  return `- ${runInfo.name}: ${runInfo.conclusion} (${runInfo.html_url})`;
}

function renderReport(gitState, githubState) {
  const findings = [];

  if (!gitState.watchBranchSha) {
    findings.push(`Watch branch is not on origin: ${config.watchBranch}`);
  }
  if (gitState.forkBehind > 0) {
    findings.push(`Fork main is behind upstream/main by ${gitState.forkBehind} commit(s).`);
  }
  if (githubState.failedRuns.length > 0) {
    findings.push(
      `${githubState.failedRuns.length} completed workflow run(s) failed in the last ${recentWindowHours}h.`,
    );
  }

  const verdict = findings.length > 0 ? "FLAG" : "PASS";
  const localNow = now.toLocaleString("en-US", {
    timeZone: config.ownerTimezone,
    hour12: false,
  });

  return {
    verdict,
    body: [
      `# Overnight Fork Watch: ${verdict}`,
      "",
      `Generated: ${now.toISOString()} (${config.ownerTimezone}: ${localNow})`,
      `Repository: ${repoSlug}`,
      `Upstream: ${config.upstreamRepo}`,
      `Watch branch: \`${config.watchBranch}\``,
      "",
      "## Drift",
      "",
      `- origin/main: \`${gitState.forkHead}\``,
      `- upstream/main: \`${gitState.upstreamHead}\``,
      `- fork ahead: ${gitState.forkAhead}`,
      `- fork behind: ${gitState.forkBehind}`,
      `- watch branch on origin: ${
        gitState.watchBranchSha ? `yes (\`${gitState.watchBranchSha}\`)` : "no"
      }`,
      "",
      "## Findings",
      "",
      findings.length ? findings.map((item) => `- ${item}`).join("\n") : "- None.",
      "",
      `## Fork Workflow Failures (${recentWindowHours}h)`,
      "",
      githubState.failedRuns.length
        ? githubState.failedRuns.map(lineForRun).join("\n")
        : "- None.",
      "",
      `## Upstream PRs Updated (${recentWindowHours}h)`,
      "",
      githubState.upstreamPulls.length
        ? githubState.upstreamPulls.map(lineForPull).join("\n")
        : "- None.",
      "",
      `## Fork PRs Updated (${recentWindowHours}h)`,
      "",
      githubState.forkPulls.length
        ? githubState.forkPulls.map(lineForPull).join("\n")
        : "- None.",
      "",
      "## Operator Notes",
      "",
      "- This issue is safe for public tracking: no local paths, host/IP values, or secrets are included.",
      "- A GitHub issue created by `GITHUB_TOKEN` does not trigger secondary workflows, so this watch mirrors to Linear directly when `LINEAR_API_KEY` is available.",
    ].join("\n"),
  };
}

function ensureLabel(name, color, description) {
  tryRun("gh", [
    "label",
    "create",
    name,
    "--repo",
    repoSlug,
    "--color",
    color,
    "--description",
    description,
  ]);
}

function issueHasLinearMarker(issueNumber) {
  const comments = ghJson(`/repos/${repoSlug}/issues/${issueNumber}/comments?per_page=100`);
  return comments.some((comment) => comment.body.includes("Linear:"));
}

async function mirrorToLinear(issueNumber, title, body) {
  if (!config.linearApiKey || issueHasLinearMarker(issueNumber)) {
    return;
  }

  const mutation = `
    mutation IssueCreate($input: IssueCreateInput!) {
      issueCreate(input: $input) {
        success
        issue { id identifier url }
      }
    }
  `;

  const response = await fetch("https://api.linear.app/graphql", {
    method: "POST",
    headers: {
      Authorization: config.linearApiKey,
      "Content-Type": "application/json",
      "x-apollo-operation-name": "IssueCreate",
    },
    body: JSON.stringify({
      query: mutation,
      variables: {
        input: {
          title,
          description: [
            `**Source**: https://github.com/${repoSlug}/issues/${issueNumber}`,
            "",
            "---",
            "",
            body,
          ].join("\n"),
          teamId: config.linearTeamId,
          projectId: config.linearProjectId,
          priority: 3,
        },
      },
    }),
  });

  const data = await response.json();
  if (!response.ok || data.errors || !data?.data?.issueCreate?.success) {
    ensureLabel("sync-failed", "D93F0B", "Linear sync workflow failed for this issue");
    tryRun("gh", [
      "issue",
      "edit",
      String(issueNumber),
      "--repo",
      repoSlug,
      "--add-label",
      "sync-failed",
    ]);
    throw new Error(`Linear API error: ${JSON.stringify(data)}`);
  }

  const linearIssue = data.data.issueCreate.issue;
  const marker = `Linear: [${linearIssue.identifier}](${linearIssue.url})\n\n_Auto-created by overnight-watch._`;
  run("gh", ["issue", "comment", String(issueNumber), "--repo", repoSlug, "--body", marker]);
}

function findExistingIssue(title) {
  const issues = JSON.parse(
    run("gh", [
      "issue",
      "list",
      "--repo",
      repoSlug,
      "--state",
      "open",
      "--label",
      "overnight-watch",
      "--json",
      "number,title",
      "--limit",
      "20",
    ]),
  );
  return issues.find((issue) => issue.title === title);
}

async function createOrUpdateTrackingIssue(report) {
  if (!config.createTrackingIssue || report.verdict === "PASS") {
    return;
  }

  ensureLabel("overnight-watch", "1D76DB", "Automated overnight fork watch");
  ensureLabel("tracking", "5319E7", "Long-lived tracking item");
  ensureLabel("pr-mirror", "0E8A16", "Mirrored into Linear or Slack tracking");

  const date = now.toISOString().slice(0, 10);
  const title = `[watch] Overnight fork patrol: ${date}`;
  const existing = findExistingIssue(title);
  const tempDir = mkdtempSync(join(tmpdir(), "everos-watch-"));
  const bodyFile = join(tempDir, "body.md");
  writeFileSync(bodyFile, report.body);

  if (existing) {
    run("gh", [
      "issue",
      "comment",
      String(existing.number),
      "--repo",
      repoSlug,
      "--body-file",
      bodyFile,
    ]);
    await mirrorToLinear(existing.number, title, report.body);
    return;
  }

  const created = run("gh", [
    "issue",
    "create",
    "--repo",
    repoSlug,
    "--title",
    title,
    "--body-file",
    bodyFile,
    "--label",
    "overnight-watch",
    "--label",
    "tracking",
    "--label",
    "pr-mirror",
  ]);
  const match = created.match(/\/issues\/(\d+)/);
  if (match) {
    await mirrorToLinear(Number(match[1]), title, report.body);
  }
}

async function main() {
  const gitState = fetchGitState();
  const githubState = fetchGitHubState();
  const report = renderReport(gitState, githubState);
  console.log(report.body);
  await createOrUpdateTrackingIssue(report);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
