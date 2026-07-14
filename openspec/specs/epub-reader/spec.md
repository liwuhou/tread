## Purpose

Defines EPUB parsing, chapter caching, style serialization, and reader progress behavior.
## Requirements
### Requirement: XHTML content rendering
系统 SHALL 将 EPUB 中的 XHTML 内容转换为 `Vec<LineContent>` 进行终端渲染。**新增**：对 calibre 等工具生成的非自然段落进行智能合并。

#### Scenario: 合并连续空段落（新增）
- **WHEN** XHTML 包含多个连续的空 `<p>` 标签（如 `<p> </p><p> </p><p> </p>`）
- **THEN** 合并为单个空行，而非每个都输出空行

#### Scenario: 合并句子延续的假分段（新增）
- **WHEN** 前一段以逗号等延续标点结尾（如 `从何处下手。`→`从何处下手，`），后一段是句子延续
- **THEN** 两段合并为同一段落，不加空行

#### Scenario: 保留真正的段落分隔（新增）
- **WHEN** 前一段以句号/问号/感叹号结尾，后一段是新内容
- **THEN** 两段之间保留一个空行

#### Scenario: 合并后输出自然的阅读体验（新增）
- **WHEN** 使用 calibre 转换的 EPUB（如《境界》）
- **THEN** 章节标题前后无多余空行，连续文字不被意外切断

### Requirement: Chapter parsing cache
系统 SHALL 缓存解析后的章节内容，避免重复解析。

#### Scenario: 首次打开章节
- **WHEN** 首次打开 EPUB 的某章节，且缓存不存在
- **THEN** 解析 XHTML → 合并段落 → 写入缓存文件 → 渲染

#### Scenario: 再次打开章节（缓存命中）
- **WHEN** 再次打开同一章节，且缓存存在
- **THEN** 直接从缓存读取，跳过 XHTML 解析和合并

#### Scenario: 缓存文件路径
- **WHEN** 缓存写入
- **THEN** 保存到 `~/.tread/cache/epub/<book_hash>/chapter_<N>.json`

#### Scenario: EPUB 文件变化时缓存失效
- **WHEN** 用户替换了同名但内容不同的 EPUB 文件
- **THEN** 文件路径 hash 不变但缓存应能处理（可选：检查文件大小/修改时间）

### Requirement: Style serialization
系统 SHALL 能将 `ratatui::style::Style` 序列化/反序列化为 JSON 格式。

#### Scenario: 序列化基本样式
- **WHEN** Style 包含 fg=Red + bold
- **THEN** 序列化为 `{"fg": "red", "bold": true}`

#### Scenario: 反序列化恢复样式
- **WHEN** 从 JSON 读取 `{"fg": "cyan", "italic": true}`
- **THEN** 恢复为 fg=Cyan + Italic 的 Style

#### Scenario: 默认 Style 不存储
- **WHEN** Style 为 default（无颜色无修饰）
- **THEN** 序列化为空对象 `{}`

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

