## Why

`tread` currently requires a file, EPUB, or URL argument, so users must remember and re-enter targets instead of resuming from a reading home screen. A dashboard makes `tread` feel like a reader entry point: continue favorite or recent reading, or open a new local path/URL from inside the TUI.

## What Changes

- **BREAKING**: Running `tread` with no arguments will open the dashboard TUI instead of printing usage and exiting with code 1.
- Add a dashboard that lists starred reading entries separately from recent reading entries.
- Add a unified reading history index at `~/.tread/history.json` for web, EPUB, and Markdown targets.
- Add starred and hidden entry states:
  - `s` toggles starred/unstarred for the selected entry.
  - `r` removes the selected entry from the dashboard by setting `hidden = true` and `starred = false`.
  - Hidden entries do not appear in either dashboard section.
  - Reopening a hidden target through the CLI or dashboard open prompt restores it to recent reading with `hidden = false` and `starred = false`.
- Add Markdown progress persistence so dashboard entries can resume Markdown files like web pages and EPUB books.
- Add an `o` open prompt in the dashboard for entering URLs or local paths.
- Add current-directory-only local path suggestions in the open prompt, including `~` expansion, relative paths, absolute paths, directories, and local reading files.
- Preserve existing target-based CLI behavior for `tread <file.md|file.epub|url>` and existing web/EPUB progress behavior.
- Preserve Markdown explicit line override behavior (`tread notes.md 50`) and make it take precedence over saved history for that launch.

## Capabilities

### New Capabilities
- `reading-dashboard`: Dashboard entry screen, starred/recent sections, open prompt, path suggestions, and dashboard item actions.
- `reading-history`: Unified persisted history model for web, EPUB, and Markdown reading targets.

### Modified Capabilities
- `tui-navigation`: No-argument CLI behavior changes from usage error to dashboard TUI; dashboard/open-prompt keyboard behavior becomes part of the terminal UI contract.
- `web-reader`: Web reading updates the unified history index while preserving existing web cache/progress behavior.
- `epub-reader`: EPUB reading updates the unified history index with title, path, chapter, and progress metadata while preserving existing EPUB progress behavior.

## Impact

- Affected code:
  - `src/main.rs`: CLI intent parsing, dashboard entry dispatch, reader resume/open flow, Markdown progress handling.
  - `src/app.rs`: likely needs either dashboard state integration or a sibling dashboard app state for list/prompt interactions.
  - `src/ui.rs`: dashboard list, starred/recent sections, open prompt, suggestions, and status/error rendering.
  - `src/web.rs` and `src/epub.rs`: history updates around open/exit metadata and positions.
  - New history/dashboard module(s) are likely needed to keep storage and UI state separate from reader state.
- User-facing docs: README usage and key list must reflect bare `tread`, dashboard keys, and the open prompt.
- Data files: introduces `~/.tread/history.json`; existing progress/cache files remain compatible.
- No new external dependency is required for v1.
