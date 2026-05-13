# May Agent Packet Review

## Verdict

FLAG: #16-#22 are useful as a May Agent architecture packet, but they should not
merge as-is. Treat them as draft review material until the private references,
unverified external claims, and EverCore API contract mismatches are fixed.

## Merge Order

| Order | PR | File | Verdict | Reason |
|-------|----|------|---------|--------|
| 1 | #16 | `.planning/may-agent/00-vision.md` | Review first | Strategy gate; later docs depend on its scope and success criteria. |
| 2 | #17 | `.planning/may-agent/10-architecture.md` | Review after #16 | Converts the strategy into system design and data flow. |
| 3 | #18 | `.planning/may-agent/20-rust-runtime-scaffold.md` | Review after #17 | Implementation scaffold depends on architecture choices. |
| 4 | #19 | `.planning/may-agent/30-evercore-integration-contract.md` | Review after #17/#18 | Contract should match live EverCore controllers before merge. |
| 5 | #20 | `.planning/may-agent/40-benchmark-strategy.md` | Review after #16/#17 | Benchmark narrative is useful, but proposed harness claims need proof. |
| 6 | #21 | `.planning/may-agent/90-risk-log.md` | Review before merge packet | Risk register should absorb API and evidence gaps found below. |
| 7 | #22 | `.planning/may-agent/INDEX.md` | Merge last only | Index references the other docs and should not land before source docs. |

## Contradictions And Risks

| Area | Finding | Evidence |
|------|---------|----------|
| EverCore search method | #19 documents `GET /api/v1/memories/search`, while live code also has `POST /api/v1/memories/search` and current plugin docs use POST. | `memory_search_controller.py` says POST; `api_specs/dtos/memory.py` documents both GET and POST sections. |
| Health route prefix | #19 says `GET /api/v1/health`, but live controller prefix is `/health`. | `methods/EverCore/src/infra_layer/adapters/input/api/health/health_controller.py` uses `prefix="/health"`. |
| Private reference | #16 and #21 reference a local memory path under a home directory. | `00-vision.md` and `90-risk-log.md` include private machine-local memory references. |
| Missing repo references | Packet references `.planning/hermes-recon/architecture.md` and `CLAUDE_DESKTOP_SANDBOX_SOURCE_TRUTH.md`, but those files are not present on current `origin/main`. | `find` did not locate either path in the checked-out fork. |
| External claims | Hermes star count, Hermes LOC, OpenClaw comparison, and May 2026 release claims are not supported by current repo evidence. | Present in #16/#17/#18 bodies, absent from repo-local proof. |

## Missing Evidence Before Merge

- Replace private memory references with repo-safe sources or remove them.
- Pin or cite exact Hermes source SHA, license proof, and measured LOC command.
- Verify the live EverCore route contract and choose one search method in the docs.
- Add evidence for WeCom/Feishu priority, or label it as product hypothesis.
- Mark Evil Agent Bench as proposed only until a harness exists.
- Confirm benchmark commands are runnable from the current repo layout.

## Upstream Pitch Framing

Pitch the packet as an internal architecture decision bundle, not as an
implementation-ready upstream submission. The strongest framing is:

- EverCore is the memory differentiator.
- The May Agent runtime is a hybrid design: Python agent core plus Rust runtime
  shell for CLI, sandbox, gateway, and embedding surfaces.
- The packet is ready for stakeholder review after evidence cleanup, not ready
  for direct merge.

## Should Not Merge As-Is

- Do not merge #16 while it contains private local memory references or
  unverified market/runtime claims.
- Do not merge #19 until the EverCore route table is reconciled with live code.
- Do not merge #20 as proof of security; it is a proposed benchmark plan only.
- Do not merge #22 before #16-#21, because it indexes files that would not exist
  yet on `main`.
