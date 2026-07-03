# 单位规范与自适应方案

**核心原则：源码统一写 `px`，H5 构建时 PostCSS 自动转 `rem`**

- **共享组件**：必须使用 `px`，PostCSS 自动处理转换
- **H5 应用级代码**：应直接使用 `rem`
- **PC 端代码**：自由使用 `px`
- **小程序端**：使用 `vmin`

## Figma 设计稿宽度换算

rootValue 固定 37.5（对应 375px 设计稿）：

- **375px 设计稿**：px 值直接使用
- **750px 设计稿**：所有 px 值 ÷2
- **其他宽度**：px 值 ÷(frameWidth/375)
- **1px 边框**：不换算

## 新建 H5 工程规范

必须使用 `defineH5Config` 代替 `defineConfig`，确保 PostCSS px→rem 转换、html root font-size 注入、CSS 头部注释自动生效。

纯 CSS/JS 消费方必须自行设置：`html { font-size: calc(100vw / 10); }`
