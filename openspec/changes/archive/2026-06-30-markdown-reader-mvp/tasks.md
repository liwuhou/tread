## 1. 项目初始化

- [x] 1.1 创建 `Cargo.toml`（依赖：ratatui 0.29, crossterm 0.28, pulldown-cmark 0.12, unicode-width 0.2, anyhow 1）
- [x] 1.2 创建 `.gitignore`（/target, Cargo.lock）
- [x] 1.3 创建目录结构 `src/main.rs`, `src/app.rs`, `src/parser.rs`, `src/ui.rs`
- [x] 1.4 创建 `sample.md` 测试文档（覆盖所有 Markdown 元素）

## 2. Markdown 解析器（TDD）

- [x] 2.1 编写 parser 单元测试骨架（`tests/parser_test.rs` 或 `parser.rs` 内 `#[cfg(test)]`）
- [x] 2.2 测试：解析标题（H1–H6）输出正确样式（颜色、Bold、Underline）
- [x] 2.3 测试：解析粗体/斜体/删除线/行内代码/链接
- [x] 2.4 测试：解析围栏代码块（带语言标签 + 边框线）
- [x] 2.5 测试：解析有序/无序列表（缩进 + 标记）
- [x] 2.6 测试：解析引用块（`▎` 前缀）
- [x] 2.7 测试：解析表格（`│` 分隔）
- [x] 2.8 测试：解析分隔线
- [x] 2.9 测试：空输入返回空 Vec
- [x] 2.10 实现 `parse_markdown(source: &str) -> Vec<Vec<(String, Style)>>`，使所有测试通过

## 3. 行换行算法（TDD）

- [x] 3.1 编写换行单元测试骨架（`app.rs` 内 `#[cfg(test)]`）
- [x] 3.2 测试：英文按空格分词换行
- [x] 3.3 测试：CJK 字符按字符换行（unicode-width 计算）
- [x] 3.4 测试：中英混排正确换行
- [x] 3.5 测试：超长单词字符级断行
- [x] 3.6 测试：空行保持为空行
- [x] 3.7 实现 `wrap_lines(lines, content_height) -> Vec<Vec<(String, Style)>>`，使所有测试通过

## 4. App 状态管理（TDD）

- [x] 4.1 编写 App 测试骨架
- [x] 4.2 测试：`handle_key` — j/k 上下滚动 1 行
- [x] 4.3 测试：`handle_key` — Ctrl+d/u 半页滚动
- [x] 4.4 测试：`handle_key` — Ctrl+f/b 整页滚动
- [x] 4.5 测试：`handle_key` — g/Home 到顶部，G/End 到底部
- [x] 4.6 测试：`handle_key` — PageUp/PageDown 半页滚动
- [x] 4.7 测试：滚动边界（不超过 0，不超过 max_scroll）
- [x] 4.8 测试：`help_visible` 切换（按 `?` 打开，再按任意键关闭）
- [x] 4.9 测试：`set_height` 触发重新换行并 clamp scroll
- [x] 4.10 实现 `App` 结构体和方法，使所有测试通过

## 5. UI 渲染

- [x] 5.1 实现 `draw_markdown` — 渲染 scroll window 内的 styled lines
- [x] 5.2 实现 `draw_status_bar` — 文件名 + 行号/总行数 + 百分比 + 提示
- [x] 5.3 实现 `draw_help` — 居中浮层显示快捷键列表
- [x] 5.4 实现 `centered_rect` — 计算居中浮层位置

## 6. 入口与终端生命周期

- [x] 6.1 实现 CLI 参数解析（文件路径 + 可选行号）
- [x] 6.2 实现终端初始化（enable_raw_mode + alt screen + mouse capture）
- [x] 6.3 实现终端清理（disable_raw_mode + leave alt screen + show cursor），确保 panic 时也能清理
- [x] 6.4 实现主事件循环（crossterm::event::read → App::handle_key → draw）
- [x] 6.5 无参数时输出 usage 信息到 stderr，退出码 1
- [x] 6.6 文件不存在时输出错误到 stderr，退出码 1，不进入 TUI

## 7. 集成验证

- [x] 7.1 `cargo build` 无 warning 编译通过
- [x] 7.2 `cargo test` 所有测试通过
- [x] 7.3 手动测试：`tread sample.md` 正常渲染，滚动正常（需要用户在终端验证）
- [x] 7.4 手动测试：`tread` 无参数输出 usage
- [x] 7.5 手动测试：`tread nonexistent.md` 输出错误
- [x] 7.6 手动测试：终端 resize 后布局正确更新（需要用户在终端验证）
