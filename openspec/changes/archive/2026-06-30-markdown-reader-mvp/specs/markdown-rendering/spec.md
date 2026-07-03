## ADDED Requirements

### Requirement: Parse Markdown source into styled lines
系统 SHALL 使用 `pulldown-cmark` 将 Markdown 源文本解析为 `Vec<Line>`，其中每个 `Line` 是 `Vec<(String, Style)>`（styled spans）。解析过程 MUST 为纯函数，不依赖终端状态。

#### Scenario: 解析包含所有基本元素的 Markdown
- **WHEN** 输入包含标题（H1–H6）、段落、粗体、斜体、删除线、行内代码、围栏代码块、有序列表、无序列表、引用块、表格、分隔线的 Markdown 文本
- **THEN** 系统返回对应的 styled lines，每种元素具有 design.md 中定义的样式

#### Scenario: 空输入
- **WHEN** 输入为空字符串
- **THEN** 系统返回空的 `Vec<Line>`

#### Scenario: 仅空白行
- **WHEN** 输入为多个空行
- **THEN** 系统返回对应数量的空 `Line`

### Requirement: Heading styles
系统 SHALL 为 H1–H6 应用不同颜色和修饰样式。

#### Scenario: H1 渲染
- **WHEN** 解析到 `# Title`
- **THEN** 输出行包含 `"# "` 前缀和标题文本，样式为 Yellow + Bold + Underline

#### Scenario: H2 渲染
- **WHEN** 解析到 `## Subtitle`
- **THEN** 输出行样式为 Cyan + Bold

#### Scenario: H3–H6 渲染
- **WHEN** 解析到 `### ` 到 `###### ` 的标题
- **THEN** 分别应用 Green/Magenta/Blue/White + Bold

### Requirement: Inline text styles
系统 SHALL 正确解析并渲染粗体、斜体、删除线、行内代码和链接。

#### Scenario: 粗体文本
- **WHEN** 解析到 `**bold text**`
- **THEN** 输出 span 包含 "bold text"，样式为 Bold

#### Scenario: 斜体文本
- **WHEN** 解析到 `*italic text*`
- **THEN** 输出 span 包含 "italic text"，样式为 Italic

#### Scenario: 行内代码
- **WHEN** 解析到 `` `code` ``
- **THEN** 输出 span 包含 " code "（前后带空格），样式为 Green fg + Black bg

#### Scenario: 链接
- **WHEN** 解析到 `[text](url)`
- **THEN** 输出 span 包含 "text"，样式为 Blue + Underline

#### Scenario: 删除线
- **WHEN** 解析到 `~~struck~~`
- **THEN** 输出 span 包含 "struck"，样式为 CrossedOut

### Requirement: Code block rendering
系统 SHALL 将围栏代码块渲染为带边框和可选语言标签的区域。

#### Scenario: 带语言的代码块
- **WHEN** 解析到 ````rust\ncode\n````
- **THEN** 输出包含：语言标签行（"── rust"，DarkGray + Italic）、顶部边框线、代码行（Green fg + Black bg）、底部边框线

#### Scenario: 无语言的代码块
- **WHEN** 解析到 ````\ncode\n````
- **THEN** 输出不包含语言标签行，仅包含边框线和代码行

### Requirement: List rendering
系统 SHALL 渲染有序和无序列表，支持缩进。

#### Scenario: 无序列表项
- **WHEN** 解析到 `- item`
- **THEN** 输出行包含缩进 + "• " 标记（Magenta） + 文本内容

#### Scenario: 嵌套列表
- **WHEN** 解析到嵌套列表（两层）
- **THEN** 第二层列表项的缩进比第一层多 2 个空格

### Requirement: Blockquote rendering
系统 SHALL 使用 `▎` 前缀和暗色样式渲染引用块。

#### Scenario: 引用段落
- **WHEN** 解析到 `> quoted text`
- **THEN** 输出行包含 `"  ▎ "` 前缀（DarkGray），后跟引用文本

### Requirement: Table rendering
系统 SHALL 以 `│` 分隔单元格的方式渲染表格。

#### Scenario: 简单表格
- **WHEN** 解析到含 2 列 2 行的表格
- **THEN** 每行以 `│` 分隔单元格内容，表格前后有边框线

### Requirement: Horizontal rule rendering
系统 SHALL 渲染分隔线。

#### Scenario: 分隔线
- **WHEN** 解析到 `---`
- **THEN** 输出行为 `────────` 重复至终端宽度（最大 8 字符重复），样式为 DarkGray

### Requirement: CJK-aware word wrapping
系统 SHALL 在将 styled lines 换行为终端宽度时，对英文按空格分词换行，对 CJK / 全角字符按字符换行。

#### Scenario: 英文文本换行
- **WHEN** 终端宽度为 20，输入行 "hello world foo bar baz"
- **THEN** 换行后第一行为 "hello world foo bar"，第二行为 "baz"（在空格处断行）

#### Scenario: 中文文本换行
- **WHEN** 终端宽度为 10，输入行 "你好世界这是一个测试"（每个中文字符占 2 列宽）
- **THEN** 每行最多 5 个中文字符（10 列宽），在字符边界换行

#### Scenario: 中英混排换行
- **WHEN** 终端宽度为 20，输入行 "hello 你好 world 世界"
- **THEN** 英文在空格处断行，中文在字符间断行，同一行可混合

#### Scenario: 超长单词强制断行
- **WHEN** 终端宽度为 10，输入行 "abcdefghijklmnop"（16 字符，超过行宽）
- **THEN** 系统按字符断行，每行最多填满行宽
