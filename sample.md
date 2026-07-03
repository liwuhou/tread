# 终端阅读器 — 完整测试文档

这是一篇用于测试 **theft-read** 终端阅读器的长文档。内容涵盖了所有章节类型和排版元素，足够长以验证滚动、翻页、跳转等功能。

---

## 第一章：阅读器的诞生

在计算机科学发展的早期，人们就在终端上阅读文本。从最初的 Teletype 到后来的 VT100，终端一直是程序员最亲密的伙伴。如今，虽然 GUI 应用层出不穷，但终端阅读器凭借其独特的魅力，依然在开发者社区中占有一席之地。

终端阅读器的优势在于：

- **轻量**：无需加载庞大的渲染引擎
- **快速**：打开文件几乎是瞬间完成的
- **键盘驱动**：不需要离开键盘去摸鼠标
- **SSH 友好**：可以在远程服务器上直接阅读
- **可脚本化**：可以与其他命令行工具组合使用

### 1.1 历史上的终端阅读器

说到终端阅读器，不得不提几个经典之作：

1. **less** — Unix 世界最经典的分页器，`less is more` 这句名言就来自于此
2. **vim** — 虽然是编辑器，但很多人用它来阅读代码和文档
3. **lynx** — 最早的终端网页浏览器之一
4. **w3m** — 另一个经典的终端浏览器，支持表格渲染
5. **mutt** — 终端邮件客户端，本质上也是一个阅读器

这些工具的共同特点是：它们都在有限的终端空间中，尽可能高效地呈现信息。

### 1.2 为什么还要造轮子

你可能会问：已经有这么多工具了，为什么还要写一个新的？原因很简单：

> 现有的工具要么只专注于某一种格式（如 less 只看纯文本），要么操作方式不够直觉（如 vim 的学习曲线）。我们希望创造一个统一的、易用的、专门为阅读优化的终端工具。

theft-read 的目标是成为终端里的 "Kindle"——一个让你可以舒适地阅读各种格式文档的工具。

---

## 第二章：Rust 与终端开发

### 2.1 为什么选择 Rust

Rust 是近年来最受欢迎的编程语言之一。它在终端应用开发中有几个突出的优势：

**性能**：Rust 编译为原生机器码，没有垃圾回收的开销。对于需要实时响应用户输入的 TUI 应用来说，这一点尤为重要。

**安全**：Rust 的所有权系统在编译时就能防止大量常见的 bug，比如空指针、数据竞争等。这意味着更少的运行时崩溃。

**生态**：Rust 的 crate 生态非常丰富，尤其在终端开发领域：

| 库名 | 功能 | Star 数 |
|------|------|---------|
| ratatui | TUI 框架 | 10k+ |
| crossterm | 终端控制 | 2k+ |
| pulldown-cmark | Markdown 解析 | 2k+ |
| clap | 命令行解析 | 14k+ |
| anyhow | 错误处理 | 9k+ |
| tui-textarea | 文本编辑 | 500+ |

### 2.2 核心依赖介绍

#### ratatui

`ratatui` 是一个用于构建终端用户界面的 Rust 库。它采用了 React 式的声明渲染模型——你只需要描述界面应该是什么样子，框架会处理渲染差异。

```rust
// ratatui 的典型用法
terminal.draw(|frame| {
    let paragraph = Paragraph::new("Hello, world!")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
})?;
```

#### pulldown-cmark

`pulldown-cmark` 是一个基于事件流的 Markdown 解析器。它不会一次性把整个文档加载到内存，而是逐个产生解析事件，这对于大文件非常友好。

```rust
use pulldown_cmark::{Parser, Options, Event, Tag};

let mut options = Options::empty();
options.insert(Options::ENABLE_STRIKETHROUGH);

let parser = Parser::new_ext(markdown_input, options);

for event in parser {
    match event {
        Event::Start(Tag::Heading { level, .. }) => {
            println!("Heading level: {:?}", level);
        }
        Event::Text(text) => {
            println!("Text: {}", text);
        }
        _ => {}
    }
}
```

#### crossterm

`crossterm` 提供了跨平台的终端操作能力，包括：

- 光标控制（移动、隐藏/显示）
- 颜色与样式
- 键盘与鼠标事件
- 终端屏幕切换（alternate screen）
- 原始模式（raw mode）

