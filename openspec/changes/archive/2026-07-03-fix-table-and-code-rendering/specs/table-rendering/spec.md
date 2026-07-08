## ADDED Requirements

### Requirement: 解析 HTML 表格结构
系统 SHALL 识别并解析 HTML 表格标签（table、thead、tbody、tr、th、td），提取表格数据。

#### Scenario: 解析简单表格
- **WHEN** HTML 包含 `<table><tr><td>Cell 1</td><td>Cell 2</td></tr></table>`
- **THEN** 系统提取出 1 行 2 列的表格数据：[["Cell 1", "Cell 2"]]

#### Scenario: 解析带表头的表格
- **WHEN** HTML 包含 `<table><thead><tr><th>Header 1</th><th>Header 2</th></tr></thead><tbody><tr><td>Data 1</td><td>Data 2</td></tr></tbody></table>`
- **THEN** 系统提取出 2 行 2 列的表格数据，第一行为表头：[["Header 1", "Header 2"], ["Data 1", "Data 2"]]

#### Scenario: 忽略嵌套标签
- **WHEN** 表格单元格包含嵌套标签（如 `<td><strong>Bold</strong> text</td>`）
- **THEN** 系统提取单元格的文本内容，忽略嵌套标签："Bold text"

### Requirement: 渲染表格到终端
系统 SHALL 将解析的表格数据渲染为终端友好的文本格式，使用 Unicode 边框字符。

#### Scenario: 渲染单行表格
- **WHEN** 表格数据为 [["A", "B"]]
- **THEN** 系统渲染为：
  ```
  ┌───┬───┐
  │ A │ B │
  └───┴───┘
  ```

#### Scenario: 渲染多行表格带表头
- **WHEN** 表格数据为 [["H1", "H2"], ["D1", "D2"]]
- **THEN** 系统渲染为：
  ```
  ┌────┬────┐
  │ H1 │ H2 │
  ├────┼────┤
  │ D1 │ D2 │
  └────┴────┘
  ```

#### Scenario: 自动计算列宽
- **WHEN** 表格内容为 [["Short", "Very Long Content"], ["A", "B"]]
- **THEN** 系统计算每列的最大宽度，第一列宽度为 5，第二列宽度为 15

### Requirement: 处理宽表格
系统 SHALL 在表格宽度超过终端宽度时，按比例压缩列宽或截断内容。

#### Scenario: 压缩超宽表格
- **WHEN** 表格总宽度超过 80 列（假设终端宽度）
- **THEN** 系统按比例压缩各列宽度，确保总宽度不超过 80 列

#### Scenario: 截断过长内容
- **WHEN** 单元格内容超过列宽限制
- **THEN** 系统截断内容并添加 "..." 后缀

### Requirement: 保持表格样式
系统 SHALL 为表格应用适当的样式，使其在终端中清晰可见。

#### Scenario: 表头样式
- **WHEN** 表格有表头行（来自 thead 或 th 标签）
- **THEN** 表头文本使用粗体样式

#### Scenario: 表格边框
- **WHEN** 渲染表格边框
- **THEN** 使用 Unicode 边框字符（┌─┬┐│├┼┤└─┴┘），颜色为默认终端颜色
