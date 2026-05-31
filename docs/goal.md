You are working in the Launcherg-Mod repository.

Goal:
Improve the Launcherg-Mod UI by using the Deep Research report and the DeepWiki comparison between Launcherg-Mod and vnite, but implement ONLY a safe UI-only first phase.

This task is strictly UI-only.
Existing database data may be read, but the database schema and stored values must not be changed.

Primary objective:
Make Launcherg-Mod’s game/detail experience and surrounding library UI feel more polished and closer to a “game collection / work page” experience, inspired by vnite, while preserving Launcherg-Mod’s existing Tauri/Rust/Svelte architecture and ErogameScape-centered identity.

Important references:
- Deep Research report placed in this repository.
- DeepWiki for Launcherg-Mod:
  https://deepwiki.com/nnnSMM/Launcherg-Mod
- DeepWiki for vnite:
  https://deepwiki.com/ximu3/vnite

Read the Deep Research report first.
Search inside it for:
- Launcherg-Mod と vnite の DeepWiki 比較設計報告
- 詳細ページ完全仕様
- 詳細ページ外 UI の磨き込み
- 詳細ページ全体
- HeroHeader
- Tabs
- GridPoster
- BigPoster
- GamePoster
- FilterAdder
- BackgroundPresenter
- WorkLayout
- Work.svelte
- Home dashboard
- Sidebar
- ZappingGameItem
- GlassInfo
- Info
- Detail
- Actions

Absolute safety constraints:
- Do not add any database migration.
- Do not create, delete, rename, or modify any database table.
- Do not modify SQL schema files.
- Do not modify existing migration files.
- Do not add new migration files.
- Do not change existing stored data.
- Do not change play_status values.
- Do not change play time calculation.
- Do not change total_play_time_seconds.
- Do not change daily play time behavior.
- Do not modify memo storage.
- Do not modify screenshot storage.
- Do not modify image storage paths.
- Do not modify thumbnail/background persistence behavior.
- Do not modify existing Rust database repository logic unless it is strictly necessary for type compatibility, and even then it must remain read-only.
- Do not add new backend commands.
- Do not add network fetching.
- Do not implement background image acquisition.
- Do not implement image crop, WebP generation, media asset tables, or image variant generation.
- Do not implement play-session reconciliation.
- Do not implement extended play-status.
- Do not implement save management.
- Do not implement cloud sync.
- Do not introduce a plugin system.
- Do not rewrite the whole app.
- Do not replace the existing architecture.

Allowed scope:
- Svelte/UI component refactoring.
- CSS styling improvements.
- Layout restructuring.
- Component extraction.
- Reorganizing existing displayed data into a more polished layout.
- Adding a tab shell that uses existing data only.
- Adding visual-only placeholder tabs if needed, as long as they do not require new DB fields or backend commands.
- Improving card/poster/list presentation using existing data only.
- Improving hover overlays using existing data only.
- Improving empty/fallback states using existing data only.
- Improving accessibility of tabs/buttons/cards.
- Adding TODO comments or docs for future DB-backed features, but not implementing them.

Current app strengths that must be preserved:
- Existing Tauri + Rust backend.
- Existing ErogameScape metadata integration.
- Existing game launch behavior.
- Existing play tracking behavior.
- Existing play/memo/like/delete/shortcut/external-link actions.
- Existing Work/detail page background/canvas presentation.
- Existing screenshot/memo functionality.
- Existing Home dashboard.
- Existing sidebar/search behavior.
- Existing virtualized or masonry grid behavior.
- Existing data loading behavior.

Target UX direction:
Use vnite as inspiration, but do not copy it blindly.

The desired direction is:
- The detail page should feel like a polished “work page” or “game collection page”, not just a metadata/settings page.
- The existing WorkLayout/background presentation should remain, but the information structure should be more organized.
- The detail page should have a clear Hero/Header area.
- Existing metadata and actions should be reorganized into cards or panes.
- Existing detail information should be grouped into an Overview-like area.
- Existing play time and last played information should be grouped into a Record-like area.
- Existing memo entry point should be easier to discover.
- Existing actions should remain available and visually cleaner.
- Library cards should be more poster-like and polished.
- Recently played and main grid surfaces should feel more intentional.
- Sidebar/list/card surfaces should better distinguish primary information, secondary metadata, and quick actions.

Implementation targets:

1. Repository audit
First inspect the repository and identify the current files/components responsible for:
- Work/detail page
- WorkLayout
- Hero
- GlassInfo
- Info
- Detail
- Actions
- Memo entry/action
- Home dashboard
- Recently played
- Game/card grid
- ZappingGameItem or equivalent poster/card component
- Sidebar/search
- Existing CSS/theme structure

Report the actual files you found before making changes.

2. Detail page UI-only refactor
Implement a safer, more structured detail page using existing data only.

Desired component direction:
- DetailPageShell or equivalent wrapper
- HeroHeader or improved existing Hero section
- TabBar or section switcher
- Overview section/tab
- Record section/tab
- Memo/Screenshot entry section/tab if feasible using existing behavior only
- Launch/Actions section if feasible using existing behavior only

Rules:
- If a full tab system is too risky, create visually separated sections instead.
- If tabs are implemented, preserve the same data and actions.
- Tabs must not require new routes or new DB fields.
- The default tab/section should show all information currently visible or make it reachable without loss.
- Existing Play, administrator launch toggle, Memo, Like, Delete, Shortcut setting, and external links must remain functional.
- Do not remove any existing user-visible information unless it is duplicated elsewhere.
- Do not change data semantics.

