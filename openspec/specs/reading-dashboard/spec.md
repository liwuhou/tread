# reading-dashboard Specification

## Purpose
TBD - created by archiving change add-reading-dashboard. Update Purpose after archive.
## Requirements
### Requirement: Dashboard entry screen
系统 SHALL 在用户无参数运行 `tread` 时进入 dashboard TUI，作为继续阅读和打开新读物的入口。Dashboard SHALL treat terminal height as a layout constraint and degrade auxiliary UI before hiding the recent-reading area.

#### Scenario: Bare command opens dashboard
- **WHEN** 用户运行 `tread` 且没有提供文件、EPUB、URL 或其它参数
- **THEN** 系统 SHALL 进入 dashboard TUI
- **AND** 系统 SHALL NOT 将无参数视为 usage error
- **AND** 用户按 `q` 或 `Esc` 退出 dashboard 后进程 SHALL 以退出码 0 结束

#### Scenario: Empty history dashboard
- **WHEN** 用户运行 `tread` 且 `~/.tread/history.json` 不存在或没有可显示条目
- **THEN** dashboard SHALL 显示空状态
- **AND** 空状态 SHALL 提示用户可使用 `o` 输入路径或 URL 打开读物
- **AND** 空状态 SHALL 提示用户仍可使用 `tread <file.md|file.epub|url>` 直接打开读物

#### Scenario: Dashboard preserves terminal cleanup
- **WHEN** 用户进入 dashboard 后按 `q`、`Esc` 或 `Ctrl+C` 退出
- **THEN** 系统 SHALL 恢复 terminal raw mode、alternate screen、mouse capture 和 cursor 状态

#### Scenario: Constrained height preserves recent-reading area
- **WHEN** dashboard terminal height is smaller than the full dashboard layout but at least 3 rows tall
- **THEN** dashboard SHALL preserve at least 3 visible lines for the recent-reading area
- **AND** those 3 lines SHALL include the recent-reading section context plus the currently relevant entry title and target line
- **AND** dashboard SHALL hide or truncate auxiliary blocks before violating that 3-line reserve

#### Scenario: Extremely small terminal fallback
- **WHEN** dashboard terminal height is smaller than 3 rows
- **THEN** dashboard SHALL render a minimal fallback state instead of a partially clipped normal dashboard
### Requirement: Starred and recent sections
Dashboard SHALL display visible history entries in separate starred and recent sections without duplicating the same entry.

#### Scenario: Starred entries are shown first
- **WHEN** history contains visible entries with `starred = true`
- **THEN** dashboard SHALL show those entries under a starred section before the recent section

#### Scenario: Recent excludes starred entries
- **WHEN** history contains a visible entry with `starred = true`
- **THEN** dashboard SHALL NOT also show that entry in the recent section

#### Scenario: Recent entries are ordered by update time
- **WHEN** history contains multiple visible unstarred entries
- **THEN** dashboard SHALL show them in descending `updated_at` order

#### Scenario: Hidden entries are not displayed
- **WHEN** history contains an entry with `hidden = true`
- **THEN** dashboard SHALL NOT show that entry in the starred section
- **AND** dashboard SHALL NOT show that entry in the recent section

### Requirement: Dashboard item actions
Dashboard SHALL support keyboard actions for selecting, opening, starring, unstarring, and removing visible entries.

#### Scenario: Selection navigation
- **WHEN** dashboard has visible entries and the user presses `j` or `Down`
- **THEN** selection SHALL move to the next visible entry when one exists
- **WHEN** the user presses `k` or `Up`
- **THEN** selection SHALL move to the previous visible entry when one exists

#### Scenario: Open selected entry
- **WHEN** the user selects a visible dashboard entry and presses `Enter`
- **THEN** system SHALL open that entry's target in the appropriate reader mode
- **AND** system SHALL restore the saved reading position for that entry when possible

