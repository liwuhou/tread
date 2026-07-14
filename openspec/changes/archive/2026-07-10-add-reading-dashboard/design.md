## Context

`tread` currently dispatches directly from CLI arguments to one reader mode:

```text
tread <target>
  ├─ URL      -> run_web()
  ├─ *.epub   -> run_epub()
  └─ other    -> run_markdown()
```

Running `tread` without a target is currently specified as a usage error. Web and EPUB readers persist progress independently, but Markdown has only a positional start-line argument and no saved progress. This change introduces a dashboard as the no-argument entry point, plus a unified history index used by that dashboard.

There is no Figma design file for this terminal UI change. The dashboard should reuse the existing ratatui/crossterm visual language: bordered overlays, simple list rows, status/footer hints, and keyboard-first interactions.

## Goals / Non-Goals

**Goals:**
- Make bare `tread` open a dashboard/home screen.
- Let users resume starred and recent reading entries for web, EPUB, and Markdown.
- Let users star/unstar entries and remove entries from the dashboard without deleting reader data.
- Let users press `o` to open a URL or local path from the dashboard.
- Provide current-directory-only local path suggestions in the open prompt.
- Persist Markdown reading progress.
- Keep existing target-based CLI flows working.

**Non-Goals:**
- Recursive fuzzy search across the filesystem.
- A full file manager.
- Deleting cache/progress files from the dashboard.
- Syncing history across machines.
- Replacing the existing web and EPUB progress files in this change.
- Adding SQLite or another external storage dependency.

## Decisions

### Parse CLI into command intent before opening readers

Introduce an internal command-intent layer instead of letting `main` directly branch into reader functions.

```text
CommandIntent
  ├─ Dashboard
  └─ Open {
       target,
       options,
       initial_position_override
     }
```

Dashboard selection and dashboard open-prompt submission should construct the same `Open` intent used by CLI target arguments.

Alternatives considered:
- Make dashboard synthesize CLI argument vectors and re-enter existing parsing. This is brittle because dashboard state already has structured target identity and position data.
- Keep direct `main` branching and bolt dashboard onto the side. This would duplicate target resolution and resume logic.

Rationale: one structured dispatch path keeps CLI opens, dashboard resumes, and dashboard prompt opens consistent.

### Use `~/.tread/history.json` as dashboard source of truth

Add a unified JSON index for dashboard entries. The dashboard reads this file for both starred and recent sections. Existing web/EPUB progress files continue to exist and continue to support reader-specific resume behavior.

Suggested entry shape:

```json
{
  "id": "web:<hash>",
  "kind": "web",
  "title": "Article title",
  "target": "https://example.com/article",
  "position": { "scroll": 123 },
  "progress_percent": 42,
  "updated_at": "2026-07-09T00:00:00Z",
  "starred": false,
  "hidden": false
}
```

EPUB entries include chapter metadata; Markdown entries include scroll metadata. Percent is a display hint computed at save time and should be clamped to 0-100.

Identity rules should be deterministic:
- Web entries use the same URL string identity that existing web cache/progress keys use.
- Local file entries use canonical absolute paths when canonicalization succeeds.
- Local file entries fall back to resolved absolute paths when canonicalization fails.
- Display targets remain separate from identity source paths so the dashboard can show user-friendly paths.

History writes should use a temporary file in the same directory followed by atomic replacement. A malformed `history.json` should be moved aside to a timestamped backup before the app writes a replacement history file.

Alternatives considered:
- Scan existing progress/cache files to build dashboard rows. This loses EPUB path/title metadata and misses Markdown entirely.
- Use SQLite. This is unnecessary for a single-user local CLI with small history volume.

Rationale: JSON matches the existing storage style and is enough for deterministic dashboard rendering.

For web and EPUB, existing reader-specific progress files remain the source of truth for the actual restored reading position. Unified history is the dashboard display index and is refreshed from reader state after exit. For Markdown, unified history is the source of truth because no legacy Markdown progress file exists.

### Keep hidden and starred as separate states, with hidden taking precedence

