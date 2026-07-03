## Context

theft-read 已完成 Markdown + EPUB + 图片预览 + 链接交互。现在要添加第三种内容来源：网页。核心挑战是网页 HTML 充满噪音（导航栏、广告、侧边栏），需要用 Readability 算法提取正文。

**已有基础设施**：
- `ureq` HTTP 客户端（图片下载时引入）
- `sha2` 哈希（URL → 缓存文件名）
- `LineContent` 枚举（Styled / Image / Link）
- App 焦点系统、进度保存机制
- 图片缓存 + 系统查看器流程

**约束**：
- 同步架构（不引入异步运行时）
- 网页 HTML 比 XHTML 更不规范，需要宽容的解析器
- 某些网站拒绝无 User-Agent 的请求

## Goals / Non-Goals

**Goals:**
- 通过 `tread <url>` 抓取并阅读网页正文
- Readability 算法提取正文，过滤导航/广告/页脚
- HTTP 请求带 Firefox User-Agent
- 缓存抓取的 HTML，24 小时过期
- `--refresh` 强制刷新跳过缓存
- 保存阅读进度（滚动位置），再次打开时恢复
- 网页图片通过已有缓存/查看器流程处理
- Readability 提取失败时提示"无法提取正文"

**Non-Goals:**
- 动态内容/JavaScript 渲染（需要 headless browser）
- 网页表单交互
- Cookie/登录态管理
- 完整的网页浏览器（地址栏、前进/后退）
- 网站级爬虫

## Decisions

### 1. 内容提取：dom_smoothie

**选择**：`dom_smoothie` crate（Mozilla Readability.js 的 Rust 移植）

**理由**：
- 在 [13 个 Rust 提取库的对比评测](https://emschwartz.me/comparing-13-rust-crates-for-extracting-text-from-html/) 中效果最好
- 纯 Rust 实现，无需外部依赖
- 提取结果包含 title 和 content HTML，方便后续处理
- 基于 `dom_query`（底层 HTML DOM 解析，处理畸形 HTML）

**替代方案**：
- `readability-rust` — 功能有限
- `rs-trafilatura` — 效果好但依赖较重
- 自己写文本密度算法 — 复杂度高，效果不确定

### 2. HTTP 客户端：复用 ureq

**选择**：复用已有的 `ureq`，添加 User-Agent header

```rust
let response = ureq::get(url)
    .set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0")
    .call()?;
```

**理由**：同步 API，与现有架构一致，无需引入新依赖。

### 3. 缓存策略

**选择**：`~/.tread/cache/web/<sha256(url)>.html`

```
cache_key = sha256(url)[..16]
缓存路径: ~/.tread/cache/web/<cache_key>.html
元数据:   ~/.tread/cache/web/<cache_key>.meta.json
```

元数据内容：
```json
{
    "url": "https://example.com/article",
    "title": "Article Title",
    "fetched_at": "2026-06-30T10:00:00Z",
    "content_type": "text/html"
}
```

过期判断：`now - fetched_at > 24h` → 过期

**理由**：与 EPUB 图片缓存一致的模式，简单可靠。

### 4. HTML → LineContent 转换

**选择**：复用 `xhtml_to_lines`，因为 Readability 输出的 HTML 通常比较规范。如果实际测试发现不兼容，再写专门的 `html_to_lines`。

**理由**：减少代码量，Readability 输出已经是清洗过的干净 HTML。

### 5. 进度保存

**选择**：`~/.tread/progress/web_<cache_key>.json`

```json
{
    "url": "https://example.com/article",
    "scroll": 42,
    "saved_at": "2026-06-30T10:00:00Z"
}
```

**理由**：与 EPUB 进度保存一致的模式。网页是单页的，所以只需保存 scroll，不需要 chapter。

### 6. CLI 参数扩展

**选择**：

```
tread <file.md|file.epub|url> [line_number] [-r|--refresh]
```

URL 识别逻辑：参数以 `http://` 或 `https://` 开头 → 网页模式。

`--refresh` / `-r`：跳过缓存，强制重新抓取。

### 7. 图片 URL 解析

**选择**：根据网页的 base URL 将相对路径转为绝对路径

```
base_url:  https://example.com/article/
img src:   /images/pic.jpg
         → https://example.com/images/pic.jpg

img src:   ../assets/pic.jpg
         → https://example.com/assets/pic.jpg

img src:   https://cdn.example.com/pic.jpg
         → 直接使用
```

提取的图片 URL 通过已有的 `download_image` 流程缓存和查看。

## Risks / Trade-offs

- **[Readability 失败]** 某些页面（首页、搜索结果页、SPA）Readability 无法提取 → 显示"无法提取正文"，不崩溃
- **[网站反爬]** 某些网站可能需要更多 header（Accept、Referer） → 先加 User-Agent，不够再加
- **[编码问题]** 某些网页使用非 UTF-8 编码 → `ureq` 的 `into_string()` 会自动处理，极端情况用 `content-type` 的 charset
- **[大图加载]** 网页中可能有大量图片，全部下载会慢 → 按需下载（Tab 聚焦时），不预加载
- **[dom_smoothie 维护]** 相对较新的 crate，社区不大 → 如果出问题，可以回退到自己写简单的正文提取
