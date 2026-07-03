---
description: 查看 @xiaoe/live-devkit 提供的所有可用指令
---

# @xiaoe/live-devkit 指令总览

## Skills（自动触发）

| Skill | 说明 |
|-------|------|
| `openspec-apply-change` | 实现 OpenSpec change 的任务 |
| `openspec-new-change` | 创建新的 OpenSpec change |
| `openspec-propose` | 快速创建 change 并生成所有 artifacts |
| `openspec-continue-change` | 继续推进 change 的下一个 artifact |
| `openspec-ff-change` | 快速推进 change 的所有 artifacts |
| `openspec-explore` | 进入探索模式，深入思考问题 |
| `openspec-verify-change` | 验证实现是否匹配 change artifacts |
| `openspec-archive-change` | 归档已完成的 change |
| `openspec-bulk-archive-change` | 批量归档多个 change |
| `openspec-sync-specs` | 将 delta specs 同步到主 specs |
| `openspec-onboard` | OpenSpec 引导式上手 |
| `figma-spec` | 解析 Figma 设计稿生成 UI 规格文档 |
| `figma-use` | 隔离 Figma MCP 调用，防止上下文撑满 |
| `create-plugin` | 创建标准 SDK 插件 package |
| `alive-system-bridge` | 直播后台系统开发桥接 |

## Slash Commands（手动 /xxx 触发）

**opsx 命名空间：**

| 命令 | 说明 |
|------|------|
| `/opsx:apply` | 实现 change 的任务 |
| `/opsx:new` | 创建新 change |
| `/opsx:propose` | 快速创建 change + 所有 artifacts |
| `/opsx:continue` | 继续推进 change |
| `/opsx:ff` | 快速推进所有 artifacts |
| `/opsx:explore` | 探索模式 |
| `/opsx:verify` | 验证实现 |
| `/opsx:archive` | 归档 change |
| `/opsx:bulk-archive` | 批量归档 |
| `/opsx:sync` | 同步 delta specs 到主 specs |
| `/opsx:onboard` | OpenSpec 上手引导 |

**devkit 命名空间：**

| 命令 | 说明 |
|------|------|
| `/devkit:help` | 查看所有可用指令（本页面） |
| `/devkit:uninstall` | 卸载 devkit 并还原配置 |

## CLI 命令（终端执行）

```bash
live-devkit config show             # 查看合并后的配置及来源
live-devkit skill list              # 列出所有官方 skills
live-devkit rule reset <name>       # 重置指定规则到最新版本
live-devkit docs sync               # 同步官方文档到本地
live-devkit openspec sync [--force] # 同步 OpenSpec 配置
live-devkit merge-config            # 手动触发配置合并
live-devkit uninstall [--dry-run]   # 卸载 devkit 并还原配置
```
