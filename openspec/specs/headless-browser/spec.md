## Purpose

Defines how tread reads browser cookies, injects them into interactive Headless Chrome sessions, extracts rendered content, and preserves the existing login-assist user experience.

## Requirements

### Requirement: Browser cookie reading
系统 SHALL 使用 `cookie-scoop` 读取用户浏览器的 cookie，按域名过滤后附加到 HTTP 请求或 Headless Chrome 会话中。

#### Scenario: 读取 Chrome cookie
- **WHEN** 用户在 Chrome 中已登录 `example.com`，运行 `tread https://example.com/article`
- **THEN** 系统读取 Chrome 中 `example.com` 的 cookie，附加到请求中，获取已登录状态的页面

#### Scenario: 读取 Firefox cookie
- **WHEN** 用户在 Firefox 中已登录目标网站
- **THEN** 系统从 Firefox 读取 cookie

#### Scenario: Cookie 读取失败静默降级
- **WHEN** cookie-scoop 读取失败（无权限、无浏览器等）
- **THEN** 不报错，使用无 cookie 的普通请求

#### Scenario: 按域名过滤
- **WHEN** 浏览器有 100 个不同域名的 cookie
- **THEN** 只提取目标 URL 域名的 cookie

#### Scenario: 成功读取 cookies
- **WHEN** 用户调用 `get_cookies_for_url(url)` 且本地浏览器有该 URL 的 cookies
- **THEN** 系统返回包含所有 cookies 的 `Vec<(String, String)>`，每个元素为 `(name, value)` 对

#### Scenario: 读取失败时返回空
- **WHEN** 用户调用 `get_cookies_for_url(url)` 但读取失败（如浏览器未安装、权限不足等）
- **THEN** 系统返回空的 `Vec::new()`，不报错

### Requirement: Headless browser mode
系统 SHALL 支持 `-i` / `--interactive` 参数启动 Headless Chrome 加载动态内容。

#### Scenario: 基本 Headless 加载
- **WHEN** 用户运行 `tread https://spa-site.com/article -i`
- **THEN** 系统启动 Headless Chrome，导航到 URL，等待 JS 渲染，提取 HTML

#### Scenario: -i 短参数
- **WHEN** 用户运行 `tread https://spa-site.com/article --interactive`
- **THEN** 等同于 `-i`

#### Scenario: Chrome 未安装
- **WHEN** `-i` 模式但系统未安装 Chrome/Chromium
- **THEN** 输出错误信息到 stderr，提示安装 Chrome

#### Scenario: 页面加载超时
- **WHEN** SPA 页面加载超过 30 秒
- **THEN** 放弃加载，返回错误

### Requirement: Cookie + Headless 组合
系统 SHALL 在 `-i` 模式下自动注入浏览器 cookie 到 Headless Chrome。

#### Scenario: 带 cookie 的 Headless 请求
- **WHEN** 用户已登录目标网站，使用 `-i` 模式访问
- **THEN** Headless Chrome 带上浏览器 cookie 访问页面，获取已登录状态的内容

#### Scenario: 无 cookie 的 Headless 请求
- **WHEN** cookie-scoop 未找到目标域名的 cookie
- **THEN** Headless Chrome 正常访问（无 cookie），获取公开内容

#### Scenario: 成功注入 cookies
- **WHEN** `headless_fetch()` 被调用且 `cookies` 参数非空
- **THEN** 系统 SHALL 将每个 `(name, value)` 转换为 `CookieParam`，为每个 `CookieParam` 设置 `url` 为当前页面 URL，调用 `tab.set_cookies(cookie_params)` 注入所有 cookies，并调用 `tab.reload()` 重新加载页面使 cookies 生效

#### Scenario: 无 cookies 时跳过注入
- **WHEN** `headless_fetch()` 被调用且 `cookies` 参数为空
- **THEN** 系统跳过 cookies 注入步骤，不重新加载页面

#### Scenario: 注入失败时静默降级
- **WHEN** `tab.set_cookies()` 或 `tab.reload()` 返回错误
- **THEN** 系统 SHALL 忽略错误，继续执行后续流程（等待用户操作和提取内容）

### Requirement: Rendered content extraction
系统 SHALL 从 Headless Chrome 获取渲染后的 HTML，然后用 Readability 提取正文。

#### Scenario: SPA 内容提取
- **WHEN** SPA 页面的初始 HTML 为空壳 `<div id="app"></div>`，JS 渲染后包含文章正文
- **THEN** Headless Chrome 提取渲染后的 DOM，Readability 提取正文

#### Scenario: 等待 JS 渲染
- **WHEN** 页面需要 JS 异步加载内容
- **THEN** Headless Chrome 等待至少 2 秒让 JS 执行完毕，然后提取 HTML

### Requirement: 保持现有用户体验
系统 SHALL 在 cookies 注入成功后，继续显示原有的登录提示，允许用户在页面需要进一步认证时手动操作。

#### Scenario: 注入后显示提示
- **WHEN** cookies 注入完成（无论成功或失败）
- **THEN** 系统继续显示原有的提示信息：如果页面需要登录，请在浏览器窗口中完成登录；登录完成且页面加载后，按 Enter 键继续提取内容；或等待 5 分钟自动超时

#### Scenario: 用户按 Enter 继续
- **WHEN** 用户在提示后按 Enter 键
- **THEN** 系统提取当前页面内容并返回

### Requirement: 使用 CookieParam 的正确结构
系统 SHALL 使用 `headless_chrome::protocol::cdp::Network::CookieParam` 结构体创建 cookies。

#### Scenario: 创建 CookieParam
- **WHEN** 系统需要将 cookie `(name, value)` 转换为 `CookieParam`
- **THEN** 系统创建 `CookieParam`，设置 `name`、`value` 和 `url: Some(url.to_string())`，其他可选字段保持 `None`

### Requirement: Session persistence
系统 SHALL 可选地保存和复用登录 session。

#### Scenario: 保存 session
- **WHEN** 成功使用浏览器 cookie 获取到页面内容
- **THEN** 将 cookie 保存到 `~/.tread/sessions/<domain>.json`

#### Scenario: 复用 session
- **WHEN** 再次访问同域名且 session 未过期（24h）
- **THEN** 直接使用保存的 session cookie，不重新读取浏览器

#### Scenario: Session 过期
- **WHEN** session 超过 24 小时
- **THEN** 忽略 session，重新读取浏览器 cookie

### Requirement: CLI parameter parsing
系统 SHALL 正确解析 `-i` / `--interactive` 参数。

#### Scenario: -i 参数
- **WHEN** 用户运行 `tread https://example.com -i`
- **THEN** 进入 Headless 模式

#### Scenario: --interactive 长参数
- **WHEN** 用户运行 `tread https://example.com --interactive`
- **THEN** 等同于 `-i`

#### Scenario: -i 对非 URL 无效
- **WHEN** 用户运行 `tread sample.md -i`
- **THEN** 忽略 `-i`，正常打开 Markdown 文件

#### Scenario: -i 和 --refresh 组合
- **WHEN** 用户运行 `tread https://example.com -i --refresh`
- **THEN** 同时启用 Headless 模式和强制刷新
