# Plan 9: Tag Conventions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The tag convention as a doctrine, and `tools/tags/` to propose, apply and check it on the knowledge folder
([spec](../specs/2026-10-11-tag-conventions-design.md)).

**Architecture:** Pure Python modules, each tested: `convention.py` (the rules), `signals.py` (proposals from the five
signals), `tagfile.py` (rewriting tag lines in YAML text), and `cli.py` (suggest, apply, check) gluing them to the
index, the knowledge folder and the cached Stack Exchange synonyms. Run through `uv run --with`, no lockfile.

**Tech Stack:** Python 3.13, `inflect`, `rapidfuzz`, `pytest`; stdlib `sqlite3`, `tomllib`, `urllib`, `difflib`.

**Rules:** test first; `uv run -q --with pytest --with inflect --with rapidfuzz python -I -m pytest tools/tags -q` after
each step; `hk check --all` before each commit; commits as the repo's (≤ 72, sentence case, Co-Authored-By trailer).
Nothing from the knowledge folder enters the repository: tests use made-up tags.

## File map

| File                                        | Responsibility                                                   |
| ------------------------------------------- | ---------------------------------------------------------------- |
| `docs/doctrine/tag-conventions.doctrine.md` | the convention, its sources, how to apply it                     |
| `tools/tags/convention.py`                  | `normalize(tag)`, `violations(tag)`                              |
| `tools/tags/signals.py`                     | `propose(uses, cooccur, synonyms) -> list[Proposal]`             |
| `tools/tags/tagfile.py`                     | `rewrite(text, aliases) -> str`: tag lines only                  |
| `tools/tags/cli.py`                         | `suggest`, `apply [--write]`, `check`; paths from kb's config    |
| `tools/tags/test_*.py`                      | pytest, beside the code                                          |
| `tools/tags/tasks.toml`, `mise.toml`        | `tags:suggest`, `tags:apply`, `tags:check`, `tags:test`; include |
| `AGENTS.md`, `README.md`                    | pointers                                                         |

---

### Task 1: The doctrine

- [ ] Write `docs/doctrine/tag-conventions.doctrine.md` (front-matter `applies_to: ['tools/tags/**']`, `topics: [tags,
      naming, taxonomy]`): Principle (one preferred tag per concept; aliases recorded, not used), the Rules table
      (charset, separator, length, naming, number, aliases, one-offs) each with its source (Stack Overflow, GitHub
      topics, NISO Z39.19 / ISO 25964, W3C SKOS), Why (search and filters split on variants; suffix stripping made
      `maco`), How to apply (`tags:check` before adding a tag; `tags:suggest` → review → `tags:apply`).
- [ ] Commit `docs: Add the tag conventions doctrine`.

### Task 2: `convention.py`

- [ ] Test (`tools/tags/test_convention.py`):

```python
from convention import normalize, violations

def test_normalizes_case_separators_and_spaces():
    assert normalize(" Semantic_Release ") == "semantic-release"
    assert normalize("virtual desktops") == "virtual-desktops"
    assert normalize("c++") == "c++"

def test_reports_what_breaks_the_convention():
    assert violations("semantic-release") == []
    assert violations("Semantic_Release") == ["not lowercase", "underscore: use a hyphen"]
    assert violations("a") == ["shorter than 2"]
    assert violations("x" * 31) == ["longer than 30"]
    assert violations("café") == ["not ASCII"]
```

- [ ] Implement: `normalize` lowercases, trims, turns `_` and runs of spaces into `-`; `violations` returns the list
      above (ASCII, lowercase, underscore or space, 2–30 characters; `+`, `#`, `.` and digits allowed, as `c++`, `c#`,
      `node.js`, `s3` need them).
- [ ] Tests pass; commit `feat(tools): Add the tag convention rules`.

### Task 3: `signals.py`

- [ ] Test (`tools/tags/test_signals.py`), each signal on made-up tags:

```python
from signals import propose

def by_alias(proposals):
    return {p.alias: (p.preferred, p.signal) for p in proposals}

def test_merges_case_and_separator_variants_into_the_most_used():
    got = by_alias(propose({"semantic-release": 3, "semantic_release": 1}, {}, {}))
    assert got == {"semantic_release": ("semantic-release", "fingerprint")}

def test_merges_a_singular_into_its_existing_plural():
    got = by_alias(propose({"jobs": 5, "job": 2, "maco": 9, "macos": 4}, {}, {}))
    assert got["job"] == ("jobs", "plural")
    assert got["maco"] == ("macos", "plural")

def test_follows_stack_overflows_synonyms():
    got = by_alias(propose({"k8s": 2, "kubernetes": 1}, {}, {"k8s": "kubernetes"}))
    assert got["k8s"] == ("kubernetes", "stackoverflow")

def test_flags_near_spellings_without_merging_distinct_short_tags():
    got = by_alias(propose({"postgres": 5, "postgress": 1, "ai": 9, "ui": 4}, {}, {}))
    assert got["postgress"] == ("postgres", "similar")
    assert "ui" not in got and "ai" not in got

def test_flags_tags_that_always_come_together():
    got = by_alias(propose({"fabric": 10, "fab": 10}, {("fab", "fabric"): 10}, {}))
    assert got["fab"][1] == "together"
```