Dashboard display rules:

```text
if hidden:
    show nowhere
else if starred:
    show in ★ 收藏
else:
    show in 最近阅读
```

The `s` key toggles `starred`. The `r` key removes from the dashboard by setting `hidden = true` and `starred = false`. Reopening the same target through CLI or the dashboard open prompt sets `hidden = false` and `starred = false`.

Alternatives considered:
- Treat `r` in starred rows as unstar only. This gives the same key different meanings by section.
- Delete entries entirely. This can surprise users because remove-from-entry should not delete progress/cache.

Rationale: `s` always means star/unstar; `r` always means remove from dashboard. Hidden supersedes starred, so removed favorites cannot silently reappear in the starred section.

### Add Markdown progress through the same history mechanism

Markdown opens should persist scroll position into history. Explicit CLI line arguments still take precedence for that launch:

```text
tread notes.md 50
  -> start at line 50
  -> save exit position to history
```

If no explicit line override exists, Markdown opens may use saved history position.

Rationale: dashboard cannot resume Markdown without a persisted Markdown position. Keeping the explicit line override preserves the existing CLI affordance.

### Open prompt uses one-level path suggestions

The dashboard `o` prompt accepts either URL or local path input. URL input starts with `http://` or `https://`. Local path input supports `~`, relative paths from the process current working directory, and absolute paths.

Path suggestions should list only the current directory implied by the input prefix. Suggestions should include directories and likely readable files; the final open behavior may continue existing CLI behavior where non-EPUB local files are parsed as Markdown.

```text
Input: ~/Books/bo
Directory searched: ~/Books
Matches:
  book.epub
  book-notes.md
  bookshelf/
```

Alternatives considered:
- Recursive fuzzy search. This adds performance, ignore-rule, and async complexity beyond the dashboard v1 goal.
- Shell-compatible expansion. This would require quoting/env-var/glob semantics not needed for a TUI launcher.

Rationale: one-level suggestions are predictable, cheap, and sufficient for opening nearby files.

Suggestion generation should sort directories before files, sort lexicographically within each group, and display at most 50 suggestions after sorting. Directory or metadata read errors should become recoverable prompt errors rather than escaping the TUI loop.

### Keep side effects behind testable boundaries

The implementation should isolate filesystem, clock, path resolution, history storage, suggestion generation, and terminal event loop boundaries. Core state transitions should be testable without starting a real terminal session.

Rationale: most dashboard behavior is deterministic state management. Isolating side effects keeps tests focused and avoids relying on brittle end-to-end TUI automation for every branch.

## Risks / Trade-offs

- **No-argument behavior is breaking.** → Mark the spec and README update explicitly; successful dashboard exit should use exit code 0.
- **History and existing progress can diverge.** → Treat history as dashboard metadata and keep reader-specific progress writes; update both at reader exit where applicable.
- **Path identity can drift for moved files.** → Use canonical path when available and preserve the display target; stale file opens should show an error and keep the user in dashboard.
- **Large directories can make suggestions noisy.** → Limit suggestion count and sort directories before files, then lexicographically.
- **Remove semantics can surprise if starred entries reappear.** → `r` always clears `starred`; reopening hidden targets restores them as unstarred recent entries.
- **TUI state can become tangled with reader state.** → Prefer a separate dashboard state/module instead of expanding the existing reader `App` with unrelated dashboard fields.

## Migration Plan

- Existing users with no `history.json` see the dashboard empty state on bare `tread`.
- Existing web and EPUB progress files remain valid.
- History entries are created or refreshed the next time a target is successfully opened.
- No automatic scan/migration of old progress files is required for v1 because old EPUB progress lacks enough display metadata.
- Rollback is simple: remove dashboard dispatch and ignore `history.json`; existing progress/cache files remain untouched.

## Open Questions

None. Current decisions:方案 B JSON history, one-level path suggestions, `s` for star/unstar, `r` for remove-from-dashboard, hidden supersedes starred, reopening hidden targets clears both hidden and starred state.
