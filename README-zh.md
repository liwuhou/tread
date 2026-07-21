# tread

[English](README.md)

`tread` 是一款用于阅读 Markdown、EPUB 和网页内容的终端阅读器。

## 功能

- 在终端 UI 中阅读本地 Markdown 文件。
- 阅读 EPUB 文件，并支持目录导航。
- 从 URL 获取并渲染网页。
- 可选地使用由 Chrome 驱动的交互式获取功能处理动态页面。
- 在操作系统支持时，从阅读界面打开链接和图片。

## 安装

### Cargo

```sh
cargo install tread
```

### Homebrew

```sh
brew install liwuhou/tap/tread
```

### GitHub Releases

从 GitHub Releases 页面下载适用于你平台的归档文件：

- macOS Apple Silicon：`aarch64-apple-darwin`
- macOS Intel：`x86_64-apple-darwin`
- Linux x86_64：`x86_64-unknown-linux-gnu`
- Linux arm64：`aarch64-unknown-linux-gnu`
- Windows x86_64：`x86_64-pc-windows-msvc`

## 使用

```sh
tread                         # open the reading dashboard
tread <file.md|file.epub|url> [-r|--refresh] [-i|--interactive]
```

选项：

- `-r`, `--refresh`：强制刷新网页并跳过缓存。
- `-i`, `--interactive`：对动态页面使用 Chrome。

### 仪表盘快捷键

- `j` / `k` / 方向键：在历史记录条目之间移动。
- `Enter`：从已保存的进度继续阅读所选的 URL、EPUB 或 Markdown 文件。
- `o`：从仪表盘提示中打开新的本地路径或 URL；URL 可包含 `-i`/`--interactive` 和 `-r`/`--refresh`，例如 `https://example.com -i`。
- `s`：收藏或取消收藏所选条目；收藏条目会优先显示。
- `r`：从仪表盘移除所选条目，但不删除文件。
- `q`：退出仪表盘。

仪表盘将统一的阅读历史记录存储在 `~/.tread/history.json`。重新打开已隐藏的目标会使其再次显示，并清除其收藏状态。
当终端高度非常小时，仪表盘会优先显示最近阅读区域，并可能隐藏页脚提示、输入提示建议或输入/错误文本，直到有更多可用高度。

### 阅读界面快捷键

- `j` / `k` / 方向键：滚动一行。
- `Ctrl+d` / `Ctrl+u`：滚动半页。
- `g` / `G`：跳转到顶部或底部。
- `D`：返回仪表盘。
- `Tab`：聚焦下一张图片或链接。
- `Enter`：打开已聚焦的图片或链接。
- `Ctrl+n` / `Ctrl+p`：下一章或上一章 EPUB 章节。
- `t`：打开 EPUB 目录。
- `?`：帮助。
- `q`：退出。

## 许可证

MIT
