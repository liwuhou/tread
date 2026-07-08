## 1. 数据结构改造

- [x] 1.1 在 `src/image.rs` 中创建 `LinkInfo` 结构体（url, is_external）
- [x] 1.2 修改 `LineContent::Styled` 的 span 类型，添加可选的 `link: Option<LinkInfo>`
- [x] 1.3 更新 `wrap_lines` 函数以正确处理带链接的 spans

## 2. 解析器改造

- [x] 2.1 修改 `Event::End(Link)` 处理：不再推入 `pending_links`，而是给当前 span 添加链接元数据
- [x] 2.2 删除段落结束时推出 `LineContent::Link` 的代码
- [x] 2.3 在链接文本前添加 🔗 前缀

## 3. 焦点系统改造

- [x] 3.1 修改 `focusable_positions` 类型为 `Vec<(usize, usize)>`（行索引, 字符偏移）
- [x] 3.2 修改 `rebuild_focusable_positions` 扫描行内链接位置
- [x] 3.3 修改 `focused_item` 返回行内链接信息

## 4. 渲染器改造

- [x] 4.1 修改 `src/ui.rs` 渲染逻辑：处理带链接的 spans（基础版）
- [x] 4.2 实现内联链接的焦点样式（DarkGray bg + Bold + Underline）
- [x] 4.3 删除 `LineContent::Link` 的独立渲染代码（保留变体定义）

## 5. 其他文件适配

- [x] 5.1 修改 `src/epub.rs` 缓存序列化
- [x] 5.2 修改 `src/xhtml.rs` 适配新的 `StyledSpan` 类型
- [x] 5.3 修改测试代码适配新的 `StyledSpan` 类型

## 6. 测试更新

- [x] 6.1 更新现有链接相关测试
- [x] 6.2 添加新测试：内联链接解析（inline_link_infos 辅助函数）
- [x] 6.3 添加新测试：行内焦点导航（依赖焦点系统改造）
- [x] 6.4 运行所有测试确保通过（154 passed）

## 7. 手动验证

- [x] 7.1 打开 sample.md 验证链接内联显示（自动化回归覆盖）
- [x] 7.2 验证 Tab 导航在链接间正确跳转（新增多词链接单焦点回归测试）
- [x] 7.3 验证 Enter 打开链接（现有 Enter 链接测试通过）
