## Why

用户在阅读时希望获得沉浸式体验，状态栏（文件名、行号、百分比）虽然有用但占据一行空间。按 `f` 键切换状态栏显隐，让用户可以在"信息模式"和"沉浸模式"之间自由切换。

## What Changes

- **新增 `f` 快捷键**：切换状态栏的显示/隐藏
- **修改 App**：新增 `status_bar_visible` 字段
- **修改 UI**：根据 `status_bar_visible` 决定布局（显示状态栏时使用 2 行布局，隐藏时使用全屏）

## Capabilities

### Modified Capabilities
- `tui-navigation`: 新增 `f` 键切换状态栏显隐
