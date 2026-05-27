# Dependabot Alert Audit — Fearvox/EverOS

**Date**: 2026-05-13T06:47Z
**Snapshot**: 127 total alerts (64 open, 63 fixed)
**Source**: `gh api /repos/Fearvox/EverOS/dependabot/alerts`

## Executive Summary

| Severity | Total | Open | Fixed | Unique Families |
|----------|-------|------|-------|-----------------|
| Critical | 4 | 2 | 2 | 2 |
| High | 48 | 24 | 24 | 24 |
| Medium | 45 | 23 | 22 | 23 |
| Low | 30 | 15 | 15 | 15 |
| **Total** | **127** | **64** | **63** | **64** |

**Ecosystem split**: pip 126, npm 1
**Manifest split**: EverCore `uv.lock` 63 open, evermemos `uv.lock` 63 fixed, game-of-throne-demo `package.json` 1 open

The 63 fixed alerts are all in `methods/evermemos/uv.lock` — a directory not present on the fork's `main` branch which suggests they were fixed upstream and the alerts auto-dismissed. The 64 open alerts are concentrated in `methods/EverCore/uv.lock` (63) plus one npm alert.

## Open Alerts by Severity

### CRITICAL (2 open)

| GHSA | Package | Summary | Manifest |
|------|---------|---------|-----------|
| GHSA-c67j-w6g6-q2cm | langchain-core | Serialization injection enables secret extraction in dumps/loads APIs | EverCore uv.lock |
| GHSA-7p94-766c-hgjp | nltk | Zip Slip vulnerability | EverCore uv.lock |

**Action**: Both are upstream in EverMind-AI/EverOS dependencies. Fix requires EverCore dependency bump. langchain-core serialization is the higher-impact of the two (RCE-adjacent via `load()`). NLTK Zip Slip is path-traversal during archive extraction — only exploitable if the app calls vulnerable NLTK APIs with untrusted input.

### HIGH (24 open)

Grouped by dependency family:

**nltk (4 alerts)** — path traversal, arbitrary file read/write, unauthenticated RCE in wordnet_app
- GHSA-68j8-pq59-fqgm: Path Traversal
- GHSA-h8wq-7xc4-p3qx: Arbitrary File Read via Absolute Path
- GHSA-469j-vmhf-r6v7: Downloader Path Traversal (Arbitrary File Overwrite)
- GHSA-jm6w-m3j8-898g: Unauthenticated remote shutdown in wordnet_app

**urllib3 (4 alerts)** — sensitive header forwarding, decompression bombs (3 variants)
- GHSA-qccp-gfcp-xxvc: Sensitive headers forwarded across origins
- GHSA-38jv-5279-wg99: Decompression-bomb bypass on redirect
- GHSA-2xpw-w6gg-jr37: Streaming API decompression handling
- GHSA-gm62-xv2j-4w53: Unbounded decompression chain links

**ujson (3 alerts)** — memory leak, integer overflow, DoS
- GHSA-c38f-wx89-p2xg: Memory leak in ujson.dump()
- GHSA-c8rr-9gxc-jprv: Integer overflow → buffer overflow / infinite loop
- GHSA-wgvc-ghv9-3pmm: Memory leak parsing large integers (DoS)

**langchain-core (2 alerts)** — deserialization + path traversal
- GHSA-pjwx-r37v-7724: Unsafe deserialization via overly broad `load()` allowlists
- GHSA-qh6h-p6c9-ff54: Path traversal in legacy `load_prompt`

**python-multipart (2 alerts)** — DoS + arbitrary file write
- GHSA-pp6c-gr5w-3c5g: DoS via unbounded multipart part headers (critical for EverCore API)
- GHSA-wp53-j4wj-2cfg: Arbitrary file write via non-default config

**pyasn1 (2 alerts)** — DoS via unbounded recursion + decoder
- GHSA-jr27-m4p2-rc6r: Unbounded recursion
- GHSA-63vm-454h-vhhq: Decoder DoS

