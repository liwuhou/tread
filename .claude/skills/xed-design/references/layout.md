# 布局规则

---

## 页面结构

### PC 后台列表页（标准）

```
[Page Title / Breadcrumb]
[Tabs or Inner Navigation]        (可选)
[Filter / Tips / Action Area]     (可选)
[Main Content Body]               ← 默认表格，非卡片网格
[Pagination / Bottom Action]      (可选)
```

### 间距节奏

页面层级优先靠间距和排版区分，避免过度依赖颜色或边框。

模块之间以留白分层，不通过重边框堆叠层次。

---

## 画布尺寸

| 端 | 画布宽度 | TopNav | Sidebar | 内容内边距 |
|----|---------|--------|---------|-----------|
| PC 后台 | 1440px | 56px | 152px | 24px |
| H5 | 375px | 44px navbar + 44px status bar | — | 16px |
| App | 375px | 44px navbar + 44px status bar | TabBar 83px | 16px |
| 小程序 | 375px design / 750rpx dev | 44px navbar | 34px safe area | 16px |

---

## 响应式断点

| 名称 | 宽度 | 关键变化 |
|------|------|---------|
| Mobile Small | < 600px | 单列，紧凑 padding (16px)，精简表格 |
| Mobile | 600–768px | 标准 H5 布局 |
| Tablet | 768–1024px | 两栏开始，admin sidebar 可折叠 |
| Desktop Small | 1024–1280px | 完整卡片网格，24px padding |
| Desktop | 1280–1440px | 完整布局，1440px 画布 |
| Large Desktop | > 1440px | 居中内容，最大内容宽度 1288px |

### 折叠策略

- **管理台表格**：列从右向左折叠，操作列最后移除
- **筛选区**：1024px 以下折叠为可展开面板
- **分页**：768px 以下切换为"加载更多"或简化分页

---

## 深度层级与边框

边框可以作为阴影的替代，尤其适合深色背景：

| 场景 | 浅色背景 | 深色背景 |
|------|---------|---------|
| 分隔边界 | `1px solid var(--color-border)` | `1px solid rgba(255,255,255,0.08)` |
| 容器描边 | `1px solid var(--color-border-light)` | `1px solid rgba(255,255,255,0.12)` |
| 选中态 | `1px solid var(--color-primary)` | `1px solid var(--color-primary)` |

---

## 跨端适配口径

| 维度 | Web 管理台 | Mobile/H5/小程序 |
|------|-----------|------------------|
| 信息密度 | 中高，支持批量和多列 | 中低，强调触控可达 |
| 字体层级 | 强调表格与分组层次 | 强调主流程与可点击元素 |
| 操作反馈 | Message / Notification | Toast / NoticeBar / Modal |
| 布局策略 | 多栏和固定操作区 | 单列滚动与底部操作 |

---

## B端 / C端 用语

- B 端（管理台）→ "您"
- C 端（学员端）→ "你"
