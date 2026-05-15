#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';

const REQUIRED = [
  'id',
  'title',
  'goal',
  'status',
  'owners',
  'memory_providers',
  'lanes',
  'gates',
  'artifacts',
  'evidence_refs',
  'next_actions',
];

const ENUMS = {
  status: ['captured', 'dispatching', 'executing', 'reviewing', 'done', 'blocked'],
  owner: ['codex', 'pi', 'opencode', 'hermes', 'muw', 'human'],
  mutation_policy: ['read_only', 'local_only', 'external_requires_approval'],
  lane_verdict: ['pass', 'flag', 'block', 'active'],
  gate_status: ['pass', 'flag', 'block', 'not_run'],
};

function usage() {
  console.log(`Usage:
  raven-run validate <run.json>
  raven-run render <run.json>
  raven-run verify <run.json>
  raven-run summary <run.json>
  raven-run sample`);
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function validateRun(run) {
  const errors = [];
  for (const field of REQUIRED) {
    if (!(field in run)) errors.push(`missing ${field}`);
  }
  for (const field of ['id', 'title', 'goal', 'status']) {
    if (field in run && typeof run[field] !== 'string') errors.push(`${field} must be string`);
  }
  if (typeof run.id === 'string' && !/^[a-z0-9][a-z0-9._-]{2,127}$/.test(run.id)) {
    errors.push('id has invalid format');
  }
  if (run.status && !ENUMS.status.includes(run.status)) errors.push('status invalid');

  for (const field of ['owners', 'memory_providers', 'lanes', 'gates', 'artifacts', 'evidence_refs', 'next_actions']) {
    if (field in run && !Array.isArray(run[field])) errors.push(`${field} must be array`);
  }

  if (Array.isArray(run.owners)) {
    for (const owner of run.owners) {
      if (!ENUMS.owner.includes(owner)) errors.push(`owner invalid: ${owner}`);
    }
  }

  if (Array.isArray(run.lanes)) {
    for (const lane of run.lanes) {
      if (!lane.id) errors.push('lane missing id');
      if (!ENUMS.owner.includes(lane.owner)) errors.push(`lane owner invalid: ${lane.owner}`);
      if (!ENUMS.mutation_policy.includes(lane.mutation_policy)) {
        errors.push(`lane mutation policy invalid: ${lane.mutation_policy}`);
      }
      if (!ENUMS.lane_verdict.includes(lane.verdict)) {
        errors.push(`lane verdict invalid: ${lane.verdict}`);
      }
    }
  }

  if (Array.isArray(run.gates)) {
    for (const gate of run.gates) {
      if (!gate.id) errors.push('gate missing id');
      if (!gate.name) errors.push(`gate missing name: ${gate.id || 'unknown'}`);
      if (!ENUMS.gate_status.includes(gate.status)) errors.push(`gate status invalid: ${gate.status}`);
      if (typeof gate.blocks_completion !== 'boolean') {
        errors.push(`gate blocks_completion must be boolean: ${gate.id || 'unknown'}`);
      }
    }
  }

  return errors;
}

function verdict(run) {
  const lanes = Array.isArray(run.lanes) ? run.lanes : [];
  const gates = Array.isArray(run.gates) ? run.gates : [];

  if (lanes.some((lane) => lane.verdict === 'block')) return 'BLOCK';
  if (gates.some((gate) => gate.blocks_completion && gate.status === 'block')) return 'BLOCK';
  if (lanes.some((lane) => lane.verdict === 'flag' || lane.verdict === 'active')) return 'FLAG';
  if (gates.some((gate) => gate.blocks_completion && ['flag', 'not_run'].includes(gate.status))) return 'FLAG';
  return 'PASS';
}

function table(rows) {
  return rows.map((row) => `| ${row.join(' | ')} |`).join('\n');
}

function renderRun(run) {
  const laneRows = [
    ['Lane', 'Owner', 'Policy', 'Verdict', 'Scope'],
    ['---', '---', '---', '---', '---'],
    ...run.lanes.map((lane) => [
      lane.id,
      lane.owner,
      lane.mutation_policy,
      lane.verdict.toUpperCase(),
      lane.scope,
    ]),
  ];
  const gateRows = [
    ['Gate', 'Status', 'Blocks', 'Evidence'],
    ['---', '---', '---', '---'],
    ...run.gates.map((gate) => [
      gate.name,
      gate.status.toUpperCase(),
      gate.blocks_completion ? 'yes' : 'no',
      gate.evidence,
    ]),
  ];
  const artifactRows = [
    ['Artifact', 'Public Safe', 'Purpose'],
    ['---', '---', '---'],
    ...run.artifacts.map((artifact) => [
      artifact.path,
      artifact.public_safe ? 'yes' : 'no',
      artifact.purpose,
    ]),
  ];

  return [
    `# ${run.title}`,
    '',
    `VERDICT: ${verdict(run)}.`,
    '',
    `Status: ${run.status}`,
    `Run id: ${run.id}`,
    `Owners: ${run.owners.join(', ')}`,
    `Memory providers: ${run.memory_providers.join(', ')}`,
    '',
    '## Goal',
    '',
    run.goal,
    '',
    '## Lanes',
    '',
    table(laneRows),
    '',
    '## Gates',
    '',
    table(gateRows),
    '',
    '## Artifacts',
    '',
    table(artifactRows),
    '',
    '## Next',
    '',
    ...run.next_actions.map((action) => `- ${action}`),
  ].join('\n');
}

function summary(run) {
  return {
    ok: true,
    id: run.id,
    status: run.status,
    verdict: verdict(run),
    lanes: run.lanes.length,
    gates: run.gates.length,
    blocking_gates_open: run.gates.filter(
      (gate) => gate.blocks_completion && gate.status !== 'pass',
    ).map((gate) => gate.id),
  };
}

function renderGateVerification(run) {
  const gateRows = [
    ['Gate', 'Status', 'Blocks', 'Command', 'Evidence'],
    ['---', '---', '---', '---', '---'],
    ...run.gates.map((gate) => [
      gate.id,
      gate.status.toUpperCase(),
      gate.blocks_completion ? 'yes' : 'no',
      gate.command || 'manual',
      gate.evidence,
    ]),
  ];
  return [
    `VERDICT: ${verdict(run)}.`,
    '',
    table(gateRows),
    '',
    `Blocking gates open: ${summary(run).blocking_gates_open.join(', ') || 'none'}`,
  ].join('\n');
}

function main() {
  const [command, file] = process.argv.slice(2);
  if (!command || command === '--help' || command === '-h') {
    usage();
    return;
  }
  if (command === 'sample') {
    const sample = new URL('../raven/fixtures/doomsday-run.json', import.meta.url);
    console.log(fs.readFileSync(sample, 'utf8'));
    return;
  }
  if (!file) throw new Error(`${command} requires run.json`);
  const run = readJson(file);
  const errors = validateRun(run);
  if (errors.length) {
    console.error(JSON.stringify({ ok: false, errors }, null, 2));
    process.exit(1);
  }
  if (command === 'validate') {
    console.log(JSON.stringify(summary(run), null, 2));
    return;
  }
  if (command === 'summary') {
    console.log(JSON.stringify(summary(run), null, 2));
    return;
  }
  if (command === 'render') {
    console.log(renderRun(run));
    return;
  }
  if (command === 'verify') {
    console.log(renderGateVerification(run));
    const result = verdict(run);
    if (result === 'BLOCK') process.exit(2);
    if (result === 'FLAG') process.exit(1);
    return;
  }
  throw new Error(`unknown command: ${command}`);
}

main();
