# reading-history Specification

## Purpose
TBD - created by archiving change add-reading-dashboard. Update Purpose after archive.
## Requirements
### Requirement: Unified reading history index
系统 SHALL maintain a unified local reading history index for dashboard display and resume metadata.

#### Scenario: History file location
- **WHEN** reading history is persisted
- **THEN** system SHALL write it under `~/.tread/history.json`

#### Scenario: History entry fields
- **WHEN** a history entry is stored
- **THEN** it SHALL include a stable `id`, `kind`, `title`, `target`, reading position, `progress_percent`, `updated_at`, `starred`, and `hidden`
- **AND** `kind` SHALL distinguish `web`, `epub`, and `markdown`

#### Scenario: History file missing
- **WHEN** `~/.tread/history.json` does not exist
- **THEN** system SHALL treat history as empty
- **AND** system SHALL NOT fail dashboard startup

#### Scenario: Malformed history file
- **WHEN** `~/.tread/history.json` cannot be parsed
- **THEN** system SHALL avoid crashing
- **AND** system SHALL move the malformed file aside to a timestamped backup such as `history.json.bak.<timestamp>` before writing a replacement history file
- **AND** dashboard SHALL show a recoverable warning that the previous history file was backed up

#### Scenario: Atomic history write
- **WHEN** system saves `~/.tread/history.json`
- **THEN** system SHALL write a complete temporary file in the same directory and atomically replace the history file
- **AND** system SHALL NOT leave a partially-written JSON file at `~/.tread/history.json` if the write fails

### Requirement: Stable content identity
系统 SHALL use stable target identities so reopening the same target updates the same history entry.

#### Scenario: Web identity
- **WHEN** storing a web entry
- **THEN** system SHALL derive its id using the same URL string identity used by existing web cache/progress keys
- **AND** reopening the same URL string SHALL update the same history entry

#### Scenario: Local file identity
- **WHEN** storing an EPUB or Markdown entry for an existing local file
- **THEN** system SHALL derive its id from the canonical absolute file path when canonicalization succeeds
- **AND** system SHALL derive its id from the resolved absolute path when canonicalization fails
- **AND** system SHALL keep a display target separately from the identity source so dashboard rows can show a user-friendly path

### Requirement: History creation and updates
系统 SHALL create or update history entries when targets are opened and when reading positions change on exit.

#### Scenario: Successful open creates entry
- **WHEN** a web, EPUB, or Markdown target opens successfully
- **THEN** system SHALL create or update the corresponding history entry with target metadata
- **AND** system SHALL set `hidden = false`
- **AND** if the entry was previously hidden, system SHALL set `starred = false`

#### Scenario: Exit updates progress
- **WHEN** the user exits a reader after opening from CLI, dashboard selection, or dashboard open prompt
- **THEN** system SHALL update the corresponding history entry with the latest reading position, display progress, and `updated_at`
- **AND** the update SHALL preserve unrelated concurrent state changes such as `starred` or `hidden` when they are newer than the reader session's opening snapshot

#### Scenario: Reader-specific progress precedence
- **WHEN** a web or EPUB target has both unified history and an existing reader-specific progress file
- **THEN** the reader-specific progress file SHALL be the source of truth for the actual restored reading position
- **AND** unified history SHALL be refreshed from the reader position after the reader exits
- **WHEN** a Markdown target has unified history
- **THEN** unified history SHALL be the source of truth for restored Markdown position unless an explicit CLI line override is provided

#### Scenario: Progress percent bounds
- **WHEN** system stores `progress_percent`
- **THEN** it SHALL clamp the value between 0 and 100 inclusive

#### Scenario: Remove does not delete reader data
- **WHEN** dashboard removal sets `hidden = true`
- **THEN** system SHALL leave web cache files, EPUB cache files, and reader-specific progress files untouched

### Requirement: Markdown progress persistence
系统 SHALL persist Markdown reading progress so Markdown files can be resumed from dashboard and direct opens.

#### Scenario: Markdown exit saves progress
- **WHEN** the user exits a Markdown reader session
- **THEN** system SHALL store the Markdown scroll position in reading history

#### Scenario: Markdown direct open restores progress
- **WHEN** the user runs `tread notes.md` and a history entry exists for that file
- **THEN** system SHALL restore the saved Markdown scroll position

#### Scenario: Explicit Markdown line overrides history
- **WHEN** the user runs `tread notes.md 50` and a history entry exists for that file
- **THEN** system SHALL start at line 50 rather than the saved history position
- **AND** on exit system SHALL save the resulting current position

### Requirement: Reader-specific compatibility
系统 SHALL keep existing web and EPUB progress behavior compatible while adding unified history.

#### Scenario: Web progress compatibility
- **WHEN** the user reads a web page and exits
- **THEN** system SHALL continue saving web progress in the existing web progress format
- **AND** system SHALL update unified history for dashboard display

#### Scenario: EPUB progress compatibility
- **WHEN** the user reads an EPUB and exits
- **THEN** system SHALL continue saving EPUB progress in the existing EPUB progress format
- **AND** system SHALL update unified history for dashboard display

