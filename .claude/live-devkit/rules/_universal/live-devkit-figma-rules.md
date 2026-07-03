# Figma 设计稿还原规则

1. **禁止手写 SVG**：所有 SVG 必须通过 `/figma-use download` 从 Figma 下载，严禁手写或格式转换替代
2. **颜色保真**：原始颜色值原样写入，不得替换为 `currentColor` 或 CSS 变量
3. **文件保真**：几个独立 SVG 就生成几个文件，不得合并
4. **坐标系保真**：保持原始 viewBox，不做坐标变换
5. **CSS 组合**：多 SVG 的组合定位通过 CSS 实现
6. **Figma 操作隔离**：所有 Figma MCP 调用通过 `/figma-use` 执行
7. **单位规范**：源码统一 `px`，H5 构建自动转 `rem`，PC 保留 `px`，小程序用 `vmin`
