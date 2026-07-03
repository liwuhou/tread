## Context

theft-read 已完成 Markdown + EPUB + 图片预览。超链接 `![alt](url)` / `<a href="url">` 当前仅渲染样式，无法交互。用户需要：
- 外链（http/https）→ 系统浏览器打开
- 内链（`chapter.xhtml#section`）→ EPUB 内章节跳转

**约束**：
- Tab 导航已有图片焦点，需要与链接焦点合并
- Enter 键已有图片打开功能，需要与链接动作合并
- 保持向后兼容（无链接文档行为不变）

## Goals / Non-Goals

**Goals:**
- 解析 Markdown 和 XHTML 中的超链接
- 统一 Tab/Shift+Tab 在所有可交互元素（图片 + 链接）之间导航
- Enter 键对外链调用系统浏览器
- Enter 键对内链执行章节跳转（EPUB 模式）
- 焦点高亮样式区分图片和链接

**Non-Goals:**
- 终端内渲染网页（后续 change）
- 链接预览/tooltip
- 书签式链接历史
- 表单交互

## Decisions

### 1. 统一焦点系统：Focusable 枚举

**选择**：将 `image_focus` 扩展为通用的 `focusable_positions: Vec<usize>`，每个位置可以是图片或链接。

```rust
pub enum Focusable<'a> {
    Image(&'a ImageNode),
    Link(&'a LinkNode),
}
```

App 统一维护焦点索引 `focus_index: Option<usize>`。

**理由**：
- Tab 用户不关心当前聚焦的是图片还是链接
- 避免维护两套独立的焦点状态
- Enter 动作根据类型分发，逻辑集中

### 2. LinkNode 数据模型

**选择**：

```rust
pub struct LinkNode {
    pub text: String,      // 链接文本
    pub url: String,       // href 值
    pub is_external: bool, // http/https = true
}
```

`LineContent::Link(LinkNode)` 作为独立变体。

**理由**：
- 与 Image 平行，结构清晰
- `is_external` 预计算避免重复判断

### 3. 外链打开

**选择**：复用 macOS `open` / Linux `xdg-open`，与图片查看器同一函数（加 URL 支持）。

```rust
pub fn open_url(url: &str) -> Result<(), String> {
    // macOS: open url
    // Linux: xdg-open url
}
```

### 4. 内链跳转（EPUB）

**选择**：解析 href 的 `#anchor` 部分，在 TOC 条目中匹配或线性搜索 spine 中各章节的锚点。

**简化 MVP**：
1. 如果 href 匹配 TOC 条目 href → 跳转到该章节
2. 否则 → 线性搜索 spine 各章节 HTML 中是否有匹配的 id/name → 跳转
3. 找不到 → 状态栏提示"未找到目标"

### 5. 焦点样式

| 类型 | 普通 | 焦点 |
|------|------|------|
| 图片 | Cyan fg | DarkGray bg + Bold |
| 链接 | Blue fg + Underline | DarkGray bg + Bold + Underline |

## Risks / Trade-offs

- **[内链匹配精度]** 简单 href 字符串匹配可能不精确（fragment vs id vs name）→ MVP 接受，后续可加完整解析
- **[Tab 循环顺序]** 图片和链接混合时 Tab 顺序按文档位置（从上到下），符合用户直觉
- **[性能]** 内链搜索需要加载所有章节 → 只在跳转时搜索，不预加载
