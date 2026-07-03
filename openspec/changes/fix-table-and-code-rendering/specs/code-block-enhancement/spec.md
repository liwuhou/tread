## ADDED Requirements

### Requirement: 保留代码块的原始格式
系统 SHALL 在解析 `<pre>` 和 `<code>` 标签时，保留所有空格、换行和缩进。

#### Scenario: 保留代码缩进
- **WHEN** HTML 包含 `<pre><code>function test() {\n  return 42;\n}</code></pre>`
- **THEN** 系统输出的文本保留原始缩进：
  ```
  function test() {
    return 42;
  }
  ```

#### Scenario: 保留空行
- **WHEN** 代码块中包含空行
- **THEN** 系统保留空行，不将其删除

#### Scenario: 保留多个连续空格
- **WHEN** 代码中包含多个连续空格（如对齐的代码）
- **THEN** 系统保留所有空格，不将其合并为单个空格

### Requirement: 应用代码块样式
系统 SHALL 为代码块应用特殊的样式，使其与普通文本区分。

#### Scenario: 代码块背景色
- **WHEN** 渲染代码块内容
- **THEN** 代码文本使用绿色前景色和黑色背景色（已在现有代码中实现）

#### Scenario: 代码块边界
- **WHEN** 代码块开始和结束
- **THEN** 系统在代码块前后添加空行，使其与周围内容分隔

### Requirement: 处理嵌套的 code 标签
系统 SHALL 正确处理 `<pre><code>...</code></pre>` 和单独的 `<code>...</code>` 两种情况。

#### Scenario: pre 内的 code
- **WHEN** HTML 包含 `<pre><code>code here</code></pre>`
- **THEN** 系统将整个内容作为代码块处理，保留格式

#### Scenario: 单独的 code
- **WHEN** HTML 包含 `<code>inline code</code>`（不在 pre 内）
- **THEN** 系统将其作为行内代码处理，应用代码样式但不保留多行格式

### Requirement: 处理特殊字符
系统 SHALL 正确处理代码块中的 HTML 特殊字符实体。

#### Scenario: 转义字符
- **WHEN** 代码块包含 `&lt;`、`&gt;`、`&amp;` 等 HTML 实体
- **THEN** 系统将其转换为对应的字符：`<`、`>`、`&`

#### Scenario: 不解析内部标签
- **WHEN** 代码块内包含 HTML 标签（如 `<span class="keyword">if</span>`）
- **THEN** 系统忽略标签，只提取文本内容（不解析标签的样式）