```rust
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode},
    event::{read, Event, KeyCode},
};

enable_raw_mode()?;
loop {
    if let Event::Key(key) = read()? {
        match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Down => scroll_down(),
            KeyCode::Up => scroll_up(),
            _ => {}
        }
    }
}
disable_raw_mode()?;
```

---

## 第三章：CJK 文本处理

### 3.1 字符宽度的挑战

在终端中显示中文、日文、韩文（统称 CJK）文本时，一个关键的问题是字符宽度。ASCII 字符占 1 列宽，而大多数 CJK 字符占 2 列宽。

例如：

- `hello` → 5 列宽
- `你好` → 4 列宽（每个字符 2 列）
- `hello你好` → 9 列宽（5 + 4）

Unicode 标准定义了 East Asian Width 属性，用来描述字符在东亚环境下的预期宽度。常见的宽度值包括：

1. **Naarrow (Na)** — 半角字符，1 列宽
2. **Wide (W)** — 全角字符，2 列宽
3. **Ambiguous (A)** — 视上下文而定
4. **Halfwidth (H)** — 半角，1 列宽
5. **Fullwidth (F)** — 全角，2 列宽
6. **Neutral (N)** — 无东亚宽度，通常 1 列宽

### 3.2 换行策略

对于英文文本，换行通常在空格处进行。但对于 CJK 文本，没有空格分隔词语，可以在任何字符处换行。

theft-read 采用混合换行策略：

```
英文部分：按空格分词，在词间换行
CJK 部分：每个字符都可以作为换行点
混合文本：英文按词、CJK 按字
超长单词：字符级强制断行
```

这种策略确保了无论什么语言的文本，都能在终端中正确显示。

### 3.3 一些测试文本

以下是一些用于测试 CJK 换行的文本：

**纯中文长句**：

终端阅读器是一个运行在命令行环境中的文本阅读工具，它允许用户在不离开终端的情况下浏览 Markdown 文档、代码文件以及其他纯文本格式的内容。对于经常在终端环境中工作的开发者来说，这样一个工具可以显著提升工作效率。

**中英混排**：

Rust 是一门注重安全性和性能的系统级编程语言。它的所有权模型（Ownership Model）在编译时就能保证内存安全，无需垃圾回收器（Garbage Collector）。这使得 Rust 非常适合开发终端应用、嵌入式系统、WebAssembly 模块等对性能有严格要求的场景。

**包含代码的混排**：

使用 `unicode-width` crate 可以方便地计算字符串的显示宽度。调用 `UnicodeWidthStr::width(s)` 即可获取字符串 `s` 在终端中的列宽。

---

## 第四章：快捷键系统设计

### 4.1 Vim 风格导航

theft-read 采用了广受欢迎的 Vim 风格快捷键。以下是最常用的快捷键一览：

**基础导航**：

| 快捷键 | 功能 | Vim 对应 |
|--------|------|----------|
| `j` / `↓` | 下一行 | j |
| `k` / `↑` | 上一行 | k |
| `Ctrl+d` | 向下翻半页 | Ctrl+d |
| `Ctrl+u` | 向上翻半页 | Ctrl+u |
| `Ctrl+f` | 向下翻整页 | Ctrl+f |
| `Ctrl+b` | 向上翻整页 | Ctrl+b |
| `g` / `Home` | 跳到顶部 | gg |
| `G` / `End` | 跳到底部 | G |
| `PageDown` | 向下翻半页 | Ctrl+f |
| `PageUp` | 向上翻半页 | Ctrl+b |

**其他功能**：

| 快捷键 | 功能 |
|--------|------|
| `?` | 显示/关闭帮助 |
| `q` / `Esc` | 退出 |
| `Ctrl+c` | 强制退出 |

### 4.2 设计原则

快捷键设计遵循几个原则：

1. **渐进式发现**：按 `?` 随时可以查看帮助
2. **多入口**：同一个功能有多个快捷键（如 `g` 和 `Home` 都到顶部）
3. **符合习惯**：优先使用 Vim 用户熟悉的快捷键
4. **不冲突**：所有快捷键互不冲突，不会产生歧义

---

## 第五章：代码示例集

