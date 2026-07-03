## ADDED Requirements

### Requirement: Image detection in Markdown
系统 SHALL 识别 Markdown 中的 `![alt](url)` 图片语法，并生成 `ImageNode` 节点。

#### Scenario: 识别带 alt 文本的图片
- **WHEN** 解析到 `![公司 Logo](logo.png)`
- **THEN** 在输出中生成一个 `ImageNode`，`alt = "公司 Logo"`，`url = "logo.png"`

#### Scenario: 识别空 alt 文本的图片
- **WHEN** 解析到 `![](photo.jpg)`
- **THEN** 生成 `ImageNode`，`alt = ""`，`url = "photo.jpg"`

#### Scenario: 识别远程 URL 图片
- **WHEN** 解析到 `![封面](https://example.com/cover.jpg)`
- **THEN** 生成 `ImageNode`，`url = "https://example.com/cover.jpg"`

#### Scenario: 图片与文本混合
- **WHEN** 解析到 `这是一张图 ![img](pic.png) 好看吗`
- **THEN** 图片作为独立行输出，前后文本各自成行

### Requirement: Image cache for remote URLs
系统 SHALL 将远程图片下载到 `~/.tread/cache/` 目录，文件名使用 URL 的 SHA-256 哈希。

#### Scenario: 首次下载远程图片
- **WHEN** 遇到 URL `https://example.com/photo.jpg` 且缓存中不存在
- **THEN** 下载图片到 `~/.tread/cache/<sha256>.jpg`，`ImageNode.local_path` 设为该路径

#### Scenario: 缓存命中不重复下载
- **WHEN** 遇到相同 URL 且缓存文件已存在
- **THEN** 不发起 HTTP 请求，直接使用缓存路径

#### Scenario: 本地图片不下载
- **WHEN** 遇到相对路径 `./pic.png` 或绝对路径 `/home/user/pic.png`
- **THEN** 不下载，`ImageNode.local_path` 直接设为解析后的绝对路径

#### Scenario: 下载失败时优雅降级
- **WHEN** HTTP 请求超时或返回错误
- **THEN** `ImageNode.local_path` 保持 `None`，占位符显示下载失败标记

### Requirement: Image placeholder rendering
系统 SHALL 在终端中以占位符形式渲染图片节点。

#### Scenario: 默认占位符显示
- **WHEN** 渲染一个 `ImageNode`（alt = "风景照片"）
- **THEN** 显示 `[📷 风景照片]`，样式为 Cyan fg

#### Scenario: 空 alt 的占位符
- **WHEN** 渲染一个 `ImageNode`（alt = ""）
- **THEN** 显示 `[📷 image]`（使用默认文本）

#### Scenario: 下载失败的占位符
- **WHEN** 渲染一个 `ImageNode`（`local_path = None`，且 URL 为远程地址）
- **THEN** 显示 `[📷 alt text ⚠ 下载失败]`，样式为 Red fg

#### Scenario: 焦点状态高亮
- **WHEN** 图片占位符处于焦点状态
- **THEN** 背景色变为 DarkGray，文字变为 Bold

### Requirement: Image focus navigation
系统 SHALL 支持 Tab / Shift+Tab 在图片占位符之间跳转。

#### Scenario: Tab 跳到下一个图片
- **WHEN** 文档中有 3 张图片，当前焦点在第 1 张，用户按 Tab
- **THEN** 焦点移到第 2 张图片

#### Scenario: Shift+Tab 跳到上一个图片
- **WHEN** 当前焦点在第 2 张图片，用户按 Shift+Tab
- **THEN** 焦点移到第 1 张图片

#### Scenario: Tab 到最后一张后循环
- **WHEN** 当前焦点在最后一张图片，用户按 Tab
- **THEN** 焦点回到第 1 张图片

#### Scenario: 无图片时 Tab 无效
- **WHEN** 文档中没有任何图片，用户按 Tab
- **THEN** 不发生任何变化

### Requirement: Open image with system viewer
系统 SHALL 在用户按 Enter 时，用系统默认图片浏览器打开当前焦点图片。

#### Scenario: macOS 打开图片
- **WHEN** 在 macOS 上，焦点图片 `local_path = /path/to/img.jpg`，用户按 Enter
- **THEN** 执行 `open /path/to/img.jpg`

#### Scenario: Linux 打开图片
- **WHEN** 在 Linux 上，焦点图片 `local_path = /path/to/img.png`，用户按 Enter
- **THEN** 执行 `xdg-open /path/to/img.png`

#### Scenario: 未下载的图片不可打开
- **WHEN** 焦点图片 `local_path = None`，用户按 Enter
- **THEN** 不执行任何命令，可选显示提示信息

### Requirement: Cache directory management
系统 SHALL 在首次使用时创建缓存目录 `~/.tread/cache/`。

#### Scenario: 首次运行创建缓存目录
- **WHEN** `~/.tread/cache/` 不存在，遇到远程图片
- **THEN** 自动创建 `~/.tread/` 和 `~/.tread/cache/` 目录

#### Scenario: 缓存目录已存在
- **WHEN** `~/.tread/cache/` 已存在
- **THEN** 直接使用，不重复创建
