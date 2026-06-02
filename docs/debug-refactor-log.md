# Debug / Refactor Log

## 2026-06-02: stability sweep

- Branch: `codex/debug-refactor-stability`
- Baseline verification before edits:
  - `npm run check`: pass
  - `npm run test:run`: pass, but jsdom/SimpleBar cleanup noise appeared on stderr
  - `cargo check --manifest-path src-tauri\Cargo.toml`: pass
  - `npm run build`: pass, with existing large chunk warning
  - `npx eslint --ext .js,.ts,.svelte,.cjs src`: failed because ESLint had no Svelte/TypeScript parser config

### Frontend stability

- `src/lib/utils.ts`
  - Made `localStorageWritable` resilient to invalid JSON and storage read/write failures.
  - Corrupt saved values now reset to the provided initial value instead of crashing module initialization.
- `src/lib/utils.test.ts`
  - Added regression coverage for corrupt localStorage JSON recovery.
- `src/lib/filter.ts`
  - Reworked `useFilter` so query/options subscriptions are tied to the returned readable store lifecycle.
  - Prevented duplicate query subscriptions each time options change.
- `src/lib/filter.test.ts`
  - Added regression coverage for subscription count and filtering after options updates.
- `src/lib/trieFilter.ts`
  - Applied the same lifecycle-safe subscription pattern to trie-based filtering.
- `src/components/UI/virtualScroller.ts`
  - Removed an always-live internal subscription by retrying deferred scroll restoration when virtual height is set.
- `src/components/UI/virtualScrollerMasonry.ts`
  - Removed the internal `layouts.subscribe` side effect and updates virtual height from the layout calculation itself.
- `src/components/PlayStatusBulkEditor/usePlayStatusVirtualScrollerMasonry.ts`
  - Applied the same virtual height side-effect cleanup to the play-status masonry scroller.
- `src/components/Sidebar/CollectionElements.svelte`
  - Added `SimpleBar.unMount()` cleanup to prevent observer leaks.
- `src/components/UI/ScrollableHorizontal.svelte`
  - Added explicit window wheel listener cleanup, `SimpleBar.unMount()`, and wheel default prevention while horizontally scrolling.
- `src/components/Sidebar/SearchComponent.test.ts`
  - Destroys manually mounted Svelte components so tests do not leave SimpleBar observers behind.
  - Stubs pseudo-element `getComputedStyle` access to keep jsdom output quiet.
- `src/lib/markdown.ts`
  - Escapes generated image attributes and sanitizes rendered markdown HTML before it is passed to Svelte `{@html}`.
  - Removes dangerous tags, inline event handlers, `style`, `srcset`, and unsafe `href`/`src` protocols.
- `src/lib/markdown.test.ts`
  - Added regression coverage for HTML/script/event-handler sanitization.
- `src/components/Work/Info.svelte`
  - Marks the sanitized markdown `{@html}` usage explicitly.
- `src/store/skyway.ts`
  - Removed an unnecessary async Promise executor when sending image chunks.
- `src/components/UI/ButtonBase.svelte`, `src/components/Work/Detail.svelte`
  - Scoped `switch` default branches so ESLint can validate them cleanly.

### Rust/Tauri stability

- `src-tauri/src/main.rs`
  - Replaced panic-prone `unwrap()` calls in single-instance emission, window hiding, and tray icon setup with logging/fallback behavior.
- `src-tauri/src/infrastructure/util.rs`
  - Replaced directory creation/canonicalization `unwrap()` calls with logged fallbacks.
- `src-tauri/src/infrastructure/repositoryimpl/driver.rs`
  - Switched SQLite connection setup from URI string parsing to `SqliteConnectOptions::filename(...).create_if_missing(true)`, avoiding path escaping edge cases.
- `src-tauri/src/infrastructure/explorerimpl/file.rs`
  - Propagates directory creation failures with `?` instead of panicking.
- `src-tauri/src/usecase/pause_manager.rs`
  - Recovers poisoned mutexes with `into_inner()` instead of panicking.
- `src-tauri/src/usecase/screenshot_watcher.rs`
  - Converts watcher mutex poisoning into a returned error/log instead of panicking.
- `src-tauri/src/domain/repository/collection.rs`, `src-tauri/src/interface/command.rs`
  - Normalized by `cargo fmt`.

### Tooling

- `.eslintrc.cjs`
  - Added ESLint parser configuration for TypeScript and Svelte using existing dependencies.
- `package.json`
  - Changed `lint` to check-only.
  - Added `lint:fix` for the previous auto-fix behavior.

### Final verification

- `npm run check`: pass
- `npm run lint`: pass
- `npm run test:run`: pass, 35 files / 231 tests
- `cargo check --manifest-path src-tauri\Cargo.toml`: pass
- `cargo test --manifest-path src-tauri\Cargo.toml`: pass, 83 tests
- `npm run build`: pass, existing large chunk warning remains
- `npm run graph:check`: pass, 19 markdown files checked

## 2026-06-02: expanded stability / performance sweep

- Branch: `codex/debug-refactor-stability`
- Scope: second large pass after the first stability sweep, focused on cross-cutting crash prevention, stale subscription cleanup, initial bundle size, demo transfer size, and Rust static-analysis debt.

### 2026-06-02: removed unused screenshot-candidate prefetch

- Removed the screenshot-candidate prefetch entry points from manual import, drag-and-drop import, automatic import, and work re-assignment.
- Deleted the unused `useGameScreenshots` candidate-fetch/parsing module, its tests, and the unmounted `GameHoverPreview` component.
- Kept the existing `game_screenshot_caches` database migrations and backend Tauri commands as compatibility shims so existing user databases and older callers are not broken.
- Result: new imports no longer trigger external FANZA/DLsite/Steam screenshot-candidate lookups, and the unused candidate chunk is no longer pulled into the frontend build.

