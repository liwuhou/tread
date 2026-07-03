## 1. 项目依赖与模块结构

- [x] 1.1 在 `Cargo.toml` 中添加依赖：`cookie-scoop`（浏览器 cookie 读取）、`headless_chrome`（Headless 浏览器）
- [x] 1.2 创建 `src/headless.rs` 模块骨架

## 2. Cookie 读取（TDD）

- [x] 2.1 编写测试：`get_cookies_for_url` 从 cookie-scoop 获取指定域名的 cookie
- [x] 2.2 编写测试：`cookies_to_header` 将 cookie 列表转为 HTTP Cookie header 字符串
- [x] 2.3 编写测试：cookie 读取失败时返回空列表（静默降级）
- [x] 2.4 实现 cookie 读取函数

## 3. Cookie 注入 HTTP 请求（TDD）

- [x] 3.1 编写测试：`fetch_html_with_cookies` 使用 cookie 发起请求
- [x] 3.2 编写测试：无 cookie 时正常请求（降级）
- [x] 3.3 实现带 cookie 的 HTTP 请求

## 4. Headless 浏览器加载（TDD）

- [x] 4.1 编写测试：`headless_fetch` 启动 Chrome、导航、获取 HTML
- [x] 4.2 编写测试：Chrome 未安装时返回错误
- [x] 4.3 编写测试：等待 JS 渲染后提取内容
- [x] 4.4 实现 `headless_fetch(url, cookies)` 函数

## 5. Session 持久化（TDD）

- [x] 5.1 编写测试：`save_session` 保存 cookie 到 `~/.tread/sessions/<domain>.json`
- [x] 5.2 编写测试：`load_session` 读取 session，24h 内有效
- [x] 5.3 编写测试：session 过期时返回 None
- [x] 5.4 实现 session 持久化函数

## 6. CLI 集成

- [x] 6.1 `main.rs` 解析 `-i` / `--interactive` 参数
- [x] 6.2 `run_web` 函数支持 `interactive` 模式分支
- [x] 6.3 `-i` 模式：cookie 注入 → Headless Chrome → 提取正文
- [x] 6.4 非 `-i` 模式：cookie 注入 → ureq → Readability（现有流程增强）
- [x] 6.5 提取失败时提示"尝试 -i 模式"

## 7. 集成验证

- [x] 7.1 `cargo build` 无 warning 编译通过
- [x] 7.2 `cargo test` 所有测试通过
- [x] 7.3 手动测试：`tread <url>` 自动带 cookie（已登录网站）
- [x] 7.4 手动测试：`tread <spa-url> -i` Headless 加载 SPA
- [x] 7.5 手动测试：`tread <url> -i` 已登录 SPA（cookie + Headless）
- [x] 7.6 手动测试：Chrome 未安装时 `-i` 模式给出友好提示
