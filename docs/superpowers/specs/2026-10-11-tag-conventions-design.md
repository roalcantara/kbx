# Tag conventions — design

- Status: approved
- Date: 2026-10-11

## Goal

One convention for kb's tags, taken from established practice rather than invented, and a repeatable way to bring the
existing tags to it: every concept one preferred tag, its variants recorded as aliases, the source files rewritten once
after the owner reviews every change. Tag icons are a separate design that starts from the clean names.

## Decisions

**T1. The convention adopts existing ones.** From Stack Overflow's tagging rules and GitHub topics: lowercase ASCII,
words joined by hyphens, the name people search for, the tool's own name (the Stack Overflow master tag when one
exists); 2 to 30 characters. From NISO Z39.19 / ISO 25964 (controlled vocabularies): one preferred term per concept with
its synonyms recorded as aliases; countable nouns plural (`containers`, `tools`), mass nouns and proper names as they
are (`security`, `macos`, `kubernetes`). From W3C SKOS: the alias map's shape, a preferred label with alternative
labels. The doctrine `docs/doctrine/tag-conventions.doctrine.md` states it, with these sources.

**T2. Suggested by tools, decided by the owner.** A script proposes a canonical form for every tag from four signals,
each a known technique: OpenRefine's fingerprint key collision (case, punctuation and token order), nearest-neighbour
string similarity (typos and separators, Levenshtein), Stack Exchange's tag synonyms (`k8s` → `kubernetes`), dictionary
inflection (`pluralize`, never suffix stripping, which produced today's `maco` and `kubernete`), plus co-occurrence on the
same entries for redundant pairs. The proposals arrive as a commented map, `tags.proposed.toml`: the owner keeps, edits or
deletes each block and saves the result as `tags.toml`.

**T3. The data stays private.** The alias map (`tags.toml`) and the proposals live in the knowledge folder beside the
YAML they govern; kb's repository holds only the doctrine and the scripts, which read the map from there.

**T4. One rewrite, of the source files.** An apply step rewrites tag lines only, in place, after `git init` of the
knowledge folder and a dry-run diff the owner approves. kb the app still never writes sources; this is a separate,
explicit tool. Raycast's kb and kb then show the same clean tags, and the engine keeps matching its goldens.

**T5. One-off tags are reviewed, not dropped.** Tags used once are flagged in the table; the owner decides.

## Scope

In: the doctrine, `tools/tags/` (suggest, apply, check), the mise tasks, the first review and rewrite. Out: tag icons
(next design), any change to kb the app or the engine.

## Tooling

| Task                    | Does                                                                                                                                 |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `mise run tags:suggest` | Reads every tag and its uses from the index, writes `<knowledge>/tags.proposed.toml`: each proposal with its uses, signal and reason |
| `mise run tags:apply`   | Reads the reviewed `<knowledge>/tags.toml`; prints the diff of tag lines; with `--write`, rewrites them                              |
| `mise run tags:check`   | Lists tags that break the convention or are an alias: for new tags, any time                                                         |

TypeScript run by Bun (AGENTS.md): `bun:sqlite`, `Bun.TOML`, `fetch`, and the npm packages `pluralize` and
`fastest-levenshtein` in `tools/tags/package.json`; tests with `bun test`. The knowledge folder is `sources_path` from kb's config. Stack Exchange's synonyms are fetched
once and cached in `tmp/`.

## Tag lines

Tags appear as `tags: [a, b]`, `tag: a`, or a block list under `tags:`. The apply step edits those lines alone, keeping
order, indentation and comments; a tag that becomes a duplicate on its entry is dropped there. Every other line stays
byte for byte.
