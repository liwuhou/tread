# 组件样式参考

> 每个组件必须覆盖完整状态：default / hover / active / disabled / focus / error（如适用）。

---

## Button 按钮

### Primary Button

| 属性 | 值 |
|------|-----|
| background | `var(--color-primary)` → `#1472FF` |
| text color | `#ffffff` |
| border-radius | `var(--radius-md)` → 8px |
| padding | 8px 16px |
| font | `var(--xe-font-size-button)` 14px/500 |
| hover | `var(--color-primary-hover)` → `#428EFF` |
| active | `var(--color-primary-active)` → `#005AE0` |
| disabled | `var(--color-bg-disabled)` bg, `var(--color-text-disabled)` text |
| focus | visible ring using primary color |

> 每个容器最多一个 Primary Button。

### Secondary / Ghost Button

| 属性 | 值 |
|------|-----|
| background | `transparent` |
| border | `1px solid var(--color-border)` → `1px solid #EBEBEB` |
| text color | `var(--color-text-primary)` → `#333333` |
| border-radius | `var(--radius-md)` → 8px |
| hover | `var(--color-bg-hover)` background |

### Text Button

| 属性 | 值 |
|------|-----|
| background | transparent, no border |
| text color | `var(--color-primary)` → `#1472FF` |
| hover | underline |

### Danger Button

| 属性 | 值 |
|------|-----|
| background | `var(--color-danger)` → `#FF4747` |
| text color | `#ffffff` |

> 仅用于删除操作，必须二次确认。

---

## Input 输入框

| 属性 | 值 |
|------|-----|
| background | `var(--color-bg-container)` → `#FFFFFF` |
| border | `1px solid var(--color-border)` → `1px solid #EBEBEB` |
| border-radius | `var(--radius-md)` → 8px |
| padding | 10px 12px |
| font | `var(--xe-font-size-body)` 14px/400 |
| focus | `1px solid var(--color-primary)` border + `var(--elevation-1)` shadow |
| error | `1px solid var(--color-danger)` border |
| disabled | `var(--color-bg-disabled)` bg, `var(--color-text-disabled)` text |

### Select / Dropdown

| 属性 | 值 |
|------|-----|
| trigger | 同 Input |
| panel | `var(--color-bg-container)` bg, `var(--elevation-4)` shadow, 8px radius |
| item hover | `var(--color-bg-hover)` background |

---

## Card 卡片 / 容器

| 属性 | 值 |
|------|-----|
| background | `var(--color-bg-container)` → `#FFFFFF` |
| border | `1px solid var(--color-border)` → `1px solid #EBEBEB` |
| border-radius | `var(--radius-lg)` → 12px |
| padding | `var(--spacing-xl)` → 24px |
| shadow (default) | flat (无阴影) |
| shadow (hover) | `var(--elevation-3)` |
| title font | `var(--xe-font-size-h3)` 14px/600 |
| body font | `var(--xe-font-size-body)` 14px/400 |

---

## Table 表格

| 属性 | 值 |
|------|-----|
| background | `var(--color-bg-container)` → `#FFFFFF` |
| header bg | `var(--color-bg-secondary)` → `#FAFAFA` |
| row border | `1px solid var(--color-border)` → `1px solid #EBEBEB` |
| row hover | `var(--color-bg-hover)` → `#F8F8F8` |
| pagination margin | `var(--spacing-xl)` → 24px top |

### 操作列规则

- 按钮数 ≤ 3：平铺
- 按钮数 > 3：前 2 个平铺，其余收进"更多"
- 危险操作（删除/禁用）：必须 Popconfirm 或 Dialog 二次确认
- 选择列固定在第一列，操作列固定在最后一列

### 状态徽标

| 状态 | 颜色 | 用途 |
|------|------|------|
| pending | `warning` (橙) | 待处理/待审核 |
| processing | `primary` (蓝) | 进行中 |
| success | `success` (绿) | 已完成/已通过 |
| danger | `danger` (红) | 已拒绝/失败 |
| default | `default` (灰) | 已关闭/草稿 |

---

## Dialog / Drawer

### Dialog

| 属性 | 值 |
|------|-----|
| background | `var(--color-bg-container)` → `#FFFFFF` |
| border-radius | `var(--radius-xl)` → 16px |
| shadow | `var(--elevation-5)` |
| widths | 480 / 640 / 800 / 960px |
| header padding | 20px 24px, H2 标题 |
| body padding | 24px |
| footer padding | 16px 24px, 按钮右对齐 |
| ESC | 关闭弹窗 |
| backdrop | `rgba(0,0,0,0.5)` |

### Drawer

| 属性 | 值 |
|------|-----|
| background | `var(--color-bg-container)` → `#FFFFFF` |
| border-radius | `var(--radius-lg)` top-left → 12px |
| widths | 400 / 520 / 640px |
| shadow | `var(--elevation-5)` |

---

## Badge / Tag 徽标与标签

### 语义状态徽标

| 属性 | 值 |
|------|-----|
| success bg/text | `var(--color-success-light)` / `var(--color-success)` |
| warning bg/text | `var(--color-warning-light)` / `var(--color-warning)` |
| danger bg/text | `var(--color-danger-light)` / `var(--color-danger)` |
| border-radius | `var(--radius-sm)` → 4px |
| padding | 2px 8px |
| font | `var(--xe-font-size-caption)` → 12px |

### 胶囊形徽标

- border-radius: `9999px`（其余同上）

### 中性标签

| 属性 | 值 |
|------|-----|
| background | `var(--color-bg-secondary)` → `#FAFAFA` |
| text | `var(--color-text-secondary)` → `#666666` |
| border-radius | `var(--radius-sm)` → 4px |
| padding | 2px 8px |

---

## 空状态

| 类型 | 标题 | 操作 |
|------|------|------|
| 无数据 | "暂无数据" | Primary Button "新建" |
| 无匹配 | "无匹配结果" | Link "清空筛选" |
| 无权限 | "无访问权限" | 无操作按钮 |
