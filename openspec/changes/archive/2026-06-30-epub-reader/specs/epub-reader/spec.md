## ADDED Requirements

### Requirement: EPUB file detection and loading
系统 SHALL 根据文件扩展名 `.epub` 自动识别 EPUB 格式并加载。

#### Scenario: 打开 EPUB 文件
- **WHEN** 用户运行 `tread book.epub`
- **THEN** 系统解析 EPUB 结构并显示第一章节内容

#### Scenario: 非 EPUB 文件回退
- **WHEN** 用户运行 `tread file.md`
- **THEN** 系统使用 Markdown 解析器（现有行为不变）

#### Scenario: 损坏的 EPUB 文件
- **WHEN** EPUB 文件缺少 `META-INF/container.xml`
- **THEN** 系统输出错误信息到 stderr，退出码 1

### Requirement: EPUB structure parsing
系统 SHALL 按 EPUB 规范解析 ZIP 内的结构：`container.xml` → OPF → spine。

#### Scenario: 解析 container.xml
- **WHEN** 打开有效 EPUB 文件
- **THEN** 系统从 `META-INF/container.xml` 中找到 OPF 文件路径

#### Scenario: 解析 OPF manifest 和 spine
- **WHEN** 解析 OPF 文件
- **THEN** 系统提取 manifest（资源清单）和 spine（阅读顺序），spine 中的 itemref 指向 manifest 中的 XHTML 文件

#### Scenario: 解析 metadata
- **WHEN** 解析 OPF 的 `<metadata>` 元素
- **THEN** 系统提取书名（`dc:title`）、作者（`dc:creator`）、语言（`dc:language`）

### Requirement: XHTML content rendering
系统 SHALL 将 EPUB 中的 XHTML 内容转换为 `Vec<LineContent>` 进行终端渲染。

#### Scenario: 段落渲染
- **WHEN** XHTML 包含 `<p>文本内容</p>`
- **THEN** 输出为 `LineContent::Styled` 行

#### Scenario: 标题渲染
- **WHEN** XHTML 包含 `<h1>` 到 `<h6>` 标签
- **THEN** 输出带有与 Markdown 标题相同颜色和样式的 `LineContent::Styled` 行

#### Scenario: 粗体和斜体
- **WHEN** XHTML 包含 `<strong>` 和 `<em>` 标签
- **THEN** 输出保留 Bold 和 Italic 样式

#### Scenario: 列表渲染
- **WHEN** XHTML 包含 `<ul>`/`<ol>` 和 `<li>` 标签
- **THEN** 输出与 Markdown 列表相同格式的缩进 + 标记

#### Scenario: 代码块渲染
- **WHEN** XHTML 包含 `<pre><code>` 标签
- **THEN** 输出带有边框和代码样式的行

#### Scenario: 链接渲染
- **WHEN** XHTML 包含 `<a href="url">文本</a>`
- **THEN** 输出带 Blue + Underline 样式的链接文本

#### Scenario: 图片渲染
- **WHEN** XHTML 包含 `<img src="path" alt="描述">`
- **THEN** 输出 `LineContent::Image` 节点，`url` 指向 EPUB 内提取后的本地路径

#### Scenario: 未知元素降级
- **WHEN** XHTML 包含不认识的标签（如 `<ruby>`, `<aside>`）
- **THEN** 忽略标签但保留内部文本内容

### Requirement: Chapter navigation
系统 SHALL 支持在 EPUB 章节之间导航。

#### Scenario: 下一章
- **WHEN** 用户在 EPUB 中按 `Ctrl+n`
- **THEN** 切换到 spine 中的下一个章节，scroll 重置到 0

#### Scenario: 上一章
- **WHEN** 用户在 EPUB 中按 `Ctrl+p`
- **THEN** 切换到 spine 中的上一个章节，scroll 重置到 0

#### Scenario: 第一章时按上一章
- **WHEN** 用户在第一章按 `Ctrl+p`
- **THEN** 保持在第一章，状态栏显示提示

#### Scenario: 最后一章时按下一章
- **WHEN** 用户在最后一章按 `Ctrl+n`
- **THEN** 保持在最后一章，状态栏显示提示

### Requirement: Table of Contents
系统 SHALL 解析并显示 EPUB 目录。

#### Scenario: 显示目录
- **WHEN** 用户在 EPUB 中按 `t`
- **THEN** 弹出目录浮层，列出所有章节标题

#### Scenario: 跳转到目录项
- **WHEN** 目录浮层可见，用户选择某项并按 Enter
- **THEN** 跳转到对应的章节和位置

#### Scenario: 无目录的 EPUB
- **WHEN** EPUB 不包含 NCX 或 nav document
- **THEN** 按 `t` 时状态栏显示"本书无目录"

### Requirement: EPUB image extraction
系统 SHALL 从 EPUB ZIP 中提取图片到本地缓存。

#### Scenario: 提取图片到缓存
- **WHEN** EPUB 中包含 `<img src="images/cover.jpg">`
- **THEN** 系统将 `images/cover.jpg` 从 ZIP 中提取到 `~/.tread/cache/epub/<book_hash>/images/cover.jpg`

#### Scenario: 图片路径解析
- **WHEN** img src 为相对路径
- **THEN** 系统根据 OPF 文件所在目录解析为 ZIP 内的绝对路径

#### Scenario: 图片不存在
- **WHEN** img src 指向的文件在 ZIP 中不存在
- **THEN** ImageNode 的 `local_path` 为 None，`download_failed` 为 true

### Requirement: Reading progress persistence
系统 SHALL 记住每本 EPUB 的上次阅读位置。

#### Scenario: 保存进度
- **WHEN** 用户在 EPUB 中阅读到第三章第 50 行，按 `q` 退出
- **THEN** 系统将 `{chapter: 2, scroll: 50}` 保存到 `~/.tread/progress/<book_hash>.json`

#### Scenario: 恢复进度
- **WHEN** 用户再次打开同一本 EPUB
- **THEN** 系统自动跳转到第三章第 50 行

#### Scenario: 进度文件不存在
- **WHEN** 首次打开一本 EPUB
- **THEN** 从第一章第一行开始阅读

### Requirement: Status bar for EPUB
系统 SHALL 在状态栏显示 EPUB 特有信息。

#### Scenario: EPUB 状态栏
- **WHEN** 阅读 EPUB 第三章（共 20 章）
- **THEN** 状态栏显示 `书名 | 第3章/共20章 | 行号/总行数 | 百分比`
