---
name: everos-prompts-sync
description: Use when editing EverCore prompts under methods/EverCore/src/memory_layer/prompts/en or .../zh, or before opening a PR that touches that tree. Verifies EN/ZH file-name mirror + export-symbol parity, surfaces missing zh files and divergent __all__ lists, and falls back to the existing src/devops_scripts/i18n/i18n_tool.py for code-comment translation drift.
---

# everos-prompts-sync

Keeps `methods/EverCore/src/memory_layer/prompts/{en,zh}/` in lockstep at the
**file-name + symbol-export** layer. Does **not** judge translation quality —
that's a content review.

This skill encodes a recurring EverCore failure mode: a contributor adds a new
prompt constant to `en/` and forgets to add a matching entry under `zh/`,
which silently breaks imports the moment a tenant uses the ZH locale.

## When to invoke

- A diff under `methods/EverCore/src/memory_layer/prompts/` is in flight.
- A new prompt constant is being added to `en/<file>.py`.
- A PR is about to be opened and the prompt tree has any change at all.

If neither side of `prompts/` changed, skip — this skill has nothing to say.

## Procedure

1. **Confirm scope.** From repo root:

   ```bash
   cd methods/EverCore/src/memory_layer/prompts
   ```

2. **File-name mirror.** Both directories must have identical file lists
   (excluding `__pycache__`):

   ```bash
   diff <(ls en/ | grep -v __pycache__) <(ls zh/ | grep -v __pycache__)
   ```

   Any difference is a bug. The fix is **always** to add the missing file to
   the side that lacks it. The new file can be a translation OR an explicit
   re-export from the other side (the existing convention — see
   `zh/agent_prompts.py` for the re-export pattern).

3. **Export-symbol parity.** For each file pair `en/X.py` and `zh/X.py`,
   their public exports must be the same set:

   ```bash
   python -c "
   import ast, sys, pathlib
   for f in pathlib.Path('en').glob('*.py'):
       if f.name == '__init__.py': continue
       z = pathlib.Path('zh') / f.name
       if not z.exists(): print(f'MISSING zh: {f.name}'); continue
       def syms(p):
           tree = ast.parse(p.read_text())
           return {t.id for n in tree.body if isinstance(n, ast.Assign)
                   for t in n.targets if isinstance(t, ast.Name) and t.id.isupper()}
       en_syms, zh_syms = syms(f), syms(z)
       if en_syms != zh_syms:
           missing_in_zh = en_syms - zh_syms
           missing_in_en = zh_syms - en_syms
           if missing_in_zh: print(f'{f.name}: zh missing {missing_in_zh}')
           if missing_in_en: print(f'{f.name}: en missing {missing_in_en}')
   "
   ```

   Re-exports count: `zh/agent_prompts.py` doing
   `from ...en.agent_prompts import FOO, BAR` exposes `FOO` and `BAR` as
   ZH symbols — that satisfies parity even though the strings live on the EN
   side only. The AST scan above catches direct top-level assignments;
   re-exports need either a `__all__` list or a wider AST walk if you want to
   be exhaustive.

4. **Report.** Output one of:
   - `PASS: EN/ZH prompt parity OK` (no further action)
   - `FAIL: <list of mismatches>` (fix before merge)

5. **Adjacent tooling.** This skill does **not** translate Chinese code
   comments to English. That's `src/devops_scripts/i18n/i18n_tool.py`, which
   is already wired into `make lint`. Use that for code-comment drift, this
   skill for prompt-constant drift.

## What this skill explicitly does NOT do

- Translate prompts from EN to ZH or vice versa. That's a human/LLM content
  task, not a parity check.
- Validate template variables (`{messages_json}`, `{new_count}`, etc.) match
  between EN and ZH versions. That's a deeper content check worth a separate
  skill if it turns out to be a recurring failure mode.
- Block commits. This is informational. Wire it into a hook only after the
  false-positive rate is known to be near zero.

## Recurrence threshold

If the parity check has surfaced the same root cause **three times** (e.g.,
"forgot to add zh re-export when adding a new EN prompt constant"), upgrade
this skill into a pre-commit hook under `.claude/hooks/`.
