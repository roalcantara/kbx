// The tag convention (docs/doctrine/tag-conventions.doctrine.md) as code: the rules, the merge proposals and the
// rewriting of tag lines in YAML text. The mise tasks in tools/tags/tasks.toml read the index and the knowledge folder
// and call these.
// No npm packages: `bun test` does not install pinned imports (AGENTS.md).
import { Database } from 'bun:sqlite'
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { dirname, join } from 'node:path'

export const MIN_LENGTH = 2
export const MAX_LENGTH = 30

/** The tag in the convention's shape: trimmed, lowercase, words joined by single hyphens. */
export const normalize = (tag: string): string => tag.trim().toLowerCase().replace(/[\s_]+/g, '-')

/** What breaks the convention in `tag`; empty when it follows it. */
export const violations = (tag: string): string[] => {
  const found: string[] = []
  if (!/^[\x00-\x7f]*$/.test(tag)) found.push('not ASCII')
  if (tag !== tag.toLowerCase()) found.push('not lowercase')
  if (/[\s_]/.test(tag)) found.push('underscore or space: use a hyphen')
  if (tag.length < MIN_LENGTH) found.push(`shorter than ${MIN_LENGTH}`)
  if (tag.length > MAX_LENGTH) found.push(`longer than ${MAX_LENGTH}`)
  return found
}

export type Signal = 'fingerprint' | 'convention' | 'plural' | 'stackoverflow' | 'similar' | 'together'

/** A proposed merge: `alias` becomes `preferred`. */
export interface Proposal {
  alias: string
  preferred: string
  signal: Signal
  reason: string
  /** Spelling signals are trusted; those that read meaning (synonyms, co-occurrence, plural against usage) are only
   * suggested, because they cannot know what a tag means in this corpus (`gpt` is not Google Publisher Tag). */
  confident: boolean
}

/** Uses per tag; co-occurrences keyed `"a b"` with `a < b`; Stack Overflow synonyms, alias to master. */
export type Uses = Record<string, number>
export type Cooccurrences = Record<string, number>
export type Synonyms = Record<string, string>

/**
 * Proposes a preferred tag for every tag that should merge, from the first signal that fires, in order:
 * fingerprint (OpenRefine key collision), convention (the tag's normalized form), plural (Z39.19: a singular whose
 * plural is also a tag, never a form made by stripping letters), stackoverflow (its tag synonyms), similar (one or two edits apart) and together (the same
 * entries). Chains resolve to their end, so no tag is both an alias and preferred.
 */
export const propose = (uses: Uses, cooccur: Cooccurrences, synonyms: Synonyms): Proposal[] => {
  const tags = Object.keys(uses)
  const merged = new Map<string, Proposal>()
  const add = (alias: string, preferred: string, signal: Signal, reason: string, confident: boolean) => {
    if (alias !== preferred && !merged.has(alias)) merged.set(alias, { alias, preferred, signal, reason, confident })
  }
  const free = (tag: string) => !merged.has(tag)
  const byUse = (a: string, b: string) => (uses[b] ?? 0) - (uses[a] ?? 0) || a.localeCompare(b)

  // fingerprint: the same tag but for case, separators and spaces
  const groups = new Map<string, string[]>()
  for (const tag of tags) {
    const key = normalize(tag).replaceAll('-', '')
    groups.set(key, [...(groups.get(key) ?? []), tag])
  }
  for (const group of groups.values()) {
    if (group.length < 2) continue
    const preferred = normalize([...group].sort(byUse)[0]!)
    for (const tag of group) add(tag, preferred, 'fingerprint', `spelled ${group.join(', ')}`, true)
  }

  // convention: alone, but not in the convention's shape
  for (const tag of tags.filter(free)) {
    if (normalize(tag) !== tag) add(tag, normalize(tag), 'convention', violations(tag).join('; '), true)
  }

  // plural: a singular whose plural is also a tag (also repairs stripped proper names: maco → macos). Forms the
  // signals above produced count too: best_practice → best-practice joins best-practices.
  const produced = [...new Set([...merged.values()].map((p) => p.preferred))].filter((t) => !(t in uses))
  for (const tag of [...tags.filter(free), ...produced.filter(free)]) {
    // A tag ending in s is no singular here: postgres + s is the typo postgress, not a plural.
    const plural = tag.endsWith('s') ? undefined : plurals(tag).find((p) => p in uses)
    if (plural) {
      const against = (uses[plural] ?? 0) < (uses[tag] ?? 0)
      const reason = `${plural} is also a tag (${uses[plural]} uses)${
        against ? `, though ${tag} has more: a name?` : ''
      }`
      add(tag, plural, 'plural', reason, !against)
    }
  }

  // stackoverflow: its synonym list names a master tag
  for (const tag of tags.filter(free)) {
    const master = synonyms[normalize(tag)]
    if (master && master !== tag) add(tag, master, 'stackoverflow', `Stack Overflow's master tag for ${tag}`, false)
  }

  // similar: one edit apart from 6 characters, two from 10; into the more used
  const candidates = tags.filter((t) => t.length >= 6).sort(byUse)
  for (const [i, more] of candidates.entries()) {
    for (const less of candidates.slice(i + 1)) {
      if (!free(less) || !free(more)) continue
      const limit = Math.min(more.length, less.length) >= 10 ? 2 : 1
      if (distance(more, less) <= limit) {
        add(less, more, 'similar', `${distance(more, less)} edit(s) from ${more}`, true)
      }
    }
  }

  // together: two tags on exactly the same entries; the shorter becomes the alias
  for (const [pair, count] of Object.entries(cooccur)) {
    const [a, b] = pair.split(' ') as [string, string]
    if (count < 2 || count !== uses[a] || count !== uses[b] || !free(a) || !free(b)) continue
    const [alias, preferred] = a.length <= b.length ? [a, b] : [b, a]
    add(alias, preferred, 'together', `always with ${preferred} (${count} entries): redundant, or related?`, false)
  }

  // chains: follow each preferred tag to its end
  const end = (tag: string, seen = new Set<string>()): string => {
    const next = merged.get(tag)
    return next && !seen.has(tag) ? end(next.preferred, seen.add(tag)) : tag
  }
  return [...merged.values()].map((p) => ({ ...p, preferred: end(p.preferred) })).filter((p) => p.alias !== p.preferred)
}

