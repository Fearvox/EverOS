# methods/HyperMem — Local CLAUDE.md

Local-only context. Root `CLAUDE.md` and `AGENTS.md` cover the cross-repo map.

## What this module is

Official implementation of the **ACL 2026** paper *HyperMem: Hypergraph Memory
for Long-Term Conversations*. Three-level hypergraph (**topics → episodes →
facts**) with weighted hyperedges, retrieved via coarse-to-fine top-down
traversal. LoCoMo headline number: **92.73% LLM-as-judge accuracy** (vs.
HyperGraphRAG 86.49%, MemOS 75.80%).

This is research code with a publication frozen behind it. Treat it as a
reference implementation — refactors that change numerics need to re-run the
LoCoMo eval before merge.

## Internal map

```
hypermem/
├── main/         entry points for construction + retrieval + evaluation
├── structure.py  hypergraph data structures (topics, episodes, facts, edges)
├── types.py      typed schemas
├── config.py     run config
├── extractors/   episode detection + topic aggregation + fact extraction
├── llm/          LLM client adapters
├── prompts/      extractor + retrieval prompts
└── utils/        shared helpers

scripts/
├── run_eval.sh           one-shot eval runner
├── serve_embedding.sh    local embedding service
└── serve_reranker.sh     local reranker service
```

Read order for a new task: `structure.py` → `main/` → the extractor or
retrieval stage you are touching → `prompts/` only if changing prompt schema.

## Hard rules

- **Numerics are paper-load-bearing.** Changes that touch propagation
  (`λ = 0.5`), attention weighting, BM25-dense RRF fusion, or top-k thresholds
  must re-run the LoCoMo eval and report the delta in the PR.
- **Python 3.12+.** ML stack (torch, transformers, sentence-transformers) —
  CPU works for smoke; GPU recommended for full eval.
- **Embedding + reranker services are external.** `scripts/serve_*.sh` boots
  them locally. Do not vendor the model weights into the repo.

## Working commands

```bash
# from this directory:
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

# boot the embedding + reranker services in separate terminals:
bash scripts/serve_embedding.sh
bash scripts/serve_reranker.sh

# run a full eval (LoCoMo or your dataset):
bash scripts/run_eval.sh
```

## Common gotchas

- The hypergraph construction is **streaming** — episode boundary detection
  runs as the dialogue is ingested. Do not batch-rewrite that loop without
  re-validating boundary placement on the paper's eval set.
- BM25 + dense RRF fusion is implementation-sensitive. Changing the k constant
  in RRF (default 60) shifts retrieval and downstream accuracy.
- Hyperedge weights are in `[0, 1]` and used as attention logits before
  softmax. Negative or unbounded values silently break propagation.

## Cross-directory contract

- `benchmarks/EverMemBench/` may import HyperMem as one of the memory systems
  under evaluation; keep the public `main/` entry signatures stable.
- HyperMem does not depend on `methods/EverCore/`. They are independent
  memory architectures, both reachable as benchmark targets.

## What does NOT belong here

- Production multi-tenant memory APIs — that is EverCore's role.
- New benchmark datasets — put those in `benchmarks/`.
- Hermes / use-case integrations — they should consume HyperMem through a
  benchmark adapter, not import internal modules directly.
