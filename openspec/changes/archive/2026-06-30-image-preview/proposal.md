## Why

Markdown 文档中大量使用图片（`![alt](url)`），但当前 tread MVP 完全忽略了图片语法。用户希望在阅读带图片的文档时，能看到图片占位标记，并能快速用系统自带看图工具查看原图。这一功能同时为后续 EPUB 和网页阅读中的图片处理打下基础。

## What Changes

- **新增图片检测**：parser 识别 `![alt](url)` 并生成图片节点
- **新增图片缓存模块**：远程图片下载到 `~/.tread/cache/<hash>.ext`，本地图片直接引用，相同 URL 不重复下载
- **新增图片占位渲染**：在图片位置显示 `[📷 alt text]` 样式的可交互占位符
- **新增图片交互**：用户可导航到图片占位符，按 Enter 调用系统图片浏览器打开
- **新增系统打开能力**：macOS 使用 `open`，Linux 使用 `xdg-open`
- **修改 markdown-rendering**：parser 需处理 `Event::Start(Tag::Image)` 和 `Event::End(TagEnd::Image)`
- **修改 tui-navigation**：新增 Enter 键绑定，用于打开当前选中的图片

## Capabilities

### New Capabilities
- `image-preview`: 图片检测、缓存、占位渲染、系统浏览器打开的完整流程

### Modified Capabilities
- `markdown-rendering`: parser 需识别图片语法并生成图片节点（而非忽略）
- `tui-navigation`: 新增图片导航（Tab/Shift+Tab 切换图片焦点）和 Enter 键打开图片

## Impact

- **新增依赖**：`ureq`（HTTP 下载）、`dirs`（home 目录）、`sha2`（URL hash）
- **新增文件**：`src/image.rs`（缓存 + 打开逻辑）
- **修改文件**：`src/parser.rs`（图片事件处理）、`src/app.rs`（图片焦点状态）、`src/ui.rs`（图片占位渲染）
- **新增目录**：`~/.tread/cache/`（运行时创建）
- **现有功能不受影响**：无图片的文档行为不变
