## Why

Calibre 等工具转换的 EPUB 文件，其 XHTML 结构与自然段落不一致：用空 `<p>` 做间距、用多个 `<p>` 拆分同一句话。当前 parser 将每个 `<p>` 视为独立段落并插入空行，导致：连续空行过多（章节标题前后 10+ 空行）、一句话被切断到两行显示。需要智能合并假分段，并缓存解析结果避免重复计算。

## What Changes

- **新增段落合并器**：在 XHTML → LineContent 转换后，分析相邻段落的关系，合并"假分段"（句子延续）和多余空行
- **新增章节解析缓存**：将合并后的 `Vec<LineContent>` 序列化到 `~/.tread/cache/epub/<hash>/chapter_N.json`，下次打开直接读取缓存
- **修改 EPUB 章节加载**：优先读缓存，缓存未命中时走 XHTML 解析 + 合并 + 写缓存流程

## Capabilities

### Modified Capabilities
- `epub-reader`: 新增段落合并和解析缓存，改善 EPUB 阅读体验

## Impact

- **修改文件**：`src/xhtml.rs`（段落合并逻辑）、`src/epub.rs`（缓存读写）、`src/image.rs`（Style 序列化辅助）
- **新增目录**：`~/.tread/cache/epub/<hash>/chapter_N.json`（缓存文件）
- **现有功能不受影响**：Markdown 和网页阅读完全不变，EPUB 首次打开稍慢（解析+合并+缓存），后续打开秒开
- **缓存失效**：EPUB 文件路径变化 → hash 不同 → 自动重新解析
