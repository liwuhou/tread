## Context

当前 tread 在使用 `-i` (interactive) 模式访问网页时，会通过 `headless_chrome` 启动一个新的 Chrome 实例。该实例使用临时的 user profile，不包含用户本地 Chrome 的 cookies 和登录状态。

现有流程：
1. `get_cookies_for_url()` 使用 `cookie_scoop` 读取本地浏览器的 cookies ✓
2. `headless_fetch()` 启动新的 Chrome 实例 ✓
3. 导航到目标 URL ✓
4. **Cookies 参数被忽略**，用户需要重新登录 ✗

目标：在启动 Chrome 后，将读取到的 cookies 注入到新实例中，实现无缝认证复用。

## Goals / Non-Goals

**Goals:**
- 复用本地浏览器的登录状态，避免重复登录
- 注入失败时静默降级，仍显示原有的登录提示
- 保持现有 API 和行为不变

**Non-Goals:**
- 不支持多 profile 切换（只使用默认 profile）
- 不支持跨浏览器 cookies 迁移（如 Firefox → Chrome）
- 不修改 `cookie_scoop` 或 `headless_chrome` 库本身
- 不持久化注入的 cookies（每次从浏览器重新读取）

## Decisions

### 1. 使用 CDP `Network.setCookie` 注入 cookies

**决策**：通过 `tab.set_cookies()` 方法将 cookies 注入到 Chrome 实例中。

**理由**：
- `headless_chrome` 已提供高级 API 封装，无需直接调用 CDP
- 相比使用 `--user-data-dir` 指定 profile 目录，此方案不受 Chrome 运行状态限制
- 相比复制到临时 profile，此方案更轻量且实时性更好

**替代方案**：
- **使用 Chrome Profile 目录**：需要 Chrome 未运行，用户体验差
- **复制 Profile 到临时目录**：实现复杂，且可能包含大量无关数据
- **连接到已运行的 Chrome**：需要用户手动配置远程调试端口

### 2. 注入后自动重新加载页面

**决策**：调用 `tab.reload()` 使 cookies 生效。

**理由**：
- Cookies 需要在页面加载时发送给服务器
- 部分网站在首次请求时就会检查认证状态
- 重新加载确保 cookies 被正确使用

### 3. 设置 cookies 的 `url` 参数

**决策**：在创建 `CookieParam` 时，设置 `url: Some(url.to_string())`。

**理由**：
- Chrome 会根据 URL 自动推断 `domain` 和 `path`
- 简化实现，无需手动解析 domain
- 对于大多数场景足够（除非需要设置跨域 cookies）

### 4. 静默降级策略

**决策**：如果 cookies 注入失败，不报错，继续显示原有的登录提示。

**理由**：
- Cookies 可能因为各种原因失效（过期、SameSite 限制等）
- 保持向后兼容，不影响现有工作流
- 用户仍可通过手动登录完成认证

## Risks / Trade-offs

**[Risk] Cookie 属性丢失**
我们只有 `name` 和 `value`，缺少 `domain`、`path`、`secure`、`httpOnly`、`sameSite`、`expires` 等属性。
→ **Mitigation**: 设置 `url` 参数让 Chrome 自动推断；对于大多数场景足够；注入失败时静默降级。

**[Risk] SameSite 策略限制**
现代浏览器对跨站 cookies 有严格限制，某些 cookies 可能无法成功注入。
→ **Mitigation**: 由于 cookies 来自同一浏览器，属性应该是一致的；注入失败时显示登录提示。

**[Risk] Session Cookies 过期**
从浏览器读取的 session cookies 可能在 Chrome 重启后失效。
→ **Mitigation**: 每次调用时重新读取 cookies；如果失效，用户仍可手动登录。

**[Risk] HttpOnly Cookies 无法读取**
某些关键的认证 cookies 可能标记为 `HttpOnly`，无法通过 JavaScript 读取。
→ **Mitigation**: `cookie_scoop` 直接读取浏览器的 cookie 数据库，应该能获取 HttpOnly cookies。

**[Trade-off] 实现简单性 vs 完整性**
我们选择不保存 cookie 的完整属性，简化实现但可能降低成功率。
→ **接受**：对于大多数网站，`name` + `value` + `url` 足够；复杂场景用户仍可手动登录。