/** The edit distance between two tags (Levenshtein). */
export const distance = (a: string, b: string): number => {
  let previous = Array.from({ length: b.length + 1 }, (_, j) => j)
  for (const [i, ca] of [...a].entries()) {
    const current = [i + 1]
    for (const [j, cb] of [...b].entries()) {
      current.push(Math.min(previous[j + 1]! + 1, current[j]! + 1, previous[j]! + (ca === cb ? 0 : 1)))
    }
    previous = current
  }
  return previous[b.length]!
}

/** The plural forms a tag may have among the others: `+s`, `+es`, `y` → `ies`. Only existing tags count. */
const plurals = (
  tag: string
): string[] => [`${tag}s`, `${tag}es`, ...(tag.endsWith('y') ? [`${tag.slice(0, -1)}ies`] : [])]

/** Each alias as its preferred tag, compared without case. */
export type Aliases = Record<string, string>

const mapTag = (raw: string, aliases: Aliases): string => {
  const tag = raw.trim().replace(/^(['"])(.*)\1$/, '$2')
  return aliases[tag.toLowerCase()] ?? tag
}

/**
 * Rewrites the tag lines of a YAML source file: `tags: [a, b]`, `tag: a` and block lists under `tags:`. Each alias
 * becomes its preferred tag and a repeat on the same entry is dropped; every other line stays byte for byte.
 */
export const rewrite = (text: string, aliases: Aliases): string => {
  const lines = text.split('\n')
  const out: string[] = []
  let block: { indent: number; seen: Set<string> } | undefined
  for (const line of lines) {
    const indent = line.length - line.trimStart().length
    if (block && line.trim() !== '' && indent <= block.indent) block = undefined

    const item = block && line.match(/^(\s*-\s+)([^#]*?)(\s*(#.*)?)$/)
    if (block && item) {
      const tag = mapTag(item[2]!, aliases)
      if (block.seen.has(tag)) continue
      block.seen.add(tag)
      out.push(tag === item[2]!.trim() ? line : `${item[1]}${tag}${item[3]}`)
      continue
    }

    const flow = line.match(/^(\s*tags?:\s*)\[(.*)\](.*)$/)
    if (flow) {
      const before = flow[2]!.split(',').map((t) => t.trim()).filter(Boolean)
      const after = [...new Set(before.map((t) => mapTag(t, aliases)))]
      out.push(after.join() === before.join() ? line : `${flow[1]}[${after.join(', ')}]${flow[3]}`)
      continue
    }

    const single = line.match(/^(\s*tag:\s*)([^\s#[][^#]*?)(\s*(#.*)?)$/)
    if (single) {
      const tag = mapTag(single[2]!, aliases)
      out.push(tag === single[2] ? line : `${single[1]}${tag}${single[3]}`)
      continue
    }

    if (/^\s*tags?:\s*(#.*)?$/.test(line)) block = { indent, seen: new Set() }
    out.push(line)
  }
  return out.join('\n')
}

// ---- Input and output: kb's config and index, the knowledge folder, Stack Overflow. Used by tools/tags/tasks.toml.

const home = homedir()
const expand = (path: string) => path.replace(/^~(?=\/|$)/, home)

/** The knowledge folder: `sources_path` in kb's config, else the Raycast default, symlinks resolved. */
export const sourcesDir = (): string => {
  const config = join(home, '.config/kb/config.toml')
  const configured = existsSync(config)
    ? (Bun.TOML.parse(readFileSync(config, 'utf8')) as { sources_path?: string }).sources_path
    : undefined
  return realpathSync(expand(configured ?? '~/.config/kodexb/sources'))
}

/** Every tag's uses and every pair's co-occurrences, from kb's index. */
export const readIndex = (): { uses: Uses; cooccur: Cooccurrences } => {
  const db = new Database(join(home, 'Library/Application Support/kb-native/kb.db'), { readonly: true })
  const uses = Object.fromEntries(
    db.query<{ tag: string; n: number }, []>('select tag, count(*) n from entry_tags group by tag').all()
      .map((r) => [r.tag, r.n])
  )
  const cooccur = Object.fromEntries(
    db.query<{ a: string; b: string; n: number }, []>(
      'select a.tag a, b.tag b, count(*) n from entry_tags a join entry_tags b'
        + ' on a.entry_id = b.entry_id and a.tag < b.tag group by 1, 2'
    ).all().map((r) => [`${r.a} ${r.b}`, r.n])
  )
  db.close()
  return { uses, cooccur }
}

/**
 * Stack Overflow's tag synonyms, alias to master, cached in `cache`. Anonymous requests stop at page 25 (2,500 of
 * 5,300+), so three orders are merged: the most applied, the oldest and the newest.
 */
export const stackOverflowSynonyms = async (cache: string): Promise<Synonyms> => {
  if (existsSync(cache)) return JSON.parse(readFileSync(cache, 'utf8'))
  const synonyms: Synonyms = {}
  for (const order of ['sort=applied&order=desc', 'sort=creation&order=asc', 'sort=creation&order=desc']) {
    for (let page = 1; page <= 25; page++) {
      const url =
        `https://api.stackexchange.com/2.3/tags/synonyms?site=stackoverflow&pagesize=100&page=${page}&${order}`
      const body = await (await fetch(url)).json() as {
        items?: { from_tag: string; to_tag: string }[]
        has_more?: boolean
        backoff?: number
        error_message?: string
      }
      if (body.error_message) throw new Error(`Stack Exchange: ${body.error_message}`)
      for (const { from_tag, to_tag } of body.items ?? []) synonyms[from_tag] = to_tag
      if (!body.has_more) break
      await Bun.sleep(((body.backoff ?? 0) + 0.1) * 1000)
    }
  }
  mkdirSync(dirname(cache), { recursive: true })
  writeFileSync(cache, JSON.stringify(synonyms))
  return synonyms
}

/** The reviewed map, `[preferred] aliases = [...]`, as each alias to its preferred tag. */
export const readAliases = (file: string): Aliases => {
  const map = Bun.TOML.parse(readFileSync(file, 'utf8')) as Record<string, { aliases?: string[] }>
  return Object.fromEntries(
    Object.entries(map).flatMap(([preferred, { aliases = [] }]) => aliases.map((a) => [a.toLowerCase(), preferred]))
  )
}

/** The proposals as a reviewable map: trusted ones as tables, the rest as commented tables to opt into. */
export const proposalsToml = (proposals: Proposal[], uses: Uses, loose: string[]): string => {
  const quote = (s: string) => JSON.stringify(s)
  const tables = (ps: Proposal[]) =>
    [...Map.groupBy(ps, (p) => p.preferred).entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([preferred, group]) =>
        [
          ...group.map((p) => `# ${p.alias} (${uses[p.alias] ?? 0}) → ${p.signal}: ${p.reason}`),
          `[${quote(preferred)}]`,
          `aliases = [${group.map((p) => quote(p.alias)).join(', ')}]`
        ].join('\n')
      )
  const commented = (block: string) => block.split('\n').map((l) => (l.startsWith('#') ? l : `# ${l}`)).join('\n')
  return [
    '# Proposed by `mise run tags:suggest` (docs/doctrine/tag-conventions.doctrine.md). Keep, edit or delete each',
    '# table, uncomment the suggestions you agree with, then save this file as tags.toml and run `mise run tags:apply`.',
    '',
    tables(proposals.filter((p) => p.confident)).join('\n\n'),
    '',
    '# ---- Suggestions: signals that read meaning, which only you can confirm. Uncomment a table to accept it.',
    '',
    tables(proposals.filter((p) => !p.confident)).map(commented).join('\n\n'),
    '',
    '# ---- Used once, with no proposal: keep if it names something real, else add it as an alias above.',
    ...loose.map((t) => `#   ${t}`),
    ''
  ].join('\n')
}
