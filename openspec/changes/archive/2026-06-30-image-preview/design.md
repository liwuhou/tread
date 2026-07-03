## Context

theft-read 已完成 Markdown 阅读器 MVP（`markdown-reader-mvp` 已归档）。当前 parser 能处理标题、粗体、代码块、列表、引用、表格等元素，但遇到 `Event::Start(Tag::Image)` 时直接忽略。用户需要一个完整的图片查看流程：检测 → 缓存 → 占位 → 打开。

**约束**：
- 不引入异步运行时（保持同步架构，与 MVP 一致）
- 终端内不渲染图片像素（避免 Sixel/Kitty 协议兼容性复杂度）
- 图片缓存需要跨平台（macOS + Linux）
- 保持现有功能不变（无图片文档行为不受影响）

## Goals / Non-Goals

**Goals:**
- 解析 Markdown 中的 `![alt](url)` 图片语法
- 远程图片下载到本地缓存，相同 URL 不重复下载
- 在终端中以可交互占位符形式展示图片位置
- 按 Enter 用系统默认看图工具打开图片原图
- 为后续 EPUB / 网页图片来源预留扩展接口

**Non-Goals:**
- 终端内渲染图片像素（Sixel / Kitty / ASCII art）
- EPUB 内图片提取（后续 change）
- 网页图片下载（后续 change）
- 图片缩放 / 适应终端宽度
- 图片格式转换

## Decisions

### 1. 图片显示策略：占位符 + 外部查看器

**选择**：终端内显示 `[📷 alt text]` 占位符，Enter 键打开系统看图工具。

**理由**：
- 终端内渲染图片需要特定协议支持（Sixel / Kitty / iTerm2），兼容性差
- 外部查看器提供完整分辨率和缩放能力
- 实现简单，无额外终端兼容性问题

**替代方案**：
- Unicode 块字符画（`image` crate + `▀▄█`）→ 质量低，终端占用大
- Sixel 协议 → 仅部分终端支持
- Kitty 图形协议 → 仅 Kitty/WezTerm 支持

### 2. HTTP 下载库：ureq

**选择**：`ureq`（同步 HTTP 客户端）

**理由**：
- 同步 API 与现有架构一致（MVP 无异步运行时）
- 轻量（依赖少，编译快）
- 功能足够（GET 请求 + 写入文件）

**替代方案**：
- `reqwest` + `tokio` → 引入异步运行时，对 MVP 架构改动太大
- `curl` 子进程 → 依赖外部命令，不够优雅

### 3. 缓存策略：URL hash 去重

**选择**：`~/.tread/cache/<sha256(url)>.<ext>`

**理由**：
- SHA-256 哈希碰撞概率极低，可靠去重
- 文件名与 URL 脱钩，避免路径/特殊字符问题
- 扩展名从 URL 或 Content-Type 推断，保持文件可识别

### 4. 图片数据模型：ImageNode

**选择**：在 styled lines 中插入特殊的图片行：

```rust
pub enum LineContent {
    Styled(Vec<(String, Style)>),  // 普通文本行
    Image(ImageNode),               // 图片占位行
}

pub struct ImageNode {
    pub alt: String,
    pub url: String,
    pub local_path: Option<PathBuf>,  // 缓存后/本地的路径
    pub id: usize,                     // 全局唯一 ID，用于焦点导航
}
```

**理由**：
- 将图片与普通文本分离，渲染和交互逻辑更清晰
- `local_path` 在缓存完成前为 `None`，支持异步加载（MVP 中同步完成）
- `id` 支持 Tab 导航时的焦点跳转

### 5. 图片焦点导航

**选择**：Tab / Shift+Tab 在图片占位符之间跳转，Enter 打开当前焦点图片。

**理由**：
- Tab 是标准的 "跳转下一个可交互元素" 快捷键
- 避免与现有 j/k 滚动冲突
- 焦点状态用高亮背景色区分

### 6. 系统打开命令

**选择**：
- macOS: `open <path>`
- Linux: `xdg-open <path>`
- 通过 `std::env::consts::OS` 判断平台

## Risks / Trade-offs

- **[网络失败]** 远程图片下载可能超时或失败 → 占位符显示 `[📷 alt text ⚠ 下载失败]`，不阻塞阅读
- **[缓存膨胀]** 缓存目录可能无限增长 → MVP 不自动清理，后续可加 TTL 或大小限制
- **[图片格式]** 某些格式（SVG、WebP）系统查看器可能不支持 → 交给系统处理，提示用户
- **[并发下载]** MVP 同步下载会阻塞 UI → 后续可改为后台线程 + 进度提示
- **[URL 变化]** 同一内容换了 URL 会重复缓存 → 接受，内容级去重需要下载后计算 hash，复杂度高
