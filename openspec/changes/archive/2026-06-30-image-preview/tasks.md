## 1. 项目依赖与数据模型

- [x] 1.1 在 `Cargo.toml` 中添加依赖：`ureq`（HTTP 下载）、`dirs`（home 目录）、`sha2`（URL 哈希）
- [x] 1.2 创建 `src/image.rs` 模块，定义 `ImageNode` 结构体（alt, url, local_path, id）
- [x] 1.3 定义 `LineContent` 枚举（`Styled` / `Image`），修改 parser 返回类型

## 2. 图片缓存（TDD）

- [x] 2.1 编写缓存模块测试骨架
- [x] 2.2 测试：`resolve_image_path` 对本地路径直接返回绝对路径
- [x] 2.3 测试：`cache_dir()` 返回 `~/.tread/cache/`
- [x] 2.4 测试：`url_to_cache_path` 生成正确的 hash 文件名
- [x] 2.5 测试：`ensure_cache_dir` 创建目录（幂等）
- [x] 2.6 测试：`download_image` 下载远程图片到缓存（使用 mock 或跳过网络测试）
- [x] 2.7 测试：缓存命中时不重复下载
- [x] 2.8 实现缓存模块，使所有测试通过

## 3. Parser 图片识别（TDD）

- [x] 3.1 编写 parser 图片测试：识别 `![alt](url)` 生成 ImageNode
- [x] 3.2 编写 parser 图片测试：空 alt 文本
- [x] 3.3 编写 parser 图片测试：远程 URL 图片
- [x] 3.4 编写 parser 图片测试：图片与文本混合
- [x] 3.5 修改 `parse_markdown` 处理 `Event::Start(Tag::Image)` / `Event::End(TagEnd::Image)`，使测试通过

## 4. App 图片焦点（TDD）

- [x] 4.1 在 App 中添加图片列表和图片焦点索引
- [x] 4.2 测试：Tab 跳到下一个图片
- [x] 4.3 测试：Shift+Tab 跳到上一个图片
- [x] 4.4 测试：Tab 循环（最后一张 → 第一张）
- [x] 4.5 测试：无图片时 Tab 无效
- [x] 4.6 测试：Enter 打开焦点图片（验证调用了正确的系统命令）
- [x] 4.7 测试：`local_path = None` 时 Enter 不执行命令
- [x] 4.8 实现图片焦点逻辑，使测试通过

## 5. UI 图片占位渲染

- [x] 5.1 渲染 `ImageNode` 为 `[📷 alt text]` 占位符（Cyan fg）
- [x] 5.2 空 alt 显示 `[📷 image]`
- [x] 5.3 下载失败显示 `[📷 alt ⚠ 下载失败]`（Red fg）
- [x] 5.4 焦点状态高亮（DarkGray bg + Bold）

## 6. 系统打开命令

- [x] 6.1 实现 `open_with_viewer(path)` — macOS 用 `open`，Linux 用 `xdg-open`
- [x] 6.2 处理命令不存在 / 执行失败的情况

## 7. 集成与清理

- [x] 7.1 `cargo build` 无 warning 编译通过
- [x] 7.2 `cargo test` 所有测试通过
- [x] 7.3 更新 `sample.md` 添加图片测试用例（本地图片 + 远程图片）
- [x] 7.4 手动测试：Tab 导航图片焦点、Enter 打开图片、滚动不受影响
