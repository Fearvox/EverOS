#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

const DEFAULT_REPO = process.env.GH_REPO || "Fearvox/EverOS";
const DEFAULT_PATCH_BYTES = 120_000;
const MARKER = "muw-review-lane:v1";

function usage(exitCode = 0) {
  const text = `
Usage:
  node .github/scripts/muw-review-lane.mjs collect --pr <number> [--repo owner/name] [--out <dir>]
  node .github/scripts/muw-review-lane.mjs post --pr <number> --body-file <file> [--repo owner/name] [--force]

Collect creates a redacted PR evidence bundle and a Codex prompt.
Post publishes a MUW-formatted issue comment with an idempotency marker.
`;
  process.stderr.write(text.trimStart());
  process.exit(exitCode);
}

function parseArgs(argv) {
  const [command, ...rest] = argv;
  if (!command || command === "--help" || command === "-h") usage(0);

  const options = { command, repo: DEFAULT_REPO, force: false };
  for (let index = 0; index < rest.length; index += 1) {
    const arg = rest[index];
    if (arg === "--repo") options.repo = rest[++index];
    else if (arg === "--pr") options.pr = Number(rest[++index]);
    else if (arg === "--out") options.out = rest[++index];
    else if (arg === "--body-file") options.bodyFile = rest[++index];
    else if (arg === "--patch-bytes") options.patchBytes = Number(rest[++index]);
    else if (arg === "--force") options.force = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!["collect", "post"].includes(command)) usage(1);
  if (!Number.isInteger(options.pr) || options.pr <= 0) {
    throw new Error("--pr must be a positive pull request number");
  }
  return options;
}

function run(command, args, options = {}) {
  return execFileSync(command, args, {
    encoding: "utf8",
    input: options.input,
    env: process.env,
    stdio: ["pipe", "pipe", options.inheritStderr ? "inherit" : "pipe"],
    maxBuffer: 20 * 1024 * 1024,
  }).trim();
}

function gh(args, options = {}) {
  return run("gh", args, options);
}

function ghJson(args) {
  return JSON.parse(gh(args));
}

function redact(value) {
  return String(value ?? "")
    .replace(/github_pat_[A-Za-z0-9_]+/g, "[REDACTED_GITHUB_TOKEN]")
    .replace(/gh[pousr]_[A-Za-z0-9_]+/g, "[REDACTED_GITHUB_TOKEN]")
    .replace(/sk-[A-Za-z0-9_-]{20,}/g, "[REDACTED_API_KEY]")
    .replace(/xox[baprs]-[A-Za-z0-9-]+/g, "[REDACTED_SLACK_TOKEN]")
    .replace(/(Authorization:\s*Bearer\s+)[A-Za-z0-9._~+/=-]+/gi, "$1[REDACTED_TOKEN]")
    .replace(/(token=)[A-Za-z0-9._~+/=-]+/gi, "$1[REDACTED_TOKEN]");
}

function truncate(value, maxBytes) {
  const text = redact(value);
  const bytes = Buffer.byteLength(text, "utf8");
  if (bytes <= maxBytes) return text;
  return `${text.slice(0, maxBytes)}\n\n[TRUNCATED: ${bytes - maxBytes} bytes omitted]`;
}

function summarizeChecks(items = []) {
  if (!items.length) return "- No status checks reported.";
  return items
    .map((item) => {
      const name = item.name || item.context || item.workflowName || item.__typename;
      const state = item.conclusion || item.state || item.status || "unknown";
      const url = item.detailsUrl || item.targetUrl || "";
      return `- ${name}: ${state}${url ? ` (${url})` : ""}`;
    })
    .join("\n");
}

function summarizeFiles(files = []) {
  if (!files.length) return "- No changed files reported.";
  return files
    .map((file) => `- ${file.path} (+${file.additions}/-${file.deletions})`)
    .join("\n");
}

function summarizeReviews(reviews = []) {
  if (!reviews.length) return "- No reviews yet.";
  return reviews
    .map((review) => {
      const author = review.author?.login || "unknown";
      const body = truncate(review.body || "", 700).replace(/\n/g, " ");
      return `- ${review.submittedAt || "unknown"} ${author} ${review.state}: ${body}`;
    })
    .join("\n");
}

function summarizeComments(comments = []) {
  if (!comments.length) return "- No issue comments yet.";
  return comments
    .slice(-20)
    .map((comment) => {
      const author = comment.author?.login || "unknown";
      const body = truncate(comment.body || "", 700).replace(/\n/g, " ");
      return `- ${comment.createdAt || "unknown"} ${author}: ${body}`;
    })
    .join("\n");
}

