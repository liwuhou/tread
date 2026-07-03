## MODIFIED Requirements

### Requirement: Parse Markdown source into styled lines
系统 SHALL 使用 `pulldown-cmark` 将 Markdown 源文本解析为带样式的行。**新增**：解析器 MUST 识别 `Event::Start(Tag::Image)` 事件并生成 `ImageNode`，而非忽略。

#### Scenario: 解析包含图片的 Markdown（新增）
- **WHEN** 输入包含 `![alt](url)` 的 Markdown 文本
- **THEN** 解析结果中包含对应的 `ImageNode`，`alt` 和 `url` 字段正确
