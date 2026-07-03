## ADDED Requirements

### Requirement: Link detection in Markdown
系统 SHALL 识别 Markdown 中的 `[text](url)` 链接语法并生成 `LinkNode`。

#### Scenario: 识别外链
- **WHEN** 解析到 `[点击这里](https://example.com)`
- **THEN** 输出 `LineContent::Link`，`text = "点击这里"`，`url = "https://example.com"`，`is_external = true`

#### Scenario: 识别内链
- **WHEN** 解析到 `[下一章](chapter2.xhtml#section1)`
- **THEN** 输出 `LineContent::Link`，`is_external = false`

#### Scenario: 链接文本保留样式
- **WHEN** 解析到 `[**粗体链接**](url)`
- **THEN** 链接文本 "粗体链接" 保留 Bold 样式，同时有链接样式

### Requirement: Link detection in XHTML
系统 SHALL 识别 XHTML 中的 `<a href="url">text</a>` 并生成 `LinkNode`。

#### Scenario: XHTML 外链
- **WHEN** XHTML 包含 `<a href="https://example.com">visit</a>`
- **THEN** 输出 `LineContent::Link`，`is_external = true`

#### Scenario: XHTML 内链
- **WHEN** XHTML 包含 `<a href="chapter2.xhtml#sec1">跳转</a>`
- **THEN** 输出 `LineContent::Link`，`is_external = false`

### Requirement: Unified focus navigation
系统 SHALL 支持 Tab / Shift+Tab 在图片和链接之间统一导航。

#### Scenario: Tab 从图片跳到链接
- **WHEN** 文档中有一个图片（焦点位置 0）和一个链接（焦点位置 1），当前焦点在图片上
- **THEN** 按 Tab 后焦点移到链接

#### Scenario: Tab 从链接跳到图片
- **WHEN** 当前焦点在链接上，下一个可交互元素是图片
- **THEN** 按 Tab 后焦点移到图片

#### Scenario: Tab 循环所有可交互元素
- **WHEN** 文档中有 3 个可交互元素（混合图片和链接），当前在最后一个
- **THEN** 按 Tab 后焦点回到第一个

#### Scenario: Shift+Tab 反向
- **WHEN** 当前焦点在位置 1
- **THEN** 按 Shift+Tab 后焦点移到位置 0

### Requirement: Enter opens external links
系统 SHALL 在焦点在外部链接时，按 Enter 调用系统浏览器打开。

#### Scenario: macOS 打开外链
- **WHEN** 焦点在一个 `is_external = true` 的链接上，用户按 Enter
- **THEN** 执行 `open https://example.com`（macOS）

#### Scenario: Linux 打开外链
- **WHEN** 焦点在外部链接上，用户按 Enter
- **THEN** 执行 `xdg-open https://example.com`（Linux）

### Requirement: Enter navigates internal links (EPUB)
系统 SHALL 在 EPUB 模式下，按 Enter 对内链执行章节跳转。

#### Scenario: 内链匹配 TOC 条目
- **WHEN** 内链 href 为 `chapter2.xhtml`，EPUB TOC 中有条目 href 为 `chapter2.xhtml`
- **THEN** 跳转到该 TOC 条目对应的章节

#### Scenario: 内链带 fragment
- **WHEN** 内链 href 为 `chapter2.xhtml#section1`
- **THEN** 跳转到 chapter2.xhtml 对应章节，尝试定位到 #section1 锚点（MVP: 跳转到章节即可）

#### Scenario: 内链找不到目标
- **WHEN** 内链 href 无法匹配任何 TOC 条目或章节
- **THEN** 状态栏显示"链接目标未找到"

### Requirement: Link focus styling
系统 SHALL 为链接焦点提供独立的视觉样式。

#### Scenario: 链接普通样式
- **WHEN** 渲染非焦点链接
- **THEN** 显示为 Blue fg + Underline

#### Scenario: 链接焦点样式
- **WHEN** 链接处于焦点状态
- **THEN** 显示为 DarkGray bg + Bold + Underline
