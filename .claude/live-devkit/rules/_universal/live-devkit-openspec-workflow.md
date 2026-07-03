# OpenSpec 工作流（SDD 规范）

本项目采用 OpenSpec 结构化开发流程（SDD - Spec-Driven Development），**所有功能开发和 Bug 修复必须通过 OpenSpec 工作流进行**。

```
/opsx:explore → /opsx:propose → /opsx:apply → /opsx:archive
```

## SDD 强制规范

当用户请求实现功能或修复 Bug 时，**必须先检查是否存在对应的 OpenSpec change**：

1. 无活跃 change → **主动建议使用 `/opsx:new` 或 `/opsx:propose` 创建 change**
2. 仅以下情况可不经 OpenSpec：紧急 hotfix（需用户确认）、简单配置/文档修改、已有 change 的小幅调整

## Context Gate（上下文门控）

进入 explore 阶段时先评估变更是否涉及 UI：
- **Clearly NO**（后端逻辑/配置/测试/文档/API）→ 跳过所有 UI 询问
- **Clearly YES**（新页面/UI 组件/视觉变更）→ 询问 Figma/Demo/设计系统
- **Ambiguous** → 问一次，不反复追问

## Language Rule（语种匹配）

Agent 与用户交互时**必须匹配用户的语言**，不得使用不同语种提问。

## UI 集成工作流（按需触发）

涉及 UI 变更时，OpenSpec 工作流自动集成以下能力（桥接细节由各 skill 控制）：

| 触发条件 | 自动行为 |
|---------|---------|
| 有 Figma 设计稿 | `/opsx:propose` 自动调用 `/figma-spec`，apply 阶段输出 Design Fidelity Checklist |
| 需要交互验证 | `/opsx:explore` 建议生成交互 demo，apply 阶段输出 Interaction Fidelity Checklist |
| 无 Figma 但涉及 UI | `/opsx:explore` 询问设计系统参考，apply 阶段输出 Design Fidelity Checklist |

优先级：Figma design-spec > 设计系统参考 > Agent 自行推断
