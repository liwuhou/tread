## 1. 项目依赖与模块结构

- [x] 1.1 在 `Cargo.toml` 中添加依赖：`dom_smoothie`（Readability 内容提取）
- [x] 1.2 创建 `src/web.rs` 模块骨架

## 2. HTTP 抓取（TDD）

- [x] 2.1 编写测试：`fetch_html` 发起 HTTP 请求并返回 HTML 字符串
- [x] 2.2 编写测试：`fetch_html` 设置 Firefox User-Agent header
- [x] 2.3 编写测试：网络失败返回错误（使用无效 URL）
- [x] 2.4 实现 `fetch_html(url)` 函数，使测试通过

## 3. Readability 正文提取（TDD）

- [x] 3.1 编写测试：`extract_content` 从 HTML 中提取标题和正文
- [x] 3.2 编写测试：提取失败时返回错误
- [x] 3.3 编写测试：提取结果的 content_html 为有效 HTML 片段
- [x] 3.4 实现 `extract_content(html)` 函数，使测试通过

## 4. 缓存管理（TDD）

- [x] 4.1 编写测试：`web_cache_dir` 返回 `~/.tread/cache/web/`
- [x] 4.2 编写测试：`web_cache_key` 对 URL 生成 SHA-256 短哈希
- [x] 4.3 编写测试：`save_web_cache` 保存 HTML 和元数据
- [x] 4.4 编写测试：`load_web_cache` 读取缓存，24h 内返回命中
- [x] 4.5 编写测试：缓存过期时返回 None
- [x] 4.6 实现缓存管理函数，使测试通过

## 5. 图片 URL 解析（TDD）

- [x] 5.1 编写测试：绝对 URL 直接返回
- [x] 5.2 编写测试：以 `/` 开头的路径基于 base URL 解析
- [x] 5.3 编写测试：相对路径 `../` 正确解析
- [x] 5.4 实现 `resolve_image_url(url, base_url)` 函数

## 6. HTML → LineContent 转换

- [x] 6.1 编写测试：Readability 输出喂给 `xhtml_to_lines` 正确转换
- [x] 6.2 编写测试：标题、粗体、链接保留样式
- [x] 6.3 编写测试：图片节点带解析后的绝对 URL
- [x] 6.4 集成 `html_to_lines(html, base_url)` 函数

## 7. 进度保存（TDD）

- [x] 7.1 编写测试：`save_web_progress` 保存到 `~/.tread/progress/web_<hash>.json`
- [x] 7.2 编写测试：`load_web_progress` 读取进度
- [x] 7.3 编写测试：进度不存在时返回 None
- [x] 7.4 实现进度保存函数

## 8. CLI 集成与入口

- [x] 8.1 `main.rs` 检测 URL 参数（`http://` / `https://` 开头）
- [x] 8.2 解析 `--refresh` / `-r` 参数
- [x] 8.3 实现 `run_web(url, refresh)` 函数：抓取→提取→缓存→渲染
- [x] 8.4 Readability 提取失败时显示"无法提取正文"
- [x] 8.5 状态栏显示网页标题

## 9. 集成验证

- [x] 9.1 `cargo build` 无 warning 编译通过
- [x] 9.2 `cargo test` 所有测试通过
- [x] 9.3 手动测试：`tread https://example.com` 抓取并显示正文
- [x] 9.4 手动测试：`tread <url> --refresh` 强制刷新
- [x] 9.5 手动测试：缓存命中时不发起 HTTP 请求
- [x] 9.6 手动测试：进度保存和恢复
- [x] 9.7 手动测试：网页中的图片 Tab 导航 + Enter 查看
