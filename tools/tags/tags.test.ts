import { describe, expect, test } from 'bun:test'
import { normalize, type Proposal, propose, rewrite, violations } from './tags.ts'

const byAlias = (proposals: Proposal[]) => Object.fromEntries(proposals.map((p) => [p.alias, [p.preferred, p.signal]]))

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
    expect(byAlias(propose({ 'semantic-release': 3, semantic_release: 1 }, {}, {}))).toEqual({
      semantic_release: ['semantic-release', 'fingerprint']
    })
  })

  test('merge a singular into its existing plural, repairing stripped proper names', () => {
    const got = byAlias(propose({ jobs: 5, job: 2, maco: 9, macos: 4 }, {}, {}))
    expect(got.job).toEqual(['jobs', 'plural'])
    expect(got.maco).toEqual(['macos', 'plural'])
  })

  test("follow Stack Overflow's synonyms", () => {
    const got = byAlias(propose({ k8s: 2, kubernetes: 1 }, {}, { k8s: 'kubernetes' }))
    expect(got.k8s).toEqual(['kubernetes', 'stackoverflow'])
  })

  test('flag near spellings, never short distinct tags', () => {
    const got = byAlias(propose({ postgres: 5, postgress: 1, ai: 9, ui: 4 }, {}, {}))
    expect(got.postgress).toEqual(['postgres', 'similar'])
    expect(got.ui).toBeUndefined()
    expect(got.ai).toBeUndefined()
  })

  test('flag tags that always come together', () => {
    const got = byAlias(propose({ fabric: 10, fab: 10 }, { 'fab fabric': 10 }, {}))
    expect(got.fab?.[1]).toBe('together')
  })

  test('trust spelling signals, and only suggest the ones that read meaning', () => {
    const confident = (uses: Record<string, number>, synonyms = {}, cooccur = {}) =>
      Object.fromEntries(propose(uses, cooccur, synonyms).map((p) => [p.alias, p.confident]))
    expect(confident({ jobs: 5, job: 2 })).toEqual({ job: true })
    expect(confident({ shells: 3, shell: 68 })).toEqual({ shell: false })
    expect(confident({ gpt: 2 }, { gpt: 'google-publisher-tag' })).toEqual({ gpt: false })
    expect(confident({ png: 4, svg: 4 }, {}, { 'png svg': 4 })).toEqual({ png: false })
  })

  test('resolve chains to their end, so no tag is both alias and preferred', () => {
    const got = byAlias(propose({ k8s: 1, kube: 3, kubernetes: 2 }, {}, { k8s: 'kube', kube: 'kubernetes' }))
    expect(got.k8s?.[0]).toBe('kubernetes')
    expect(got.kube?.[0]).toBe('kubernetes')
  })
})

describe('tag lines', () => {
  const aliases = { maco: 'macos', job: 'jobs' }

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

  test('leave text without aliases byte for byte', () => {
    const text = 'a:\n  tags: [shell]   # spacing kept\n'
    expect(rewrite(text, aliases)).toBe(text)
  })
})
