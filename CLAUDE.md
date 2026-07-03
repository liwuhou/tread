<!-- @xiaoe/live-devkit:start -->
## AI 开发工具链

本项目使用 [@xiaoe/live-devkit](https://www.npmjs.com/package/@xiaoe/live-devkit)，启用以下工作流：

- **OpenSpec SDD**：功能开发和 Bug 修复走 spec-driven development：
  `/opsx:new` → `/opsx:propose` → `/opsx:apply` → `/opsx:archive`
- **TDD**：默认采用测试驱动开发，遵循 `.claude/rules/live-devkit-development.md`
- **Figma 设计稿还原**：UI 开发自动集成设计稿解析，遵循 `.claude/rules/live-devkit-figma-rules.md`

**Context Gate**：对不涉及 UI 的变更（后端逻辑、配置、测试、文档），不要询问 Figma 或交互 Demo。

可用 Skills、Commands、Rules 详见 `.claude/skills/`、`.claude/commands/`、`.claude/rules/`。
<!-- @xiaoe/live-devkit:end -->