## Why

当前 Tab 键焦点导航存在体验问题：用户已滚动到文档中间位置时，按 Tab 会从文档顶部的第一个可聚焦元素开始，导致需要多次按 Tab 才能到达当前阅读位置附近的链接/图片，造成阅读进度丢失。

## What Changes

- 修改 Tab 键首次聚焦逻辑：当 `focus_index = None` 时，不再固定选择第一个元素，而是选择当前可视区域内或之后的第一个可聚焦元素
- 修改 Shift+Tab 首次聚焦逻辑：当 `focus_index = None` 时，选择当前可视区域内或之前的最后一个可聚焦元素
- 边界处理：当可视区域前后都没有可聚焦元素时，选择最近的元素（Tab 选最后一个，Shift+Tab 选第一个）

## Capabilities

### New Capabilities

（无新增 capability）

### Modified Capabilities

- `link-interaction`: 修改 "Unified focus navigation" 需求，Tab/Shift+Tab 首次聚焦应基于当前可视区域位置
- `tui-navigation`: 修改 "Vim-style scroll navigation" 中 Tab 相关场景，首次聚焦从可视区域开始

## Impact

- **代码**: `src/app.rs` 中 `handle_key` 方法的 `KeyCode::Tab` 和 `KeyCode::BackTab` 分支
- **测试**: 需要更新现有测试 `tab_focuses_first_image` 等，添加新测试覆盖可视区域感知聚焦逻辑
- **行为变更**: 非破坏性变更，仅改进首次聚焦的起始位置选择逻辑
