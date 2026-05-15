#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const REQUIRED = [
  'id',
  'name',
  'summary',
  'visibility',
  'status',
  'version',
  'source',
  'domains',
  'install_targets',
  'evidence_refs',
  'body_markdown',
];

const ENUMS = {
  visibility: ['private', 'link', 'community'],
  status: ['draft', 'active', 'needs_eval', 'archived'],
  source: ['manual', 'evercore_extracted', 'imported', 'community'],
  install_targets: ['hermes', 'raven', 'claude_code', 'evercore', 'openclaw'],
};

function usage() {
  console.log(`Usage:
  skillhub-packet validate <packet.json>
  skillhub-packet render <packet.json>
  skillhub-packet views <packet.json>
  skillhub-packet from-skill <SKILL.md> [--domain <name>] [--target <name>] [--owner <id>]
  skillhub-packet sample`);
}

export function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

export function slugify(input) {
  return String(input || '')
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 120) || 'skill';
}

function findRepoRoot(startDir) {
  let dir = path.resolve(startDir);
  while (dir !== path.dirname(dir)) {
    if (fs.existsSync(path.join(dir, '.git'))) return dir;
    dir = path.dirname(dir);
  }
  return path.resolve(startDir);
}

function repoRelative(file) {
  const absolute = path.resolve(file);
  const root = findRepoRoot(process.cwd());
  return path.relative(root, absolute);
}

export function parseFrontmatter(markdown) {
  if (!markdown.startsWith('---\n')) {
    return { frontmatter: {}, body: markdown.trim() };
  }
  const end = markdown.indexOf('\n---', 4);
  if (end === -1) {
    return { frontmatter: {}, body: markdown.trim() };
  }
  const raw = markdown.slice(4, end).split(/\r?\n/);
  const body = markdown.slice(end + 4).trim();
  const frontmatter = {};
  let pendingKey = null;
  for (const line of raw) {
    if (!line.trim()) continue;
    if (/^\s+/.test(line) && pendingKey) {
      frontmatter[pendingKey] = `${frontmatter[pendingKey] || ''} ${line.trim()}`.trim();
      continue;
    }
    const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (!match) continue;
    const [, key, value] = match;
    pendingKey = key;
    if (value === '>') {
      frontmatter[key] = '';
    } else if (value === 'true' || value === 'false') {
      frontmatter[key] = value === 'true';
    } else {
      frontmatter[key] = value.replace(/^["']|["']$/g, '');
    }
  }
  return { frontmatter, body };
}

export function validatePacket(packet) {
  const errors = [];
  for (const field of REQUIRED) {
    if (!(field in packet)) errors.push(`missing ${field}`);
  }
  for (const field of ['id', 'name', 'summary', 'version', 'body_markdown']) {
    if (field in packet && typeof packet[field] !== 'string') errors.push(`${field} must be string`);
  }
  if (typeof packet.id === 'string' && !/^[a-z0-9][a-z0-9._-]{2,127}$/.test(packet.id)) {
    errors.push('id has invalid format');
  }
  if (typeof packet.version === 'string' && !/^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9._-]+)?$/.test(packet.version)) {
    errors.push('version must be semver-like');
  }
  for (const field of ['domains', 'install_targets', 'evidence_refs']) {
    if (field in packet && !Array.isArray(packet[field])) errors.push(`${field} must be array`);
  }
  for (const field of ['visibility', 'status', 'source']) {
    if (field in packet && !ENUMS[field].includes(packet[field])) errors.push(`${field} invalid`);
  }
  if (Array.isArray(packet.install_targets)) {
    for (const target of packet.install_targets) {
      if (!ENUMS.install_targets.includes(target)) errors.push(`install target invalid: ${target}`);
    }
  }
  return errors;
}

export function renderPacket(packet) {
  return [
    `# ${packet.name}`,
    '',
    `id: ${packet.id}`,
    `status: ${packet.status}`,
    `visibility: ${packet.visibility}`,
    `version: ${packet.version}`,
    `source: ${packet.source}`,
    `domains: ${packet.domains.join(', ')}`,
    `install_targets: ${packet.install_targets.join(', ')}`,
    `evidence_refs: ${packet.evidence_refs.length}`,
    '',
    packet.summary,
  ].join('\n');
}

function valueOrUnknown(value) {
  if (value === undefined || value === null || value === '') return 'unknown';
  return String(value);
}

