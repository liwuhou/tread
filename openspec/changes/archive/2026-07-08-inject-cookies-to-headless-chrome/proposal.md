## Why

当用户使用 `-i` (interactive) 模式打开需要登录的网页时，tread 会启动一个新的 Chrome 实例，但该实例不会继承用户本地 Chrome 的登录状态。用户每次都需要重新登录，体验不友好。

## What Changes

- **Cookies 注入**：在 `headless_fetch()` 函数中，将已读取的浏览器 cookies 注入到新启动的 Chrome 实例中
- **页面重载**：注入 cookies 后自动重新加载页面，使 cookies 生效
- **无缝登录**：用户无需重复登录，tread 自动复用本地浏览器的认证状态

## Capabilities

### New Capabilities
- `cookie-injection`: 将本地浏览器的 cookies 注入到 headless Chrome 中，实现无缝认证复用

### Modified Capabilities

（无）

## Impact

- **代码**：`src/headless.rs` 中的 `headless_fetch()` 函数
- **依赖**：使用现有的 `headless_chrome` 和 `cookie_scoop` 库，无需新增依赖
- **用户体验**：使用 `-i` 模式访问需要登录的网站时，自动复用本地浏览器的登录状态
- **兼容性**：不影响现有功能，cookies 注入失败时静默降级（仍显示登录提示）
