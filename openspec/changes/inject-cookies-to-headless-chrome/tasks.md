## 1. 测试（TDD - 先写测试）

- [x] 1.1 在 `src/headless.rs` 的 `tests` 模块中添加测试：验证 `headless_fetch()` 在有 cookies 参数时能成功注入（mock 或使用测试服务器）
- [x] 1.2 添加测试：验证 cookies 为空时跳过注入逻辑
- [x] 1.3 添加测试：验证 cookies 注入失败时不报错，继续执行（静默降级）

## 2. 核心实现

- [x] 2.1 在 `src/headless.rs` 的 `headless_fetch()` 函数中，移除 `_cookies` 参数的下划线前缀
- [x] 2.2 在 `tab.navigate_to()` 和 `tab.wait_until_navigated()` 之后，添加 cookies 注入逻辑：
  - 检查 `cookies` 是否非空
  - 将 `&[(String, String)]` 转换为 `Vec<CookieParam>`
  - 为每个 `CookieParam` 设置 `url` 参数
  - 调用 `tab.set_cookies(cookie_params)`
  - 调用 `tab.reload()` 重新加载页面
  - 再次调用 `tab.wait_until_navigated()` 等待页面加载完成
- [x] 2.3 使用 `?` 操作符或 `.ok()` 处理注入错误，实现静默降级（注入失败不中断流程）

## 3. 导入和类型

- [x] 3.1 在 `src/headless.rs` 顶部添加必要的导入：`use headless_chrome::protocol::cdp::Network::CookieParam;`

## 4. 手动验证

- [x] 4.1 使用 `-i` 模式访问一个需要登录的网站（如已登录的 GitHub、Twitter 等），验证无需重新登录
- [x] 4.2 验证在无 cookies 时（如隐私模式或无登录记录），仍显示原有的登录提示
- [x] 4.3 验证 cookies 注入失败时（如浏览器未运行），程序不崩溃，继续显示登录提示

## 5. 文档和提交

- [x] 5.1 更新函数注释，说明 cookies 注入行为
- [x] 5.2 使用 Conventional Commits 格式提交：`feat(headless): inject browser cookies to avoid re-login in interactive mode`
