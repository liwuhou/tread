## MODIFIED Requirements

### Requirement: Vim-style scroll navigation
系统 SHALL 支持 vim 风格的键盘导航。**新增**：Tab / Shift+Tab 在图片占位符之间跳转，Enter 打开当前焦点图片。

#### Scenario: Tab 跳到下一个图片（新增）
- **WHEN** 文档中有图片，用户按 Tab
- **THEN** 焦点跳到下一个图片占位符

#### Scenario: Shift+Tab 跳到上一个图片（新增）
- **WHEN** 文档中有图片，用户按 Shift+Tab
- **THEN** 焦点跳到上一个图片占位符

#### Scenario: Enter 打开焦点图片（新增）
- **WHEN** 焦点在某图片占位符上，用户按 Enter
- **THEN** 用系统默认图片浏览器打开该图片