- [ ] Implement `Proposal(alias, preferred, signal, reason)` and `propose(uses, cooccur, synonyms)`:
  1. **fingerprint** (OpenRefine key collision): key = `normalize(tag)` with `-` removed; a group of 2+ merges into its
     most-used member.
  2. **plural**: if `tag + "s"` (or `inflect`'s plural of `tag`) is also a tag, merge `tag` into it (Z39.19 plural for
     countables; repairs suffix-stripped proper names).
  3. **stackoverflow**: if `synonyms[normalize(tag)]` names a different tag, propose it (the target need not exist yet).
  4. **similar**: `rapidfuzz` ratio ≥ 90 between tags of length ≥ 5, merge into the more used.
  5. **together**: two tags whose entries are the same set (`cooccur[(a, b)] == uses[a] == uses[b]`): flag the less
     common or shorter as an alias, for review.
     The first signal that fires wins; no tag is both alias and preferred (chains resolve to the end).
- [ ] Tests pass; commit `feat(tools): Propose tag merges from known signals`.

### Task 4: `tagfile.py`

- [ ] Test (`tools/tags/test_tagfile.py`):

```python
from tagfile import rewrite

ALIASES = {"maco": "macos", "job": "jobs"}

def test_rewrites_flow_lists_single_tags_and_block_lists_only():
    text = (
        "a:\n  title: maco tips # maco\n  tags: [maco, shell]\n"
        "b:\n  tag: job\n"
        "c:\n  tags:\n    - job\n    - jobs\n  notes: job\n"
    )
    assert rewrite(text, ALIASES) == (
        "a:\n  title: maco tips # maco\n  tags: [macos, shell]\n"
        "b:\n  tag: jobs\n"
        "c:\n  tags:\n    - jobs\n  notes: job\n"
    )

def test_leaves_text_without_aliases_byte_for_byte():
    text = "a:\n  tags: [shell]   # spacing kept\n"
    assert rewrite(text, ALIASES) == text
```

- [ ] Implement line by line: `^(\s*)tags?:\s*\[(.*)\](.*)$` (flow list), `^(\s*)tag:\s*(\S.*?)(\s*#.*)?$` (one tag),
      and a block list after `^\s*tags:\s*$` (items `^\s+- `, indented deeper than the key). Map each tag through the
      aliases (case-insensitive key, normalized output), drop a repeat within the entry, keep everything else as
      written.
- [ ] Tests pass; commit `feat(tools): Rewrite tag lines in YAML text`.

### Task 5: `cli.py` and the tasks

- [ ] `cli.py`: paths from `~/.config/kb/config.toml` (`sources_path`, `~` expanded; default
      `~/.config/kodexb/sources`) and the index `~/Library/Application Support/kb-native/kb.db`.
  - `suggest`: reads `select tag, count(*) from entry_tags group by tag` and co-occurrence
    (`select a.tag, b.tag, count(*) from entry_tags a join entry_tags b on a.entry_id = b.entry_id and a.tag < b.tag
    group by 1, 2`); Stack Exchange synonyms fetched once from `api.stackexchange.com/2.3/tags/synonyms?site=stackoverflow`
    (all pages, cached `tmp/tags/so-synonyms.json`); writes `<sources>/tags.proposed.toml`: one table per preferred tag,
    `aliases = [...]`, each preceded by `# <alias> (<uses>) → <signal>: <reason>`; then a commented list of one-off tags
    and of tags breaking the convention with no proposal.
  - `apply [--write]`: reads `<sources>/tags.toml`, refuses when the folder is not a git work tree with a clean status,
    prints the unified diff of every changed `.yml`/`.yaml`, writes only with `--write`.
  - `check`: prints each indexed tag that `violations` flags or that `tags.toml` lists as an alias; exit 1 when any.
- [ ] `tools/tags/tasks.toml` (`tags:test`, `tags:suggest`, `tags:apply`, `tags:check`, each
      `uv run -q --with … python -I tools/tags/cli.py <cmd>`), included from `mise.toml`.
- [ ] Run `mise run tags:test`, then `mise run tags:suggest`; read the proposal counts. Commit
      `feat(tools): Add the tags suggest, apply and check tasks`.

### Task 6: Pointers, TODO, PR

- [ ] AGENTS.md repository map: `tools/tags/`; README Development Workflow: the three tasks.
- [ ] TODO: "Review tags.proposed.toml, git init the knowledge folder, run tags:apply --write" (owner); "Tag icons:
      brainstorm the second sub-project".
- [ ] `hk check --all`; push; PR; merge on green.

## Self-review

T1 → Task 1; T2 → Tasks 3, 5 (suggest); T3 → Task 5 (files in `<sources>`), Rules; T4 → Tasks 4, 5 (git and clean
checks, dry run); T5 → Task 5 (one-off list). Types: `Proposal`, `propose`, `rewrite`, `normalize`, `violations` are
named the same everywhere.
