## MODIFIED Requirements

### Requirement: Vim-style scroll navigation
系统 SHALL 支持 vim 风格的键盘导航。**修改**：Tab / Shift+Tab 在图片和链接之间跳转时，首次聚焦基于当前可视区域位置。

#### Scenario: 单行滚动
- **WHEN** 用户按 `j` 或 `↓`
- **THEN** 内容向上滚动 1 行
- **WHEN** 用户按 `k` 或 `↑`
- **THEN** 内容向下滚动 1 行

#### Scenario: 半页滚动
- **WHEN** 用户按 `Ctrl+d`
- **THEN** 内容向下滚动 half_page 行（half_page = 终端内容高度 / 2）
- **WHEN** 用户按 `Ctrl+u`
- **THEN** 内容向上滚动 half_page 行

#### Scenario: 整页滚动
- **WHEN** 用户按 `Ctrl+f`
- **THEN** 内容向下滚动一整页（终端内容高度行）
- **WHEN** 用户按 `Ctrl+b`
- **THEN** 内容向上滚动一整页

#### Scenario: 跳转到顶部/底部
- **WHEN** 用户按 `g` 或 `Home`
- **THEN** 滚动到文件顶部（scroll = 0）
- **WHEN** 用户按 `G` 或 `End`
- **THEN** 滚动到文件底部（scroll = max_scroll）

#### Scenario: PageUp/PageDown
- **WHEN** 用户按 `PageDown`
- **THEN** 内容向下滚动 half_page 行
- **WHEN** 用户按 `PageUp`
- **THEN** 内容向上滚动 half_page 行

#### Scenario: Tab 跳到下一个可聚焦元素
- **WHEN** 文档中有可聚焦元素（图片/链接），用户按 Tab
- **THEN** 如果当前无焦点，焦点跳到可视区域内或之后的第一个可聚焦元素；如果已有焦点，焦点跳到下一个可聚焦元素

#### Scenario: Shift+Tab 跳到上一个可聚焦元素
- **WHEN** 文档中有可聚焦元素（图片/链接），用户按 Shift+Tab
- **THEN** 如果当前无焦点，焦点跳到可视区域内或之前的最后一个可聚焦元素；如果已有焦点，焦点跳到上一个可聚焦元素

#### Scenario: Enter 打开焦点元素
- **WHEN** 焦点在某可聚焦元素（图片或链接）上，用户按 Enter
- **THEN** 用系统默认程序打开该元素（图片查看器或浏览器）
