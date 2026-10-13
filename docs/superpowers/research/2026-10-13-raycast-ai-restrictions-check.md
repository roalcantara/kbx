# Does Raycast forbid any AI-related feature? (phase 0 check)

- Date: 2026-10-13
- Purpose: a short attempt to refute the assumption the planned "AI launchers" investigation rests on (owner,
  2026-10-13): **at least one AI-related feature is forbidden by Raycast** — refused as a third-party extension, by
  policy or by design. One proven case is enough for "yes".

## Verdict: holds (refined)

Raycast does not forbid AI extensions in general, but it refuses, case by case, AI extensions that duplicate its own
AI features, some of which are paid.

| Evidence                                                                                                         | What it shows                                                                                                                                                                                                                             |
| ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [raycast/extensions#25592](https://github.com/raycast/extensions/pull/25592) `local-ai`, closed unmerged         | Local models through Ollama refused: "This is already possible directly within Raycast as default… Raycast has native local model support via Ollama in the AI features."                                                                 |
| [raycast/extensions#24097](https://github.com/raycast/extensions/pull/24097) `rewrite-anywhere`, closed unmerged | AI rewriting of selected text refused: "there is nothing that's not possible out of the box in Raycast already."                                                                                                                          |
| [raycast/extensions#30802](https://github.com/raycast/extensions/pull/30802) `ai-commands-byok`, open            | Built because "Raycast's built-in AI Commands … moved behind Pro"; the reviewer steers it towards existing extensions rather than accepting a new listing.                                                                                |
| [Extensions Guidelines](https://manual.raycast.com/extensions-guidelines)                                        | The rule used: rejected when "there is already a feature to accomplish this directly in Raycast, which offers comparable value"; extensions "should do more than Raycast's native features"; the word "Assistant" is restricted in names. |

## What limits the verdict

- **Not absolute:** [Ollama AI and LM Studio](https://raycast.com//changelog/macos/2-5) extensions are in the Store,
  and Raycast 2.5 lets extensions supply models to AI Chat. The same kind of feature can be accepted or refused
  depending on the review.
- **No AI-specific rule:** the refusals use the generic "native feature" rule.
- **Agent-session tools are accepted:** ClaudeCast, Claude Code Launcher, PromptCast and Agent Sessions are in the
  Store. They attach to an external terminal: the extension API (lists, details, forms, grids) cannot draw one inside
  Raycast.
- Searched: closed unmerged PRs in `raycast/extensions` with AI terms and staff comments (2024-08 → 2026-10); a
  systematic count of refusals is phase 3's job, not this check's.

## Consequence for the investigation

The assumption survives in a refined form: the gap is **AI features Raycast keeps native (some behind Pro) and refuses
from third parties**, plus what its extension API cannot draw (an embedded terminal). The planned investigation
(taxonomy → audit → Raycast coverage → GPUI census → port scoring → synthesis) stays justified; its Raycast column
gains a status "refused as a duplicate of a native feature", with the PR as evidence.
