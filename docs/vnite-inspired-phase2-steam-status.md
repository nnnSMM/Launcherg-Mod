# vnite-inspired phase 2: Steam thumbnail, status UI, and titlebar cleanup

This phase keeps the existing Launcherg-Mod storage model and only adds safe,
incremental behavior around the current UI and import flow.

## Changed

- Detect Steam-installed games during add/import by inspecting paths under
  `steamapps/common/<install dir>` and matching the sibling `appmanifest_*.acf`
  files by `installdir`.
- Detect Steam `.url` shortcuts by reading `steam://rungameid/<appid>` or
  `steam://run/<appid>`.
- When a Steam AppID is found, try Steam CDN thumbnail candidates before the
  existing ErogameScape thumbnail URL:
  - `library_600x900_2x.jpg`
  - `library_600x900.jpg`
  - `capsule_616x353.jpg`
  - `header.jpg`
- Extend the existing play status UI to five values while still using the
  existing `collection_elements.play_status` INTEGER column:
  - `0`: unplayed
  - `1`: playing
  - `2`: cleared
  - `3`: multiple
  - `4`: shelved
- Update detail, home, card, sidebar filter, and bulk edit surfaces so the new
  status labels and colors are visible consistently.
- Reorganize the current titlebar actions in a vnite-like shape: back/forward
  and home controls on the left, add as a compact action, visible
  shortcut/display/play-status/help actions on the right, and existing window
  controls fixed at the far right.
- Polish the surrounding library frame with a clearer sidebar header, visible
  shown/registered counts, grouped search filters, a compact collapsed sidebar,
  and an empty state for filtered-out results.
- Refine the game detail page toward vnite's practical detail-header pattern:
  keep the existing Hero/background presentation, move status/play metadata
  into a light record row below it, keep persistent actions in a separate
  compact action panel, and use tabbed detail pages for overview, record, memo,
  and screenshots.
- Show a description area in the overview tab. When the already-fetched
  ErogameScape HTML exposes a description/meta description it is parsed into the
  optional UI model field. Missing descriptions are not replaced with fabricated
  summary text.

## Main touched areas

- `src-tauri/src/domain/file.rs`
- `src-tauri/src/usecase/collection.rs`
- `src-tauri/src/interface/command.rs`
- `src/lib/types.ts`
- `src/lib/playStatus.ts`
- `src/components/TitleBar.svelte`
- `src/components/Sidebar/SubHeader.svelte`
- `src/components/Sidebar/Search.svelte`
- `src/components/Sidebar/SearchInput.svelte`
- `src/components/Sidebar/SearchAttribute.svelte`
- `src/components/Sidebar/CollectionElements.svelte`
- `src/components/Sidebar/MinimalSidebar.svelte`
- `src/components/Sidebar/searchAttributes.ts`
- `src/components/Work/*`
- `src/views/PlayStatusBulkEditor.svelte`

## Storage and data scope

This phase does not add database migrations, does not modify existing migration
files, and does not modify SQL schema files.

It does not add media asset tables, image downloading tables, crop persistence,
WebP generation, playtime reconciliation, play session ledgers, status history,
save management, launch presets, cloud sync, or plugin systems.

The Steam thumbnail handling only changes which source URL candidates are tried
when saving the already-existing thumbnail file for a newly added game. Existing
memo storage, screenshot storage, launch behavior, delete behavior, favorite
behavior, and play time calculation are intentionally left unchanged.

The titlebar/detail-page refinement is still UI-only. It changes Svelte layout,
labels, styling, and renderer metadata presentation only; it does not change
DB/schema/migration files or stored database data.

The overview description and action-panel refinement do not add migrations,
tables, backend commands, or SQL schema changes. Launch, memo, favorite, delete,
shortcut, screenshot, and play-time behavior are left as-is.

Future work to fill descriptions from VNDB, DLsite, Steam, or another metadata
source should be handled as a separate phase because it would add new metadata
fetching behavior.

Image/background experiments that changed poster or Home image rendering were
rolled back after visual regressions. The current detail background path keeps
the existing WorkLayout canvas/Hero image behavior while the metadata and action
layout work continues. Record and action surfaces now follow vnite's compact
icon/text pattern more closely without changing thumbnail/background storage.
