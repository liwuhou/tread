## MODIFIED Requirements

### Requirement: Reading progress persistence
系统 SHALL 记住每本 EPUB 的上次阅读位置，并同步更新 unified reading history 以供 dashboard 显示和恢复。

#### Scenario: 保存进度
- **WHEN** 用户在 EPUB 中阅读到第三章第 50 行，按 `q` 退出
- **THEN** 系统将 `{chapter: 2, scroll: 50}` 保存到 `~/.tread/progress/<book_hash>.json`
- **AND** 系统 SHALL update `~/.tread/history.json` with EPUB path, title, chapter, chapter count, scroll position, progress percent, and `updated_at`

#### Scenario: 恢复进度
- **WHEN** 用户再次打开同一本 EPUB
- **THEN** existing EPUB progress file SHALL be the source of truth for restored chapter and scroll position
- **AND** unified history SHALL be refreshed from the reader's final chapter and scroll position after exit

#### Scenario: 进度文件不存在
- **WHEN** 首次打开一本 EPUB
- **THEN** 从第一章第一行开始阅读

#### Scenario: Hidden EPUB entry is reopened
- **WHEN** 用户通过 CLI 或 dashboard open prompt 再次打开一个 previously hidden EPUB target
- **THEN** system SHALL set that history entry to `hidden = false` and `starred = false`
