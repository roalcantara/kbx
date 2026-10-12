import { describe, expect, test } from 'bun:test'
import { contrast, LIGHT_ACCENTS, mix, pill, tagAccent, typeAccent } from './color.ts'

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
