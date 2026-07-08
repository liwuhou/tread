## MODIFIED Requirements

### Requirement: Unified focus navigation
系统 SHALL 支持 Tab / Shift+Tab 在图片和内联链接之间统一导航。**修改**：链接作为行内元素，焦点定位基于行内范围（line_idx + start_offset/end_offset），同一视觉行内属于同一个链接的连续 spans SHALL 作为一个逻辑焦点项。

#### Scenario: Tab 从图片跳到内联链接
- **WHEN** 文档中有图片和内联链接，当前焦点在图片上
- **THEN** 按 Tab 后焦点移到下一个内联链接

#### Scenario: Tab 在内联链接间跳转
- **WHEN** 当前焦点在某个内联链接上
- **THEN** 按 Tab 后焦点移到下一个可聚焦元素（链接或图片）

#### Scenario: Shift+Tab 反向
- **WHEN** 当前焦点在某个内联链接上
- **THEN** 按 Shift+Tab 后焦点移到上一个可聚焦元素

#### Scenario: 焦点定位基于字符范围
- **WHEN** 一行中有多个内联链接
- **THEN** 每个链接有独立的焦点范围，基于行索引和 start_offset/end_offset 定位

#### Scenario: 多词链接作为单个焦点
- **WHEN** 一个内联链接文本为 `Learn more`，wrap 阶段将它拆分为 `Learn`、空格、`more` 多个 spans
- **THEN** Tab 只聚焦该链接一次，并将 `Learn more` 整体作为焦点范围

### Requirement: Enter opens focused inline link
系统 SHALL 在焦点在内联链接时，按 Enter 执行链接操作。

#### Scenario: 打开外部链接
- **WHEN** 焦点在 `is_external = true` 的内联链接上，用户按 Enter
- **THEN** 用系统默认浏览器打开该链接

#### Scenario: 导航内部链接 (EPUB)
- **WHEN** 焦点在 EPUB 内链上，用户按 Enter
- **THEN** 跳转到目标章节

### Requirement: Inline link focus styling
系统 SHALL 为内联链接焦点提供视觉反馈。

#### Scenario: 链接普通样式
- **WHEN** 渲染非焦点内联链接
- **THEN** 显示为 Blue fg + Underline，前缀 🔗

#### Scenario: 链接焦点样式
- **WHEN** 内联链接处于焦点状态
- **THEN** 该链接文本的整个焦点范围显示为 DarkGray bg + Bold + Underline
