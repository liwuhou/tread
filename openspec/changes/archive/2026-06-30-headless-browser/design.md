## Context

tread 已完成 Markdown + EPUB + 静态网页（Readability）阅读。但两类网站仍无法阅读：需登录的网站和 SPA 动态网站。

**已有基础设施**：
- `ureq` HTTP 客户端（同步）
- `dom_smoothie` Readability 正文提取
- 网页缓存 + 进度保存机制
- 图片缓存 + 系统查看器流程

**约束**：
- Cookie 读取需要系统级权限（macOS Keychain）
- Headless Chrome 需要用户安装 Chrome/Chromium
- `-i` 模式是按需加载，默认模式保持轻量

## Goals / Non-Goals

**Goals:**
- 使用 `cookie-scoop` 读取浏览器 cookie，自动附加到 HTTP 请求
- 支持 Chrome、Firefox、Safari、Edge 的 cookie 读取
- `-i` / `--interactive` 模式启动 Headless Chrome 加载动态内容
- Headless 模式下执行 JS、等待渲染、提取 DOM
- Cookie + Headless 组合覆盖"需登录的 SPA"
- 可选的 session 持久化（`~/.tread/sessions/<domain>.json`）
- 默认模式（无 `-i`）行为不变，不启动 Chrome

**Non-Goals:**
- 自动填写登录表单（用户需要在浏览器中手动登录）
- 支持所有浏览器版本（依赖 cookie-scoop 的支持范围）
- 替代完整的网页浏览器（不做前进/后退/地址栏）
- 处理 CAPTCHA / 人机验证

## Decisions

### 1. Cookie 读取：cookie-scoop

**选择**：`cookie-scoop` crate

**理由**：
- 跨平台（macOS/Windows/Linux）
- 支持 Chrome、Edge、Firefox、Safari
- 内置解密支持（macOS Keychain、Windows DPAPI、Linux Secret Service）
- 2026 年 2 月发布，活跃维护

### 2. Headless 浏览器：headless_chrome

**选择**：`headless_chrome` crate

**理由**：
- 同步 API，与现有架构一致
- 基于 Chrome DevTools Protocol
- API 简单：`Browser::new()` → `tab.navigate_to()` → `tab.get_content()`
- 社区成熟，Star 数高

**替代方案**：
- `chromiumoxide` — 异步（tokio），需要引入运行时，与同步架构冲突

### 3. Cookie 注入流程

```
tread <url> [-i]
    │
    ├─ 1. cookie-scoop 读取浏览器 cookie
    │     └─ 按域名过滤 → 只取目标 URL 的 cookie
    │
    ├─ 2. 构建 Cookie header
    │     └─ "Cookie: session=abc123; token=xyz..."
    │
    └─ 3. ureq 请求时附加 Cookie header
```

如果 cookie-scoop 失败（无权限/无浏览器），静默降级为无 cookie 请求。

### 4. Headless 模式流程

```
tread <url> -i
    │
    ├─ 1. 启动 Headless Chrome（自动查找 Chrome 路径）
    │
    ├─ 2. 如果 cookie 可用 → 注入 cookie 到浏览器
    │
    ├─ 3. 导航到 URL
    │     tab.navigate_to(url)?
    │
    ├─ 4. 等待页面加载完成
    │     tab.wait_until_navigated()?
    │
    ├─ 5. 额外等待 JS 渲染（可配置，默认 2 秒）
    │     std::thread::sleep(Duration::from_secs(2))
    │
    ├─ 6. 提取渲染后的 HTML
    │     let html = tab.get_content()?
    │
    ├─ 7. dom_smoothie 提取正文
    │
    └─ 8. 关闭 Headless Chrome
```

### 5. 模式选择逻辑

```
tread <url>
    │
    ├─ 无 -i 标志:
    │   ├─ 尝试 cookie-scoop 获取 cookie
    │   ├─ ureq + cookie → 获取 HTML
    │   ├─ dom_smoothie 提取正文
    │   └─ 如果提取失败 → 提示"尝试 -i 模式"
    │
    └─ 有 -i 标志:
        ├─ 启动 Headless Chrome
        ├─ 注入 cookie（如果可用）
        ├─ 导航 + 等待渲染
        ├─ 提取 HTML
        ├─ dom_smoothie 提取正文
        └─ 关闭 Chrome
```

### 6. Session 持久化（可选）

```
~/.tread/sessions/
├── example.com.json    ← { cookies: [...], saved_at: "..." }
└── ...
```

- 保存条件：cookie-scoop 成功读取 + 请求成功
- 加载条件：同域名 + session 未过期（24h）
- 如果 session 可用 → 直接用 session cookie，不读浏览器

## Risks / Trade-offs

- **[Keychain 授权]** macOS 首次读取 Chrome cookie 时弹授权框 → 正常行为，用户点"允许"即可
- **[Chrome 未安装]** `-i` 模式需要 Chrome/Chromium → 检测并提示安装路径
- **[Cookie 过期]** 浏览器 cookie 可能已过期 → 请求失败时静默降级，不阻塞
- **[SPA 加载时间]** 某些 SPA 加载较慢 → 可配置的等待时间（默认 2s，未来可改为检测 DOM 稳定）
- **[headless_chrome 维护]** 依赖 Chrome DevTools Protocol 稳定性 → 如果 Chrome 大版本更新导致不兼容，需要更新库版本