### 2026-06-02: fullscreen screenshot filmstrip keeps position

- Changed fullscreen screenshot filmstrip selection so clicking a thumbnail no longer aligns it to the left edge.
- The filmstrip now measures the selected thumbnail against the visible viewport and only scrolls by the clipped amount when the selected thumbnail is partially or fully out of view.
- Kept the initial/reveal behavior compatible by ensuring the current thumbnail is visible without forcing left alignment.

### 2026-06-03: public landing page mentions stats

- Added a play-trend/statistics feature card to the public landing page.
- Added the statistics screen to the main screen showcase and published `public/images/stats.png` for that section.
- Kept the existing download and demo links unchanged.

### 2026-06-03: release title cleanup

- Kept existing release tags and assets intact so historical download counts and URLs are preserved.
- Updated the release workflow to name future releases from the product version, such as `0.3.3`, instead of leaving the title as the raw `YYYYMMDD` tag.

### Additional frontend stability

- `src/lib/registerCollectionElementDetails.ts`
  - Added in-flight request deduplication so repeated initializers do not run duplicate detail-registration commands concurrently.
  - Added a test-only reset hook for deterministic regression coverage.
- `src/lib/registerCollectionElementDetails.test.ts`
  - Replaced broad integration-style assertions with command-mocked tests for row mapping, flag/id conversion, empty-id short-circuiting, concurrent dedupe, and retry-after-failure behavior.
- `src/components/Sidebar/searchAttributes.ts`
  - Added storage normalization through safe localStorage helpers.
  - Added `matchesAttribute` as a shared predicate so attribute filtering is tested once and reused by bulk filtering.
- `src/components/Sidebar/searchAttributes.test.ts`
  - Added malformed JSON and missing-key recovery tests.
- `src/components/Sidebar/Sidebar.svelte`
  - Replaced manual store subscription with a derived store and `get`, removing a long-lived component subscription.
- `src/store/skywayMessage.ts`, `src/store/skywayMessage.test.ts`
  - Extracted remote-message parsing into pure, tested functions.
- `src/store/skyway.ts`
  - Ignores malformed SkyWay data messages instead of throwing from direct `JSON.parse`.
- `src/App.svelte`
  - Fixed `onMount(async () => ...)` so the cleanup function is registered synchronously by Svelte.
  - Kept main-window initialization out of landing/special windows and moved optional UI/module loading behind dynamic imports.
- `src/components/UI/ButtonBase.svelte`
  - Made tooltip creation/destruction track changing tooltip props.
  - Moved Tippy.js loading to the first actual tooltip use.

### Rust/Tauri stability and static analysis

- `src-tauri/src/infrastructure/repositoryimpl/driver.rs`
  - `Db::new` now returns `anyhow::Result<Db>` and propagates SQLite connection/migration failures with context instead of panicking.
- `src-tauri/src/interface/module.rs`
  - `Modules::new` now returns `anyhow::Result<Self>` so DB setup failure reaches app setup cleanly.
- `src-tauri/src/main.rs`
  - Propagates module setup failure from the Tauri setup closure.
  - Removed needless clones/borrows around shortcuts and app handles after clippy audit.
- `src-tauri/src/**/*.rs`
  - Ran `cargo clippy --fix --allow-dirty --all-targets` and manually resolved the remaining warnings.
  - `cargo clippy --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings` now passes.

### Performance / data reduction

- `src/router/route.ts`
  - Converted static route component imports to `wrap({ asyncComponent })` route-level code splitting.
- `src/lib/routeHelper.ts`
  - Removed top-level `@tauri-apps/api/core` import from route predicates.
  - `getWorkDetailBgImage` now imports `convertFileSrc` only when image URL conversion is actually requested.
  - This stopped demo route checks from pulling the full mock Tauri core and demo catalog into the first JS payload.
- `src/App.svelte`
  - Lazy-loads overlay, screenshot window, tray menu, landing page, title bar, layout, import drop zone, updater store, scrape cache initializer, and work store registration.
- `src/components/Work/QRCode.svelte`, `src/views/Memo.svelte`
  - Lazy-load SkyWay only when the QR/link or memo sync path needs it.
- `src/components/Work/Info.svelte`
  - Lazy-loads markdown parsing only for non-empty memo rendering and ignores stale parse results.
- `src/store/theme.ts`
  - Lazy-loads command bindings for theme persistence.
- `src/views/ScreenshotWindow.svelte`
  - Pre-groups screenshots by game id so per-game filtering does not repeatedly scan all screenshots.
- `src/views/PlayStatusBulkEditor.svelte`
  - Reuses `matchesAttribute` and performs attribute filtering in one pass.
  - Avoids unnecessary Set/filter work when the text filter already includes all games.
- `src/main.ts`, `src/views/Memo.svelte`
  - Moved EasyMDE CSS from the global entry to the memo view chunk.
- `uno.config.ts`
  - Removed generated Google WebFont CSS and switched `font-sans` / `font-logo` to local/system fallback stacks.
  - Demo initial CSS dropped from about 492 KB / 123 KB gzip to about 166 KB / 28 KB gzip.

### Expanded verification

- `npm run check`: pass
- `npm run lint`: pass
- `npm run test:run`: pass, 36 files / 242 tests
- `npm run build`: pass; remaining warning is the deferred EasyMDE memo editor chunk
- `npm run build-demo`: pass; first JS entry is about 55 KB and first CSS is about 166 KB
- `cargo check --manifest-path src-tauri\Cargo.toml`: pass
- `cargo test --manifest-path src-tauri\Cargo.toml`: pass, 83 tests
- `cargo clippy --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings`: pass
