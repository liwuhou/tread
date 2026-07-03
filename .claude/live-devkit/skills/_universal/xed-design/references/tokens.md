# Design Tokens 速查

> 所有值必须通过 CSS 变量引用，禁止硬编码。

---

## 颜色

### 品牌色

| Token | 值 | 用途 |
|-------|-----|------|
| `--color-primary` | `#1472FF` | 主按钮、链接、品牌强调 |
| `--color-primary-hover` | `#428EFF` | 主按钮 hover |
| `--color-primary-active` | `#005AE0` | 主按钮 active |
| `--color-primary-light` | `#EBF3FF` | 浅色背景（Tag、高亮区块） |
| `--color-primary-lighter` | `#F0F7FF` | 更浅背景 |

### 语义色

| Token | 值 | 用途 |
|-------|-----|------|
| `--color-success` | `#07C160` | 成功状态 |
| `--color-success-light` | `#EBFFEB` | 成功 Tag 背景 |
| `--color-warning` | `#FFA51F` | 警告状态 |
| `--color-warning-light` | `#FFF8EE` | 警告 Tag 背景 |
| `--color-danger` | `#FF4747` | 错误/危险/删除 |
| `--color-danger-light` | `#FFEBEB` | 错误 Tag 背景 |

> `--color-info` 不存在。信息提示用 `--color-primary`。

### 文字色

| Token | 值 | 用途 |
|-------|-----|------|
| `--color-text-primary` | `#333333` | 标题、正文 |
| `--color-text-secondary` | `#666666` | 次要文字、说明 |
| `--color-text-tertiary` | `#999999` | 占位符、弱化信息 |
| `--color-text-disabled` | `#B2B2B2` | 禁用态文字 |
| `--color-text-link` | `#1472FF` | 链接文字 |

### 背景色

| Token | 值 | 用途 |
|-------|-----|------|
| `--color-bg-page` | `#F5F7FA` | 页面背景 |
| `--color-bg-container` | `#FFFFFF` | 卡片/容器表面 |
| `--color-bg-hover` | `#F8F8F8` | 行/元素 hover |
| `--color-bg-disabled` | `#F5F5F5` | 禁用态背景 |
| `--color-bg-secondary` | `#FAFAFA` | 次要背景（表头等） |

### 边框色

| Token | 值 | 用途 |
|-------|-----|------|
| `--color-border` | `#EBEBEB` | 默认边框 |
| `--color-border-light` | `#E5E5E5` | 浅色边框 |
| `--color-border-dark` | `#CCCCCC` | 深色边框 |

---

## 间距

8px 基础节奏：8 / 16 / 24 / 32。4px 和 12px 用于微调。

| Token | 值 | 用途 |
|-------|-----|------|
| `--spacing-xs` | `4px` | 紧凑间距、微对齐 |
| `--spacing-sm` | `8px` | 小间距 |
| `--spacing-md` | `12px` | 中等间距 |
| `--spacing-lg` | `16px` | 标准间距 |
| `--spacing-xl` | `24px` | 标准内容内边距、区块主间距 |
| `--spacing-2xl` | `32px` | 大留白、大区块分隔 |

---

## 字体

### 字体族

```
PingFang SC, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif
```

### 字号层级

| Token | 大小 | 字重 | 行高 | 用途 |
|-------|------|------|------|------|
| `--xe-font-size-h1` | `20px` | `600` | `28px` | 页面标题 |
| `--xe-font-size-h2` | `16px` | `600` | `22px` | 区块标题 |
| `--xe-font-size-h3` | `14px` | `600` | `20px` | 卡片标题 |
| `--xe-font-size-body-lg` | `14px` | `400` | `22px` | 描述文本 |
| `--xe-font-size-body` | `14px` | `400` | `20px` | 正文 |
| `--xe-font-size-body-sm` | `12px` | `400` | `18px` | 次要信息 |
| `--xe-font-size-caption` | `12px` | `400` | `16px` | 小标签 |
| `--xe-font-size-button` | `14px` | `500` | `20px` | 按钮文字 |

> font-size / line-height / font-weight 三者必须配套使用，禁止混搭。禁止硬编码 `14px`、`500` 等值。

---

## 圆角

| Token | 值 | 用途 |
|-------|-----|------|
| `--radius-sm` | `4px` | Tag、徽标 |
| `--radius-md` | `8px` | 按钮、输入框 |
| `--radius-lg` | `12px` | 卡片、面板 |
| `--radius-xl` | `16px` | 弹窗、抽屉 |

---

## 阴影（Elevation）

6 级深度系统，阴影颜色限 `rgba(0,0,0, 0.05~0.16)` 范围，禁止彩色阴影。

| Level | Token | CSS 值 | 用途 |
|-------|-------|--------|------|
| 0 | flat | none | 页面根背景、分隔线 |
| 1 | `--elevation-1` | `0 1px 2px rgba(0,0,0,0.05)` | 按钮/输入框 hover、focus ring |
| 2 | `--elevation-2` | `0 1px 3px rgba(0,0,0,0.08)` | 卡片默认态 |
| 3 | `--elevation-3` | `0 4px 6px rgba(0,0,0,0.10)` | 卡片 hover、Popover |
| 4 | `--elevation-4` | `0 8px 16px rgba(0,0,0,0.12)` | Dropdown、Select 面板、Tooltip |
| 5 | `--elevation-5` | `0 10px 24px rgba(0,0,0,0.16)` | Dialog、Drawer、Toast |

### 组件阴影映射

| 组件 | 默认 | hover/active |
|------|------|-------------|
| Button | flat | Level 1 |
| Input | flat | Level 1 (focus) |
| Card | Level 2 | Level 3 |
| Dropdown / Select | Level 4 | Level 4 |
| Dialog / Drawer | Level 5 | Level 5 |

### 阴影反模式

- 页面背景加阴影
- Level 0 表面上使用 Level 3+
- 非弹窗场景使用 Level 5
- 彩色阴影

---

## Token 命名映射

开发使用语义化命名（左侧），Figma 原始命名（右侧）仅供参考：

| 语义化命名（使用这个） | Figma 原始命名 | 值 |
|---------------------|--------------|-----|
| `--color-bg-page` | `--xe-bg-color-page` | `#F5F7FA` |
| `--color-bg-container` | `--xe-bg-color-container` | `#FFFFFF` |
| `--color-text-primary` | `--xe-font-color-primary(333)` | `#333333` |
| `--color-text-secondary` | `--xe-font-color-secondary(666)` | `#666666` |
| `--color-text-tertiary` | `--xe-font-color-placeholder(999)` | `#999999` |
| `--color-border` | `--xe-border-line(EB)` | `#EBEBEB` |
| `--color-primary` | `--xe-brand-color` | `#1472FF` |

---

## 使用示例

```css
/* 正确 */
.button-primary {
  background-color: var(--color-primary);
  color: white;
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  font-size: var(--xe-font-size-button);
  line-height: var(--xe-font-height-button);
  font-weight: var(--xe-font-weight-Medium);
}

/* 错误 — 禁止硬编码 */
.button-primary {
  background-color: #428EFF;       /* 应该用 var(--color-primary) */
  border-radius: 6px;              /* 应该用 var(--radius-md) */
  padding: 8px 16px;              /* 应该用 spacing token */
  font-size: 14px;                /* 应该用 font token */
  font-weight: 500;               /* 应该用 font-weight token */
}
```
