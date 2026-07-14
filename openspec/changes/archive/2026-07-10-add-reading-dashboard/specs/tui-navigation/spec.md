## MODIFIED Requirements

### Requirement: Command-line interface
系统 SHALL accept an optional file path, EPUB path, or URL target. When a target is provided, the target opens directly. When no target is provided, the system opens the dashboard entry screen.

#### Scenario: 正常打开文件
- **WHEN** 用户运行 `tread sample.md`
- **THEN** 系统打开文件并从保存的 Markdown 进度开始显示（如存在）
- **AND** 如果不存在保存进度，则从第 1 行开始显示

#### Scenario: 指定起始行号
- **WHEN** 用户运行 `tread sample.md 50`
- **THEN** 系统打开文件并滚动到第 50 行
- **AND** 显式起始行号 SHALL 优先于保存的 Markdown 进度

#### Scenario: 无参数运行
- **WHEN** 用户运行 `tread`（无参数）
- **THEN** 系统进入 dashboard TUI
- **AND** 系统 SHALL NOT 输出 usage error
- **AND** dashboard 正常退出时进程退出码 SHALL 为 0

#### Scenario: URL 参数保持直接打开
- **WHEN** 用户运行 `tread https://example.com/article`
- **THEN** 系统 SHALL 直接进入网页阅读模式，而不是先进入 dashboard

#### Scenario: EPUB 参数保持直接打开
- **WHEN** 用户运行 `tread book.epub`
- **THEN** 系统 SHALL 直接进入 EPUB 阅读模式，而不是先进入 dashboard
