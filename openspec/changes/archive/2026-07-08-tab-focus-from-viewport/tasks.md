## 1. 测试编写（TDD - 先写失败的测试）

- [x] 1.1 编写测试：Tab 首次聚焦从可视区域开始（scroll > 0，focus_index = None）
- [x] 1.2 编写测试：Shift+Tab 首次聚焦从可视区域开始（scroll > 0，focus_index = None）
- [x] 1.3 编写测试：Tab 首次聚焦边界情况 - 所有元素在屏幕上方
- [x] 1.4 编写测试：Shift+Tab 首次聚焦边界情况 - 所有元素在屏幕下方
- [x] 1.5 更新现有测试 `tab_focuses_first_image` 以适配新行为（scroll = 0 时仍聚焦第一个）

## 2. 核心实现

- [x] 2.1 在 `src/app.rs` 中添加辅助函数 `find_first_focusable_from_viewport()` - 查找可视区域内或之后的第一个可聚焦元素
- [x] 2.2 在 `src/app.rs` 中添加辅助函数 `find_last_focusable_before_viewport()` - 查找可视区域内或之前的最后一个可聚焦元素
- [x] 2.3 修改 `KeyCode::Tab` 分支：`focus_index = None` 时使用新的辅助函数
- [x] 2.4 修改 `KeyCode::BackTab` 分支：`focus_index = None` 时使用新的辅助函数

## 3. 验证

- [x] 3.1 运行所有测试确保通过
- [x] 3.2 手动测试：打开长文档，滚动到中间，按 Tab 验证焦点从可视区域开始
- [x] 3.3 手动测试：验证边界情况（滚动到顶部/底部时的行为）
