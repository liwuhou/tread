## 1. Style 序列化（TDD）

- [x] 1.1 创建 `src/style_serde.rs` 模块（或在 `src/image.rs` 中添加）
- [x] 1.2 编写测试：Style → JSON 序列化（fg/bg/bold/italic/underline/strikethrough）
- [x] 1.3 编写测试：JSON → Style 反序列化
- [x] 1.4 编写测试：默认 Style 序列化为空对象
- [x] 1.5 编写测试：RGB 颜色序列化/反序列化
- [x] 1.6 实现 Style 序列化/反序列化

## 2. LineContent 序列化（TDD）

- [x] 2.1 定义 `CachedLine` 枚举（Styled/Empty/Image/Link），可 serde 序列化
- [x] 2.2 编写测试：LineContent → CachedLine → JSON
- [x] 2.3 编写测试：JSON → CachedLine → LineContent
- [x] 2.4 编写测试：ImageNode 序列化（alt, url, local_path）
- [x] 2.5 编写测试：LinkNode 序列化（text, url, is_external）

## 3. 段落合并逻辑（TDD）

- [x] 3.1 编写测试：连续空段落合并为单个空行
- [x] 3.2 编写测试：逗号结尾的段落 + 后续段落 → 合并
- [x] 3.3 编写测试：句号结尾的段落 + 后续段落 → 保留空行
- [x] 3.4 编写测试：无结尾标点的段落 + 后续段落 → 合并
- [x] 3.5 编写测试：后续段落以"第"开头 → 保留空行（新章节）
- [x] 3.6 编写测试：完整 calibre 风格 HTML → 合并后输出自然
- [x] 3.7 实现 `merge_paragraphs(lines: Vec<LineContent>) -> Vec<LineContent>`

## 4. 章节缓存读写（TDD）

- [x] 4.1 编写测试：`save_chapter_cache(book_hash, idx, lines)` 写入 JSON
- [x] 4.2 编写测试：`load_chapter_cache(book_hash, idx)` 读取并反序列化
- [x] 4.3 编写测试：缓存不存在时返回 None
- [x] 4.4 实现缓存读写函数

## 5. EPUB 章节加载集成

- [x] 5.1 修改 `epub.rs`：加载章节时先检查缓存
- [x] 5.2 缓存未命中 → XHTML 解析 → 段落合并 → 写缓存 → 返回
- [x] 5.3 缓存命中 → 直接反序列化 → 返回（跳过解析）

## 6. 集成验证

- [x] 6.1 `cargo build` 无 warning
- [x] 6.2 `cargo test` 全部通过
- [x] 6.3 手动测试：用 test.epub 验证段落合并效果
- [x] 6.4 手动测试：首次打开慢，再次打开秒开（缓存生效）
