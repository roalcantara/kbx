# Asyar survey: claims refuted and refined

- Date: 2026-10-13
- Reviews: [2026-10-12-asyar.md](2026-10-12-asyar.md)
- Method: each claim was first tried against evidence that could refute it (GitHub repositories and their manifests,
  kb's own measurements in `docs/guides/results/archive/2026-10-10-pre-polish/`, crates.io, Raycast's engineering
  blog). A claim that survives is rated for its value to kb, now and later.

## 1. The central claim: "there is no Rust/GPUI equivalent of Raycast"

The Asyar report does not state this outright. It follows from the report's framing (Asyar is a Tauri/webview app;
kb is "far leaner" because it is native GPUI) and from our own earlier survey (memory note, 2026-10-12: "No usable
Rust/GPUI product: Nohrs pre-alpha, Loungy archived"). That earlier survey used the GPUI Kit gallery, which apps join
by opting in, so it saw only six launcher candidates.

**Refuted.** A search of GitHub's `gpui` topic (644 repositories) and of README text finds at least ten Rust/GPUI
launchers, several of them explicitly Raycast-like:

| Project                                                                                                                                                                                         | Stack (from its `Cargo.toml`)      | Licence    | Activity (2026-10-13)            | Raycast-like scope                                                                                                                                   |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- | ---------- | -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| [johnlindquist/script-kit-next](https://github.com/johnlindquist/script-kit-next)                                                                                                               | GPUI + vendored gpui-component 0.5 | Apache-2.0 | 4,328 commits, pushed 2026-09-04 | Script Kit (a Raycast competitor) rewritten on GPUI by its author; Bun scripts, prompts, MCP; macOS                                                  |
| [diegoleteliers10/corvo](https://github.com/diegoleteliers10/corvo)                                                                                                                             | Rust + GPUI                        | MIT        | 104 commits since 2026-09-21     | "A raycast alternative": apps, window tiling, clipboard, snippets, quicklinks, notes, calculator; compiled-in Rust extensions; macOS, Linux, Windows |
| [panosAthDBX/photoncast](https://github.com/panosAthDBX/photoncast)                                                                                                                             | GPUI from Zed's repository         | none       | 166 commits, pushed 2026-06-24   | Apps, clipboard (encrypted), calculator, window management, calendar, quicklinks, ABI-stable extensions, Catppuccin; macOS                           |
| [pane-app/pane](https://github.com/pane-app/pane)                                                                                                                                               | GPUI CE                            | GPL-3.0    | 508 commits since 2026-09-27     | "Inspired by Raycast": root search, settings, WASI extensions in Rust/JS/TS; macOS, Linux, Windows; early                                            |
| [MatthiasGrandl/Loungy](https://github.com/MatthiasGrandl/Loungy)                                                                                                                               | GPUI                               | MIT        | archived 2025-10                 | "In the vein of Spotlight, Alfred, Raycast"; 1,730 stars; points to Gauntlet and Vicinae                                                             |
| [frostymur/aerofi](https://github.com/frostymur/aerofi)                                                                                                                                         | GPUI from Zed's repository         | MIT        | 459 commits since 2026-08-28     | rofi-style launcher; runs Raycast Script Commands; TOML design tokens; 25–35 MB; macOS                                                               |
| [iFence/steward](https://github.com/iFence/steward)                                                                                                                                             | `gpui-pre 0.3.7` (kb's GPUI line)  | MIT        | 105 commits                      | Launcher and plugin platform; Windows                                                                                                                |
| [zortax/zlaunch](https://github.com/zortax/zlaunch)                                                                                                                                             | GPUI + gpui-component              | MIT        | 227 stars                        | Application launcher; Wayland                                                                                                                        |
| [andre-brandao/gpui-shell](https://github.com/andre-brandao/gpui-shell)                                                                                                                         | GPUI                               | none       | 148 stars                        | Desktop shell with a launcher; Hyprland, Niri                                                                                                        |
| [starc007/better-spotlight](https://github.com/starc007/better-spotlight), [Oya-Tomo/batto](https://github.com/Oya-Tomo/batto), [swarajbachu/spacekey](https://github.com/swarajbachu/spacekey) | GPUI                               | MIT / none | small                            | Spotlight and "Raycast-like" launchers; early                                                                                                        |

Clipboard launchers on GPUI exist too (ropy 194★, clippi 407★, KwikPaste, pasta).

**Refined.** What the evidence does support:

- Nobody has published a GPUI launcher that runs **Raycast's own extensions**. The projects that do run them are
  not GPUI: [Vicinae](https://github.com/vicinaehq/vicinae) (C++/Qt, GPL-3.0, 10k stars, "mostly compatible"
  extension API) and Flare (Svelte, archived). A bridge for React on GPUI exists:
  [remorses/gpuix](https://github.com/remorses/gpuix) (Node and Bun bindings for GPUI, 2.5k stars).
- None of the GPUI launchers is mature by adoption: Loungy, the most starred, is archived; most others are 2026
  projects with 0–30 stars, several visibly agent-built.
- One data point cuts the other way: [Photon](https://github.com/ryan-stoffel/photon) shipped a Rust/GPUI rewrite
  (v0.3.0) and went back to Swift in 0.3.1 because it "failed the native parity gate" (macOS panel, Dock, focus,
  hotkey behaviour).
- Raycast itself moved away from a native UI: Raycast 2.0 is native hosts (Swift on macOS, C# on Windows) around one
  React web frontend in web views, a Node backend, and a Rust core for the file indexer and the sync schema
  ([Raycast's engineering post](https://www.raycast.com/blog/a-technical-deep-dive-into-the-new-raycast)).

### Value to kb

- **Today:** none of these replaces kb. Its niche — a hotkey panel over the owner's own YAML with a view per entry
  type — is not what they build; they are app launchers with clipboard and window tools. Plan 8 stands.
- **Prior art to read** (buy over build), all MIT or Apache-2.0: Corvo (extension module: list, actions, cache,
  preferences; frecency), Script Kit GPUI (the most worked GPUI launcher; prompts, editor, terminal), aerofi (TOML
  design tokens, Carbon hotkeys without Accessibility, Raycast Script Commands), Steward (the same `gpui-pre` line as
  kb). Do not copy from PhotonCast (no licence), Pane or Vicinae (GPL-3.0).
- **Tomorrow, a real option:** kb as an _extension_ of a GPUI launcher instead of a standalone panel. Corvo takes
  compiled-in Rust crates; Pane takes WASI components. kb-domain has no UI dependency, so it could sit under either.
  This is the owner's call; it is not evaluated here.
- **On a "Rust/GPUI Raycast port":** the evidence argues against starting one. The space already has a dozen
  attempts, Raycast's extension API is React on Node (a port would need a JavaScript runtime and a React reconciler
  for GPUI, as Vicinae built for Qt), and Raycast chose web views for its own 2.0.

## 2. The report's own claims

| #  | Claim (report section)                                                                                | Verdict         | Evidence                                                                                                                                                                                               |
| -- | ----------------------------------------------------------------------------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1  | kb is "a pure Rust binary idling at ~30–40 MiB" (§7.5)                                                | **Refuted**     | kb's own tier 2 run (archived, pre-polish): idle footprint p50 52 MiB, RSS p50 94 MiB; 127 MiB footprint after 100 queries. aerofi claims 25–35 MB; kb is not there.                                   |
| 2  | GPUI gives kb "sub-10 ms response times" (§7.1)                                                       | **Refined**     | True for the action: ⌘↵ to clipboard p50 2.7 ms. False for showing the panel: p50 36.9 ms, slower than Raycast's 30.5 ms in the same archived session. Keystroke latency was not measured in that set. |
| 3  | Asyar "idles at ~150–250 MiB" (§7.5)                                                                  | Unverified      | The report cites no measurement. Raycast v1 measured 242 MiB RSS idle in kb's session, so the range is plausible, not shown.                                                                           |
| 4  | kb "fully implements" the ↵/⌘↵/⇧⌘↵ ladder and nested submenus (§7.3)                                  | **Refuted**     | Planned (spec D7–D9; Plan 8d–8e), not implemented: the code on `feat/design-system` has the old palette (Copy…, Links pages).                                                                          |
| 5  | kb "uses a compile-time validated `catalog.toml` deep-merged under `config.toml`" (§7.4)              | **Refuted**     | Planned (D18, Plan 8b), not implemented; and the plan validates at start (with a unit test for the shipped file), not at compile time.                                                                 |
| 6  | Asyar "restricts the footer to a single primary action" and has no submenus (§7.3, §3 Area 4)         | Stands          | Cited to source lines; consistent with Asyar's README. Value to kb: confirms D8–D9; nothing to adopt.                                                                                                  |
| 7  | Asyar's action popup has its search at the bottom and Esc clears the query before closing (§3 Area 4) | Stands          | Cited to source lines. Value: adopted into Plan 8e (Esc order).                                                                                                                                        |
| 8  | Paste via `CGEventPostToPid` needs no third-party crate (§1, §8.1)                                    | Stands, refined | True; it needs the Accessibility permission, which kb's probes already need. Value: high for the Paste task (TODO `SeKS`). Photon and Corvo implement the same and are MIT prior art.                  |
| 9  | `fend-core` 1.5.8 is MIT (§8.4)                                                                       | Stands          | crates.io: 1.5.8, MIT. Value to kb: low; a calculator is outside kb's scope (PRODUCT.md).                                                                                                              |
| 10 | "Top-weighted" placement (`bias: 0.16`) keeps the panel off the Dock and menu bar (§8.2)              | Stands, refined | A placement formula, not a fix for a kb bug: kb centres on the active display today. Value: medium, once multi-display matters.                                                                        |
| 11 | Process-group `phys_footprint` and `powermetrics` improve kb's bench (§8.3)                           | Stands          | kb already measures `footprint` per process (tier 2); summing a process group matters for Raycast (several processes), not kb (one). Value: medium for the comparison's fairness.                      |
| 12 | Asyar is GPL-3.0, so kb copies no code (§2)                                                           | Stands          | The repository's licence is GPL-3.0. Keep the clean-room rule.                                                                                                                                         |

## 3. What changes

- Correct the 2026-10-12 market-survey memory: GPUI launchers exist in number; none runs Raycast extensions; none is
  mature by adoption.
- The Asyar report's §7.1, §7.3, §7.4 and §7.5 should be read with the verdicts above; its adoptions (§8) stand.
- No Plan 8 decision changes. The owner may want to weigh "kb as an extension of Corvo or Pane" after the
  experiment's report.
