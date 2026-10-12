import { describe, expect, test } from 'bun:test'
import { contrast, description, LIGHT_ACCENTS, mix, pill, tagAccent, tint, typeAccent } from './color.ts'

describe('colour arithmetic', () => {
  test('mixes and measures contrast as WCAG does', () => {
    expect(mix('#FFFFFF', '#000000', 0.5)).toBe('#808080')
    expect(contrast('#FFFFFF', '#000000')).toBeCloseTo(21, 5)
    expect(contrast('#000000', '#FFFFFF')).toBe(contrast('#FFFFFF', '#000000'))
  })

  test('lifts pill text only as far as AA needs', () => {
    const { background, text } = pill(LIGHT_ACCENTS, 'yellow')
    expect(background).toBe(mix(LIGHT_ACCENTS.yellow!, '#FFFFFF', 0.2))
    expect(contrast(text, background)).toBeGreaterThanOrEqual(4.5)
    expect(pill({ window: '#000000', text: '#FFFFFF', yellow: '#FFFF00' }, 'yellow').text).toBe('#FFFF00')
  })
})

describe('descriptions', () => {
  const roles = {
    window: '#23273A',
    selection: '#2A3047',
    raised: '#24293E',
    text: '#CBD4FF',
    textMuted: '#B2BADB',
    textFaint: '#7A86B6'
  }

  test('dim from textMuted toward textFaint only as far as AA holds on every surface', () => {
    const shade = description(roles)
    expect(shade).toBe('#909BC5')
    for (const surface of ['window', 'selection', 'raised']) {
      expect(contrast(shade, roles[surface as keyof typeof roles])).toBeGreaterThanOrEqual(4.5)
    }
    expect(contrast(mix(roles.textFaint, roles.textMuted, 0.7), roles.selection)).toBeLessThan(4.5)
  })
})

describe('translucent highlights', () => {
  test('find the tint that shows as the target over its surface', () => {
    expect(tint('#2A3047', '#23273A', 0.2)).toBe('#46547B33')
    expect(mix('#46547B', '#23273A', 0.2)).toBe('#2A3047')
  })

  test('refuse a target no tint at that opacity can reach', () => {
    expect(() => tint('#FFFFFF', '#000000', 0.2)).toThrow('#FFFFFF')
  })
})

describe('accents', () => {
  test('hash a tag to one accent whatever its case, as the Raycast extension', () => {
    expect(tagAccent('Git')).toBe(tagAccent('git'))
    expect(tagAccent('git')).toBe('red') // crates/kb-app/tests/fixtures/tag-accents.json
  })

  test('tint the known types, and hash the others', () => {
    expect(typeAccent('bookmark')).toBe('purple')
    expect(typeAccent('skill')).toBe(tagAccent('skill'))
  })
})
