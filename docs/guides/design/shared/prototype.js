// kb prototypes: expands the component tags each screen is written in, then draws the Lucide icons
// the app draws (GPUI Kit's IconName set). One tag per app component (Plan 8, `crates/kb-app/src/ui/`).
//
//   <kb-row type="commands" title="docker ps -a" desc="…" tags="docker,containers" links="2" selected compact>
//   <kb-row favicon="https://…" …>          a bookmark's favicon, drawn bare instead of the tile
//   <kb-tag name="git">                      a tag pill in its hashed accent
//   <kb-keys keys="⌘ K">                     one keycap per key
//   <kb-action icon="copy" label="…" keys="⌘ ↵" selected drill mark="on|off">
//   <kb-section label="Types">               a popover section header (label-xs)
//   <kb-header query="kube" placeholder="…" filter="All Entries" open back-only>
//   <kb-footer status="ready|busy|none" count="12" context-type="commands" context="docker ps -a"
//              favicon="…" primary="Show Details" actions>

// Raycast's TYPE_TINTS and kb's icons (presentation.rs, type_badge).
const TYPES = {
  bookmark: ['globe', 'purple'],
  command: ['square-terminal', 'green'],
  cheat: ['file-text', 'blue'],
  note: ['notebook-text', 'orange'],
  task: ['circle-check', 'magenta']
}
// The tag hash of the Raycast extension (presentation.rs, tag_hue; tools/design/color.ts).
const TAG_ORDER = ['blue', 'orange', 'green', 'purple', 'magenta', 'yellow', 'red']
const hash = (s) => [...s].reduce((acc, c) => (acc * 31 + c.codePointAt(0)) % 100003, 0)
const tagAccent = (tag) => TAG_ORDER[hash(tag.toLowerCase()) % TAG_ORDER.length]
const singular = (t) => t.trim().toLowerCase().replace(/s$/, '')
const typeBadge = (t) => TYPES[singular(t)] ?? ['file-text', tagAccent(singular(t))]
const MAX_ROW_TAGS = 3
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])

const tile = (icon, accent) => `<span class="tile ${accent}"><i data-lucide="${icon}"></i></span>`
// A favicon, drawn bare; the type's tile when it fails to load, as the app's `img().with_fallback`.
const favicon_ = (url, icon, accent) =>
  `<img class="favicon" src="${esc(url)}" alt="" data-fallback="${icon} ${accent}">`

const keys = (spec) =>
  `<span class="keys">${spec.split(' ').filter(Boolean).map((k) => `<kbd>${esc(k)}</kbd>`).join('')}</span>`
const tag = (name) => `<span class="tag ${tagAccent(name)}">#${esc(name)}</span>`

function row(el) {
  const [icon, accent] = typeBadge(el.getAttribute('type') ?? 'notes')
  const favicon = el.getAttribute('favicon')
  const lead = favicon ? favicon_(favicon, icon, accent) : tile(icon, accent)
  const compact = el.hasAttribute('compact')
  const selected = el.hasAttribute('selected')
  const tags = (el.getAttribute('tags') ?? '').split(',').map((t) => t.trim()).filter(Boolean)
  const shown = tags.slice(0, MAX_ROW_TAGS)
  const more = tags.length - shown.length
  const links = Number(el.getAttribute('links') ?? 0)
  const desc = el.getAttribute('desc')
  const trail = compact
    ? ''
    : [
      ...shown.map(tag),
      more > 0 ? `<span class="count label-sm">+${more}</span>` : '',
      links > 0 ? `<span class="count label-sm"><i data-lucide="link"></i>${links}</span>` : '',
      selected ? keys('↵') : ''
    ].join('')
  return `<div class="row${selected ? ' selected' : ''}"><div class="fill">
    <span class="lead">${lead}</span>
    <span class="text"><span class="title body-lg">${esc(el.getAttribute('title'))}</span>${
    !compact && desc ? `<span class="desc body-sm description">${esc(desc)}</span>` : ''
  }</span>
    ${trail ? `<span class="trail">${trail}</span>` : ''}
  </div></div>`
}

