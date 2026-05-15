#!/usr/bin/env node

import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { readJson, renderPacket, validatePacket } from './skillhub-packet.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DEFAULT_PACKET_DIR = path.resolve(__dirname, '../skillhub/fixtures');
const DEFAULT_HOST = process.env.SKILLHUB_HOST || '127.0.0.1';
const DEFAULT_PORT = Number(process.env.SKILLHUB_PORT || 18765);

function usage() {
  console.log(`Usage:
  skillhub-mock-api [--dir <packets-dir>] [--host <host>] [--port <port>]
  skillhub-mock-api --check [--dir <packets-dir>]`);
}

function parseArgs(args) {
  const options = {
    dir: DEFAULT_PACKET_DIR,
    host: DEFAULT_HOST,
    port: DEFAULT_PORT,
    check: false,
  };
  for (let i = 0; i < args.length; i += 1) {
    const item = args[i];
    if (item === '--dir') options.dir = path.resolve(args[++i]);
    else if (item === '--host') options.host = args[++i];
    else if (item === '--port') options.port = Number(args[++i]);
    else if (item === '--check') options.check = true;
    else if (item === '--help' || item === '-h') {
      usage();
      process.exit(0);
    } else {
      throw new Error(`unknown option: ${item}`);
    }
  }
  if (!Number.isInteger(options.port) || options.port <= 0) {
    throw new Error('port must be a positive integer');
  }
  return options;
}

function packetSummary(packet) {
  return {
    id: packet.id,
    name: packet.name,
    summary: packet.summary,
    visibility: packet.visibility,
    status: packet.status,
    version: packet.version,
    source: packet.source,
    domains: packet.domains,
    install_targets: packet.install_targets,
    evidence_refs: packet.evidence_refs,
    eval_score: packet.eval_score,
    rating: packet.rating,
    votes: packet.votes,
  };
}

function loadPackets(dir) {
  const files = fs
    .readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.json'))
    .map((entry) => path.join(dir, entry.name))
    .sort();

  const packets = new Map();
  for (const file of files) {
    const packet = readJson(file);
    const errors = validatePacket(packet);
    if (errors.length) {
      throw new Error(`${path.relative(process.cwd(), file)} invalid: ${errors.join('; ')}`);
    }
    if (packets.has(packet.id)) {
      throw new Error(`duplicate packet id: ${packet.id}`);
    }
    packets.set(packet.id, { file, packet });
  }
  return packets;
}

function listPackets(packets, url) {
  const target = url.searchParams.get('target');
  const domain = url.searchParams.get('domain');
  const includeBody = url.searchParams.get('include_body') === 'true';

  return [...packets.values()]
    .map(({ packet }) => packet)
    .filter((packet) => !target || packet.install_targets.includes(target))
    .filter((packet) => !domain || packet.domains.includes(domain))
    .map((packet) => (includeBody ? packet : packetSummary(packet)));
}

function sendJson(res, statusCode, payload) {
  const body = JSON.stringify(payload, null, 2);
  res.writeHead(statusCode, {
    'content-type': 'application/json; charset=utf-8',
    'cache-control': 'no-store',
  });
  res.end(`${body}\n`);
}

function sendText(res, statusCode, body) {
  res.writeHead(statusCode, {
    'content-type': 'text/markdown; charset=utf-8',
    'cache-control': 'no-store',
  });
  res.end(`${body}\n`);
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.setEncoding('utf8');
    req.on('data', (chunk) => {
      body += chunk;
      if (body.length > 2_000_000) {
        reject(new Error('request body too large'));
        req.destroy();
      }
    });
    req.on('end', () => resolve(body));
    req.on('error', reject);
  });
}

async function route(req, res, packets) {
  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'access-control-allow-origin': 'http://127.0.0.1',
      'access-control-allow-methods': 'GET,POST,OPTIONS',
      'access-control-allow-headers': 'content-type',
    });
    res.end();
    return;
  }

  const url = new URL(req.url || '/', 'http://skillhub.local');
  const segments = url.pathname.split('/').filter(Boolean).map(decodeURIComponent);

  if (req.method === 'GET' && url.pathname === '/health') {
    sendJson(res, 200, {
      ok: true,
      service: 'everme-skillhub-mock',
      packet_count: packets.size,
    });
    return;
  }

  if (req.method === 'GET' && segments.length === 1 && segments[0] === 'skills') {
    sendJson(res, 200, { ok: true, skills: listPackets(packets, url) });
    return;
  }

  if (req.method === 'GET' && segments.length === 2 && segments[0] === 'skills') {
    const item = packets.get(segments[1]);
    if (!item) {
      sendJson(res, 404, { ok: false, error: 'skill not found' });
      return;
    }
    sendJson(res, 200, { ok: true, skill: item.packet });
    return;
  }

  if (
    req.method === 'GET'
    && segments.length === 3
    && segments[0] === 'skills'
    && segments[2] === 'render'
  ) {
    const item = packets.get(segments[1]);
    if (!item) {
      sendJson(res, 404, { ok: false, error: 'skill not found' });
      return;
    }
    sendText(res, 200, renderPacket(item.packet));
    return;
  }

  if (req.method === 'POST' && url.pathname === '/skills/validate') {
    const body = await readBody(req);
    const packet = JSON.parse(body);
    const errors = validatePacket(packet);
    sendJson(res, errors.length ? 422 : 200, { ok: errors.length === 0, errors });
    return;
  }

  sendJson(res, 404, { ok: false, error: 'not found' });
}

function writeCheckResult(packets) {
  process.stdout.write(JSON.stringify({
    ok: true,
    service: 'everme-skillhub-mock',
    packet_count: packets.size,
    packet_ids: [...packets.keys()],
  }, null, 2) + '\n');
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const packets = loadPackets(options.dir);

  if (options.check) {
    writeCheckResult(packets);
    return;
  }

  const server = http.createServer((req, res) => {
    route(req, res, packets).catch((error) => {
      sendJson(res, 500, { ok: false, error: error.message });
    });
  });

  server.listen(options.port, options.host, () => {
    const address = server.address();
    process.stdout.write(JSON.stringify({
      ok: true,
      service: 'everme-skillhub-mock',
      url: `http://${address.address}:${address.port}`,
      packet_count: packets.size,
    }) + '\n');
  });
}

main();
