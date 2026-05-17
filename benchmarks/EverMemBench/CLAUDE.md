# benchmarks/EverMemBench — Local CLAUDE.md

Local-only context. Root `CLAUDE.md` and `AGENTS.md` cover the cross-repo map.

## What this module is

A multi-person group-chat memory evaluation framework. Pits 5 memory systems
(**Memos, Mem0, Memobase, EverCore, Zep**) plus an LLM long-context baseline
against the **EverMemBench-Dynamic** dataset on HuggingFace
(`EverMind-AI/EverMemBench-Dynamic`).

Pipeline: **Add → Search → Answer → Evaluate**. Two question types: multiple
choice (direct comparison) and open-ended (LLM-judge).

## Internal map

```
eval/
├── cli.py        main entry — orchestrates the 4-stage pipeline
├── config/       YAML configs per memory system + per dataset slice
└── src/          stage implementations (add/search/answer/evaluate)

tools/
└── analyze_results.py   post-run accuracy + breakdown reporter
```

## Hard rules

- **Numbers are reportable.** Any code change that affects retrieval, answer
  generation, or evaluation logic must report a paired before/after run in
  the PR. Treat this like HyperMem — research artifact, not utility code.
- **Datasets are not in-repo.** Source comes from HuggingFace. Do not vendor
  the full dataset; cache it via `datasets` library or a pinned snapshot
  path.
- **OpenRouter is the default LLM gateway.** `LLM_API_KEY` in `.env` must
  point at OpenRouter (or compatible). Per-system keys (`MEMOS_API_KEY`,
  `MEM0_API_KEY`, etc.) are only needed for the systems being benchmarked.
- **Smoke mode exists for a reason.** Use it (`--smoke` or equivalent in
  CLI) before any full run. Full runs are expensive.

## Working commands

```bash
# from this directory:
cp env.template .env       # fill in LLM_API_KEY + system-specific keys
pip install -r requirements.txt
# install only the SDKs for systems you are evaluating:
pip install mem0ai memobase zep-cloud   # subset as needed

# pipeline (smoke first, then full):
python -m eval.cli add    --config eval/config/<config>.yaml --smoke
python -m eval.cli search --config eval/config/<config>.yaml --smoke
python -m eval.cli answer --config eval/config/<config>.yaml --smoke
python -m eval.cli evaluate --config eval/config/<config>.yaml --smoke

# post-run analysis:
python tools/analyze_results.py <run-output-dir>
```

## Common gotchas

- **Message format differs per system.** Memos wants
  `[Group: X][Speaker: Y]content`; Mem0 wants `run_id="${user_id}_${groupId}"`
  + `name=<Speaker>`. The README has the full matrix — do not paper over the
  differences with a generic adapter.
- **Timestamp handling is per-system.** Memos uses native `chat_time`, Mem0
  uses Unix timestamps per batch. Misaligned timestamps silently kill recall.
- **Rate limits matter.** OpenRouter and the memory system providers all rate
  limit. `aiolimiter` is wired in — do not bypass it.

## Cross-directory contract

- `methods/EverCore/` is one of the systems under evaluation. EverCore DTO
  changes can break the EverCore adapter here; treat the EverCore HTTP API
  as a frozen contract for benchmark runs.
- `methods/HyperMem/` may be added as a benchmark target via its `main/`
  entry; add adapters in `eval/src/` not by importing HyperMem internals.

## What does NOT belong here

- Memory system implementations. Adapters only.
- Live agent demos — that is `use-cases/`.
- The dataset itself — keep it on HuggingFace.
