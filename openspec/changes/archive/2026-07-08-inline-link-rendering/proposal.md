## Why

当前链接渲染存在体验问题：链接文本已经在段落中显示（带蓝色下划线样式），但又作为独立行重复出现（`🔗link`），导致视觉冗余和阅读中断。

**当前效果**：
```
This is a __link__ about we talk.
🔗link
```

**期望效果**：
```
This is a 🔗link about we talk.
```

## What Changes

- 修改解析器：链接不再作为独立行推出，而是内联在文本流中
- 修改数据结构：`LineContent::Styled` 的 span 需要携带链接元数据
- 修改焦点系统：支持行内元素的焦点定位（基于字符位置而非行位置）
- 修改渲染器：正确处理内联链接的渲染和焦点样式

## Capabilities

### New Capabilities

（无新增 capability）

### Modified Capabilities

- `link-interaction`: 链接渲染从块级改为内联，焦点管理基于字符位置
- `markdown-rendering`: 链接作为 span 嵌入文本行，不再独立成行

## Impact

- **数据结构**: `src/image.rs` - `LineContent` 和 span 结构需要携带链接信息
- **解析器**: `src/parser.rs` - 链接不再推入 `pending_links`，而是嵌入 styled spans
- **焦点系统**: `src/app.rs` - `focusable_positions` 需要支持 (行, 字符位置) 定位
- **渲染器**: `src/ui.rs` - 处理内联链接的渲染和焦点样式
- **测试**: 需要更新现有链接相关测试
