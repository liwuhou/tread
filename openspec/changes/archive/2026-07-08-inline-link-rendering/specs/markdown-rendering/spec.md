## MODIFIED Requirements

### Requirement: Inline link rendering
系统 SHALL 将 Markdown 中的链接内联渲染在文本流中，不单独成行。

#### Scenario: 基本内联链接
- **WHEN** 解析到 `This is a [link](https://example.com) in text`
- **THEN** 渲染为单行：`This is a 🔗link in text`，链接部分有蓝色下划线样式

#### Scenario: 多个链接在同一段落
- **WHEN** 段落中有多个链接 `[a](url1) and [b](url2)`
- **THEN** 渲染为：`🔗a and 🔗b`，每个链接内联显示

#### Scenario: 链接与其他样式组合
- **WHEN** 链接包含在粗体中 `[**bold link**](url)`
- **THEN** 渲染为 `🔗bold link`，同时有粗体和链接样式

#### Scenario: 链接文本为空时使用 URL
- **WHEN** 链接文本为空 `[](https://example.com)`
- **THEN** 渲染为 `🔗https://example.com`

### Requirement: Link no longer as separate line
系统 SHALL 不再将链接作为独立的 `LineContent::Link` 行推出。

#### Scenario: 段落中链接不单独成行
- **WHEN** 解析包含链接的段落
- **THEN** 输出中链接嵌入在 `LineContent::Styled` 中，不产生独立的 `LineContent::Link`