**Singletons**:
- GHSA-752w-5fwx-jx9f: PyJWT — accepts unknown `crit` header extensions
- GHSA-6mq8-rvhq-8wgg: aiohttp — zip bomb via auto_decompress
- GHSA-3936-cmfr-pm3m: black — arbitrary file writes from unsanitized cache file name
- GHSA-r6ph-v2qm-q3c2: cryptography — subgroup attack for SECT curves
- GHSA-wj6h-64fc-37mp: ecdsa — Minerva timing attack on P-256
- GHSA-hx9q-6w63-j58v: orjson — unlimited recursion depth
- GHSA-7gcm-g887-7qv7: protobuf — JSON recursion depth bypass

### MEDIUM (23 open)

Notable clusters:
- **aiohttp (7 alerts)**: SSRF/NTLM credential theft, duplicate Host headers, chunked message DoS, large payload DoS, trailer header memory, multipart header size bypass, UNC path handling
- **langchain/langgraph/langsmith (5 alerts)**: checkpoint deserialization RCE (Medium rather than High due to precondition), f-string injection, SSRF via header injection, token redaction bypass
- **filelock (2 alerts)**: TOCTOU symlink attacks
- **nltk (2 alerts)**: XSS in web interface, unbounded recursion
- **python-multipart (1 alert)**: DoS via large preamble/epilogue
- **vite (1 alert)**: Path traversal in optimized deps `.map` handling (npm, game-of-throne-demo)
- **Singletons**: ecdsa DoS, pytest tmpdir, python-dotenv symlink, requests insecure temp file, virtualenv TOCTOU

### LOW (15 open)

Mostly aiohttp (10 alerts: CRLF injection, cookie leaks, DNS cache DoS, cookie parser warning storm, brute-force path leak, unicode regex parsing, header value discrepancies) plus cryptography DNS constraint issue, langchain SSRF (x2), Pygments ReDoS, mem0ai input validation.

## Dependency Hotspots

| Package | Total Alerts | Critical | High | Medium | Low |
|---------|-------------|----------|------|--------|-----|
| aiohttp | 15 | 0 | 1 | 7 | 7 |
| nltk | 7 | 1 | 4 | 2 | 0 |
| langchain* | 8 | 1 | 2 | 5 | 2 |
| urllib3 | 4 | 0 | 4 | 0 | 0 |
| ujson | 3 | 0 | 3 | 0 | 0 |
| python-multipart | 3 | 0 | 2 | 1 | 0 |
| filelock | 2 | 0 | 0 | 2 | 0 |
| pyasn1 | 2 | 0 | 2 | 0 | 0 |

## Recommendations

1. **Immediate (this week)**: Bump langchain-core to fix the critical serialization injection (GHSA-c67j-w6g6-q2cm) and NLTK for Zip Slip (GHSA-7p94-766c-hgjp). These are the only two critical open alerts.

2. **High priority (within 30d)**: Address the 24 high-severity alerts, prioritizing:
   - python-multipart (EverCore API surface — DoS + arbitrary file write)
   - urllib3 (sensitive header forwarding + decompression bomb chain)
   - nltk (4 alerts with path traversal + RCE surface)

3. **Medium (next cycle)**: The aiohttp cluster (15 alerts) is mostly medium/low — batch update. The langgraph checkpoint deserialization (GHSA-g48c-2wqr-h844) warrants attention as it could be RCE under specific conditions.

4. **evermemos directory**: 63 fixed alerts reference `methods/evermemos/uv.lock` which is not present on the fork's `main` branch. These were likely fixed upstream and auto-dismissed by Dependabot. No action needed.

5. **npm**: Single vite alert (GHSA-4w7w-66w2-5vf9) in game-of-throne-demo — already tracked by open Dependabot PR #1.

## Raw Data

Alerts JSON snapshot saved at `.planning/dependabot-alerts.json` (not committed to feature branch — CI-only artifact).