### 5.1 Hello World

```rust
fn main() {
    println!("Hello, world!");
}
```

### 5.2 斐波那契数列

```rust
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    for i in 0..20 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
}
```

### 5.3 文件读取

```rust
use std::fs;
use std::path::Path;

fn read_file(path: &Path) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

fn main() {
    match read_file(Path::new("sample.md")) {
        Ok(content) => println!("Read {} bytes", content.len()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 5.4 命令行参数解析

```rust
use std::env;

struct Config {
    file_path: String,
    line_number: Option<usize>,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Usage: app <file> [line]".to_string());
    }

    let file_path = args[1].clone();
    let line_number = if args.len() > 2 {
        Some(args[2].parse().map_err(|_| "Invalid line number")?)
    } else {
        None
    };

    Ok(Config { file_path, line_number })
}
```

### 5.5 终端 TUI 基础

```rust
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};
use crossterm::terminal;
use std::io;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    terminal::write(&mut stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        let area = frame.area();
        let block = Block::default()
            .title("My App")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new("Hello from TUI!")
            .block(block);
        frame.render_widget(paragraph, area);
    })?;

    // ... event loop ...

    terminal::disable_raw_mode()?;
    terminal::write(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
```

---

## 第六章：引用与名言

以下是一些关于阅读和编程的名言：

> 程序必须写得让人能看懂，顺便让机器能执行。
>
> — Harold Abelson

> 好的代码本身就是最好的文档。
>
> — Steve McConnell

> 先让它能工作，再让它正确，最后让它快。
>
> — Kent Beck

> 简化！简化！再简化！
>
> — 某位不愿透露姓名的程序员

> 任何傻瓜都能写出计算机能理解的代码。好的程序员写出人类能理解的代码。
>
> — Martin Fowler

> The best way to predict the future is to invent it.
>
> — Alan Kay

---

## 附录 A：格式支持清单

| 格式 | 状态 | 说明 |
|------|------|------|
| Markdown | ✅ 已实现 | 标题、粗体、斜体、代码块、列表、引用、表格、分隔线 |
| EPUB | 🔜 计划中 | 需要 epub-rs 或自行解析 ZIP + XHTML |
| 网页 | 🔜 计划中 | 需要 HTTP 客户端 + Readability 算法 |
| PDF | ❓ 待评估 | 终端中渲染 PDF 比较困难 |
| TXT | ✅ 已支持 | 作为 Markdown 的特例自动处理 |

---

## 附录 B：已知限制

1. **不支持鼠标滚动**：MVP 版本仅支持键盘导航
2. **大文件性能**：当前全量解析，超大文件可能有初始延迟
3. **终端兼容**：依赖 ANSI 转义序列，极老的终端可能不兼容
4. **图片支持**：终端中显示图片需要特殊协议（如 Sixel、iTerm2 inline images），MVP 不支持

---

## 附录 C：未来路线图

```
Phase 1 (MVP)     → Markdown 阅读 ✅ 当前
Phase 2           → EPUB 支持
Phase 3           → 网页内容抓取
Phase 4           → 搜索功能
Phase 5           → 阅读进度持久化
Phase 6           → 书签与笔记
Phase 7           → 主题与配色方案
Phase 8           → 插件系统
```

---

## 附录 D：图片测试

本文档包含以下图片用于测试图片预览功能：

**本地图片**（如果文件存在则显示为占位符，按 Enter 可打开）：

![示例本地图片](./sample-image.png)

**远程图片**（需要网络连接，首次打开会下载到缓存）：

![Rust 语言 Logo](https://www.rust-lang.org/static/images/rust-social.jpg)

**空 alt 文本的图片**：

![](https://via.placeholder.com/150)

**不存在的本地图片**（应显示下载失败标记）：

![不存在的图片](./nonexistent.png)

使用 **Tab** 在图片之间跳转，**Shift+Tab** 反向跳转，**Enter** 打开当前焦点图片。

---

*感谢你读到这里！这份文档的目的是充分测试 theft-read 的各项功能。*
*试试按 `G` 跳到底部，再按 `g` 回到顶部，或者用 `Ctrl+u` 往上翻页。*
*按 `?` 查看所有快捷键，按 `q` 退出。*

— theft-read 团队
