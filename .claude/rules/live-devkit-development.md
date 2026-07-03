# 通用开发规范

## TDD (测试驱动开发)

所有功能开发必须遵循 TDD 流程：
1. 先写失败的测试
2. 实现最小代码使测试通过
3. 重构

核心模块测试覆盖率要求（vitest.config.ts）：
- lines: 80%
- functions: 80%
- branches: 80%
- statements: 80%

## TypeScript 配置

- `strict: true` - 严格模式
- `noUncheckedSideEffectImports: true` - 严格导入检查
- `noUnusedLocals` / `noUnusedParameters: true` - 禁止未使用变量
- 公共 API 必须导出类型定义

## Conventional Commits

提交格式：`<type>(<scope>): <subject>`

- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档变更
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具
