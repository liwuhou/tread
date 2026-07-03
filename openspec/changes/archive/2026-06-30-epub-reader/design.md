## Context

theft-read 已完成 Markdown 阅读器 MVP 和图片预览功能。现在要添加 EPUB 格式支持。EPUB 本质是一个 ZIP 包，内含：
- `META-INF/container.xml` → 指向 OPF 文件
- `*.opf`（Package Document）→ manifest（资源清单）、spine（阅读顺序）、metadata（元数据）
- XHTML 文件 → 正文内容
- CSS 文件 → 样式（终端中忽略）
- 图片文件 → 封面、插图等

**约束**：
- EPUB 3 基于 XHTML 5，但需兼容 EPUB 2（HTML 4 / XHTML 1.1）
- 不渲染 CSS（终端无法支持）
- 图片使用已有的缓存 + 外部查看器流程
- 不依赖 `epub-rs`（功能有限且维护不活跃），自行解析

## Goals / Non-Goals

**Goals:**
- 打开 `.epub` 文件并按 spine 顺序渲染正文内容
- 显示书名、作者等元信息
- 支持目录（TOC）浏览和跳转
- 支持章节间导航（上一章/下一章）
- EPUB 中的图片可通过已有的 Tab/Enter 流程查看
- 进度记忆（记住上次读到的位置）

**Non-Goals:**
- CSS 样式渲染
- JavaScript 执行
- EPUB 内嵌字体/音频/视频
- EPUB 编辑或导出
- DRM 破解

## Decisions

### 1. EPUB 解析策略：手动解析 ZIP + XML

**选择**：使用 `zip` crate 解压，`quick-xml` 解析 OPF/NCX/XHTML。

**理由**：
- `epub-rs` 依赖老版本 `xml-rs`，API 受限
- 手动解析更灵活，且逻辑清晰（container → OPF → spine → XHTML）
- `quick-xml` 是 Rust 生态最快的 XML 解析器，适合大量 XHTML 解析

**替代方案**：
- `epub-rs` — API 简单但功能有限，不活跃维护
- `libxml2` 绑定 — 太重，引入 C 依赖

### 2. XHTML → LineContent 转换

**选择**：编写 `xhtml_to_lines(html: &str) -> Vec<LineContent>` 函数，将 XHTML DOM 转换为我们的 `LineContent` 序列。

**理由**：
- 复用现有的样式体系（`LineContent::Styled` 和 `LineContent::Image`）
- 不需要经过 Markdown 中间格式
- 可以精确处理 EPUB 特有的元素（`<ruby>`、`<aside>`、`<figure>` 等）

**替代方案**：
- XHTML → Markdown → parse_markdown → 多一次转换，且丢失信息
- 直接用 `pulldown-cmark` 处理 HTML — 它是 Markdown 解析器，不处理 HTML

### 3. 章节管理：Spine 级别的章节切换

**选择**：每个 spine itemref 对应一个章节。App 维护当前章节索引，支持 `Ctrl+n`（下一章）/ `Ctrl+p`（上一章）切换。

**理由**：
- spine 是 EPUB 的自然章节划分
- 避免一次性加载所有章节内容到内存（大文件友好）
- 用户习惯的"章节"概念与 spine itemref 对齐

### 4. 目录（TOC）来源

**选择**：优先使用 NCX（EPUB 2/3 均支持），fallback 到 EPUB 3 的 nav document。

**理由**：
- NCX 是最通用的目录格式，EPUB 2 和 3 都支持
- EPUB 3 的 nav document 是 XHTML，需要额外解析
- 大多数 EPUB 同时包含两者

### 5. 图片处理

**选择**：从 EPUB ZIP 中提取图片到 `~/.tread/cache/epub/<book_hash>/<href>`，创建 `ImageNode` 时直接指向提取后的路径。

**理由**：
- 复用已有的图片预览流程（Tab/Enter/系统查看器）
- 提取到固定路径避免重复解压
- 按书名 hash 分目录，多本书的图片不会冲突

### 6. 进度持久化

**选择**：`~/.tread/progress/<book_hash>.json` 存储 `{chapter_index, scroll_position}`。

**理由**：
- JSON 格式简单可读
- 按书名 hash 索引，不污染用户目录
- 打开 EPUB 时自动恢复进度

## Risks / Trade-offs

- **[XHTML 复杂度]** EPUB 中的 XHTML 可能包含复杂嵌套、内联样式、SVG → 只处理常见元素（p/h1-h6/em/strong/a/img/ul/ol/blockquote/pre/code/table），其他元素忽略或降级为纯文本
- **[大文件性能]** 大 EPUB（如全集小说）可能包含几十个 XHTML 文件 → 按需加载当前章节，不预解析全部
- **[编码问题]** 少数 EPUB 使用非 UTF-8 编码 → zip crate 返回 bytes，需检测/转换编码
- **[EPUB 2 兼容性]** 老版 EPUB 可能缺少某些必需文件 → 优雅降级，缺少 TOC 时不显示目录
- **[图片格式]** EPUB 中可能包含 SVG 图片 → 系统查看器可能不支持 SVG，接受限制
