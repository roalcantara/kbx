# 0008. kbx stays its own code; other tools' data and content are its adoption path

- Status: accepted
- Date: 2026-10-13

## Context

Before investing further in kbx, the owner asked three questions, each answered by a ladder of cheap disproof passes
(research notes of 2026-10-13, kept in the git history):

1. **Is a Rust/GPUI launcher in Raycast's vein worth building as a product?** No, for the owner's bar (a sustainable
   niche: 2–3k engaged users and revenue covering maintenance in 12 months). Adoption needs a wedge, and the one proven
   wedge (Linux plus Raycast's extensions) is Vicinae's; sponsorship does not fund maintenance — the most-starred
   open-source launcher earns about $720 a year — so revenue would need a paid product against Raycast's free tier
   (launcher product check). Agent management, the
   AI angle examined first, is already served inside Raycast by Store extensions; the open gaps need a resident engine,
   not a launcher.
2. **Is kbx worth building, by adoption (2–3k engaged users in 12 months) and as a portfolio piece?** Yes. Keeping and
   finding one's own commands is a large job (tldr, cheat.sh, Atuin, navi in five figures of stars), established tools
   sit at the bar (navi and cheat: 3–4k Homebrew installs a year), new entrants are few, and the incumbents live in
   the terminal: none is a native panel from anywhere over commands, cheat sheets, bookmarks and tasks on macOS,
   Linux and Windows (kbx product check).
3. **Build kbx from its own code, fork a Rust/GPUI tool, or port one from another stack?** Build. Every Rust/GPUI
   candidate (Steward, Corvo, zlaunch, Loungy, Script Kit GPUI) is an app launcher, not kbx's job, and on an older GPUI
   line; the nearest job in another stack, massCode, is AGPL and Electron. But navi ships a library crate
   (Apache-2.0), Atuin keeps its history in SQLite (MIT), pet, cheat and navi use simple formats, and tldr's pages are
   open content (CC BY 4.0).

kbx's YAML format is not a defining trait (owner): its storage can change.

## Decision

- kbx stays its own code, on its current stack (Rust, GPUI Kit). No fork, no port.
- Its adoption path is other tools' data and content, not their code: importers for navi, cheat and pet files and
  Atuin's history, so users arrive with their commands; tldr's pages as bundled content, so kbx is useful on first
  launch. Each comes after the MVP, as engine changes do (they change what `kbx-domain` reads).
- A Rust/GPUI launcher product is not pursued.

## Consequences

- The importers and bundled content are TODO tasks; each needs its own design (mapping, licence notices, what the
  sync report says about imported entries).
- Reusing navi's crate adds an Apache-2.0 dependency, compatible with kbx's MIT licence (cargo-deny's allow-list may
  need it); tldr's content needs CC BY 4.0 attribution where it is shown.
