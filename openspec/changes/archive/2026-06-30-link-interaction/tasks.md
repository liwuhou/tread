## 1. 数据模型重构

- [x] 1.1 在 `src/image.rs` 中新增 `LinkNode` 结构体（text, url, is_external）
- [x] 1.2 在 `LineContent` 枚举中新增 `Link(LinkNode)` 变体
- [x] 1.3 将 `image_positions` 重命名为 `focusable_positions`，支持图片和链接的统一索引

## 2. Markdown 链接检测（TDD）

- [x] 2.1 编写测试：`[text](url)` 生成 LinkNode
- [x] 2.2 编写测试：外部链接 `is_external = true`
- [x] 2.3 编写测试：内部链接 `is_external = false`
- [x] 2.4 修改 `parse_markdown` 处理 `Event::Start(Tag::Link)` / `Event::End(TagEnd::Link)`

## 3. XHTML 链接检测（TDD）

- [x] 3.1 编写测试：`<a href="url">text</a>` 生成 LinkNode
- [x] 3.2 编写测试：XHTML 外部链接
- [x] 3.3 编写测试：XHTML 内部链接
- [x] 3.4 修改 `xhtml_to_lines` 处理 `<a>` 标签

## 4. 统一焦点导航（TDD）

- [x] 4.1 编写测试：Tab 在图片和链接之间导航
- [x] 4.2 编写测试：Shift+Tab 反向导航
- [x] 4.3 编写测试：Tab 循环所有可交互元素
- [x] 4.4 重构 `handle_key` 中 Tab/Shift+Tab 为统一的焦点导航

## 5. 外链打开（TDD）

- [x] 5.1 编写测试：`open_url()` 在 macOS 调用 `open`
- [x] 5.2 编写测试：`open_url()` 在 Linux 调用 `xdg-open`
- [x] 5.3 实现 `open_url()` 函数
- [x] 5.4 修改 Enter 键处理：外部链接调用 `open_url()`

## 6. 内链跳转（TDD）

- [x] 6.1 编写测试：内链匹配 TOC 条目跳转到对应章节
- [x] 6.2 编写测试：内链带 fragment 跳转到章节
- [x] 6.3 编写测试：内链找不到目标显示提示
- [x] 6.4 实现内链解析和跳转逻辑

## 7. UI 渲染

- [x] 7.1 渲染 LinkNode 为带样式的文本（Blue fg + Underline）
- [x] 7.2 焦点链接样式（DarkGray bg + Bold + Underline）
- [x] 7.3 更新帮助浮层显示 Tab/Enter 对链接的说明

## 8. 集成验证

- [x] 8.1 `cargo build` 无 warning 编译通过
- [x] 8.2 `cargo test` 所有测试通过
- [x] 8.3 更新 `sample.md` 添加链接测试用例
- [x] 8.4 手动测试：Tab 导航链接、Enter 打开外链、Enter 跳转内链
