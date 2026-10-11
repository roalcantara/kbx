import { describe, expect, test } from 'bun:test'
import { normalize, type Proposal, propose, rewrite, violations } from './tags.ts'

const byAlias = (proposals: Proposal[]) =>
  Object.fromEntries(proposals.map((p) => [p.alias, [p.preferred.join(' + '), p.signal]]))

describe('convention', () => {
  test('normalizes case, separators and spaces', () => {
    expect(normalize(' Semantic_Release ')).toBe('semantic-release')
    expect(normalize('virtual  desktops')).toBe('virtual-desktops')
    expect(normalize('c++')).toBe('c++')
  })

  test('reports what breaks the convention', () => {
    expect(violations('semantic-release')).toEqual([])
    expect(violations('node.js')).toEqual([])
    expect(violations('Semantic_Release')).toEqual(['not lowercase', 'underscore or space: use a hyphen'])
    expect(violations('a')).toEqual(['shorter than 2'])
    expect(violations('x'.repeat(31))).toEqual(['longer than 30'])
    expect(violations('café')).toEqual(['not ASCII'])
  })
})

describe('signals', () => {
  test('merge case and separator variants into the most used', () => {
    expect(byAlias(propose({ 'semantic-release': 3, semantic_release: 1 }, {}, { synonyms: {} }))).toEqual({
      semantic_release: ['semantic-release', 'fingerprint']
    })
  })

  test('merge a singular into its existing plural, repairing stripped proper names', () => {
    const got = byAlias(propose({ jobs: 5, job: 2, maco: 9, macos: 4 }, {}, { synonyms: {} }))
    expect(got.job).toEqual(['jobs', 'plural'])
    expect(got.maco).toEqual(['macos', 'plural'])
  })

  test('merge a renamed singular into its plural', () => {
    const got = byAlias(propose({ best_practice: 1, 'best practices': 1, 'best-practices': 2 }, {}, { synonyms: {} }))
    expect(got.best_practice?.[0]).toBe('best-practices')
    expect(got['best practices']?.[0]).toBe('best-practices')
  })

  test("follow Stack Overflow's synonyms", () => {
    const got = byAlias(propose({ k8s: 2, kubernetes: 1 }, {}, { synonyms: { k8s: 'kubernetes' } }))
    expect(got.k8s).toEqual(['kubernetes', 'stackoverflow'])
  })

  test('flag near spellings, never short distinct tags', () => {
    const got = byAlias(propose({ postgres: 5, postgress: 1, ai: 9, ui: 4 }, {}, { synonyms: {} }))
    expect(got.postgress).toEqual(['postgres', 'similar'])
    expect(got.ui).toBeUndefined()
    expect(got.ai).toBeUndefined()
  })

  test('flag tags that always come together', () => {
    const got = byAlias(propose({ fabric: 10, fab: 10 }, { 'fab fabric': 10 }, { synonyms: {} }))
    expect(got.fab?.[1]).toBe('together')
  })

  test('trust spelling signals, and only suggest the ones that read meaning', () => {
    const confident = (uses: Record<string, number>, synonyms = {}, cooccur = {}) =>
      Object.fromEntries(propose(uses, cooccur, { synonyms }).map((p) => [p.alias, p.confident]))
    expect(confident({ jobs: 5, job: 2 })).toEqual({ job: true })
    expect(confident({ shells: 3, shell: 68 })).toEqual({ shell: false })
    expect(confident({ gpt: 2 }, { gpt: 'google-publisher-tag' })).toEqual({ gpt: false })
    expect(confident({ png: 4, svg: 4 }, {}, { 'png svg': 4 })).toEqual({ png: false })
  })

  test('split a compound whose words are all tags', () => {
    const got = byAlias(
      propose({ coding_patterns: 2, 'coding-pattern': 1, coding: 9, pattern: 30, patterns: 3 }, {}, {})
    )
    expect(got['coding-pattern']).toEqual(['coding + patterns', 'split'])
    expect(got.coding_patterns?.[0]).toBe('coding + patterns')
  })

  test('shorten to an established abbreviation', () => {
    const got = byAlias(propose({ 'amazon-web-services': 4 }, {}, { abbreviations: { 'amazon-web-services': 'aws' } }))
    expect(got['amazon-web-services']).toEqual(['aws', 'shorten'])
  })

  test('cluster tags naming the same concept, in any language, into the shortest', () => {
    const concepts = { database: 'Q8513', db: 'Q8513', 'banco-de-dados': 'Q8513', data: 'Q5227137' }
    const got = byAlias(propose({ database: 5, db: 9, 'banco-de-dados': 2, data: 4 }, {}, { concepts }))
    expect(got.database).toEqual(['db', 'concept'])
    expect(got['banco-de-dados']).toEqual(['db', 'concept'])
    expect(got.data).toBeUndefined()
  })

  test('let a reviewed judgement win over the guessing signals, without loops', () => {
    const semantic = { shells: { preferred: ['shell'], reason: 'the Unix shell' } }
    const got = byAlias(propose({ shell: 68, shells: 3 }, {}, { semantic }))
    expect(got).toEqual({ shells: ['shell', 'semantic'] })
  })

  test('drop a tag judged vague', () => {
    const semantic = { general: { preferred: [], reason: 'says nothing' } }
    const got = propose({ general: 2 }, {}, { semantic })
    expect(got).toEqual([{
      alias: 'general',
      preferred: [],
      signal: 'semantic',
      reason: 'says nothing',
      confident: false
    }])
  })

  test('resolve chains to their end, so no tag is both alias and preferred', () => {
    const got = byAlias(
      propose({ k8s: 1, kube: 3, kubernetes: 2 }, {}, { synonyms: { k8s: 'kube', kube: 'kubernetes' } })
    )
    expect(got.k8s?.[0]).toBe('kubernetes')
    expect(got.kube?.[0]).toBe('kubernetes')
  })
})

describe('tag lines', () => {
  const aliases = { maco: ['macos'], job: ['jobs'] }

  test('rewrite flow lists, single tags and block lists, and nothing else', () => {
    const text = 'a:\n  title: maco tips # maco\n  tags: [maco, shell]\n'
      + 'b:\n  tag: job\n'
      + 'c:\n  tags:\n    - job\n    - jobs\n  notes: job\n'
    expect(rewrite(text, aliases)).toBe(
      'a:\n  title: maco tips # maco\n  tags: [macos, shell]\n'
        + 'b:\n  tag: jobs\n'
        + 'c:\n  tags:\n    - jobs\n  notes: job\n'
    )
  })

  test('expand an alias into several tags', () => {
    const split = { 'coding-pattern': ['coding', 'patterns'] }
    expect(rewrite('a:\n  tags: [coding-pattern, coding]\n', split)).toBe('a:\n  tags: [coding, patterns]\n')
    expect(rewrite('a:\n  tag: coding-pattern\n', split)).toBe('a:\n  tags: [coding, patterns]\n')
    expect(rewrite('a:\n  tags:\n    - coding-pattern\n', split)).toBe('a:\n  tags:\n    - coding\n    - patterns\n')
  })

  test('drop a tag mapped to nothing', () => {
    const drop = { general: [] }
    expect(rewrite('a:\n  tags: [general, git]\n', drop)).toBe('a:\n  tags: [git]\n')
    expect(rewrite('a:\n  tags:\n    - general\n    - git\n', drop)).toBe('a:\n  tags:\n    - git\n')
  })

  test('leave text without aliases byte for byte', () => {
    const text = 'a:\n  tags: [shell]   # spacing kept\n'
    expect(rewrite(text, aliases)).toBe(text)
  })
})
