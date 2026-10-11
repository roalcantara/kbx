---
name: tag-conventions
applies_to: ['tools/tags/**']
topics: [tags, naming, taxonomy, vocabulary]
always: false
---

# Tag conventions

## Principle

As few tags as possible, each as short as possible: one tag per concept, in its shortest established form. Variants,
synonyms, translations and long forms are recorded as aliases in the knowledge folder's `tags.toml`, never used on
entries. The convention is borrowed, not invented: each rule names its source.

## Why

Tags filter and group entries. Variants split them: `semantic-release` and `semantic_release` each hold half the
entries, `ia` and `ai` never meet. Mechanical "fixes" make it worse: stripping a final `s` turned `macos` into `maco`
and `kubernetes` into `kubernete`. Established conventions already settle these questions for millions of tags.

## Rules

| Rule                                                                                                                      | Source                                                                          |
| ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| Lowercase ASCII letters and digits; `+`, `#` and `.` only where the name has them (`c++`, `node.js`)                      | Stack Overflow tags; GitHub topics                                              |
| Words joined by hyphens: `semantic-release`, never `semantic_release` or a space                                          | Stack Overflow tags; GitHub topics                                              |
| 2 to 30 characters                                                                                                        | GitHub topics (50), Stack Overflow (35), tightened for pills                    |
| The shortest established form: an abbreviation a public source lists (`crm`, `aws`, `k8s`, `db`), that no other tag means | Wikipedia's computing abbreviations; Stack Overflow tag synonyms                |
| Countable things plural (`containers`, `tools`); mass nouns and proper names as they are (`security`, `macos`)            | NISO Z39.19 / ISO 25964, noun forms                                             |
| Never derive a form by stripping or adding letters: use a dictionary, or the name as written                              | Folksonomy normalization research (lemmatize, do not stem)                      |
| One preferred tag per concept; every other form is an alias of it                                                         | NISO Z39.19 equivalence ("use" / "used for"); W3C SKOS `prefLabel` / `altLabel` |
| One concept, one tag, in any language: `db`, `database`, `banco-de-dados` are one; related concepts (`data`) stay apart   | Wikidata reconciliation (OpenRefine); Z39.19 equivalence                        |
| A compound whose words are tags becomes those tags: `coding-pattern` → `coding` + `patterns`                              | Z39.19 (split compound terms into their facets)                                 |
| A tag that says nothing on its own (`general`, `info`) is dropped                                                         | Folksonomy research (non-descriptive tags)                                      |
| A tag used once is reviewed: kept if it names something real, else merged or dropped                                      | Folksonomy clustering research (idiosyncratic tags)                             |

## How to apply

- Before adding a tag, run `mise run tags:check`: it lists the tags that break these rules or are an alias.
- To clean up: `mise run tags:suggest` writes `tags.proposed.toml` in the knowledge folder, each proposal with its
  reason. It reads, beside the index, Stack Overflow's synonyms, Wikipedia's abbreviations, Wikidata's concepts and
  the knowledge folder's `tags.semantic.toml` (one-off judgements, e.g. an LLM pass), all as suggestions. Keep, edit or delete each block, save it as `tags.toml` (an alias under several tables becomes all of them; under `["-"]`, it is dropped), then `mise run tags:apply` shows the diff and
  `mise run tags:apply -- --write` rewrites the tag lines. The knowledge folder must be a clean git work tree.
- kb itself never writes source files: these tools do, only when asked.
