## Context

当前 EPUB 阅读体验差，根因是 calibre 等工具生成的 XHTML 结构与自然段落不一致。我们的 parser 对每个 `<p>` 都插入空行，导致：

1. **过多空行**：calibre 用 `<p class="block_2"> </p>` 做间距，一章可能有几十个空段落
2. **句子断裂**：一个完整句子被拆到多个 `<p>` 中（不同 CSS class 控制样式），parser 在 `</p>` 处强制分段

**约束**：
- 合并逻辑要保守（宁可保留多余空行，也不要合并真正的段落）
- 缓存要可靠（EPUB 文件不变则缓存永不过期）
- Style 序列化需要覆盖 ratatui 的所有颜色和修饰符

## Goals / Non-Goals

**Goals:**
- 合并连续空段落为单个空行
- 检测"假分段"并合并（同一句话被拆到多个 `<p>`）
- 缓存合并后的 `Vec<LineContent>` 到磁盘
- 首次打开章节：解析 XHTML → 合并 → 缓存 → 渲染
- 再次打开章节：读缓存 → 渲染（跳过解析）

**Non-Goals:**
- 解析 CSS 来判断段落样式（工程量大，MVP 不做）
- 支持所有 EPUB 生成工具（先针对 calibre 优化）
- 缓存跨设备同步
- 缓存压缩

## Decisions

### 1. 段落合并策略：基于标点符号的保守检测

**选择**：通过分析相邻段落的标点符号判断是否应合并。

合并规则：
```
合并（不加空行）:
  - 前一段以 ，、：""） 等"延续"标点结尾
  - 或前一段没有任何结尾标点（被硬切）
  - 且后一段不以"新段落"特征开头（大写/第X章/「/引号）

保留分段（加空行）:
  - 前一段以 。！？.!?… 结尾
  - 或后一段以"新段落"特征开头
```

**理由**：标点符号是最可靠的信号，不需要解析 CSS 或理解语义。

### 2. 空段落处理

**选择**：内容为空或只有空白的 `<p>` 视为 spacer，多个连续 spacer 合并为一个空行。

**实现**：
```
遍历 LineContent:
  - Styled(spans) 且 spans 全为空 → spacer
  - 连续多个 spacer → 只保留一个空行
  - 空行前后都是文字段落 → 保留一个空行作为段落分隔
```

### 3. 缓存格式

**选择**：JSON 文件，每个章节一个文件。

```json
{
    "chapter_index": 2,
    "line_count": 150,
    "lines": [
        {"type": "styled", "spans": [{"text": "...", "fg": "cyan", "bold": true}]},
        {"type": "empty"},
        {"type": "image", "alt": "...", "url": "...", "local_path": "..."},
        {"type": "link", "text": "...", "url": "...", "is_external": true}
    ]
}
```

**Style 序列化**：
- `fg`/`bg`：颜色名（"red", "cyan", "dark_gray"）或 RGB（"#ff0000"）
- `bold`/`italic`/`underline`/`strikethrough`：布尔值

### 4. 缓存路径

**选择**：`~/.tread/cache/epub/<book_hash>/chapter_<N>.json`

- 按 EPUB 文件路径的 SHA-256 hash 分目录（与图片缓存一致）
- 按章节索引命名
- EPUB 文件不变 → hash 不变 → 缓存永不过期

### 5. 集成点

**选择**：在 `epub.rs` 的 `read_chapter` 之后、`xhtml_to_lines` 之前检查缓存。

```
read_chapter_html(idx)
  → 检查缓存 chapter_<idx>.json
  → 命中 → 反序列化 → 返回 Vec<LineContent>
  → 未命中 → xhtml_to_lines(html) → merge_paragraphs(lines) → 写缓存 → 返回
```

## Risks / Trade-offs

- **[合并误判]** 可能把真正的段落合并了 → 规则保守（只在标点明确延续时才合并）
- **[缓存膨胀]** 大 EPUB 可能有几百个章节 → 每个章节几 KB，总量可控
- **[Style 丢失]** 序列化/反序列化可能丢失复杂的 Style 组合 → 只序列化常用属性，默认值不存储
- **[首次打开慢]** 第一次打开每个章节需要解析 + 合并 → 可接受，只发生一次