function action(el) {
  const mark = el.getAttribute('mark')
  const lead = mark
    ? `<span class="icon mark ${mark}"><i data-lucide="${mark === 'on' ? 'circle-check' : 'circle'}"></i></span>`
    : `<span class="icon"><i data-lucide="${esc(el.getAttribute('icon'))}"></i></span>`
  const trail = el.hasAttribute('drill')
    ? '<span class="drill"><i data-lucide="chevron-right"></i></span>'
    : el.getAttribute('keys')
    ? keys(el.getAttribute('keys'))
    : ''
  return `<div class="action${el.hasAttribute('selected') ? ' selected' : ''}">${lead}<span class="label body-md">${
    esc(el.getAttribute('label'))
  }</span>${trail}</div>`
}

function header(el) {
  const query = el.getAttribute('query')
  const field = el.hasAttribute('back-only')
    ? ''
    : `<div class="query headline-md">${
      query
        ? `${esc(query)}<span class="caret"></span>`
        : `<span class="caret"></span><span class="placeholder">${
          esc(el.getAttribute('placeholder') ?? 'Search Knowledge Base…')
        }</span>`
    }</div>`
  const filter = el.hasAttribute('back-only')
    ? ''
    : `<div class="filter-trigger label-md"><i data-lucide="funnel"></i><span>${
      esc(el.getAttribute('filter') ?? 'All Entries')
    }</span><i class="chevron" data-lucide="${el.hasAttribute('open') ? 'chevron-up' : 'chevron-down'}"></i></div>`
  return `<div class="header"><span class="back"><i data-lucide="chevron-left"></i></span>${field}${filter}</div>`
}

const STATUS = { ready: 'Ready', busy: 'Rebuilding…', none: 'No index' }
function footer(el) {
  const status = el.getAttribute('status')
  const count = el.getAttribute('count')
  const left = status
    ? `<span class="status body-md"><span class="dot ${status}"></span><span>${STATUS[status]}</span>${
      count !== null
        ? `<span class="sep">•</span><span>${Number(count).toLocaleString('en-US')} result${
          count === '1' ? '' : 's'
        }</span>`
        : ''
    }</span>`
    : el.getAttribute('context')
    ? `<span class="context body-md">${
      el.getAttribute('favicon')
        ? favicon_(el.getAttribute('favicon'), ...typeBadge(el.getAttribute('context-type') ?? 'bookmarks'))
        : tile(...typeBadge(el.getAttribute('context-type')))
    }<span class="name">${esc(el.getAttribute('context'))}</span></span>`
    : ''
  const primary = el.getAttribute('primary')
  const right = [
    primary ? `<span class="item label-md">${esc(primary)}${keys('↵')}</span>` : '',
    `<span class="item label-md">Actions${keys('⌘ K')}</span>`
  ].join('')
  return `<div class="footer"><div class="side">${left}</div><div class="side">${right}</div></div>`
}

const RENDER = {
  'kb-header': header,
  'kb-footer': footer,
  'kb-row': row,
  'kb-tag': (el) => tag(el.getAttribute('name')),
  'kb-keys': (el) => keys(el.getAttribute('keys')),
  'kb-action': action,
  // kb's own section labels are uppercase; one taken from the user's data (a link's title or URL) keeps its case.
  'kb-section': (el) =>
    `<div class="section label-xs${el.hasAttribute('verbatim') ? ' verbatim' : ''}">${
      esc(el.getAttribute('label'))
    }</div>`
}

for (const [name, render] of Object.entries(RENDER)) {
  for (const el of [...document.querySelectorAll(name)]) el.outerHTML = render(el)
}
const icons = () => lucide.createIcons({ attrs: { 'stroke-width': 2 } })
const fallback = (img) => {
  const [icon, accent] = img.dataset.fallback.split(' ')
  img.outerHTML = tile(icon, accent)
  icons()
}
for (const img of document.querySelectorAll('img[data-fallback]')) {
  if (img.complete && img.naturalWidth === 0) fallback(img)
  else img.addEventListener('error', () => fallback(img))
}
icons()
// A popover's list scrolls its selected row into view, as the app's does.
const reveal = () => {
  for (const items of document.querySelectorAll('.popover .items')) {
    const selected = items.querySelector('.selected')
    if (!selected) continue
    const [box, row] = [items.getBoundingClientRect(), selected.getBoundingClientRect()]
    if (row.bottom > box.bottom) items.scrollTop += row.bottom - box.bottom + (box.height - row.height) / 2
  }
}
document.fonts.ready.then(reveal)
