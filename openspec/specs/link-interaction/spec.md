## Purpose

Defines how tread detects, renders, focuses, and opens links from Markdown and EPUB/XHTML content.

## Requirements

### Requirement: Link detection in Markdown
系统 SHALL 识别 Markdown 中的 `[text](url)` 链接语法，并将链接元数据附加到对应的内联 styled span。

#### Scenario: 识别外链
- **WHEN** 解析到 `[点击这里](https://example.com)`
- **THEN** 输出包含文本 `🔗点击这里` 的内联 styled span，链接元数据 `url = "https://example.com"` 且 `is_external = true`

#### Scenario: 识别内链
- **WHEN** 解析到 `[下一章](chapter2.xhtml#section1)`
- **THEN** 输出包含文本 `🔗下一章` 的内联 styled span，链接元数据 `is_external = false`

#### Scenario: 链接文本保留样式
- **WHEN** 解析到 `[**粗体链接**](url)`
- **THEN** 链接文本 `🔗粗体链接` 保留 Bold 样式，同时有链接样式和链接元数据

### Requirement: Link detection in XHTML
系统 SHALL 识别 XHTML 中的 `<a href="url">text</a>` 并将链接元数据附加到对应的内联 styled span。

#### Scenario: XHTML 外链
- **WHEN** XHTML 包含 `<a href="https://example.com">visit</a>`
- **THEN** 输出包含文本 `🔗visit` 的内联 styled span，链接元数据 `is_external = true`

#### Scenario: XHTML 内链
- **WHEN** XHTML 包含 `<a href="chapter2.xhtml#sec1">跳转</a>`
- **THEN** 输出包含文本 `🔗跳转` 的内联 styled span，链接元数据 `is_external = false`

### Requirement: Unified focus navigation
系统 SHALL 支持 Tab / Shift+Tab 在图片和内联链接之间统一导航。链接作为行内元素，焦点定位基于行内范围（line_idx + start_offset/end_offset），同一视觉行内属于同一个链接的连续 spans SHALL 作为一个逻辑焦点项。首次聚焦 SHALL 基于当前可视区域位置选择起始元素。

#### Scenario: Tab 从图片跳到内联链接
- **WHEN** 文档中有图片和内联链接，当前焦点在图片上
- **THEN** 按 Tab 后焦点移到下一个内联链接

#### Scenario: Tab 在内联链接间跳转
- **WHEN** 当前焦点在某个内联链接上
- **THEN** 按 Tab 后焦点移到下一个可聚焦元素（链接或图片）

#### Scenario: Tab 循环所有可交互元素
- **WHEN** 文档中有 3 个可交互元素（混合图片和链接），当前在最后一个
- **THEN** 按 Tab 后焦点回到第一个

#### Scenario: Shift+Tab 反向
- **WHEN** 当前焦点在某个内联链接上
- **THEN** 按 Shift+Tab 后焦点移到上一个可聚焦元素

#### Scenario: 焦点定位基于字符范围
- **WHEN** 一行中有多个内联链接
- **THEN** 每个链接有独立的焦点范围，基于行索引和 start_offset/end_offset 定位

#### Scenario: 多词链接作为单个焦点
- **WHEN** 一个内联链接文本为 `Learn more`，wrap 阶段将它拆分为 `Learn`、空格、`more` 多个 spans
- **THEN** Tab 只聚焦该链接一次，并将 `Learn more` 整体作为焦点范围

#### Scenario: Tab 首次聚焦从可视区域开始
- **WHEN** 用户已滚动到文档中间（scroll > 0），当前无焦点（focus_index = None）
- **THEN** 按 Tab 后焦点跳到可视区域内或之后的第一个可聚焦元素

#### Scenario: Shift+Tab 首次聚焦从可视区域开始
- **WHEN** 用户已滚动到文档中间（scroll > 0），当前无焦点（focus_index = None）
- **THEN** 按 Shift+Tab 后焦点跳到可视区域内或之前的最后一个可聚焦元素

#### Scenario: Tab 首次聚焦边界情况 - 所有元素在屏幕上方
- **WHEN** 用户已滚动到文档底部，所有可聚焦元素都在当前可视区域上方
- **THEN** 按 Tab 后焦点跳到最后一个可聚焦元素

#### Scenario: Shift+Tab 首次聚焦边界情况 - 所有元素在屏幕下方
- **WHEN** 用户在文档顶部（scroll = 0），所有可聚焦元素都在当前可视区域下方
- **THEN** 按 Shift+Tab 后焦点跳到第一个可聚焦元素

### Requirement: Enter opens focused inline link
系统 SHALL 在焦点位于内联链接时，按 Enter 执行链接操作。

#### Scenario: 打开外部链接
- **WHEN** 焦点在 `is_external = true` 的内联链接上，用户按 Enter
- **THEN** 用系统默认浏览器打开该链接

#### Scenario: 导航内部链接 (EPUB)
- **WHEN** 焦点在 EPUB 内链上，用户按 Enter
- **THEN** 跳转到目标章节

#### Scenario: 内链找不到目标
- **WHEN** 内链 href 无法匹配任何 TOC 条目或章节
- **THEN** 状态栏显示“链接目标未找到”

### Requirement: Inline link focus styling
系统 SHALL 为内联链接焦点提供视觉反馈。

#### Scenario: 链接普通样式
- **WHEN** 渲染非焦点内联链接
- **THEN** 显示为 Blue fg + Underline，前缀 🔗

#### Scenario: 链接焦点样式
- **WHEN** 内联链接处于焦点状态
- **THEN** 该链接文本的整个焦点范围显示为 DarkGray bg + Bold + Underline
