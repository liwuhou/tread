## ADDED Requirements

### Requirement: 读取本地浏览器的 cookies
系统 SHALL 使用 `cookie_scoop` 库从用户本地浏览器读取指定 URL 的 cookies。

#### Scenario: 成功读取 cookies
- **WHEN** 用户调用 `get_cookies_for_url(url)` 且本地浏览器有该 URL 的 cookies
- **THEN** 系统返回包含所有 cookies 的 `Vec<(String, String)>`，每个元素为 `(name, value)` 对

#### Scenario: 读取失败时返回空
- **WHEN** 用户调用 `get_cookies_for_url(url)` 但读取失败（如浏览器未安装、权限不足等）
- **THEN** 系统返回空的 `Vec::new()`，不报错

### Requirement: 将 cookies 注入到 headless Chrome
系统 SHALL 在启动 headless Chrome 并导航到目标 URL 后，将读取到的 cookies 注入到浏览器实例中。

#### Scenario: 成功注入 cookies
- **WHEN** `headless_fetch()` 被调用且 `cookies` 参数非空
- **THEN** 系统 SHALL：
  1. 将每个 `(name, value)` 转换为 `CookieParam`
  2. 为每个 `CookieParam` 设置 `url` 为当前页面 URL
  3. 调用 `tab.set_cookies(cookie_params)` 注入所有 cookies
  4. 调用 `tab.reload()` 重新加载页面使 cookies 生效

#### Scenario: 无 cookies 时跳过注入
- **WHEN** `headless_fetch()` 被调用且 `cookies` 参数为空
- **THEN** 系统跳过 cookies 注入步骤，不重新加载页面

#### Scenario: 注入失败时静默降级
- **WHEN** `tab.set_cookies()` 或 `tab.reload()` 返回错误
- **THEN** 系统 SHALL 忽略错误，继续执行后续流程（等待用户操作和提取内容）

### Requirement: 保持现有用户体验
系统 SHALL 在 cookies 注入成功后，继续显示原有的登录提示，允许用户在页面需要进一步认证时手动操作。

#### Scenario: 注入后显示提示
- **WHEN** cookies 注入完成（无论成功或失败）
- **THEN** 系统继续显示原有的提示信息：
  - "📌 如果页面需要登录，请在浏览器窗口中完成登录。"
  - "📌 登录完成且页面加载后，按 Enter 键继续提取内容。"
  - "📌 （或等待 5 分钟自动超时）"

#### Scenario: 用户按 Enter 继续
- **WHEN** 用户在提示后按 Enter 键
- **THEN** 系统提取当前页面内容并返回

### Requirement: 使用 CookieParam 的正确结构
系统 SHALL 使用 `headless_chrome::protocol::cdp::Network::CookieParam` 结构体创建 cookies。

#### Scenario: 创建 CookieParam
- **WHEN** 系统需要将 cookie `(name, value)` 转换为 `CookieParam`
- **THEN** 系统 SHALL 创建如下结构：
  ```rust
  CookieParam {
      name: name.to_string(),
      value: value.to_string(),
      url: Some(url.to_string()),
      domain: None,
      path: None,
      secure: None,
      http_only: None,
      same_site: None,
      expires: None,
      priority: None,
      same_party: None,
      source_scheme: None,
      source_port: None,
      partition_key: None,
  }
  ```