#### Scenario: Toggle starred state
- **WHEN** the user selects a visible unstarred entry and presses `s`
- **THEN** system SHALL set `starred = true` for that entry
- **AND** the entry SHALL appear in the starred section
- **WHEN** the user selects a visible starred entry and presses `s`
- **THEN** system SHALL set `starred = false` for that entry
- **AND** the entry SHALL appear in the recent section

#### Scenario: Remove entry from dashboard
- **WHEN** the user selects any visible entry and presses `r`
- **THEN** system SHALL set `hidden = true` for that entry
- **AND** system SHALL set `starred = false` for that entry
- **AND** the entry SHALL disappear from all dashboard sections
- **AND** system SHALL NOT delete cached content or reader progress files for that entry

### Requirement: Dashboard open prompt
Dashboard SHALL provide an `o` open prompt that accepts URLs and local paths without leaving the TUI. Under constrained height, prompt visibility SHALL be lower priority than preserving recent-reading content.

#### Scenario: Open prompt appears
- **WHEN** the user presses `o` from dashboard list mode
- **THEN** dashboard SHALL show an input prompt for a URL or local path when height permits
- **AND** normal dashboard list actions SHALL be suspended until the prompt is submitted or closed

#### Scenario: Cancel open prompt
- **WHEN** the open prompt is visible and the user presses `Esc`
- **THEN** dashboard SHALL close the prompt
- **AND** dashboard SHALL return to list mode without opening a target

#### Scenario: Submit URL
- **WHEN** the user enters a value beginning with `http://` or `https://` and presses `Enter`
- **THEN** system SHALL open that value in web reader mode

#### Scenario: Submit existing local file
- **WHEN** the user enters an existing local file path and presses `Enter`
- **THEN** system SHALL open `.epub` files in EPUB reader mode
- **AND** system SHALL open other local files with the Markdown reader behavior used by `tread <path>`

#### Scenario: Submit missing local path
- **WHEN** the user enters a local path that does not exist and presses `Enter`
- **THEN** dashboard SHALL show an inline error
- **AND** dashboard SHALL remain in the open prompt

#### Scenario: Local path I/O error
- **WHEN** resolving, inspecting, reading, or opening a local path fails for a reason other than missing path, such as permission denied or unreadable directory
- **THEN** dashboard SHALL show an inline recoverable error
- **AND** dashboard SHALL remain usable without leaving the TUI

#### Scenario: Prompt may be obscured under constrained height
- **WHEN** the open prompt is active and terminal height is too small to show both the prompt UI and the reserved recent-reading area
- **THEN** dashboard MAY partially hide the prompt, suggestions, or recoverable error text
- **AND** dashboard SHALL still preserve the reserved recent-reading area before those auxiliary prompt elements

### Requirement: Current-directory path suggestions
The open prompt SHALL provide current-directory-only suggestions for local path input.

#### Scenario: Suggest current directory matches
- **WHEN** the user types a local path prefix
- **THEN** dashboard SHALL inspect only the directory implied by that prefix
- **AND** dashboard SHALL show matching direct children of that directory
- **AND** dashboard SHALL NOT recursively search descendant directories
- **AND** dashboard SHALL sort directories before files, then sort lexicographically within each group
- **AND** dashboard SHALL display at most 50 suggestions after sorting

#### Scenario: Tilde and relative path support
- **WHEN** the user types a path beginning with `~/`
- **THEN** dashboard SHALL resolve suggestions relative to the user's home directory
- **WHEN** the user types a relative path
- **THEN** dashboard SHALL resolve suggestions relative to the process current working directory

#### Scenario: Directory suggestion
- **WHEN** a suggestion is a directory
- **THEN** dashboard SHALL display it with a trailing `/`
- **AND** submitting that directory SHALL complete or enter the directory rather than opening it as a reader target

#### Scenario: Suggestion selection and completion
- **WHEN** suggestions are visible and the user presses `Down` or `Up`
- **THEN** selection SHALL move within the suggestions list
- **WHEN** the user presses `Tab`
- **THEN** the prompt SHALL fill with the selected suggestion
- **WHEN** the user presses `Enter` with a file suggestion selected
- **THEN** system SHALL open that selected file

