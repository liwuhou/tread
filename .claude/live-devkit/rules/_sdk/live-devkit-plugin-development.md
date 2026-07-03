# 插件开发规范

## 目录规范

- **插件存放位置**: `packages/plugins/`
- **插件命名规范**: `plugin-<feature>` (例如 `plugin-lottery`, `plugin-comment`, `plugin-gift`)
- **包名规范**: `@xiaoe/live-<feature>-sdk` (例如 `@xiaoe/live-lottery-sdk`)

## 插件架构

所有插件必须遵循 `@xiaoe/live-room-common-sdk` 的插件规范：

- 使用 `#` 私有字段（不用 `private` 关键字）
- 状态管理五件套：`#state` + `#stateSubscribers` + `subscribeState` + `getState` + `#setState`
- 事件总线：事件常量 + 模块增强扩展 `BusEventMap`
- 订阅清理：`#unsubs` 数组统一清理
- 生命周期：`install` → `onReady` → `onDestroy`

**完整规范**: `.claude/skills/create-plugin/references/style-guide.md`

## 创建插件

使用 `/create-plugin` skill 自动创建标准插件包：

```
/create-plugin <插件名称> [目标目录]
```

**示例**:
```
/create-plugin lottery packages/plugins
/create-plugin comment packages/plugins
```

技能会自动：
1. 创建标准插件包结构
2. 生成平台 Demo（微信小程序/Vue2/Vue3）
3. 添加构建脚本到根 `package.json`
