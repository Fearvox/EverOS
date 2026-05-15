#!/usr/bin/env node
import http from 'node:http';

const DEFAULT_PORT = Number.parseInt(process.env.MOCK_OPENAI_PORT || '18080', 10);
const DEFAULT_HOST = process.env.MOCK_OPENAI_HOST || '127.0.0.1';
const DEFAULT_DIMENSIONS = Number.parseInt(process.env.MOCK_OPENAI_DIMENSIONS || '1024', 10);

function parseArgs(argv) {
  const args = {
    host: DEFAULT_HOST,
    port: DEFAULT_PORT,
    check: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--host') {
      args.host = argv[i + 1];
      i += 1;
    } else if (arg === '--port') {
      args.port = Number.parseInt(argv[i + 1], 10);
      i += 1;
    } else if (arg === '--check') {
      args.check = true;
    } else if (arg === '-h' || arg === '--help') {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  if (!Number.isInteger(args.port) || args.port < 1) {
    throw new Error(`invalid port: ${args.port}`);
  }
  return args;
}

function printHelp() {
  console.log(`Usage: mock-openai-compatible.mjs [--host 127.0.0.1] [--port 18080] [--check]

Tiny local OpenAI-compatible mock for EverCore dogfood:
  GET  /health
  POST /v1/chat/completions
  POST /v1/embeddings
  POST /v1/rerank

No external network and no real model keys are used.`);
}

function stableHash(text) {
  let hash = 2166136261;
  for (let i = 0; i < text.length; i += 1) {
    hash ^= text.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

function embeddingFor(text, dimensions = DEFAULT_DIMENSIONS) {
  const vector = new Array(dimensions).fill(0);
  const tokens = String(text)
    .toLowerCase()
    .split(/[^a-z0-9_./:-]+/)
    .filter(Boolean);
  for (const token of tokens.length ? tokens : ['empty']) {
    const base = stableHash(token);
    for (let i = 0; i < 8; i += 1) {
      const idx = (base + i * 97) % dimensions;
      vector[idx] += 1 / (i + 1);
    }
  }
  const norm = Math.sqrt(vector.reduce((sum, value) => sum + value * value, 0)) || 1;
  return vector.map((value) => Number((value / norm).toFixed(6)));
}

function extractPromptText(body) {
  const messages = Array.isArray(body.messages) ? body.messages : [];
  return messages
    .map((message) => {
      if (typeof message.content === 'string') return message.content;
      if (Array.isArray(message.content)) {
        return message.content.map((part) => part.text || '').join('\n');
      }
      return '';
    })
    .join('\n');
}

function extractSmokeNeedle(prompt) {
  const match = prompt.match(/Hermes EverOS dogfood smoke Raven SkillHub \d+[^.\n]*/i);
  if (match) return match[0].trim();
  const ravenLine = prompt
    .split(/\r?\n/)
    .find((line) => /Raven|SkillHub|EverOS|Hermes/i.test(line));
  return (ravenLine || 'Hermes EverOS dogfood smoke Raven SkillHub').trim().slice(0, 500);
}

function completionContent(prompt) {
  if (/should_end|episode boundar/i.test(prompt)) {
    return JSON.stringify({
      should_end: false,
      reasoning: 'Single dogfood turn; keep accumulating until flush.',
      topic_summary: '',
    });
  }
  if (/worth_extracting/i.test(prompt)) {
    return JSON.stringify({
      worth_extracting: false,
      reason: 'Smoke fixture only',
    });
  }

  const needle = extractSmokeNeedle(prompt);
  return JSON.stringify({
    title: 'Hermes EverOS Raven SkillHub dogfood',
    content: `${needle}. Provider-level store, flush, extract, index, search, and recall smoke for Raven and EverMe SkillHub.`,
    summary: `${needle}. EverOS memory dogfood smoke.`,
  });
}

function chatResponse(body) {
  const prompt = extractPromptText(body);
  const content = completionContent(prompt);
  return {
    id: `chatcmpl-mock-${Date.now()}`,
    object: 'chat.completion',
    created: Math.floor(Date.now() / 1000),
    model: body.model || 'mock-openai-compatible',
    choices: [
      {
        index: 0,
        finish_reason: 'stop',
        message: {
          role: 'assistant',
          content,
        },
      },
    ],
    usage: {
      prompt_tokens: prompt.length,
      completion_tokens: content.length,
      total_tokens: prompt.length + content.length,
    },
  };
}

function embeddingsResponse(body) {
  const input = Array.isArray(body.input) ? body.input : [body.input ?? ''];
  const dimensions = Number.isInteger(body.dimensions) && body.dimensions > 0
    ? body.dimensions
    : DEFAULT_DIMENSIONS;
  return {
    object: 'list',
    data: input.map((item, index) => ({
      object: 'embedding',
      index,
      embedding: embeddingFor(String(item), dimensions),
    })),
    model: body.model || 'mock-embedding',
    usage: {
      prompt_tokens: input.join(' ').length,
      total_tokens: input.join(' ').length,
    },
  };
}

function rerankResponse(body) {
  const documents = Array.isArray(body.documents) ? body.documents : [];
  const queryTokens = new Set(
    String(body.query || '')
      .toLowerCase()
      .split(/[^a-z0-9_./:-]+/)
      .filter(Boolean),
  );
  const results = documents
    .map((document, index) => {
      const docTokens = String(document)
        .toLowerCase()
        .split(/[^a-z0-9_./:-]+/)
        .filter(Boolean);
      const overlap = docTokens.filter((token) => queryTokens.has(token)).length;
      return { index, relevance_score: Math.min(1, 0.55 + overlap / 20) };
    })
    .sort((a, b) => b.relevance_score - a.relevance_score);
  return { results };
}

function readJson(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (chunk) => chunks.push(chunk));
    req.on('end', () => {
      const raw = Buffer.concat(chunks).toString('utf8') || '{}';
      try {
        resolve(JSON.parse(raw));
      } catch (error) {
        reject(error);
      }
    });
    req.on('error', reject);
  });
}

function sendJson(res, status, payload) {
  const body = JSON.stringify(payload);
  res.writeHead(status, {
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(body),
  });
  res.end(body);
}

function createServer() {
  return http.createServer(async (req, res) => {
    try {
      const path = new URL(req.url || '/', 'http://localhost').pathname;
      if (req.method === 'GET' && path === '/health') {
        sendJson(res, 200, { status: 'ok' });
        return;
      }
      if (req.method !== 'POST') {
        sendJson(res, 404, { error: { message: 'not found' } });
        return;
      }

      const body = await readJson(req);
      if (path === '/v1/chat/completions' || path === '/chat/completions') {
        sendJson(res, 200, chatResponse(body));
      } else if (path === '/v1/embeddings' || path === '/embeddings') {
        sendJson(res, 200, embeddingsResponse(body));
      } else if (path === '/v1/rerank' || path === '/rerank') {
        sendJson(res, 200, rerankResponse(body));
      } else {
        sendJson(res, 404, { error: { message: `unknown path: ${path}` } });
      }
    } catch (error) {
      sendJson(res, 500, { error: { message: error.message } });
    }
  });
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.check) {
    console.log('PASS mock-openai-compatible syntax');
    return;
  }
  const server = createServer();
  server.listen(args.port, args.host, () => {
    console.log(`mock-openai-compatible listening host=${args.host} port=${args.port}`);
  });
}

main();
