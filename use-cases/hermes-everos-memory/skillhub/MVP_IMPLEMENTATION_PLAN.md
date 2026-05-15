# EverMe SkillHub MVP Implementation Plan v0

SkillHub is the memory-backed skill surface for EverMe. It is not a generic
marketplace. The MVP makes skills legible, installable, improvable, and
evidence-bearing before final EverMe product UI is available.

## Product Contract

SkillHub v0 manages portable skill packets.

Each packet must answer:

- What is this skill for?
- Where can it install?
- What evidence says it works?
- What should improve next?
- Can it be shared safely?

## MVP Views

| View | Purpose | Minimum fields |
| --- | --- | --- |
| Skill Index | scan owned/community/needs-eval skills | name, status, version, domains, targets |
| Skill Detail | understand one skill | summary, body, source, evidence |
| Evolution Queue | decide next improvement | status, eval score, last evolved, evidence gaps |
| Install Packet | connect to runtime | install targets, version, body markdown |
| Trust Panel | decide whether to use/share | provenance, evidence refs, rating, votes |

The CLI renderer should expose these views before any final web UI exists.

## API Contract

The mock API is read-only until EverMe backend constraints arrive.

Current routes:

- `GET /health`
- `GET /skills`
- `GET /skills?target=hermes`
- `GET /skills/:id`
- `GET /skills/:id/render`
- `GET /skills/:id/views`
- `GET /skills/:id/install-packet?target=hermes`
- `POST /skills/validate`

Next API routes:

- `POST /skills/:id/evidence`
- `POST /skills/:id/evolution-note`

Write routes stay proposed until the canonical EverMe API exists.

## Data Additions

Keep the packet compact. Add optional fields only when they support the five
MVP views:

- `provenance`: source runtime, extractor, source artifact id;
- `evolution_history`: timestamped notes, eval deltas, next action;
- `compatibility`: runtime names and minimum versions;
- `install_notes`: target-specific setup notes;
- `trust`: rating, votes, eval score, evidence summary.

The existing core packet stays valid without these fields.

## Implementation Sequence

1. Extend the local renderer with a `views` command.
2. Add `just skillhub-views`.
3. Keep the mock API read-only and deterministic.
4. Import one real `SKILL.md` file through `from-skill`.
5. Add optional data fields only after the renderer proves their use.
6. Wait for EverMe design-system input before building final visual UI.

## Gates

| Gate | Verdict rule |
| --- | --- |
| Packet validation | schema and custom validator pass |
| Views render | five MVP views render from one packet |
| Mock API | health/list/detail/render/validate pass |
| Import path | `from-skill` produces a valid packet |
| Public safety | no secrets, host details, private paths, or raw tokens |

## First Useful Slice

```bash
node bin/skillhub-packet.mjs views skillhub/fixtures/raven-skillhub-sample.json
```

This is enough for Raven, Hermes, and EverCore to dogfood SkillHub without
pretending the final EverMe UI is done.
