## Why

tread 已支持 Markdown 和 EPUB 两种格式的阅读。但用户最常见的阅读场景之一——浏览网页文章——仍然缺失。用户在终端中看到一个感兴趣的文章链接时，不得不切换到浏览器阅读，打断了终端工作流。支持网页正文提取和阅读，将使 tread 成为一个完整的"终端阅读中心"。

## What Changes

- **新增网页抓取模块**：`src/web.rs`，负责 HTTP 请求、HTML 下载、Readability 正文提取、图片 URL 解析
- **新增 HTML → LineContent 转换**：将 Readability 提取的干净 HTML 转换为 `Vec<LineContent>`，复用现有样式体系
- **新增 CLI URL 识别**：`main.rs` 检测参数是否为 `http://`/`https://` 开头，自动分发到网页模式
- **新增网页缓存**：下载的 HTML 缓存到 `~/.tread/cache/web/`，24 小时过期，`--refresh` 强制刷新
- **新增网页进度保存**：按 URL hash 保存滚动位置到 `~/.tread/progress/web_<hash>.json`
- **新增 User-Agent 伪装**：HTTP 请求设置 Firefox User-Agent，避免被网站拒绝
- **新增错误处理**：Readability 提取失败时显示"无法提取正文"提示

## Capabilities

### New Capabilities
- `web-reader`: URL 识别、HTTP 抓取、Readability 正文提取、缓存管理、进度保存的完整网页阅读流程

## Impact

- **新增依赖**：`dom_smoothie`（Readability 内容提取，基于 Mozilla Readability.js）
- **新增文件**：`src/web.rs`（HTTP + 缓存 + 提取 + 进度）
- **修改文件**：`src/main.rs`（URL 识别、`--refresh` 参数）、`src/app.rs`（网页进度字段）
- **新增目录**：`~/.tread/cache/web/`（HTML 缓存）、`~/.tread/progress/`（复用已有目录）
- **现有功能不受影响**：Markdown 和 EPUB 阅读行为完全不变
- **图片复用**：网页中的图片通过已有的 `ImageNode` + 缓存 + 系统查看器流程处理
