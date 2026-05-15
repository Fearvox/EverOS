#!/usr/bin/env node

const DEFAULT_BASE_URL = 'http://127.0.0.1:1995';

function config() {
  return {
    baseUrl: (process.env.EVEROS_API_BASE_URL || DEFAULT_BASE_URL).replace(/\/+$/, ''),
    userId: process.env.EVEROS_USER_ID || 'hermes-user',
    agentId: process.env.EVEROS_AGENT_ID || 'hermes',
    searchMethod: process.env.EVEROS_SEARCH_METHOD || 'hybrid',
    memoryTypes: (process.env.EVEROS_MEMORY_TYPES || 'episodic_memory,profile')
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
    topK: Number.parseInt(process.env.EVEROS_TOP_K || '5', 10),
  };
}

async function requestJson(path, options = {}) {
  const cfg = config();
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 10000);
  try {
    const response = await fetch(`${cfg.baseUrl}${path}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(options.headers || {}),
      },
      signal: controller.signal,
    });
    const text = await response.text();
    let data = null;
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        data = { raw: text };
      }
    }
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${JSON.stringify(data)}`);
    }
    return data;
  } finally {
    clearTimeout(timeout);
  }
}

async function health() {
  return requestJson('/health');
}

async function search(query) {
  const cfg = config();
  return requestJson('/api/v1/memories/search', {
    method: 'POST',
    body: JSON.stringify({
      query,
      method: cfg.searchMethod,
      memory_types: cfg.memoryTypes,
      top_k: Number.isFinite(cfg.topK) ? cfg.topK : 5,
      filters: {
        user_id: cfg.userId,
      },
    }),
  });
}

async function syncTurn(user, assistant) {
  const cfg = config();
  const now = Date.now();
  return requestJson('/api/v1/memories/agent', {
    method: 'POST',
    body: JSON.stringify({
      user_id: cfg.userId,
      session_id: `hermes-smoke-${now}`,
      messages: [
        {
          role: 'user',
          sender_id: cfg.userId,
          timestamp: now,
          content: user,
        },
        {
          role: 'assistant',
          sender_id: cfg.agentId,
          timestamp: now + 1,
          content: assistant,
        },
      ],
    }),
  });
}

function summarizeSearch(data) {
  const payload = data?.data || {};
  const episodes = Array.isArray(payload.episodes) ? payload.episodes : [];
  const profiles = Array.isArray(payload.profiles) ? payload.profiles : [];
  const cases = Array.isArray(payload.agent_memory?.cases) ? payload.agent_memory.cases : [];
  const skills = Array.isArray(payload.agent_memory?.skills) ? payload.agent_memory.skills : [];
  return {
    episodes: episodes.length,
    profiles: profiles.length,
    agent_cases: cases.length,
    agent_skills: skills.length,
  };
}

async function main() {
  const [command, ...args] = process.argv.slice(2);

  if (!command || command === 'help' || command === '--help' || command === '-h') {
    console.log(`Usage:
  everos-memory health
  everos-memory search <query>
  everos-memory sync-smoke
  everos-memory self-test`);
    return;
  }

  if (command === 'health') {
    const result = await health();
    console.log(JSON.stringify({ ok: true, status: result?.status || result?.data?.status || 'unknown' }, null, 2));
    return;
  }

  if (command === 'search') {
    const query = args.join(' ').trim();
    if (!query) throw new Error('search requires a query');
    const result = await search(query);
    console.log(JSON.stringify({ ok: true, ...summarizeSearch(result) }, null, 2));
    return;
  }

  if (command === 'sync-smoke') {
    const result = await syncTurn(
      'Hermes EverOS memory provider smoke test user message.',
      'Hermes EverOS memory provider smoke test assistant response.'
    );
    console.log(JSON.stringify({ ok: true, data: result?.data || null }, null, 2));
    return;
  }

  if (command === 'self-test') {
    const h = await health();
    console.log(JSON.stringify({ health: h?.status || 'ok' }));
    const s = await search('Hermes EverOS memory provider');
    console.log(JSON.stringify({ search: summarizeSearch(s) }));
    return;
  }

  throw new Error(`unknown command: ${command}`);
}

main().catch((error) => {
  console.error(JSON.stringify({ ok: false, error: error.message }));
  process.exit(1);
});

