## Why

用户希望在终端环境中流畅阅读 Markdown 文档（小说、技术文档等），无需切换到 GUI 阅读器。当前缺少一个轻量、快速、支持 CJK 的终端 Markdown 阅读工具。本项目是 theft-read 终端阅读器的第一步，先实现 Markdown 格式，后续再扩展到 EPUB 和网页内容。

## What Changes

- **新建 Rust 项目** `theft-read`，二进制名称 `tread`
- **命令行入口**：接受 Markdown 文件路径（可选行号）作为参数
- **Markdown 渲染**：将 Markdown 源文件解析为带样式的终端输出，支持标题（H1–H6）、粗体/斜体/删除线、行内代码、围栏代码块（带语言标签）、有序/无序列表、引用块、表格、分隔线、链接
- **终端 TUI**：使用 ratatui 实现全屏渲染，包含内容区和底部状态栏
- **滚动导航**：vim 风格快捷键（j/k、Ctrl+d/u/f/b、g/G、Home/End、PageUp/PageDown）
- **状态栏**：显示文件名、当前行/总行数、滚动百分比
- **帮助浮层**：按 `?` 弹出快捷键参考
- **CJK 支持**：使用 unicode-width 正确处理中文字符宽度，词级换行（英文按空格、中文按字符）
- **终端生命周期**：进入 alt screen + raw mode，退出时完整恢复终端状态

## Capabilities

### New Capabilities
- `markdown-rendering`: 将 Markdown 文件解析并渲染为带 ANSI 样式的终端文本，覆盖所有常见 Markdown 元素及 CJK 文本换行
- `tui-navigation`: 终端全屏 UI，包含内容滚动、状态栏、帮助浮层，以及完整的键盘事件处理

### Modified Capabilities
（无 — 这是全新项目）

## Impact

- **新增依赖**：`ratatui`（TUI 框架）、`pulldown-cmark`（Markdown 解析）、`crossterm`（终端控制）、`unicode-width`（字符宽度）、`anyhow`（错误处理）
- **新增文件**：`Cargo.toml`、`src/main.rs`、`src/app.rs`、`src/parser.rs`、`src/ui.rs`
- **无现有代码受影响**：当前目录为空项目
- **运行时需求**：需要支持 ANSI 转义序列的终端（绝大多数现代终端均满足）
