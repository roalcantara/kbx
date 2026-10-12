// Colour arithmetic shared by the design kit's tasks (tasks.toml), so they agree on every derived colour.
export const ACCENTS = ['blue', 'green', 'orange', 'purple', 'magenta', 'yellow', 'red'] as const
export type Accent = (typeof ACCENTS)[number]
/** A palette's roles: surfaces, text and accents, as `#RRGGBB`. */
export type Roles = Record<string, string>

// A tag pill is its accent at 20% over the window; its text is the accent, lifted toward `text` in 5% steps only as far
// as WCAG AA (4.5:1) needs (DESIGN.md, Tags).
export const PILL_ALPHA = 0.2
export const AA = 4.5

export const rgb = (h: string): [number, number, number] => {
  const n = Number.parseInt(h.slice(1, 7), 16)
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}
export const toHex = (c: number[]): string =>
  `#${c.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('').toUpperCase()}`
export const mix = (fg: string, bg: string, a: number): string =>
  toHex(rgb(fg).map((v, i) => v * a + rgb(bg)[i]! * (1 - a)))

/** The colour that, drawn at `alpha` over `under`, shows as `target`, as `#RRGGBBAA`: for the theme keys GPUI Kit draws
 * as translucent highlights (it caps their opacity). Throws when no colour at that opacity reaches the target. */
export const tint = (target: string, under: string, alpha: number): string => {
  const channels = rgb(target).map((v, i) => rgb(under)[i]! + (v - rgb(under)[i]!) / alpha)
  if (channels.some((v) => v < -0.5 || v > 255.5)) throw new Error(`No tint at ${alpha} shows ${target} over ${under}`)
  return `${toHex(channels)}${Math.round(alpha * 255).toString(16).padStart(2, '0').toUpperCase()}`
}

export const lin = (v: number): number => {
  const c = v / 255
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
}
export const luminance = (h: string): number => {
  const [r, g, b] = rgb(h).map(lin) as [number, number, number]
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}
export const contrast = (a: string, b: string): number => {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p) as [number, number]
  return (x + 0.05) / (y + 0.05)
}

// Type tints and the tag hash, as the Raycast extension's accessories.ts: a tag keeps one accent everywhere.
export const TYPE_ACCENTS: Record<string, Accent> = {
  bookmark: 'purple',
  command: 'green',
  cheat: 'blue',
  note: 'orange',
  task: 'magenta'
}
export const TAG_ORDER: Accent[] = ['blue', 'orange', 'green', 'purple', 'magenta', 'yellow', 'red']
export const MAX_TAGS = 3
const hash = (value: string): number => [...value].reduce((acc, char) => (acc * 31 + char.codePointAt(0)!) % 100_003, 0)
export const tagAccent = (tag: string): Accent => TAG_ORDER[hash(tag.toLowerCase()) % TAG_ORDER.length]!
// `type` is the type key: the entry type singular and lowercased (`commands` → `command`).
export const typeAccent = (type: string): Accent => TYPE_ACCENTS[type] ?? TAG_ORDER[hash(type) % TAG_ORDER.length]!

export const pill = (roles: Roles, accent: string): { background: string; text: string } => {
  const background = mix(roles[accent]!, roles.window!, PILL_ALPHA)
  let text = roles[accent]!
  for (let lift = 0.05; contrast(text, background) < AA && lift <= 1; lift += 0.05) {
    text = mix(roles.text!, roles[accent]!, lift)
  }
  return { background, text }
}

/** The description's text (list rows, placeholders: GPUI Kit's `muted.foreground`): textMuted dimmed toward textFaint
 * in 10% steps, as far as WCAG AA holds on the window, the selection and popovers, so it reads apart from titles. */
export const description = (roles: Roles): string => {
  let shade = roles.textMuted!
  for (let step = 0.1; step <= 1.0001; step += 0.1) {
    const next = mix(roles.textFaint!, roles.textMuted!, step)
    if (['window', 'selection', 'raised'].some((s) => contrast(next, roles[s]!) < AA)) break
    shade = next
  }
  return shade
}

// Light accents (Tailwind 600) over a white window, for GPUI Kit's default light theme: kb is designed for dark.
export const LIGHT_ACCENTS: Roles = {
  window: '#FFFFFF',
  text: '#0A0A0A',
  blue: '#2563EB',
  green: '#16A34A',
  orange: '#EA580C',
  purple: '#9333EA',
  magenta: '#DB2777',
  yellow: '#CA8A04',
  red: '#DC2626'
}
