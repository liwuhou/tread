## MODIFIED Requirements

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
