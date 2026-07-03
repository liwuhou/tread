## ADDED Requirements

### Requirement: URL detection and web mode
系统 SHALL 检测 CLI 参数是否为 URL（以 `http://` 或 `https://` 开头），自动进入网页阅读模式。

#### Scenario: URL 参数进入网页模式
- **WHEN** 用户运行 `tread https://example.com/article`
- **THEN** 系统识别为网页模式，抓取并显示网页正文

#### Scenario: HTTP URL
- **WHEN** 用户运行 `tread http://example.com/article`
- **THEN** 系统正常抓取（HTTP）

#### Scenario: 非 URL 参数保持现有行为
- **WHEN** 用户运行 `tread sample.md` 或 `tread book.epub`
- **THEN** 系统使用对应的 Markdown/EPUB 解析器（行为不变）

### Requirement: HTTP fetch with User-Agent
系统 SHALL 使用 Firefox User-Agent 发送 HTTP 请求。

#### Scenario: 设置 User-Agent header
- **WHEN** 抓取网页时
- **THEN** HTTP 请求包含 `User-Agent: Mozilla/5.0 ... Firefox/...` header

#### Scenario: 网络失败
- **WHEN** HTTP 请求失败（超时、DNS 失败、连接拒绝）
- **THEN** 输出错误信息到 stderr，退出码 1

### Requirement: Readability content extraction
系统 SHALL 使用 Readability 算法从网页 HTML 中提取正文。

#### Scenario: 成功提取正文
- **WHEN** 网页包含可读的文章内容
- **THEN** 提取 title 和正文 HTML，过滤导航栏/广告/页脚

#### Scenario: 提取失败
- **WHEN** Readability 无法提取正文（首页、搜索结果页等）
- **THEN** 显示"无法提取正文"提示

#### Scenario: 提取的正文渲染
- **WHEN** Readability 提取出正文 HTML
- **THEN** 转换为 `Vec<LineContent>` 进行终端渲染，保留标题、粗体、斜体、链接、图片

### Requirement: Web page caching
系统 SHALL 缓存抓取的网页 HTML 到 `~/.tread/cache/web/`。

#### Scenario: 首次访问
- **WHEN** 访问一个 URL 且缓存中不存在
- **THEN** HTTP 抓取，保存到 `~/.tread/cache/web/<hash>.html` 和 `.meta.json`

#### Scenario: 缓存命中（未过期）
- **WHEN** 访问一个 URL 且缓存存在且未超过 24 小时
- **THEN** 直接使用缓存，不发起 HTTP 请求

#### Scenario: 缓存过期
- **WHEN** 缓存存在但已超过 24 小时
- **THEN** 重新 HTTP 抓取，更新缓存

#### Scenario: 强制刷新
- **WHEN** 用户使用 `--refresh` 或 `-r` 参数
- **THEN** 跳过缓存，强制 HTTP 抓取，更新缓存

### Requirement: Web page progress persistence
系统 SHALL 保存网页的阅读进度（滚动位置）。

#### Scenario: 保存进度
- **WHEN** 用户在网页中阅读到某位置，按 `q` 退出
- **THEN** 系统将 `{url, scroll, saved_at}` 保存到 `~/.tread/progress/web_<hash>.json`

#### Scenario: 恢复进度
- **WHEN** 用户再次打开同一 URL
- **THEN** 自动恢复到上次滚动位置

#### Scenario: 首次访问无进度
- **WHEN** 首次访问一个 URL
- **THEN** 从顶部开始阅读

### Requirement: Web image handling
系统 SHALL 正确处理网页中的图片。

#### Scenario: 绝对 URL 图片
- **WHEN** 网页包含 `<img src="https://cdn.example.com/pic.jpg">`
- **THEN** 直接使用该 URL 下载图片到缓存

#### Scenario: 相对路径图片
- **WHEN** 网页包含 `<img src="/images/pic.jpg">`，base URL 为 `https://example.com/article/`
- **THEN** 解析为 `https://example.com/images/pic.jpg` 后下载

#### Scenario: 相对路径图片（../）
- **WHEN** 网页包含 `<img src="../assets/pic.jpg">`，base URL 为 `https://example.com/article/page/`
- **THEN** 解析为 `https://example.com/article/assets/pic.jpg` 后下载

### Requirement: CLI --refresh flag
系统 SHALL 支持 `--refresh` / `-r` 参数强制刷新网页。

#### Scenario: --refresh 参数
- **WHEN** 用户运行 `tread https://example.com/article --refresh`
- **THEN** 跳过缓存，强制重新抓取

#### Scenario: -r 短参数
- **WHEN** 用户运行 `tread https://example.com/article -r`
- **THEN** 等同于 `--refresh`

#### Scenario: --refresh 对非网页无效
- **WHEN** 用户运行 `tread sample.md --refresh`
- **THEN** 忽略 `--refresh`，正常打开 Markdown 文件

### Requirement: Web mode status bar
系统 SHALL 在状态栏显示网页特有信息。

#### Scenario: 网页状态栏
- **WHEN** 阅读网页文章
- **THEN** 状态栏显示 `文章标题 | 行号/总行数 | 百分比`
