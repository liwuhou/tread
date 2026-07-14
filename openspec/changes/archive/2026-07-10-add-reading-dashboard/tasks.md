## 1. History Store and Target Identity

- [x] 1.1 Add a history module with `HistoryEntry`, target kind, position, starred, hidden, and timestamp data structures
- [x] 1.2 Implement `~/.tread/history.json` load/save with missing-file-as-empty, malformed-file backup, temporary-file write, and atomic replacement behavior
- [x] 1.3 Implement stable IDs for web URL, EPUB path, and Markdown path targets using the specified web URL identity and local canonical/resolved absolute path rules
- [x] 1.4 Implement history upsert-on-open semantics, including hidden target reopen behavior (`hidden = false`, `starred = false`)
- [x] 1.5 Implement position/progress updates with percent clamping, updated-at ordering data, and preservation of newer unrelated `starred`/`hidden` state
- [x] 1.6 Add focused unit tests for history serialization, missing/malformed files, atomic write behavior, star/hide state transitions, and ID stability
- [x] 1.7 Add injectable boundaries for filesystem, clock, home/current directory resolution, and history store operations

## 2. Command Intent and Reader Integration

- [x] 2.1 Refactor CLI parsing into a command intent that distinguishes dashboard entry from direct target open
- [x] 2.2 Preserve direct URL, EPUB, Markdown, `--refresh`, and `--interactive` CLI behavior for target opens
- [x] 2.3 Change no-argument `tread` behavior to dispatch to dashboard instead of usage error
- [x] 2.4 Integrate history upsert and progress update calls into web reader open/exit flow while preserving existing web progress files as the web restore source of truth
- [x] 2.5 Integrate history upsert and progress update calls into EPUB reader open/exit flow while preserving existing EPUB progress files as the EPUB restore source of truth
- [x] 2.6 Add Markdown progress restore/save through history, with explicit `tread notes.md 50` line argument taking precedence
- [x] 2.7 Add tests for command intent parsing, reader-specific progress precedence, and Markdown history-vs-explicit-line precedence

## 3. Dashboard State and Rendering

- [x] 3.1 Add dashboard app state separate from the reader `App`, including list mode, prompt mode, selected row, prompt input, suggestions, and status/error message
- [x] 3.2 Render dashboard empty state with `o` and direct CLI usage hints
- [x] 3.3 Render starred and recent sections from visible history entries without duplicates
- [x] 3.4 Sort visible unstarred recent entries by descending `updated_at`
- [x] 3.5 Render row metadata for kind, title/target, progress percent, and star marker
- [x] 3.6 Add dashboard terminal setup/teardown using the same raw mode and alternate screen guarantees as existing readers

## 4. Dashboard Interaction

- [x] 4.1 Implement `j`/`k` and arrow-key selection over visible dashboard entries
- [x] 4.2 Implement `Enter` on a selected entry to open the corresponding reader at the saved position
- [x] 4.3 Implement `s` to toggle starred state for selected entries
- [x] 4.4 Implement `r` to remove selected entries from dashboard by setting `hidden = true` and `starred = false` without deleting cache/progress files
- [x] 4.5 Implement `q`, `Esc`, and `Ctrl+C` dashboard exit behavior with exit code 0 and terminal cleanup
- [x] 4.6 Add interaction tests for section movement, star/unstar, remove semantics, and hidden starred entries not reappearing in starred section

## 5. Open Prompt and Path Suggestions

- [x] 5.1 Implement `o` to enter dashboard open-prompt mode and `Esc` to return to dashboard list mode
- [x] 5.2 Implement URL submission for `http://` and `https://` inputs
- [x] 5.3 Implement local path submission with `~`, relative path, and absolute path resolution
- [x] 5.4 Implement missing-path inline errors that keep the prompt open
- [x] 5.5 Implement current-directory-only suggestions for local path prefixes, with no recursive search, directory-first sorting, lexicographic ordering, and a maximum of 50 displayed suggestions
- [x] 5.6 Implement directory suggestions with trailing `/` and directory-enter/complete behavior
- [x] 5.7 Implement suggestion selection with arrow keys and completion with `Tab`
- [x] 5.8 Add recoverable prompt errors for missing paths, permission errors, unreadable directories, unreadable files, and other local path I/O failures
- [x] 5.9 Add tests for URL detection, path resolution, one-level suggestions, suggestion ordering/limit, directory completion, I/O error handling, and file submission

## 6. Documentation and Verification

- [x] 6.1 Update README usage to document bare `tread`, dashboard keys, open prompt, starred entries, and remove semantics
- [x] 6.2 Update terminal help text or dashboard footer hints to include dashboard/open-prompt keys where applicable
- [x] 6.3 Run formatting checks for the Rust code touched by the implementation
- [x] 6.4 Run targeted unit tests for history, command intent parsing, dashboard state, path suggestions, and injected side-effect boundaries
- [x] 6.5 Run an interactive smoke scenario: `tread` opens dashboard, `o` opens a local Markdown file, exit saves history, reopening `tread` shows the entry
- [x] 6.6 Run OpenSpec validation for `add-reading-dashboard`
