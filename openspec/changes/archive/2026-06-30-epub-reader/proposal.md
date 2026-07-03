## Why

用户希望 theft-read 不仅支持 Markdown，还能阅读 EPUB 格式的电子书。EPUB 是最主流的电子书格式，绝大多数正版/免费电子书都以 EPUB 分发。支持 EPUB 将使 theft-read 成为一个真正的通用终端阅读器。

## What Changes

- **新增 EPUB 解析模块**：读取 EPUB 文件（ZIP 格式），解析 `container.xml` → OPF → spine，按阅读顺序提取 XHTML 内容
- **新增 XHTML → 内容行转换**：将 EPUB 中的 XHTML 内容转换为 `Vec<LineContent>`，复用现有样式体系
- **新增 EPUB 元数据支持**：显示书名、作者、语言等元信息（首屏/状态栏）
- **新增目录（TOC）支持**：解析 NCX/nav 文档，支持目录跳转
- **新增 EPUB 图片支持**：从 EPUB ZIP 中提取图片到缓存，复用现有图片预览流程
- **修改 CLI 入口**：根据文件扩展名（`.epub` / `.md`）自动选择解析器
- **修改 App**：支持多章节切换（上一章/下一章）

## Capabilities

### New Capabilities
- `epub-reader`: EPUB 文件的完整解析、渲染、导航流程（ZIP 解压 → OPF 解析 → XHTML 渲染 → 章节导航 → 目录 → 图片提取）

### Modified Capabilities
- `markdown-rendering`: 新增 XHTML → LineContent 转换能力（或直接复用 Markdown parser 的样式逻辑）

## Impact

- **新增依赖**：`zip`（EPUB ZIP 解压）、`quick-xml`（OPF/NCX XML 解析）
- **新增文件**：`src/epub.rs`（EPUB 解析）、`src/xhtml.rs`（XHTML → LineContent 转换）
- **修改文件**：`src/main.rs`（文件类型分发）、`src/app.rs`（章节导航状态）
- **新增目录**：`~/.tread/cache/epub/`（EPUB 图片缓存子目录）
- **现有功能不受影响**：Markdown 阅读行为完全不变