function collect(options) {
  const outDir = resolve(
    options.out || join(tmpdir(), `muw-review-pr-${options.pr}`),
  );
  mkdirSync(outDir, { recursive: true });

  const fields = [
    "number",
    "title",
    "url",
    "author",
    "baseRefName",
    "headRefName",
    "headRefOid",
    "isDraft",
    "body",
    "labels",
    "mergeStateStatus",
    "statusCheckRollup",
    "files",
    "commits",
    "comments",
    "reviews",
  ].join(",");

  const pr = ghJson([
    "pr",
    "view",
    String(options.pr),
    "--repo",
    options.repo,
    "--json",
    fields,
  ]);
  const patch = gh([
    "pr",
    "diff",
    String(options.pr),
    "--repo",
    options.repo,
    "--patch",
  ]);

  const patchBytes = options.patchBytes || DEFAULT_PATCH_BYTES;
  const contextPath = join(outDir, `pr-${options.pr}-context.md`);
  const promptPath = join(outDir, `pr-${options.pr}-prompt.md`);
  const metadataPath = join(outDir, `pr-${options.pr}-metadata.json`);

  const context = [
    `# MUW Review Context: ${options.repo}#${pr.number}`,
    "",
    `- Title: ${pr.title}`,
    `- URL: ${pr.url}`,
    `- Author: ${pr.author?.login || "unknown"}`,
    `- Draft: ${pr.isDraft}`,
    `- Base: ${pr.baseRefName}`,
    `- Head: ${pr.headRefName}`,
    `- Head SHA: ${pr.headRefOid}`,
    `- Merge state: ${pr.mergeStateStatus || "unknown"}`,
    `- Labels: ${(pr.labels || []).map((label) => label.name).join(", ") || "none"}`,
    "",
    "## PR Body",
    "",
    truncate(pr.body || "_No body._", 12_000),
    "",
    "## Files",
    "",
    summarizeFiles(pr.files),
    "",
    "## Checks",
    "",
    summarizeChecks(pr.statusCheckRollup),
    "",
    "## Existing Reviews",
    "",
    summarizeReviews(pr.reviews),
    "",
    "## Recent Comments",
    "",
    summarizeComments(pr.comments),
    "",
    "## Patch",
    "",
    "```diff",
    truncate(patch, patchBytes),
    "```",
    "",
  ].join("\n");

  const prompt = [
    "You are reviewing a GitHub pull request using the MUW review contract.",
    "",
    "Read the context file listed below. Return only a GitHub-ready comment body.",
    "Do not include Markdown fences around the whole answer.",
    "",
    "Required shape:",
    "",
    "VERDICT: PASS / FLAG / BLOCK",
    "VERDICT_SUMMARY: three lines or fewer; what passed, what is risky, and the next action",
    "EVIDENCE:",
    "",
    "Rules:",
    "- Do not mark PASS from author summary alone.",
    "- Ground claims in files, checks, comments, or patch evidence.",
    "- Findings first, ordered by severity.",
    "- For each finding include severity, file/path, evidence, why it matters, and fix guidance.",
    "- If clean, include evidence checked and residual risk/test gap.",
    "- Keep the answer concise and public-safe.",
    "",
    `Context file: ${contextPath}`,
    "",
  ].join("\n");

  writeFileSync(contextPath, context);
  writeFileSync(promptPath, prompt);
  writeFileSync(
    metadataPath,
    `${JSON.stringify(
      {
        repo: options.repo,
        pr: pr.number,
        headRefOid: pr.headRefOid,
        contextPath,
        promptPath,
      },
      null,
      2,
    )}\n`,
  );

  process.stdout.write(
    [
      `context=${contextPath}`,
      `prompt=${promptPath}`,
      `metadata=${metadataPath}`,
      `head=${pr.headRefOid}`,
    ].join("\n") + "\n",
  );
}

function markerFor(options, headRefOid) {
  return `<!-- ${MARKER} repo=${options.repo} pr=${options.pr} head=${headRefOid} -->`;
}

function validateMuwBody(body) {
  const missing = ["VERDICT:", "VERDICT_SUMMARY:", "EVIDENCE:"].filter(
    (needle) => !body.includes(needle),
  );
  if (missing.length) {
    throw new Error(`Body is missing required MUW field(s): ${missing.join(", ")}`);
  }
}

function existingMarkerComment(options, marker) {
  const [owner, repo] = options.repo.split("/");
  const comments = ghJson([
    "api",
    `/repos/${owner}/${repo}/issues/${options.pr}/comments?per_page=100`,
  ]);
  return comments.find((comment) => String(comment.body || "").includes(marker));
}

function post(options) {
  if (!options.bodyFile) throw new Error("--body-file is required for post");
  const body = redact(readFileSync(resolve(options.bodyFile), "utf8")).trim();
  validateMuwBody(body);

  const pr = ghJson([
    "pr",
    "view",
    String(options.pr),
    "--repo",
    options.repo,
    "--json",
    "headRefOid,url",
  ]);
  const marker = markerFor(options, pr.headRefOid);
  const existing = existingMarkerComment(options, marker);
  if (existing && !options.force) {
    process.stdout.write(
      `skip=existing-comment\nurl=${existing.html_url}\nhead=${pr.headRefOid}\n`,
    );
    return;
  }

  const comment = `${marker}\n${body}\n`;
  const url = gh(
    ["pr", "comment", String(options.pr), "--repo", options.repo, "--body", comment],
    { inheritStderr: true },
  );
  process.stdout.write(`posted=${url}\nhead=${pr.headRefOid}\n`);
}

try {
  const options = parseArgs(process.argv.slice(2));
  if (options.command === "collect") collect(options);
  else if (options.command === "post") post(options);
} catch (error) {
  process.stderr.write(`muw-review-lane: ${error.message}\n`);
  process.exit(1);
}
