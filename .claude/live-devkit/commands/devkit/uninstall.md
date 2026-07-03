---
description: 卸载 @xiaoe/live-devkit 并还原用户原始配置
---

# 卸载 @xiaoe/live-devkit

卸载将还原以下内容：

- `.claude/settings.json` 中 devkit 写入的字段（hooks 等）
- `.claude/skills/` 中 devkit 复制的目录
- `.claude/commands/` 中 devkit 复制的目录
- `.claude/rules/big-class-*.md` 规则文件
- 状态追踪文件和备份文件

## 操作步骤

**1. 先预览将移除的内容（推荐）：**

```bash
npx live-devkit uninstall --dry-run
```

**2. 确认无误后执行卸载：**

```bash
npx live-devkit uninstall
```

**3. 卸载完成后移除 npm 包：**

```bash
npm uninstall @xiaoe/live-devkit
# 或
pnpm remove @xiaoe/live-devkit
```

## 注意事项

- 如果 `.claude-devkit-state.json` 丢失，将无法精确还原，需手动清理
- 已修改过的 devkit 复制文件会被直接删除（建议先 `--dry-run` 检查）
- 卸载不影响 `openspec/config.yaml` 和用户自定义的 rules 文件
