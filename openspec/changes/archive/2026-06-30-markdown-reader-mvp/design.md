## Context

tread 是一个全新的 Rust 终端阅读器项目，当前仓库为空（仅含 `.claude` 配置和 OpenSpec 脚手架）。MVP 目标：实现一个可以在终端中流畅阅读 Markdown 文件的 CLI 工具。

**约束**：
- 使用 Rust 语言
- 纯终端 UI（无 GUI 依赖）
- 需正确处理 CJK 字符宽度
- 目标平台：macOS / Linux 现代终端

## Goals / Non-Goals

**Goals:**
- 将任意 Markdown 文件渲染为带样式的终端输出
- 提供流畅的 vim 风格滚动导航
- 正确处理中英文混排的换行
- 完整的终端生命周期管理（进入/退出无残留）

**Non-Goals:**
- EPUB 解析（后续迭代）
- 网页内容抓取（后续迭代）
- 搜索功能（后续迭代）
- 阅读进度持久化（后续迭代）
- 鼠标滚动支持（MVP 不需要）

## Decisions

### 1. TUI 框架：ratatui + crossterm

**选择**：ratatui 0.29 + crossterm 0.28

**理由**：ratatui 是 Rust 生态最成熟的 TUI 框架，基于 crossterm（跨平台终端控制），社区活跃、文档齐全。

**替代方案**：
- `termion` — 仅支持 Unix，不支持 Windows
- `terminal-ansi` — 底层库，需要大量手写渲染逻辑

### 2. Markdown 解析：pulldown-cmark

**选择**：pulldown-cmark 0.12

**理由**：Rust 生态最主流的 Markdown 解析器，基于事件流（SAX 风格）设计，内存效率高，支持 GFM 扩展（表格、删除线、任务列表）。

**替代方案**：
- `comrak` — 功能更全但依赖更重，对 MVP 来说过于复杂

### 3. 架构：三层分离（parser / app / ui）

**选择**：
```
src/
├── main.rs      # 入口、终端生命周期、事件循环
├── parser.rs    # Markdown → 带样式的行（纯函数，可测试）
├── app.rs       # 状态管理、按键处理、行换行算法
└── ui.rs        # ratatui 渲染（纯展示，无状态）
```

**理由**：parser 是纯函数方便单测；app 持有所有状态方便推理；ui 只做展示便于替换渲染后端。

### 4. 行换行策略：词级换行 + CJK 字符级回退

**选择**：
- 英文按空格分词，在空格处换行
- CJK / 全角字符每个字符可独立换行
- 超长单词（>行宽）字符级强制断行

**理由**：中文没有空格分隔，纯英文词级换行对中文无效。混合策略同时照顾两种语言。

### 5. 渲染管线：Markdown → Styled Lines → Wrap → Scroll Window

**选择**：
1. `parser.rs`：pulldown-cmark 事件 → `Vec<Line>`（每行是 `Vec<(String, Style)>`）
2. `app.rs`：Styled Lines → 按终端宽度换行 → `Vec<Line>`（wrapped）
3. `ui.rs`：取 scroll 到 scroll+height 的切片渲染

**理由**：三层管线每层职责单一。解析结果可缓存，仅窗口变化时重新滚动。

### 6. 样式方案

| 元素 | 样式 |
|------|------|
| H1 | Yellow + Bold + Underline |
| H2 | Cyan + Bold |
| H3 | Green + Bold |
| H4 | Magenta + Bold |
| H5/H6 | Blue/White + Bold |
| **粗体** | Bold |
| *斜体* | Italic |
| ~~删除线~~ | CrossedOut |
| `行内代码` | Green fg + Black bg |
| 代码块 | Green fg + Black bg + 边框线 |
| 引用 | DarkGray + `▎` 前缀 |
| 列表标记 | Magenta |
| 链接 | Blue + Underline |
| 分隔线 | DarkGray |

### 7. 错误处理：anyhow

**选择**：`anyhow` crate

**理由**：应用级错误处理，`?` 运算符简洁，适合 CLI 工具。

## Risks / Trade-offs

- **[pulldown-cmark 版本兼容性]** → 锁定 0.12.x，避免 0.13 breaking changes 影响构建
- **[大文件性能]** → MVP 采用全量解析+全量换行，超大文件可能有初始延迟。Mitigation：MVP 后增加流式解析/分块渲染
- **[终端尺寸变化]** → 窗口 resize 时需重新换行。Mitigation：监听 terminal size 变化事件，触发 re-wrap
- **[CJK 字符宽度边界情况]** → 某些特殊字符（emoji、组合字符）unicode-width 计算可能不准。Mitigation：接受小误差，后续可引入更精确的 wcwidth 实现
