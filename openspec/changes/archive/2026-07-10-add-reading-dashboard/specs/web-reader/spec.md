## MODIFIED Requirements

### Requirement: Web page progress persistence
系统 SHALL 保存网页的阅读进度（滚动位置），并同步更新 unified reading history 以供 dashboard 显示和恢复。

#### Scenario: 保存进度
- **WHEN** 用户在网页中阅读到某位置，按 `q` 退出
- **THEN** 系统将 `{url, scroll, saved_at}` 保存到 `~/.tread/progress/web_<hash>.json`
- **AND** 系统 SHALL update `~/.tread/history.json` with the web target, title, scroll position, progress percent, and `updated_at`

#### Scenario: 恢复进度
- **WHEN** 用户再次打开同一 URL
- **THEN** existing web progress file SHALL be the source of truth for the restored scroll position
- **AND** unified history SHALL be refreshed from the reader's final scroll position after exit

#### Scenario: 首次访问无进度
- **WHEN** 首次访问一个 URL
- **THEN** 从顶部开始阅读

#### Scenario: Hidden web entry is reopened
- **WHEN** 用户通过 CLI 或 dashboard open prompt 再次打开一个 previously hidden web target
- **THEN** system SHALL set that history entry to `hidden = false` and `starred = false`
