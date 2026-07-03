---
name: xed-design
description: UI 实现时参考设计系统还原样式。当涉及 UI 变更且无 Figma 设计稿时，提供 XED 设计系统（内置）或用户自定义设计规范作为样式对照源。
---

# 设计系统参考

## 触发条件

当检测到以下场景时激活本 skill：

- UI 变更任务且无 `design-spec.md`（无 Figma 设计稿）
- `/opsx:apply` 实现 UI 组件时需要样式参考
- 用户主动要求参考设计系统

## 交互逻辑

检测到 UI 变更且无设计稿时，询问用户：

> "这个变更涉及 UI 样式，是否需要参考设计系统实现？"
> 1) XED 设计系统（项目内置）
> 2) 自定义 — 请提供设计规范目录路径或文件
> 3) 不需要参考

| 选择 | 行为 |
|------|------|
| XED 设计系统 | 加载本 skill `references/` 下的文件 |
| 自定义 | 读取用户指定的路径/文件，遵循同样原则 |
| 不需要 | 不加载任何设计系统，纯交互 demo 驱动 |

选择结果记录到当前 change 的 `design-context.md` 中，后续 apply 阶段自动复用。

## 渐进式导航地图

根据任务类型按需读取，**不要一次性加载所有 references**：

| 任务 | 读取文件 | 优先级 |
|------|---------|--------|
| 任何 UI 任务 | `references/tokens.md` | 必须 |
| 实现按钮/输入框/卡片/表格/弹窗/徽标 | `references/components.md` | 按需 |
| 搭建页面结构/布局 | `references/layout.md` | 按需 |
| 做设计决策/Review | `references/principles.md` | 按需 |
| H5/小程序/移动端 | `references/mobile.md` | 按需 |

**读取策略**：先读 tokens.md（任何 UI 任务都需要），再根据具体组件/场景追加读取。

## 5 条铁律

1. **Token-first** — 颜色、间距、字号、圆角、阴影必须使用 CSS 变量引用，禁止硬编码数值。如无对应 token，定义新 token 并注明
2. **语义化命名** — 使用 `var(--color-primary)` 而非 `#1472FF`，使用 `var(--spacing-lg)` 而非 `16px`
3. **状态完整** — 每个 UI 组件必须覆盖：default / hover / active / disabled / focus / error（如适用）
4. **单主按钮** — 每个容器最多一个 Primary Button，其余用 Secondary 或 Text Button
5. **待确认优先** — 任何不确定的样式值写 `待确认`，不猜测

## 与 OpenSpec 工作流的桥接

| 阶段 | 行为 | 条件 |
|------|------|------|
| explore | 询问是否参考设计系统 | UI 变更 + 无 Figma |
| propose | 记录设计系统选择到 change | 用户已选择 |
| apply | 加载对应设计系统文件，输出 Design Fidelity Checklist | 有 UI 任务 |
| verify | 增加 Design Fidelity 维度 | 已使用设计系统 |

## 自定义设计系统

当用户选择"自定义"时：

1. 读取用户指定的目录或文件
2. 提取 token 定义（颜色/间距/字号/圆角/阴影）
3. 提取组件样式模式（如有）
4. 遵循同样的 token-first 原则
5. 将提取结果写入 change 的 `design-context.md`

用户提供的自定义规范格式可以是：
- CSS 变量文件（`.css` / `.scss`）
- JSON token 文件
- Markdown 设计规范文档
- 任意目录结构（skill 会扫描并提取 token）
