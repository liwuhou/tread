## Why

当前 tread 的网页阅读依赖 `ureq` + `dom_smoothie`（Readability），能处理大部分静态网页。但两类网站无法阅读：

1. **需要登录的网站**：HTTP 请求被重定向到登录页，Readability 提取到的是登录表单
2. **SPA / 异步加载网站**：HTML 只有一个空壳 `<div id="app"></div>`，实际内容由 JavaScript 异步获取并渲染

需要引入两个能力来覆盖这两种场景。

## What Changes

- **新增浏览器 Cookie 读取**：使用 `cookie-scoop` 库读取用户浏览器（Chrome/Firefox/Safari/Edge）的 cookie，自动附加到 HTTP 请求中，复用用户已有的登录态
- **新增 Headless 浏览器模式**：`-i` / `--interactive` 参数启动 Headless Chrome，执行页面中的 JavaScript，等待动态内容渲染完成后再提取正文
- **两种模式组合**：`-i` 模式下同时使用 cookie（如果可用），覆盖"需要登录的 SPA"场景
- **新增 Session 持久化**：保存成功登录的 session，下次访问同域名时自动复用

## Capabilities

### New Capabilities
- `headless-browser`: Cookie 读取、Headless 浏览器启动、动态内容等待、Session 持久化的完整流程

## Impact

- **新增依赖**：`cookie-scoop`（浏览器 cookie 读取 + 解密）、`headless_chrome`（Headless Chrome 控制）
- **新增文件**：`src/headless.rs`（Headless 浏览器逻辑）、扩展 `src/web.rs`（cookie 集成）
- **修改文件**：`src/main.rs`（`-i`/`--interactive` 参数解析）
- **系统要求**：用户需安装 Chrome/Chromium（`-i` 模式需要）
- **首次运行**：macOS 可能弹出 Keychain 授权框（读取 Chrome cookie 需要）
- **现有功能不受影响**：默认模式（无 `-i`）行为完全不变
