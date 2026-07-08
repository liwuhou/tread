## MODIFIED Requirements

### Requirement: Unified focus navigation
系统 SHALL 支持 Tab / Shift+Tab 在图片和链接之间统一导航。**修改**：首次聚焦时基于当前可视区域位置选择起始元素。

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