Suggested layout:
- Top Hero/Header:
  - cover/thumbnail using existing image source
  - title
  - brand/release date if already available
  - play status using existing values only
  - total play time using existing value only
  - last played if already available
  - primary Play button
  - secondary actions grouped cleanly

- Overview:
  - description or detail fields already available
  - brand
  - release date
  - creators
  - voice actors
  - ErogameScape/VNDB/other links already available
  - rank/median/score fields already available

- Record:
  - existing total play time
  - existing first played
  - existing last played
  - existing play status
  - any existing daily play time display if already available
  - no new reconciliation logic

- Memo/Screenshot:
  - link or embedded entry point to existing memo functionality
  - preserve existing memo behavior
  - preserve existing screenshot insertion/paste behavior if already present
  - do not change memo storage

- Launch/Actions:
  - existing launch and shortcut actions
  - existing admin toggle
  - existing delete/favorite actions
  - no new launch profile system

3. Detail page visual improvements
Improve visual polish while preserving behavior:
- clearer visual hierarchy
- better spacing
- card-like grouping
- cleaner action grouping
- stronger hero/header composition
- graceful fallback when images are missing
- readable overlay over existing background/canvas
- responsive behavior for narrow/wide layouts
- avoid layout breakage with long Japanese titles
- avoid layout breakage with missing metadata
- avoid layout breakage with very small or unusual images

Image handling rules:
- Use existing image paths only.
- Do not download images.
- Do not add background image search.
- Do not generate new image files.
- Do not crop or persist image variants.
- You may use CSS-only techniques:
  - object-fit
  - object-position
  - aspect-ratio
  - blurred visual fallback using existing source
  - gradient overlay
  - dark overlay
  - backdrop-filter if already compatible
- Do not change stored image data.

4. UI outside detail page
Improve surrounding UI using existing data only:
- library grid cards
- recently played row
- poster/card hover overlay
- sidebar/list item readability
- filter chip readability if present

Use vnite-inspired concepts but keep implementation safe:
- GridPoster-like presentation for normal cards
- BigPoster-like presentation for recently played/featured items if easy
- NavItem-like compact rows if applicable
- Do not rewrite the entire Home dashboard.
- Do not break virtualization/masonry.
- Do not change search/filter semantics.
- Do not persist new view settings.

Card/poster suggestions:
- normal state:
  - cover/thumbnail
  - title
  - small metadata if already available
- hover/focus state:
  - Play action if already available
  - total play time if already available
  - last played if already available
  - status badge using existing values only
- keyboard focus should show equivalent action affordance where feasible.

5. Accessibility and UX quality
Include:
- keyboard accessible tab buttons if tabs are added
- aria labels for icon-only buttons
- visible focus states
- reduced-motion consideration for animations
- no hover-only critical actions without keyboard access
- text overflow handling for Japanese titles
- graceful empty states

6. Documentation
Add or update a small markdown note, for example:
docs/ui-only-vnite-inspired-phase1.md

The note should include:
- what was changed
- which existing files/components were touched
- what remains intentionally out of scope
- future phases that require DB changes but were not implemented:
  - extended play status
  - play session ledger
  - playtime reconciliation
  - media asset tables
  - background image picker
  - crop/WebP/variant generation
  - save management

This document must clearly state:
“This phase is UI-only. It does not add migrations or change existing stored data.”

7. Testing and verification
Run the project’s available checks if possible.

At minimum, try to run:
- npm install or bun install only if dependencies are missing and the project expects it
- npm run check / npm run build / npm run tauri build / cargo check, depending on what scripts exist
- If exact commands are unclear, inspect package.json and Cargo.toml first.

Report:
- commands run
- success/failure
- any errors
- files changed
- any skipped checks and why

Acceptance criteria:
- No migration file is added.
- No SQL schema file is modified.
- No existing migration is modified.
- Existing games still load.
- Existing detail page route still works.
- Existing Play action still works.
- Existing memo action still works.
- Existing favorite/like action still works.
- Existing delete action still works.
- Existing shortcut/external link actions still work.
- Existing play time values are displayed unchanged.
- Existing play status values are displayed unchanged.
- Existing thumbnails/backgrounds still work.
- Missing image fallback works.
- Missing metadata fallback works.
- Long Japanese titles do not break the layout.
- Card/grid layout remains usable.
- Sidebar/search behavior remains unchanged.
- Build/typecheck passes, or failures are clearly reported.

Non-goals:
- Do not implement timer unrecorded play time reconciliation in this PR.
- Do not implement play status extension in this PR.
- Do not implement background image acquisition in this PR.
- Do not implement background image persistence changes in this PR.
- Do not implement image crop or WebP generation in this PR.
- Do not implement media asset database tables in this PR.
- Do not implement save management in this PR.
- Do not implement launch presets in this PR.
- Do not implement cloud sync in this PR.

Output expected:
1. First, summarize the repository audit:
   - actual files found
   - current component responsibilities
   - safest insertion points

2. Then implement the UI-only changes.

3. Finally, provide:
   - summary of changes
   - files changed
   - checks run
   - acceptance criteria status
   - remaining risks
   - recommended next PR, still UI-only if possible

Be conservative.
Prefer small, reversible changes.
Preserve behavior over visual ambition.
If there is any risk of changing stored data or backend behavior, do not do it.