function renderSkillViews(packet) {
  const evidence = packet.evidence_refs.length
    ? packet.evidence_refs.map((ref) => `- ${ref}`).join('\n')
    : '- no evidence refs yet';
  const score = typeof packet.eval_score === 'number' ? packet.eval_score.toFixed(2) : 'not scored';
  const rating = typeof packet.rating === 'number' ? packet.rating.toFixed(1) : 'not rated';
  const votes = Number.isInteger(packet.votes) ? packet.votes : 0;
  const lastEvolved = valueOrUnknown(packet.last_evolved_at);
  return [
    `# ${packet.name} SkillHub Views`,
    '',
    '## Skill Index Row',
    '',
    `- id: ${packet.id}`,
    `- status: ${packet.status}`,
    `- version: ${packet.version}`,
    `- domains: ${packet.domains.join(', ')}`,
    `- install targets: ${packet.install_targets.join(', ')}`,
    '',
    '## Skill Detail',
    '',
    packet.summary,
    '',
    packet.body_markdown.trim(),
    '',
    '## Evolution Queue',
    '',
    `- status: ${packet.status}`,
    `- eval score: ${score}`,
    `- last evolved: ${lastEvolved}`,
    `- next action: ${packet.status === 'needs_eval' ? 'run eval before promoting' : 'watch for fresh evidence'}`,
    '',
    '## Install Packet',
    '',
    `- compatible targets: ${packet.install_targets.join(', ')}`,
    `- source: ${packet.source}`,
    `- visibility: ${packet.visibility}`,
    `- version: ${packet.version}`,
    '',
    '## Trust Panel',
    '',
    `- rating: ${rating}`,
    `- votes: ${votes}`,
    `- evidence refs: ${packet.evidence_refs.length}`,
    evidence,
  ].join('\n');
}

export function packetFromSkill(file, options) {
  const markdown = fs.readFileSync(file, 'utf8');
  const { frontmatter, body } = parseFrontmatter(markdown);
  const name = String(frontmatter.name || path.basename(path.dirname(file)) || 'skill');
  return {
    id: `${slugify(options.owner || 'everme-local')}.${slugify(name)}`,
    name,
    summary: String(frontmatter.description || name).replace(/\s+/g, ' ').trim(),
    owner_id: options.owner || 'everme-local',
    visibility: 'private',
    status: 'needs_eval',
    version: '0.1.0',
    source: 'evercore_extracted',
    domains: [options.domain || inferDomain(file)],
    install_targets: [options.target || 'hermes'],
    evidence_refs: [repoRelative(file)],
    body_markdown: body || markdown.trim(),
    frontmatter,
  };
}

function inferDomain(file) {
  const parts = file.split(path.sep);
  const idx = parts.indexOf('skills_sample');
  if (idx >= 0 && parts[idx + 1]) return parts[idx + 1].toLowerCase();
  return 'general';
}

function parseOptions(args) {
  const options = {};
  for (let i = 0; i < args.length; i += 1) {
    const item = args[i];
    if (item === '--domain') options.domain = args[++i];
    else if (item === '--target') options.target = args[++i];
    else if (item === '--owner') options.owner = args[++i];
    else throw new Error(`unknown option: ${item}`);
  }
  return options;
}

function main() {
  const [command, file, ...rest] = process.argv.slice(2);
  if (!command || command === '--help' || command === '-h') {
    usage();
    return;
  }
  if (command === 'sample') {
    const sample = new URL('../skillhub/fixtures/raven-skillhub-sample.json', import.meta.url);
    console.log(fs.readFileSync(sample, 'utf8'));
    return;
  }
  if (command === 'validate') {
    if (!file) throw new Error('validate requires packet.json');
    const packet = readJson(file);
    const errors = validatePacket(packet);
    if (errors.length) {
      console.error(JSON.stringify({ ok: false, errors }, null, 2));
      process.exit(1);
    }
    console.log(JSON.stringify({ ok: true, id: packet.id, targets: packet.install_targets }, null, 2));
    return;
  }
  if (command === 'render') {
    if (!file) throw new Error('render requires packet.json');
    const packet = readJson(file);
    const errors = validatePacket(packet);
    if (errors.length) {
      console.error(JSON.stringify({ ok: false, errors }, null, 2));
      process.exit(1);
    }
    console.log(renderPacket(packet));
    return;
  }
  if (command === 'views') {
    if (!file) throw new Error('views requires packet.json');
    const packet = readJson(file);
    const errors = validatePacket(packet);
    if (errors.length) {
      console.error(JSON.stringify({ ok: false, errors }, null, 2));
      process.exit(1);
    }
    console.log(renderSkillViews(packet));
    return;
  }
  if (command === 'from-skill') {
    if (!file) throw new Error('from-skill requires SKILL.md');
    const packet = packetFromSkill(file, parseOptions(rest));
    const errors = validatePacket(packet);
    if (errors.length) {
      console.error(JSON.stringify({ ok: false, errors }, null, 2));
      process.exit(1);
    }
    console.log(JSON.stringify(packet, null, 2));
    return;
  }
  throw new Error(`unknown command: ${command}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
