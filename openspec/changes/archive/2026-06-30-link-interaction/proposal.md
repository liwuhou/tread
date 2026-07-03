## Why

EPUB 和 Markdown 文件中包含大量超链接（`<a href="url">`），当前 theft-read 仅渲染为蓝色下划线文本，无法交互。用户需要：外链（http/https）用系统浏览器打开，内链（章节跳转）在阅读器内导航。这是阅读器从"只能看"到"可以交互"的关键一步。

## What Changes

- **新增 LinkNode 数据模型**：在 parser/xhtml.rs 中识别 `<a>` 标签，生成 `LineContent::Link` 节点
- **新增统一焦点系统**：将图片和链接合并为"可交互元素"（Focusable），共享 Tab/Shift+Tab 导航
- **新增 Enter 交互**：Enter 键对链接节点执行对应动作（外链 → 系统浏览器，内链 → 章节跳转）
- **新增外链打开能力**：`open_url(url)` 调用系统浏览器（macOS `open`，Linux `xdg-open`）
- **新增内链解析**：解析 href 中的 `#anchor` 部分，匹配 TOC/spine 定位目标章节

## Capabilities

### New Capabilities
- `link-interaction`: 超链接检测、焦点导航、外链打开、内链跳转的完整流程

## Impact

- **修改文件**：`src/image.rs`（新增 LinkNode、Focusable 枚举）、`src/xhtml.rs`（识别 `<a>` 标签）、`src/parser.rs`（Markdown 链接处理）、`src/app.rs`（统一焦点导航）、`src/ui.rs`（链接样式渲染）、`src/main.rs`（内链跳转支持）
- **现有功能不受影响**：图片导航和 Markdown 阅读行为完全不变
- **EPUB 内链跳转**：需要 EPUB 解析器暴露章节锚点映射
