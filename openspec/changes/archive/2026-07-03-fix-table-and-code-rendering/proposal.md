## Why

当使用 tread 的 `-i` 模式读取网页内容时，表格（table、tr、td、th）和代码块（pre、code）的格式没有被正确保留和渲染。表格数据变得混乱，代码块失去原有的缩进和格式，严重影响技术文档和数据的可读性。

这是一个常见的网页内容类型，特别是对于技术博客、文档网站和学习平台（如 sitor.cc），表格和代码块是核心内容组成部分。

## What Changes

- **表格渲染**：在 `src/xhtml.rs` 中添加对 HTML 表格标签（table、thead、tbody、tr、th、td）的解析和渲染逻辑，将表格转换为终端友好的文本格式
- **代码块渲染**：优化 pre/code 标签的处理逻辑，确保代码块的缩进、换行和语法高亮标记被正确保留
- **样式支持**：为表格和代码块添加适当的样式（如代码块使用等宽字体、表格使用边框字符）

## Capabilities

### New Capabilities
- `table-rendering`: HTML 表格到终端文本的转换，支持基本的表格结构和简单的样式
- `code-block-enhancement`: 改进代码块的解析和渲染，保留缩进、换行和语法结构

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- **代码**：主要修改 `src/xhtml.rs` 中的 HTML 解析逻辑
- **依赖**：无需新增依赖
- **用户体验**：显著提升技术文档和数据的可读性
- **兼容性**：不影响现有功能，仅增强特定 HTML 元素的渲染
