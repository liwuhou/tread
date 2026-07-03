## 1. 项目依赖与数据模型

- [x] 1.1 在 `Cargo.toml` 中添加依赖：`zip`（EPUB 解压）、`quick-xml`（XML 解析）、`serde` + `serde_json`（进度持久化）
- [x] 1.2 创建 `src/epub.rs` 模块，定义 `EpubBook` 结构体（metadata, spine, manifest, resources）
- [x] 1.3 定义 `EpubMetadata`（title, author, language）、`SpineItem`（href, id）、`TocEntry`（title, href, level）

## 2. EPUB 结构解析（TDD）

- [x] 2.1 编写测试：解析 `container.xml` 获取 OPF 路径
- [x] 2.2 编写测试：解析 OPF manifest（提取 id, href, media-type）
- [x] 2.3 编写测试：解析 OPF spine（按顺序提取 itemref）
- [x] 2.4 编写测试：解析 OPF metadata（title, creator, language）
- [x] 2.5 编写测试：解析 NCX 目录（navPoint → title + content src）
- [x] 2.6 实现 `EpubBook::open(path)` 方法，使所有测试通过

## 3. XHTML → LineContent 转换（TDD）

- [x] 3.1 创建 `src/xhtml.rs` 模块
- [x] 3.2 编写测试：`<p>` 段落 → Styled 行
- [x] 3.3 编写测试：`<h1>`–`<h6>` → 带颜色的 Styled 行
- [x] 3.4 编写测试：`<strong>`/`<em>` → Bold/Italic 样式
- [x] 3.5 编写测试：`<ul>`/`<ol>`/`<li>` → 列表格式
- [x] 3.6 编写测试：`<pre><code>` → 代码块格式
- [x] 3.7 编写测试：`<a href>` → 链接样式
- [x] 3.8 编写测试：`<img src alt>` → ImageNode
- [x] 3.9 编写测试：未知标签降级（保留文本）
- [x] 3.10 实现 `xhtml_to_lines(html, base_path)` 函数

## 4. EPUB 图片提取

- [x] 4.1 编写测试：从 ZIP 提取图片到 `~/.tread/cache/epub/<hash>/`
- [x] 4.2 编写测试：相对路径解析（根据 OPF 位置）
- [x] 4.3 编写测试：图片不存在时标记 download_failed
- [x] 4.4 实现图片提取逻辑，复用现有 ImageNode 和缓存体系

## 5. 章节导航（TDD）

- [x] 5.1 App 添加 `chapter_index`, `chapters: Vec<Vec<LineContent>>` 字段
- [x] 5.2 编写测试：`Ctrl+n` 切换到下一章
- [x] 5.3 编写测试：`Ctrl+p` 切换到上一章
- [x] 5.4 编写测试：边界保护（第一章/最后一章）
- [x] 5.5 实现章节切换逻辑

## 6. 目录（TOC）浮层

- [x] 6.1 编写测试：按 `t` 打开目录浮层
- [x] 6.2 编写测试：目录项选择后跳转到对应章节
- [x] 6.3 编写测试：无目录时按 `t` 显示提示
- [x] 6.4 实现目录浮层 UI（类似帮助浮层，可上下选择）

## 7. 进度持久化

- [x] 7.1 编写测试：保存进度到 `~/.tread/progress/<hash>.json`
- [x] 7.2 编写测试：打开 EPUB 时恢复进度
- [x] 7.3 实现 `save_progress()` 和 `load_progress()` 函数

## 8. CLI 集成与状态栏

- [x] 8.1 `main.rs` 根据文件扩展名分发解析器（`.epub` → EPUB, 其他 → Markdown）
- [x] 8.2 状态栏显示 EPUB 信息（书名、章节号/总章节数）
- [x] 8.3 帮助浮层添加 EPUB 专属快捷键（Ctrl+n/p, t）

## 9. 测试文档与集成验证

- [x] 9.1 创建测试用 EPUB 文件（包含多章节、图片、目录）
- [x] 9.2 `cargo build` 无 warning 编译通过
- [x] 9.3 `cargo test` 所有测试通过
- [x] 9.4 手动测试：打开 EPUB，章节导航、目录跳转、图片查看、进度恢复
