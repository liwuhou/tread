## 1. App 修改

- [x] 1.1 在 App 中添加 `status_bar_visible: bool` 字段（默认 true）
- [x] 1.2 `handle_key` 中添加 `f` 键处理：切换 `status_bar_visible`

## 2. UI 修改

- [x] 2.1 `draw` 函数根据 `status_bar_visible` 决定布局
- [x] 2.2 隐藏时内容区域占满全屏
- [x] 2.3 更新帮助浮层添加 `f` 键说明

## 3. 集成验证

- [x] 3.1 `cargo build` 无 warning
- [x] 3.2 `cargo test` 全部通过
- [x] 3.3 手动测试：按 `f` 隐藏/显示状态栏
