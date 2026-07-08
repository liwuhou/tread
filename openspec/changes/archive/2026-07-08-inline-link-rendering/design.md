## Context

当前链接处理逻辑：
1. 解析器在 `Event::Text` 时把链接文本推入当前行（有样式）
2. 同时在 `Event::End(Link)` 时把链接推入 `pending_links`
3. 段落结束时，`pending_links` 作为独立的 `LineContent::Link` 推出

焦点系统基于行索引：`focusable_positions: Vec<usize>` 存储可聚焦行的索引。

## Goals / Non-Goals

**Goals:**
- 链接内联显示在文本流中，不单独成行
- Tab/Shift+Tab 可以在内联链接间导航
- Enter 可以打开当前聚焦的内联链接
- 链接焦点有明确的视觉反馈

**Non-Goals:**
- 不支持链接内的富文本（链接内只有纯文本）
- 不改变图片的块级渲染方式

## Decisions

### Decision 1: 数据结构扩展

**选择**: 为 styled span 添加可选的链接元数据

```rust
// 扩展 span 类型，支持链接信息
pub struct StyledSpan {
    pub text: String,
    pub style: Style,
    pub link: Option<LinkInfo>,  // 新增：链接元数据
}

pub struct LinkInfo {
    pub url: String,
    pub is_external: bool,
}
```

**理由**: 最小化改动，保持现有结构，向后兼容。

**替代方案**:
- 完全重构为 `SpanContent` 枚举 → 改动过大
- 使用 `(String, Style, Option<LinkInfo>)` 元组 → 可读性差

### Decision 2: 焦点定位方式

**选择**: `focusable_positions` 存储结构化的 `FocusableItem`，内联链接使用范围定位而不是单点定位。

```rust
pub enum FocusableItem {
    Image { line_idx: usize },
    BlockLink { line_idx: usize, url: String, is_external: bool },
    InlineLink {
        line_idx: usize,
        start_offset: usize,
        end_offset: usize,
        url: String,
        is_external: bool,
    },
}

pub focusable_positions: Vec<FocusableItem>;
```

**理由**: wrap 阶段会把 `Learn more` 这类链接文本拆成 `Learn`、空格、`more` 等多个 styled span。焦点系统如果只记录 `(line_idx, char_offset)` 单点，会把这些拆分后的 span 分别当作可聚焦项。范围模型可以把同一行内连续、相同链接元数据的 spans 合并为一个逻辑链接焦点，从而 Tab 一次高亮整个 `Learn more`。

### Decision 3: 链接标记

**选择**: 链接文本前加 `🔗` 前缀，内联显示

```
This is a 🔗click here about...
```

**理由**: 视觉上明确标识链接，与 emoji 风格一致。

## Risks / Trade-offs

**[Risk] 焦点导航复杂度增加** → 需要计算字符位置，但性能影响可忽略。

**[Risk] 换行时链接分割** → 链接跨行时可能需要特殊处理，MVP 阶段允许分割。

**[Trade-off] 元组 vs 结构体** → 选择结构体 `StyledSpan` 以提高可读性，但需要修改现有代码。
